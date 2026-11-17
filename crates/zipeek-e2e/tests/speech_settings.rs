use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn browser_speech_configuration(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings?section=speech").await?;
    let panel = driver.query_single("[data-settings-speech]").await?;
    let text = panel.text().await?;
    zipeek_e2e::e2e_ensure!(text.contains("Browser speech synthesis"));
    zipeek_e2e::e2e_ensure!(text.contains("Browser speech recognition"));
    driver.query_single("[data-speech-test-tts]").await?;
    driver.query_single("[data-speech-test-stt]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-speech-provider-select]")
            .await?
            .is_empty(),
        "Browser mode must not expose provider selection"
    );
    Ok(())
}

#[tokio::test]
async fn browser_speech_settings_only_offer_working_providers() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("speech_settings", browser_speech_configuration)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
