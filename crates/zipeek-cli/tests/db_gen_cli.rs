use std::process::Command;

#[test]
fn generator_cli_requires_a_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let result = Command::new(env!("CARGO_BIN_EXE_zipeek-cli"))
        .args(["db-gen"])
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Usage:"));
    Ok(())
}

#[test]
fn generator_cli_rejects_unrecognized_flags() -> Result<(), Box<dyn std::error::Error>> {
    let result = Command::new(env!("CARGO_BIN_EXE_zipeek-cli"))
        .args(["db-gen", "--unknown"])
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--unknown"));
    Ok(())
}

#[test]
fn generator_cli_rejects_existing_output() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::temp_dir().join(format!(
        "zipeek-cli-existing-artifact-{}", std::process::id()
    ));
    std::fs::create_dir_all(&output)?;
    let result = Command::new(env!("CARGO_BIN_EXE_zipeek-cli"))
        .arg("db-gen")
        .arg("generate")
        .arg("--output")
        .arg(&output)
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("already exists"));
    std::fs::remove_dir(&output)?;
    Ok(())
}

#[test]
fn generator_cli_rejects_invalid_logging_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let result = Command::new(env!("CARGO_BIN_EXE_zipeek-cli"))
        .args(["db-gen", "check"])
        .env("RUST_LOG", "[")
        .output()?;
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Invalid RUST_LOG filter"));
    Ok(())
}
