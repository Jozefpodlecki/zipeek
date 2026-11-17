use zipeek_e2e::ChromeDevTools;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn dictionary_failure_retry_and_real_read_only_queries(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    ctx.checkpoint("database-initial-state").await?;
    ChromeDevTools::new(&driver).network_enabled().await?;
    let blocked = driver
        .execute(zipeek_e2e::js_scripts::platform::DATABASE_DICTIONARY_FAILURE_RETRY_AND_REAL_READ_ONLY_QUERIES_01, vec![])
        .await?;
    ChromeDevTools::new(&driver)
        .blocked_urls(blocked.json())
        .await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query_single("[data-database-state='failed']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-home]").await?.is_empty(),
        "The application must not display the home page while the dictionary has failed"
    );
    let failure_message = driver
        .find_css("[data-database-error-message]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !failure_message.trim().is_empty(),
        "Database failures must explain what failed"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-database-retry]")
            .await?
            .is_enabled()
            .await?,
        "Database failure must offer an enabled retry action"
    );
    ctx.checkpoint_visible(
        "database-download-failed",
        "[data-database-state=\"failed\"]",
    )
    .await?;
    super::bootstrap::capture_database_boundary(&ctx, "failed").await?;
    let clear = driver.execute(zipeek_e2e::js_scripts::platform::DATABASE_DICTIONARY_FAILURE_RETRY_AND_REAL_READ_ONLY_QUERIES_02, vec![]).await?;
    ChromeDevTools::new(&driver)
        .blocked_urls(clear.json())
        .await?;
    driver
        .find_css("[data-database-retry]")
        .await?
        .click()
        .await?;
    zipeek_e2e::page_objects::shell::ready(&driver).await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::dictionary_row_count(&driver).await? > 0,
        "Rust-owned initialization must expose verified dictionary row metadata after retry"
    );
    ctx.checkpoint_visible("database-retry-completed", "[data-database-state='ready']")
        .await?;
    ctx.checkpoint("database-verified-result").await?;
    Ok(())
}

pub async fn adapter_reads_all_rows_and_rejects_writes(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .goto(format!("{}/__e2e/adapter.html", ctx.adapter_url))
        .await?;
    let probe = driver
        .execute_async(
            include_str!("../../../fixtures/browser_probes/database.js"),
            vec![],
        )
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(zipeek_e2e::js_scripts::platform::DATABASE_ADAPTER_READS_ALL_ROWS_AND_REJECTS_WRITES_01, vec![probe.json().clone()])
            .await?
            .convert::<bool>()?,
        "{probe:?}"
    );
    ctx.checkpoint("database-read-only-confirmed").await?;
    Ok(())
}
