use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn stats_route_and_sections(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/stats").await?;
    let page = driver.query_single("[data-stats-page]").await?;
    zipeek_e2e::e2e_ensure!(page.text().await?.contains("Learning statistics"));
    let notice = driver.query_single("[data-stats-unavailable]").await?;
    zipeek_e2e::e2e_ensure!(
        notice
            .text()
            .await?
            .contains("Learning evidence not connected")
    );
    for tab in ["overview", "hanzi", "skills", "history", "insights"] {
        driver
            .query_single(&format!("[data-stats-tab='{tab}']"))
            .await?
            .click()
            .await?;
        let panel = driver.query_single("[data-stats-panel]").await?;
        zipeek_e2e::e2e_ensure_eq!(panel.attr("data-stats-panel").await?, Some(tab.to_owned()));
        zipeek_e2e::e2e_ensure!(panel.text().await?.contains("Not assessed"));
    }
    driver
        .query_single("[data-stats-unavailable] a[href*='/settings']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-component='main-content']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .to_string()
            .contains("/settings")
    );
    driver
        .query_single("[data-settings-tab-button='learning']")
        .await?
        .click()
        .await?;
    let diagnostics = driver.query_single("[data-settings-learning]").await?;
    zipeek_e2e::e2e_ensure!(diagnostics.text().await?.contains("Learning data"));
    driver
        .query_single("[data-learning-technical]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-learning-worker-check]")
        .await?
        .click()
        .await?;

    let status = driver.query_single("[data-learning-worker-status]").await?;
    zipeek_e2e::e2e_ensure!(!status.text().await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn stats_dashboard_does_not_fabricate_learning_progress() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("stats_dashboard", stats_route_and_sections)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
