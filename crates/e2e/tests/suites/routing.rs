use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn unknown_route_recovers_while_dictionary_fails(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.block_urls(&["*hanzi.duckdb"]).await?;
    let url = ctx.url("/#/missing-page");
    driver.goto(&url).await?;
    driver.refresh().await?;
    driver
        .query(By::Css("[data-route-placeholder] h1"))
        .with_text("Page not found")
        .single()
        .await?;
    assert_eq!(driver.current_url().await?.as_str(), url);
    assert_eq!(driver.title().await?, "Page not found · Zipeek");
    driver
        .find(By::LinkText("Back to home"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-database-state='failed']"))
        .single()
        .await?;
    ctx.block_urls(&[]).await?;
    driver
        .find(By::Css("[data-database-retry]"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("[data-home]")).single().await?;
    assert_eq!(driver.title().await?, "Home · Zipeek");
    Ok(())
}

pub async fn practice_query_decoding_preserves_shared_url(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    let shared = ctx.url("/#/practice/eng-pinyin?mode=fixed&rounds=5&seed=1&hsk=2.0&level=7&pos=verb%2Cnoun%2Cunknown&offset=40");
    driver.goto(&shared).await?;
    let practice = driver.query(By::Css("[data-practice-variant='eng-pinyin'][data-practice-mode='fixed'][data-practice-level='1'][data-practice-offset='40'][data-practice-pos='noun,verb']")).single().await?;
    assert!(practice.is_displayed().await?);
    assert_eq!(driver.current_url().await?.as_str(), shared);
    driver
        .goto(ctx.url("/#/practice/pinyin-hanzi?mode=oops&offset=-1&pos=unknown"))
        .await?;
    driver.query(By::Css("[data-practice-variant='pinyin-hanzi'][data-practice-mode='endless'][data-practice-offset='0'][data-practice-pos='']")).single().await?;
    Ok(())
}
