use std::path::PathBuf;

use zipeek_tts_stt::speech::Speech;

#[test]
fn missing_model_files_are_not_reported_as_available() {
    let missing = PathBuf::from("zipeek-test-checkpoint-does-not-exist/model.safetensors");
    let speech = Speech::with_paths(
        PathBuf::from("zipeek-test-espeak-does-not-exist"),
        PathBuf::from("zipeek-test-resources-do-not-exist"),
        missing,
    );
    assert!(!speech.synthesis_available());
    assert!(!speech.candle_stt_checkpoint_present());
}

#[test]
#[ignore = "requires downloaded Qwen3-ASR checkpoint; probes real decoder and encoder safetensors tensors"]
fn verifies_real_qwen_checkpoint_mappings() {
    let checkpoint = std::env::var_os("ZIPEEK_CANDLE_STT_MODEL")
        .expect("set ZIPEEK_CANDLE_STT_MODEL to local Qwen3-ASR model.safetensors");
    let speech = Speech::with_paths(
        PathBuf::from("zipeek-test-no-espeak"),
        PathBuf::from("zipeek-test-no-data"),
        PathBuf::from(checkpoint),
    );
    assert!(speech.candle_stt_checkpoint_present());
    speech
        .probe_candle_stt_weights(&candle_core::Device::Cpu)
        .expect("actual Qwen3-ASR checkpoint tensor mapping");
}
