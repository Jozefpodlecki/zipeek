use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
pub struct GenerationSampling {
    pub temperature: f32,
    pub top_k: usize,
    pub top_p: f32,
    pub repetition_penalty: f32,
}

impl Default for GenerationSampling {
    fn default() -> Self {
        Self {
            temperature: 0.9,
            top_k: 50,
            top_p: 1.0,
            repetition_penalty: 1.05,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SamplingRng {
    state: u64,
}

impl SamplingRng {
    pub fn seeded(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x9e3779b97f4a7c15 } else { seed } }
    }

    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos() as u64);
        Self::seeded(nanos ^ (std::process::id() as u64).rotate_left(17))
    }

    fn unit_f64(&mut self) -> f64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let value = self.state.wrapping_mul(0x2545f4914f6cdd1d);
        ((value >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
    }
}

pub fn top_k_logits(logits: &[f32], k: usize) -> Result<Vec<(u32, f32)>, String> {
    if logits.is_empty() || k == 0 || logits.len() > u32::MAX as usize {
        return Err("invalid Qwen3-TTS logits shape or top-k".into());
    }
    if logits.iter().any(|logit| !logit.is_finite()) {
        return Err("Qwen3-TTS logits contain non-finite values".into());
    }
    let mut values = logits
        .iter()
        .copied()
        .enumerate()
        .map(|(index, logit)| (index as u32, logit))
        .collect::<Vec<_>>();
    values.sort_unstable_by(|left, right| {
        right.1.total_cmp(&left.1).then_with(|| left.0.cmp(&right.0))
    });
    values.truncate(k.min(values.len()));
    Ok(values)
}

pub fn greedy_code(logits: &[f32]) -> Result<u32, String> {
    top_k_logits(logits, 1).map(|tokens| tokens[0].0)
}

pub fn sample_logits(
    logits: &[f32],
    allowed_tokens: &[usize],
    history: &[u32],
    parameters: GenerationSampling,
    rng: &mut SamplingRng,
) -> Result<u32, String> {
    if allowed_tokens.is_empty()
        || parameters.top_k == 0
        || !parameters.temperature.is_finite()
        || parameters.temperature <= 0.0
        || !parameters.top_p.is_finite()
        || !(0.0 < parameters.top_p && parameters.top_p <= 1.0)
        || !parameters.repetition_penalty.is_finite()
        || parameters.repetition_penalty < 1.0
    {
        return Err("invalid Qwen3-TTS sampling parameters".into());
    }
    let mut choices = Vec::with_capacity(allowed_tokens.len());
    for &index in allowed_tokens {
        let Some(&raw) = logits.get(index) else {
            return Err("Qwen3-TTS sample index outside vocabulary".into());
        };
        if !raw.is_finite() || index > u32::MAX as usize {
            return Err("invalid Qwen3-TTS sampling logits".into());
        }
        let mut score = raw;
        if history.contains(&(index as u32)) {
            score = if score >= 0.0 {
                score / parameters.repetition_penalty
            } else {
                score * parameters.repetition_penalty
            };
        }
        choices.push((index as u32, score / parameters.temperature));
    }
    choices.sort_unstable_by(|left, right| {
        right.1.total_cmp(&left.1).then_with(|| left.0.cmp(&right.0))
    });
    choices.truncate(parameters.top_k.min(choices.len()));
    let maximum = choices[0].1;
    let mut weights = choices
        .iter()
        .map(|(_, score)| ((*score - maximum) as f64).exp())
        .collect::<Vec<_>>();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 || !total.is_finite() {
        return Err("invalid Qwen3-TTS sampling probability".into());
    }
    if parameters.top_p < 1.0 {
        let mut cumulative = 0.0;
        for index in 0..weights.len() {
            cumulative += weights[index] / total;
            if cumulative >= f64::from(parameters.top_p) {
                weights.truncate(index + 1);
                choices.truncate(index + 1);
                break;
            }
        }
    }
    let threshold = rng.unit_f64() * weights.iter().sum::<f64>();
    let mut cumulative = 0.0;
    for (index, weight) in weights.iter().enumerate() {
        cumulative += weight;
        if threshold < cumulative {
            return Ok(choices[index].0);
        }
    }
    Ok(choices.last().expect("sampling choices cannot be empty").0)
}

pub fn sample_acoustic_code(
    logits: &[f32],
    codebook_size: usize,
    eos_token: usize,
    history: &[u32],
    parameters: GenerationSampling,
    rng: &mut SamplingRng,
) -> Result<Option<u32>, String> {
    if codebook_size == 0 || codebook_size > logits.len() || eos_token >= logits.len() || eos_token < codebook_size {
        return Err("invalid Qwen3-TTS acoustic sampling vocabulary".into());
    }
    let mut allowed = (0..codebook_size).collect::<Vec<_>>();
    allowed.push(eos_token);
    let sampled = sample_logits(logits, &allowed, history, parameters, rng)?;
    Ok((sampled as usize != eos_token).then_some(sampled))
}

pub fn sample_residual_code(
    logits: &[f32],
    parameters: GenerationSampling,
    rng: &mut SamplingRng,
) -> Result<u32, String> {
    let allowed = (0..logits.len()).collect::<Vec<_>>();
    sample_logits(logits, &allowed, &[], parameters, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_rank_and_rejects_invalid_logits() {
        assert_eq!(greedy_code(&[0.4, 2.0, 2.0]).unwrap(), 1);
        assert_eq!(top_k_logits(&[0.1, 0.3, 0.2], 2).unwrap(), vec![(1, 0.3), (2, 0.2)]);
        assert!(greedy_code(&[f32::NAN]).is_err());
        assert!(greedy_code(&[]).is_err());
    }

    #[test]
    fn fixed_seed_replays_stochastic_selection() {
        let config = GenerationSampling { top_k: 3, ..Default::default() };
        let logits = [2.0, 2.1, 2.2];
        let mut first = SamplingRng::seeded(1234);
        let mut second = SamplingRng::seeded(1234);
        let samples_a = (0..64).map(|_| sample_residual_code(&logits, config, &mut first).unwrap()).collect::<Vec<_>>();
        let samples_b = (0..64).map(|_| sample_residual_code(&logits, config, &mut second).unwrap()).collect::<Vec<_>>();
        assert_eq!(samples_a, samples_b);
        assert!(samples_a.iter().copied().collect::<std::collections::HashSet<_>>().len() > 1);
    }

    #[test]
    fn repetition_penalty_can_break_a_greedy_loop() {
        let logits = [10.0, 9.8, -1.0, 8.0];
        let mut rng = SamplingRng::seeded(7);
        let result = sample_acoustic_code(&logits, 3, 3, &[0], GenerationSampling {
            top_k: 1,
            repetition_penalty: 1.1,
            ..Default::default()
        }, &mut rng).unwrap();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn eos_is_sampled_and_invalid_values_fail() {
        let mut rng = SamplingRng::seeded(4);
        assert_eq!(sample_acoustic_code(&[1.0, 2.0, 30.0], 2, 2, &[], GenerationSampling::default(), &mut rng).unwrap(), None);
        assert!(sample_acoustic_code(&[1.0, 2.0], 2, 1, &[], GenerationSampling::default(), &mut rng).is_err());
        assert!(sample_residual_code(&[f32::NAN], GenerationSampling::default(), &mut rng).is_err());
        assert!(sample_residual_code(&[1.0], GenerationSampling { temperature: 0.0, ..Default::default() }, &mut rng).is_err());
    }
}
