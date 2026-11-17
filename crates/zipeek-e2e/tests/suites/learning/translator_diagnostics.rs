use zipeek_e2e::page_objects::WebDriverActions;

use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn prepare(ctx: &ScenarioContext, scenario: &str) -> HarnessResult<()> {
    ctx.configure_ai_fixture(scenario).await?;
    let base = &ctx.ai_url;
    zipeek_e2e::page_objects::browser::set_local_storage(
        &ctx.driver,
        "zipeek.ai.base_url.v1",
        base,
    )
    .await?;
    zipeek_e2e::page_objects::browser::set_local_storage(
        &ctx.driver,
        "zipeek.ai.model.v1",
        "fixture-model",
    )
    .await?;
    ctx.open_route("/translate-sentence").await?;
    ctx.driver
        .query_single("[data-component='translate-sentence-page']")
        .await?;
    ctx.driver
        .find_css("[data-translation-source]")
        .await?
        .send_keys("Jane is delighted.")
        .await?;
    ctx.driver
        .find_css("[data-translate-submit]")
        .await?
        .click()
        .await?;
    ctx.driver
        .query_single("[data-translation-result] li")
        .await?;
    Ok(())
}

pub async fn structured_rejection_returns_plain_translation(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    prepare(&ctx, "translator-rejected-analysis").await?;
    ctx.checkpoint_visible("translated-result-visible", "[data-translation-result]")
        .await?;
    ctx.checkpoint("translator-diagnostics-initial-state")
        .await?;
    let result = ctx
        .driver
        .find_css("[data-translation-result]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        result.contains("简非常高兴。"),
        "Unexpected result: {result}"
    );
    let page = ctx
        .driver
        .find_css("[data-component='translate-sentence-page']")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        page.contains("Translation succeeded without detailed analysis"),
        "Missing graceful fallback warning: {page}"
    );
    zipeek_e2e::e2e_ensure!(
        page.contains("HTTP 400"),
        "Provider rejection was not diagnosed: {page}"
    );
    zipeek_e2e::e2e_ensure!(
        page.contains("context length exceeds fixture model limit"),
        "Provider reason was discarded: {page}"
    );
    zipeek_e2e::e2e_ensure!(
        ctx.driver
            .find_all_css("[data-sentence-analysis]")
            .await?
            .is_empty()
    );
    let rendered = ctx
        .driver
        .find_css("[data-translation-result]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !rendered.trim().is_empty(),
        "Fallback translation must contain visible text"
    );
    ctx.checkpoint_visible(
        "translation-result-after-fallback",
        "[data-translation-result]",
    )
    .await?;
    ctx.checkpoint("translator-diagnostics-verified-result")
        .await?;
    Ok(())
}

pub async fn structured_success_shows_analysis(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare(&ctx, "translator-valid-analysis").await?;
    let result = ctx
        .driver
        .find_css("[data-translation-result]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        result.contains("简非常高兴。"),
        "Unexpected result: {result}"
    );
    ctx.driver.query_single("[data-sentence-analysis]").await?;
    let page = ctx
        .driver
        .find_css("[data-component='translate-sentence-page']")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(!page.contains("Translation succeeded without detailed analysis"));
    Ok(())
}

pub async fn streaming_progress_visible_before_result(ctx: ScenarioContext) -> HarnessResult<()> {
    ctx.configure_ai_fixture("translator-streaming").await?;
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
    ctx.open_route("/translate-sentence").await?;
    ctx.driver
        .query_single("[data-translation-source]")
        .await?
        .send_keys("Jane is delighted.")
        .await?;
    ctx.driver
        .find_css("[data-translate-submit]")
        .await?
        .click()
        .await?;
    let progress = ctx.driver.query_single("[data-ai-stream-progress]").await?;
    let active = ctx.driver.execute_async(
        zipeek_e2e::js_scripts::learning::TRANSLATOR_DIAGNOSTICS_STREAMING_PROGRESS_VISIBLE_BEFORE_RESULT_01,
        vec![],
    ).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        active,
        "streamed reasoning must update progress while request is active"
    );
    zipeek_e2e::e2e_ensure!(
        progress.text().await?.contains("Model"),
        "progress must describe current model activity"
    );
    ctx.driver
        .query_single("[data-translation-result] li")
        .await?;
    let result = ctx
        .driver
        .find_css("[data-translation-result]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        result.contains("简非常高兴。"),
        "unexpected streamed translation: {result}"
    );
    zipeek_e2e::e2e_ensure!(
        ctx.driver
            .find_all_css("[data-ai-stream-progress]")
            .await?
            .is_empty()
    );
    Ok(())
}

pub async fn fixture_requests_are_observable(ctx: ScenarioContext) -> HarnessResult<()> {
    prepare(&ctx, "translator-valid-analysis").await?;
    let script = format!(
        "const done=arguments[arguments.length-1];fetch('{}__e2e/requests').then(r=>r.json()).then(r=>done(r.records.map(x=>x.kind))).catch(()=>done([]));",
        ctx.ai_url.replace("/v1/", "/")
    );
    let kinds = ctx
        .driver
        .execute_async(script, vec![])
        .await?
        .convert::<Vec<String>>()?;
    zipeek_e2e::e2e_ensure!(
        kinds.iter().any(|kind| kind == "translator-analysis"),
        "Expected a deterministic fixture-backed analysis request; observed {kinds:?}"
    );
    ctx.checkpoint("ai-fixture-backed-result").await?;
    Ok(())
}
