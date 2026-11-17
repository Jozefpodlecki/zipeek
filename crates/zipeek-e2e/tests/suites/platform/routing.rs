use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn unknown_route_recovers_while_dictionary_fails(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.block_urls(&["*hanzi*.duckdb*"]).await?;
    ctx.checkpoint("routing-initial-state").await?;
    let url = ctx.url("/#/missing-page");
    driver.goto(&url).await?;
    driver.refresh().await?;
    driver
        .query_with_text("[data-route-placeholder] h1", "Page not found")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), url);
    zipeek_e2e::e2e_ensure_eq!(driver.title().await?, "Page not found · Zipeek");
    driver
        .find(By::LinkText("Back to home"))
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-database-state='failed']")
        .await?;
    ctx.block_urls(&[]).await?;
    driver
        .find_css("[data-database-retry]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-home]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-database-state='failed']")
            .await?
            .is_empty(),
        "A successful route recovery cannot retain the failed dictionary state"
    );
    ctx.checkpoint_visible("route-recovered-home", "[data-home]")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.title().await?, "Home · Zipeek");
    ctx.checkpoint_visible("routing-home-still-present", "[data-home]")
        .await?;
    ctx.checkpoint("routing-verified-result").await?;
    Ok(())
}

pub async fn session_only_practice_routes_reject_retired_urls(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for retired in [
        "/practice/eng-pinyin?offset=8",
        "/practice/pinyin-hanzi",
        "/practice/mcq/meaning-to-hanzi",
        "/practice/character-hunt",
        "/practice/image-vocabulary",
    ] {
        let url = ctx.url(&format!("/#{retired}"));
        driver.goto(&url).await?;
        driver
            .query_with_text("[data-route-placeholder] h1", "Page not found")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), url);
    }
    ctx.open_route("/practice").await?;
    driver.query_single("[data-practice-path='quick']").await?;
    driver.query_single("[data-component='app-header']").await?;
    ctx.checkpoint("practice-session-only-routes").await?;
    Ok(())
}
