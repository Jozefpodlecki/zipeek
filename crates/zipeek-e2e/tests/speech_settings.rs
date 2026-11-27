use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn browser_speech_configuration(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings?section=speech").await?;
    let panel = driver.query_single("[data-settings-speech]").await?;
    let text = panel.text().await?;
    zipeek_e2e::e2e_ensure!(text.contains("Mandarin speech"));
    zipeek_e2e::e2e_ensure!(text.contains("Browser speech"));
    zipeek_e2e::e2e_ensure!(text.contains("Local Mandarin speech"));
    driver
        .query_single("[data-speech-provider='browser']")
        .await?;
    driver
        .query_single("[data-speech-provider='local']")
        .await?;
    driver.query_single("[data-speech-test-tts] svg").await?;
    driver.query_single("[data-speech-test-stt] svg").await?;
    zipeek_e2e::e2e_ensure!(text.contains("今天天气晴朗。"));
    zipeek_e2e::e2e_ensure!(text.contains("Play sample"));
    zipeek_e2e::e2e_ensure!(text.contains("Start speaking"));
    zipeek_e2e::e2e_ensure!(!text.contains("Test microphone"));
    zipeek_e2e::e2e_ensure!(!text.contains("Local Mandarin WAV played"));
    zipeek_e2e::e2e_ensure!(!text.contains("voice available, microphone available"));
    driver.query_single("[data-speech-tts-status]").await?;
    driver.query_single("[data-speech-stt-status]").await?;
    driver
        .query_single("[data-speech-test-stt]:not([disabled])")
        .await?;
    zipeek_e2e::e2e_ensure!(!text.contains("JsValue("));
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
