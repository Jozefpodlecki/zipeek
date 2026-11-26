use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn setup_and_options(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/time-weaver").await?;
    let pinyin = driver
        .query_single("[data-time-weaver-pinyin-setting]")
        .await?;
    let audio = driver
        .query_single("[data-time-weaver-audio-setting]")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(pinyin.attr("aria-checked").await?.as_deref(), Some("true"));
    zipeek_e2e::e2e_ensure_eq!(audio.attr("aria-checked").await?.as_deref(), Some("false"));
    pinyin.click().await?;
    audio.click().await?;
    zipeek_e2e::e2e_ensure_eq!(pinyin.attr("aria-checked").await?.as_deref(), Some("false"));
    zipeek_e2e::e2e_ensure_eq!(audio.attr("aria-checked").await?.as_deref(), Some("true"));
    let preview = driver
        .query_single("[data-time-weaver-preview]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !preview.contains("xīng qī yī"),
        "Preview hides reading when pinyin is off"
    );
    driver
        .query_single("[data-time-weaver-count='10']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-time-weaver-size='compact']")
        .await?
        .click()
        .await?;
    ctx.open_route("/games/time-weaver").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-time-weaver-count='10']")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-time-weaver-size='compact']")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-time-weaver-pinyin-setting]")
            .await?
            .attr("aria-checked")
            .await?
            .as_deref(),
        Some("false")
    );
    driver
        .query_single("[data-time-weaver-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-time-weaver-game]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-time-weaver-progress]")
            .await?
            .text()
            .await?
            .contains("of 10")
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-time-weaver-question] button[lang='zh-Hans'] span[lang='en']")
            .await?
            .is_empty(),
        "Pinyin-disabled sessions must not render readings below the Hanzi"
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-time-weaver-progress]")
            .await?
            .text()
            .await?
            .is_empty(),
        "The active game must display question progress"
    );
    ctx.checkpoint("time-weaver-options-no-pinyin").await?;
    Ok(())
}

async fn answer_feedback_and_controls(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/time-weaver").await?;
    driver
        .query_single("[data-time-weaver-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-time-weaver-game]").await?;
    let tiles = driver
        .find_all_css("[data-time-weaver-build] [data-time-weaver-tile]")
        .await?;
    if tiles.is_empty() {
        let choices = driver
            .find_all_css("[data-time-weaver-choices] button")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(choices.len(), 3);
        choices[0].click().await?;
    } else {
        zipeek_e2e::e2e_ensure!(tiles.len() > 2);
        for index in 0..tiles.len() {
            driver
                .query_single(&format!("[data-time-weaver-tile='{index}']"))
                .await?
                .click()
                .await?;
        }
        driver
            .query_single("[data-time-weaver-submit]")
            .await?
            .click()
            .await?;
    }
    let feedback = driver
        .query_single("[data-time-weaver-review]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        feedback.contains("Correct") || feedback.contains("Incorrect"),
        "Feedback must clearly identify the outcome: {feedback}"
    );
    driver
        .query_single("[data-time-weaver-next]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-time-weaver-game]").await?;
    ctx.checkpoint("time-weaver-feedback-and-continue").await?;
    Ok(())
}

#[tokio::test]
async fn time_weaver_setup_pinyin_and_feedback() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = async {
        harness
            .run("time_weaver_setup_options", setup_and_options)
            .await?;
        harness
            .run("time_weaver_result_feedback", answer_feedback_and_controls)
            .await
    }
    .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
