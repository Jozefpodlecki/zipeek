use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn dictionary_failure_retry_and_real_read_only_queries(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    driver
        .cdp()
        .send_raw(
            "Network.enable",
            std::collections::BTreeMap::<String, String>::new(),
        )
        .await?;
    let blocked = driver
        .execute("return {urls: ['*hanzi.duckdb']};", vec![])
        .await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", blocked.json())
        .await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query(By::Css("[data-database-state='failed']"))
        .single()
        .await?;
    assert!(driver.find_all(By::Css("[data-home]")).await?.is_empty());
    super::bootstrap::capture_database_boundary(&ctx, "failed").await?;
    let clear = driver.execute("return {urls: []};", vec![]).await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", clear.json())
        .await?;
    driver
        .find(By::Css("[data-database-retry]"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("[data-database-state='ready'][data-database-rows='12100'][data-database-engine='v1.4.3']")).single().await?;
    Ok(())
}

pub async fn adapter_reads_all_rows_and_rejects_writes(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .goto(format!("{}/__e2e/adapter.html", ctx.adapter_url))
        .await?;
    let probe = driver
        .execute_async(
            include_str!("../../fixtures/browser_probes/database.js"),
            vec![],
        )
        .await?;
    assert!(
        driver
            .execute("return arguments[0].ok;", vec![probe.json().clone()])
            .await?
            .convert::<bool>()?,
        "{probe:?}"
    );
    Ok(())
}
