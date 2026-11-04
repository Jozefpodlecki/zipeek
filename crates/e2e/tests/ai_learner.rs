use e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, e2e_ensure};
use e2e::learner::TranslationPage;
use thirtyfour::prelude::*;

async fn live_learner(context: ScenarioContext) -> HarnessResult<()> {
    let endpoint = std::env::var("ZIPEEK_E2E_AI_ENDPOINT")
        .map_err(|_| "Set ZIPEEK_E2E_AI_ENDPOINT to your LM Studio /v1/ base URL")?;
    let model = std::env::var("ZIPEEK_E2E_AI_MODEL")
        .map_err(|_| "Set ZIPEEK_E2E_AI_MODEL to the loaded LM Studio model ID")?;
    let page = TranslationPage::new(&context);
    page.open("starter:everyday").await?;
    page.configure_endpoint(&model, &endpoint).await?;
    page.generate().await?;
    let observation = page.observe().await?;

    let script = context.artifacts.join("learner-observation.txt");
    let visible = observation.available_tiles.iter()
        .map(|tile| format!("{}: {}", tile.index, tile.text))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&script, format!("source: {}\ntarget: {}\ntiles:\n{visible}", observation.source, observation.target_language))?;
    let output = tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new("node");
        command
            .arg(e2e::Harness::workspace().join("e2e/fixtures/ai-learner.cjs"))
            .args([endpoint, model, observation.target_language, observation.source]);
        for tile in observation.available_tiles {
            command.arg(tile.index.to_string()).arg(tile.text);
        }
        command.output()
    }).await??;
    if !output.status.success() {
        return Err(format!("AI learner decision failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    std::fs::write(context.artifacts.join("learner-decision.txt"), &text)?;
    let indices = text.trim().split(',').map(str::parse::<usize>).collect::<Result<Vec<_>,_>>()
        .map_err(|error| format!("Invalid learner index: {error}"))?;
    for index in indices {
        let tile = context.driver.find(By::Css(format!("[data-translation-word-bank] [data-bank-tile='{index}']"))).await?;
        tile.click().await?;
    }
    let verdict = page.check().await?;
    e2e_ensure!(verdict == "correct" || verdict == "incorrect", "verdict: {verdict}");
    let stats = context.driver.find(By::Css("[data-component='translation-session-stats']")).await?.text().await?;
    e2e_ensure!(stats.contains("Completed: 1"), "stats: {stats}");
    Ok(())
}

#[tokio::test]
#[ignore = "Requires an explicit running LM Studio model; not part of deterministic CI"]
async fn ai_completes_translation_through_browser() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness.run("live_ai_translation", live_learner).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
