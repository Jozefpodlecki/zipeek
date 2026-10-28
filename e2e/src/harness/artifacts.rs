use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{Harness, HarnessError, HarnessResult, ScenarioContext};

pub fn setup_failure(directory: &std::path::Path, name: &str, error: &HarnessError) {
    eprintln!("Harness setup failed: {error}");
    if let Err(write_error) = std::fs::write(directory.join(name), error.to_string()) {
        eprintln!("Setup diagnostic write failed: {write_error}");
    }
}

pub fn run_directory(suite: &str) -> HarnessResult<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = Harness::workspace()
        .join("target/e2e")
        .join(format!("{suite}-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub async fn diagnostics(context: &ScenarioContext) {
    match context.driver.execute("return JSON.stringify({url: location.href, console: window.e2eDiagnostics || [], resources: performance.getEntriesByType('resource').map(r => ({url: r.name, duration: r.duration, bytes: r.transferSize}))}, null, 2);", vec![]).await {
        Ok(result) => match result.convert::<String>() {
            Ok(text) => {
                if let Err(error) = std::fs::write(context.artifacts.join("browser.json"), text) {
                    eprintln!("Diagnostic write failed: {error}");
                }
            }
            Err(error) => eprintln!("Diagnostic conversion failed: {error}"),
        },
        Err(error) => eprintln!("Diagnostic capture failed: {error}"),
    }
}

pub async fn failure(context: &ScenarioContext, message: &str) {
    eprintln!("FAIL {}: {message}", context.artifacts.display());
    use std::io::Write;
    let write = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(context.artifacts.join("failure.txt"))
        .and_then(|mut file| writeln!(file, "{message}"));
    if let Err(error) = write {
        eprintln!("Failure log write failed: {error}");
    }
    if let Err(error) = context
        .driver
        .screenshot(&context.artifacts.join("failure.png"))
        .await
    {
        eprintln!("Failure screenshot unavailable: {error}");
    }
}
