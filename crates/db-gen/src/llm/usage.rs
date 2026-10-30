use serde::{Deserialize, Serialize};

use aimux_core::types::{TokenUsage, Usage};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct TurnUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub cached_input_tokens: u64,
}

impl TurnUsage {
    pub fn from_usage(usage: &Usage) -> Self {
        let input = &usage.input_tokens;
        let output = &usage.output_tokens;

        let cached = u64::from(input.cache_read.unwrap_or(0));

        let reasoning = u64::from(output.reasoning.unwrap_or(0));

        let output_total = output
            .total
            .or_else(|| {
                let text = output.text.unwrap_or(0);
                let reasoning = output.reasoning.unwrap_or(0);
                if text == 0 && reasoning == 0 {
                    None
                } else {
                    Some(text + reasoning)
                }
            })
            .map(u64::from)
            .unwrap_or(0);

        Self {
            input_tokens: u64::from(input.total.unwrap_or(0)),
            output_tokens: output_total,
            reasoning_tokens: reasoning,
            cached_input_tokens: cached,
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.reasoning_tokens += other.reasoning_tokens;
        self.cached_input_tokens += other.cached_input_tokens;
    }

    pub fn is_empty(&self) -> bool {
        self.input_tokens == 0
            && self.output_tokens == 0
            && self.reasoning_tokens == 0
            && self.cached_input_tokens == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(input: TokenUsage, output: TokenUsage) -> Usage {
        Usage {
            input_tokens: input,
            output_tokens: output,
            raw: None,
        }
    }

    #[test]
    fn from_usage_pulls_totals() {
        let u = usage(
            TokenUsage {
                total: Some(100),
                cache_read: Some(40),
                ..Default::default()
            },
            TokenUsage {
                total: Some(25),
                text: Some(20),
                reasoning: Some(5),
                ..Default::default()
            },
        );
        let t = TurnUsage::from_usage(&u);
        assert_eq!(t.input_tokens, 100);
        assert_eq!(t.output_tokens, 25);
        assert_eq!(t.reasoning_tokens, 5);
        assert_eq!(t.cached_input_tokens, 40);
    }

    #[test]
    fn from_usage_computes_output_total_when_missing() {
        let u = usage(
            TokenUsage::default(),
            TokenUsage {
                text: Some(12),
                reasoning: Some(3),
                ..Default::default()
            },
        );
        let t = TurnUsage::from_usage(&u);
        assert_eq!(t.output_tokens, 15);
        assert_eq!(t.reasoning_tokens, 3);
    }

    #[test]
    fn from_usage_handles_missing_everything() {
        let u = usage(TokenUsage::default(), TokenUsage::default());
        assert!(TurnUsage::from_usage(&u).is_empty());
    }

    #[test]
    fn add_accumulates() {
        let mut a = TurnUsage {
            input_tokens: 10,
            output_tokens: 5,
            ..Default::default()
        };
        a.add(&TurnUsage {
            input_tokens: 3,
            output_tokens: 2,
            ..Default::default()
        });
        assert_eq!(a.input_tokens, 13);
        assert_eq!(a.output_tokens, 7);
    }
}