use std::{path::PathBuf, process::Command};

#[test]
fn executable_rejects_missing_gguf_instead_of_starting_fixture() {
    let exe = env!("CARGO_BIN_EXE_zipeek-ai-server");
    let missing =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/nonexistent-zipeek-test.gguf");
    let status = Command::new(exe)
        .args(["--bind", "127.0.0.1:1430"])
        .env("ZIPEEK_AI_MODEL_PATH", missing)
        .status()
        .expect("launch AI service");
    assert!(!status.success());
}
