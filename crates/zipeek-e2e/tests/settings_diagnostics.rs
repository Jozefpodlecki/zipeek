use thirtyfour::prelude::*;
use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, selectors::settings as selectors},
};

async fn font_and_tile_size_persist(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    driver.element(selectors::FONT_OPTIONS).await?;
    zipeek_e2e::page_objects::settings::choose_font(driver, "lxgw-wenkai").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .execute(
                zipeek_e2e::js_scripts::settings::SETTINGS_DIAGNOSTICS_FONT_AND_TILE_SIZE_PERSIST_01,
                vec![]
            )
            .await?
            .convert::<String>()?,
        "lxgw-wenkai"
    );
    zipeek_e2e::page_objects::settings::choose_character_size(driver, "extra-large").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .execute(
                zipeek_e2e::js_scripts::settings::SETTINGS_DIAGNOSTICS_FONT_AND_TILE_SIZE_PERSIST_02,
                vec![]
            )
            .await?
            .convert::<String>()?,
        "extra-large"
    );
    driver.refresh().await?;
    driver.element(selectors::FONT).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .element(selectors::FONT)
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .element(selectors::EXTRA_LARGE)
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    driver.click(selectors::LARGE).await?;
    driver.element(selectors::LARGE_SELECTED).await?;
    driver.refresh().await?;
    driver.element(selectors::LARGE_SELECTED).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css(selectors::EXTRA_LARGE)
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("false"),
        "Choosing Large must deselect Extra Large even after reload"
    );
    ctx.checkpoint("font-preference-and-size-persisted").await?;
    Ok(())
}

async fn debug_reports_real_database_metadata(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/debug").await?;
    let diagnostics = driver.element(selectors::DIAGNOSTICS).await?;
    zipeek_e2e::e2e_ensure!(
        diagnostics
            .text()
            .await?
            .contains("Loaded dictionary storage")
    );
    driver.element(selectors::STATISTICS).await?;
    driver
        .query(By::XPath(
            "//section[@data-duckdb-diagnostics]//dt[normalize-space()='Block size (bytes)']",
        ))
        .single()
        .await?;
    let text = diagnostics.text().await?;
    zipeek_e2e::e2e_ensure!(text.contains("Storage compatibility"));
    zipeek_e2e::e2e_ensure!(
        !text.contains("Not recorded in manifest"),
        "Installed manifest lacks DuckDB version metadata"
    );
    zipeek_e2e::e2e_ensure!(
        !text.contains("DuckDB storage metadata unavailable"),
        "Browser must report real DuckDB block statistics: {text}"
    );
    ctx.checkpoint("duckdb-storage-diagnostics").await?;
    Ok(())
}

#[tokio::test]
async fn settings_and_diagnostics() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("font_and_tile_size_persist", font_and_tile_size_persist)
        .await?;
    harness
        .run(
            "debug_database_metadata",
            debug_reports_real_database_metadata,
        )
        .await?;
    harness.finish().await
}
