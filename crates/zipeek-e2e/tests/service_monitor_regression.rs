use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{
        WebDriverActions,
        settings::{self, Section},
    },
};

async fn browser_exposes_shared_service_status_without_process_controls(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    for (section, service) in [(Section::Ai, "ai"), (Section::Images, "image")] {
        settings::select_section(driver, section).await?;
        let panel = driver
            .query_single(&format!("[data-native-service='{service}']"))
            .await?;
        zipeek_e2e::e2e_ensure!(panel.is_displayed().await?);
        zipeek_e2e::e2e_ensure_eq!(
            panel.find_all(thirtyfour::By::Css("button")).await?.len(),
            1,
            "Browser-mode service panel should expose Refresh but no process controls",
        );
        let refresh = panel.find(thirtyfour::By::Css("button")).await?;
        zipeek_e2e::e2e_ensure_eq!(refresh.text().await?, "Refresh");
        refresh.click().await?;
        driver
            .query_single(&format!(
                "[data-native-service='{service}'] [role='status']"
            ))
            .await?;
        ctx.checkpoint(&format!("browser-{service}-service-status"))
            .await?;
    }
    Ok(())
}

async fn grammar_does_not_offer_inline_ai_generation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/grammar").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-grammar-ai-example]")
            .await?
            .is_empty(),
        "The retired inline Grammar player must not offer its own AI generator",
    );
    driver
        .query_single("[data-course-details] a")
        .await?
        .click()
        .await?;
    driver.query_single("[data-grammar-lesson]").await?;
    ctx.checkpoint("grammar-authored-single-lesson").await?;
    Ok(())
}

#[tokio::test]
async fn service_monitor_regression() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let status = harness
        .run(
            "browser_service_status",
            browser_exposes_shared_service_status_without_process_controls,
        )
        .await;
    let grammar = harness
        .run(
            "grammar_no_inline_ai",
            grammar_does_not_offer_inline_ai_generation,
        )
        .await;
    let cleanup = harness.finish().await;
    status.and(grammar).and(cleanup)
}
