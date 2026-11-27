use std::time::Instant;

use zipeek_tts_stt::{
    audio_codec::{InputFormat, resample_mono},
    speech::{MAX_AUDIO_SAMPLES, Speech, decode_pcm16_wav},
};

fn normalize_reference_wav_for_stt(wav: &[u8]) -> Vec<u8> {
    assert!(wav.len() >= 44, "reference TTS audio must be WAV");
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[12..16], b"fmt ");
    assert_eq!(u32::from_le_bytes(wav[16..20].try_into().unwrap()), 16);
    assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
    assert_eq!(&wav[36..40], b"data");
    let source_rate = u32::from_le_bytes(wav[24..28].try_into().unwrap());
    assert!((8_000..=96_000).contains(&source_rate));
    assert_eq!(
        u32::from_le_bytes(wav[28..32].try_into().unwrap()),
        source_rate * 2
    );
    assert_eq!(u16::from_le_bytes(wav[32..34].try_into().unwrap()), 2);
    assert_eq!(
        u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize,
        wav.len() - 8
    );
    assert_eq!(
        u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
        wav.len() - 44
    );
    assert_eq!((wav.len() - 44) % 2, 0);

    let source: Vec<f32> = wav[44..]
        .chunks_exact(2)
        .map(|sample| i16::from_le_bytes([sample[0], sample[1]]) as f32 / 32768.0)
        .collect();
    let resampled = resample_mono(&source, source_rate, 16_000, MAX_AUDIO_SAMPLES)
        .expect("resample reference WAV to STT sample rate");
    let pcm: Vec<i16> = resampled
        .into_iter()
        .map(|sample| (sample * 32768.0).round().clamp(-32768.0, 32767.0) as i16)
        .collect();
    let data_len = u32::try_from(pcm.len() * 2).expect("reference WAV byte length");
    let mut output = Vec::with_capacity(44 + data_len as usize);
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&(36 + data_len).to_le_bytes());
    output.extend_from_slice(b"WAVEfmt ");
    output.extend_from_slice(&16_u32.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&16_000_u32.to_le_bytes());
    output.extend_from_slice(&32_000_u32.to_le_bytes());
    output.extend_from_slice(&2_u16.to_le_bytes());
    output.extend_from_slice(&16_u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&data_len.to_le_bytes());
    for sample in pcm {
        output.extend_from_slice(&sample.to_le_bytes());
    }
    assert!(
        !decode_pcm16_wav(&output)
            .expect("valid 16 kHz reference WAV")
            .is_empty()
    );
    output
}

#[test]
fn reference_audio_is_resampled_to_the_stt_wav_contract() {
    let samples = 24_000_u32;
    let mut wav = Vec::with_capacity(44 + (samples * 2) as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + samples * 2).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&24_000_u32.to_le_bytes());
    wav.extend_from_slice(&48_000_u32.to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(samples * 2).to_le_bytes());
    for index in 0..samples {
        let value = if index % 24 < 12 {
            8_000_i16
        } else {
            -8_000_i16
        };
        wav.extend_from_slice(&value.to_le_bytes());
    }
    assert!(decode_pcm16_wav(&wav).is_err());
    let converted = normalize_reference_wav_for_stt(&wav);
    assert_eq!(converted.len(), 44 + 16_000 * 2);
    assert_eq!(decode_pcm16_wav(&converted).unwrap().len(), 16_000);
}

fn transcript_matches_reference_greeting(transcript: &str) -> bool {
    (transcript.contains("你好") || transcript.contains("您好")) && transcript.contains("世界")
}

#[test]
fn reference_greeting_accepts_honorific_variant_without_accepting_unrelated_text() {
    assert!(transcript_matches_reference_greeting("你好世界"));
    assert!(transcript_matches_reference_greeting("您好，世界。"));
    assert!(!transcript_matches_reference_greeting("您好"));
    assert!(!transcript_matches_reference_greeting("今天，世界。"));
    assert!(!transcript_matches_reference_greeting("你好，朋友。"));
}

