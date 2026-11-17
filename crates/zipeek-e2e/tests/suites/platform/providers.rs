use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn navigation_and_rapid_theme_commands_preserve_dictionary(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.add_init_script(include_str!(
        "../../../fixtures/browser_probes/provider_lifetime.js"
    ))
    .await?;
    ctx.checkpoint("providers-initial-state").await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query_single("[data-database-state='loading']")
        .await?;
    driver.execute_async(zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_01, vec![]).await?;
    let workers_before_download = driver
        .execute(zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_02, vec![])
        .await?
        .convert::<u64>()?;
    zipeek_e2e::e2e_ensure_eq!(
        workers_before_download,
        0,
        "The dictionary worker should not be created until the download is verified"
    );
    for route in ["/practice", "/browse", "/"] {
        driver
            .find_css(format!("[data-navigation-link][href='#{route}']"))
            .await?
            .click()
            .await?;
        driver
            .query_single(format!(
                "[data-navigation-link][href='#{route}'][aria-current='page']"
            ))
            .await?;
        driver
            .query_single("[data-database-state='loading']")
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::preference(driver, "dark").await?
            == Some(serde_json::json!(true))
    );
    for _ in 0..2 {
        driver
            .query_single("[data-theme-toggle]")
            .await?
            .click()
            .await?;
    }
    driver
        .query_single("html.dark body.dark [data-theme-toggle][aria-checked='true']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::preference(driver, "dark").await?
            == Some(serde_json::json!(true))
    );
    driver
        .execute_async(
            include_str!("../../../fixtures/browser_probes/release_download.js"),
            vec![],
        )
        .await?;
    zipeek_e2e::page_objects::shell::ready(driver).await?;
    let workers = driver
        .execute(zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_03, vec![])
        .await?
        .convert::<u64>()?;
    zipeek_e2e::e2e_ensure_eq!(
        workers,
        1,
        "Exactly one DuckDB worker should be created after download"
    );
    driver
        .find_css("[data-navigation-link][href='#/browse']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-component='browse-page'] [data-browse-query]")
        .await?;
    driver.back().await?;
    driver.query_single("[data-home]").await?;
    driver.forward().await?;
    driver
        .query_single("[data-navigation-link][href='#/browse'][aria-current='page']")
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_04,
            vec![],
        )
        .await?;
    driver.query_single("[data-home]").await?;
    zipeek_e2e::page_objects::shell::ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .execute(zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_05, vec![])
            .await?
            .convert::<u64>()?,
        1
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .execute(zipeek_e2e::js_scripts::platform::PROVIDERS_NAVIGATION_AND_RAPID_THEME_COMMANDS_PRESERVE_DICTIONARY_06, vec![])
            .await?
            .convert::<u64>()?,
        workers
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-database-state='failed']")
            .await?
            .is_empty(),
        "Navigation and theme changes must not leave a failed database boundary"
    );
    ctx.checkpoint_visible(
        "providers-home-present",
        "[data-home], [data-component='browse-page']",
    )
    .await?;
    ctx.checkpoint("providers-verified-result").await?;
    Ok(())
}
