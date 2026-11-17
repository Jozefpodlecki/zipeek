use crate::{HarnessResult, ScenarioContext};
use serde_json::json;
use std::io::Write;

pub async fn capture_journey_step(
    ctx: &ScenarioContext,
    label: &str,
    seed: u64,
    variant: &str,
) -> HarnessResult<()> {
    if label.is_empty() || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!("Invalid journey checkpoint: {label}").into());
    }
    let folder = ctx.artifacts.join("full-run-screenshots");
    std::fs::create_dir_all(&folder)?;
    let filename = format!("{variant}-{label}.png");
    ctx.driver.screenshot(&folder.join(&filename)).await?;
    let record = json!({
        "variant": variant,
        "seed": seed,
        "step": label,
        "screenshot": filename,
        "url": ctx.driver.current_url().await?.to_string(),
    });
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(folder.join("manifest.jsonl"))?,
        "{record}"
    )?;
    Ok(())
}
