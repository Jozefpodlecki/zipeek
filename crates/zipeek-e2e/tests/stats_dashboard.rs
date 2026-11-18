use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn stats_route_and_sections(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/stats").await?;
    let page = driver.query_single("[data-stats-page]").await?;
    zipeek_e2e::e2e_ensure!(page.attr("data-stats-page").await?.is_some());
    let notice = driver.query_single("[data-stats-unavailable]").await?;
    zipeek_e2e::e2e_ensure!(
        notice
            .text()
            .await?
            .contains("Learning evidence not connected")
    );
    driver.query_single("[data-stats-preview-toggle]").await?.click().await?;
    driver.query_single("[data-stats-loading]").await?;
    driver.query_single("[data-stats-demo]").await?;
    driver.query_single("[data-stats-activity-chart]").await?;
    driver.query_single("[data-stats-tab='hanzi']").await?.click().await?;
    driver.query_single("[data-stats-vocabulary-explorer]").await?;
    driver.query_single("[role='checkbox'][aria-label='Select visible vocabulary']").await?.click().await?;
    driver.query_single("[data-stats-vocabulary-explorer] button[aria-label='Next vocabulary page']").await?.click().await?;
    driver.query_single("[data-stats-vocabulary-explorer] button[aria-label='Previous vocabulary page']").await?.click().await?;
    driver.query_single("[data-stats-vocabulary-explorer] button").await?;
    for tab in ["overview", "hanzi", "grammar", "skills", "history", "insights", "settings"] {
        driver.query_single(&format!("[data-stats-tab='{tab}']")).await?.click().await?;
        let panel = driver.query_single("[data-stats-panel]").await?;
        zipeek_e2e::e2e_ensure_eq!(panel.attr("data-stats-panel").await?, Some(tab.to_owned()));
        zipeek_e2e::e2e_ensure!(!panel.text().await?.is_empty());
        let loaded_selector = match tab {
            "overview" => "[data-stats-activity-chart]",
            "hanzi" => "[data-stats-vocabulary-explorer]",
            "grammar" => "[data-stats-grammar]",
            "skills" => "[data-stats-skills]",
            "history" => "[data-stats-history]",
            "insights" => "[data-stats-insights]",
            "settings" => "[data-stats-settings]",
            _ => unreachable!(),
        };
        driver.query_single(loaded_selector).await?;
        if tab == "grammar" {
            driver.query_single("[data-stats-grammar]").await?;
        }
        if tab == "history" {
            driver.query_single("[data-stats-history]").await?;
            driver.query_single("[data-stats-history-activity='province-quest']").await?;
            driver.query_single("[data-stats-history] button[aria-label='Next history page']").await?.click().await?;
            driver.query_single("[data-stats-history] button[aria-label='Previous history page']").await?.click().await?;
            driver.query_single("[data-stats-history] input[aria-label='Search history']").await?;
            driver.query_single("[data-stats-history] button[aria-pressed='true']").await?;
        }
        if tab == "settings" {
            driver.query_single("[data-stats-settings] [role='switch']").await?;
        }
    }
    driver.query_single("[data-stats-preview-toggle]").await?.click().await?;
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
