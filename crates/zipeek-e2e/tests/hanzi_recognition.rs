use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn recognition_uses_explicit_submission_and_verified_results(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    ctx.configure_ai_fixture("success").await?;
    zipeek_e2e::page_objects::browser::set_local_storage(
        &ctx.driver,
        "zipeek.ai.base_url.v1",
        &ctx.ai_url,
    )
    .await?;
    zipeek_e2e::page_objects::browser::set_local_storage(
        &ctx.driver,
        "zipeek.ai.model.v1",
        "fixture-model",
    )
    .await?;
    ctx.open_route("/recognize-hanzi").await?;
    ctx.driver.query_single("[data-component='recognize-hanzi-page']").await?;
    ctx.driver.query_single("[data-recognition-layout]").await?;
    ctx.driver.query_single("[data-recognition-results]").await?;
    zipeek_e2e::e2e_ensure!(
        ctx.driver.find_all_css("[data-recognition-results] article").await?.is_empty(),
        "Recognition must not fabricate candidates before submission"
    );
    ctx.driver.find_css("[data-recognition-canvas]").await?.click().await?;
    ctx.driver
        .find_css("button[data-recognition-submit]")
        .await?
        .click()
        .await?;
    ctx.driver.query_single("[data-recognition-results] article").await?;
    ctx.assert_ai_request_kind("hanzi-recognition").await?;
    let page = ctx.driver.find_css("[data-recognition-results]").await?.text().await?;
    zipeek_e2e::e2e_ensure!(page.contains("汉"), "Missing fixture recognition result: {page}");
    zipeek_e2e::e2e_ensure!(page.contains("Dictionary verified"), "Candidate was not checked against local dictionary: {page}");
    ctx.driver
        .find_css("button[aria-label='Clear drawing or image']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        ctx.driver.find_all_css("[data-recognition-results] article").await?.is_empty(),
        "Clear must discard recognition results"
    );
    ctx.checkpoint("recognition-cleared").await?;
    Ok(())
}

#[tokio::test]
async fn hanzi_recognition() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("hanzi-recognition-explicit-submission", recognition_uses_explicit_submission_and_verified_results)
        .await?;
    harness.finish().await
}