#[tokio::test]
#[ignore = "requires working CUDA Qwen3-TTS checkpoint and model-backed performance acceptance"]
async fn twenty_warm_tts_requests_reuse_worker_and_produce_audio() {
    let speech = Speech::new().expect("speech runtime configuration");
    assert!(speech.qwen_tts_checkpoint_present());
    speech
        .initialize_neural_engines()
        .await
        .expect("model warm-up");
    assert_eq!(speech.neural_tts_status(), "ready");
    let mut durations = Vec::with_capacity(20);
    for index in 0..20 {
        let start = Instant::now();
        let wav = speech
            .synthesize("你好世界")
            .await
            .expect("warm speech request");
        let elapsed = start.elapsed();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert!(
            wav[44..]
                .chunks_exact(2)
                .any(|pair| { i16::from_le_bytes([pair[0], pair[1]]).unsigned_abs() > 200 })
        );
        assert_eq!(speech.neural_tts_status(), "ready");
        durations.push(elapsed.as_secs_f64());
        eprintln!(
            "warm TTS request {}: {:.3}s",
            index + 1,
            elapsed.as_secs_f64()
        );
    }
    durations.sort_by(f64::total_cmp);
    eprintln!(
        "warm TTS latency p50 {:.3}s, p95 {:.3}s",
        durations[9], durations[18]
    );
}

#[tokio::test]
#[ignore = "requires both neural checkpoints and verified Mandarin transcription accuracy"]
async fn twenty_warm_stt_requests_preserve_reference_transcription() {
    let speech = Speech::new().expect("speech runtime configuration");
    assert!(speech.qwen_tts_checkpoint_present());
    assert!(speech.candle_stt_checkpoint_present());
    speech
        .initialize_neural_engines()
        .await
        .expect("model warm-up");
    let wav = normalize_reference_wav_for_stt(
        &speech
            .synthesize("你好世界")
            .await
            .expect("reference speech"),
    );
    let mut durations = Vec::with_capacity(20);
    let mut first_transcript: Option<String> = None;
    for index in 0..20 {
        let start = Instant::now();
        let transcript = speech
            .transcribe_audio(wav.clone(), InputFormat::Wav)
            .await
            .expect("recognize reference speech");
        let elapsed = start.elapsed();
        assert!(
            transcript_matches_reference_greeting(&transcript),
            "{transcript}"
        );
        if let Some(first) = &first_transcript {
            assert_eq!(
                &transcript, first,
                "STT changed across identical reference requests"
            );
        } else {
            eprintln!("STT reference transcript: {transcript}");
            first_transcript = Some(transcript.clone());
        }
        assert_eq!(speech.neural_stt_status(), "ready");
        durations.push(elapsed.as_secs_f64());
        eprintln!(
            "warm STT request {}: {:.3}s",
            index + 1,
            elapsed.as_secs_f64()
        );
    }
    durations.sort_by(f64::total_cmp);
    eprintln!(
        "warm STT latency p50 {:.3}s, p95 {:.3}s",
        durations[9], durations[18]
    );
}

#[tokio::test]
#[ignore = "requires both neural checkpoints and CUDA; exercises concurrent independent workers"]
async fn concurrent_tts_and_stt_complete_without_backend_contention() {
    let speech = Speech::new().expect("speech runtime configuration");
    speech
        .initialize_neural_engines()
        .await
        .expect("model warm-up");
    let reference = normalize_reference_wav_for_stt(
        &speech
            .synthesize("你好世界")
            .await
            .expect("reference speech"),
    );
    let (tts, stt) = tokio::join!(
        speech.synthesize("今天天气晴朗。"),
        speech.transcribe_audio(reference, InputFormat::Wav),
    );
    let wav = tts.expect("concurrent TTS");
    let transcript = stt.expect("concurrent STT");
    assert!(wav.len() > 44);
    assert!(
        transcript_matches_reference_greeting(&transcript),
        "{transcript}"
    );
    assert_eq!(speech.neural_tts_status(), "ready");
    assert_eq!(speech.neural_stt_status(), "ready");
}
