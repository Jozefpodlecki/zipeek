use zipeek_e2e::{HarnessResult, ScenarioContext, page_objects::WebDriverActions};

pub async fn validation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for path in [
        "/games/classifier-market",
        "/games/listening-navigator",
        "/games/sentence-relay",
    ] {
        let invalid_hsk = zipeek_e2e::AppRoute::new(path)?
            .with_query("source", "hsk")?
            .with_query("standard", "2")?
            .with_query("level", "7")?;
        ctx.open(invalid_hsk).await?;
        let error = driver.query_single("[role='alert']").await?.text().await?;
        zipeek_e2e::e2e_ensure!(
            error.contains("Could not load vocabulary"),
            "Invalid HSK source must be rejected: {error}"
        );

        ctx.open_route(&format!("{path}?source=set&id=missing"))
            .await?;
        let missing = driver.query_single("[role='alert']").await?.text().await?;
        zipeek_e2e::e2e_ensure!(
            missing.contains("Word Set no longer exists"),
            "Missing set must not fall back to starter: {missing}"
        );

        ctx.open_route(path).await?;
        driver
            .query_single("[aria-label='Game vocabulary']")
            .await?;
        driver.query_single("[data-game-start]").await?;
        let source = driver
            .query_single("[data-game-source]")
            .await?
            .text()
            .await?;
        zipeek_e2e::e2e_ensure_eq!(source, "Starter questions");
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-game-complete]")
                .await?
                .is_empty()
        );
    }
    ctx.checkpoint("mini-game-vocabulary-validation").await?;
    Ok(())
}
