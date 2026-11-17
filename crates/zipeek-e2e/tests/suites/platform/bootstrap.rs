use zipeek_e2e::ChromeDevTools;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn mounted_app_waits_for_dictionary_download(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    ctx.checkpoint("bootstrap-initial-state").await?;
    ctx.add_init_script(include_str!(
        "../../../fixtures/browser_probes/provider_lifetime.js"
    ))
    .await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query_single("[data-database-state='loading'] [role='status']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-home]").await?.is_empty(),
        "Home content must not appear until the dictionary download completes"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-theme-toggle]")
            .await?
            .is_displayed()
            .await?
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_MOUNTED_APP_WAITS_FOR_DICTIONARY_DOWNLOAD_01, vec![])
            .await?
            .convert::<u64>()?,
        0,
        "Rust should verify dictionary bytes before starting the DuckDB worker"
    );
    capture_database_boundary(&ctx, "loading").await?;
    driver
        .execute_async(
            include_str!("../../../fixtures/browser_probes/release_download.js"),
            vec![],
        )
        .await?;
    driver
        .query_single("[data-database-state='ready'] [data-home]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-welcome-enter]")
            .await?
            .is_empty(),
        "Dictionary recovery must not return to first-visit onboarding"
    );
    ctx.checkpoint_visible(
        "bootstrap-database-recovered",
        "[data-database-state='ready'] [data-home]",
    )
    .await?;
    ctx.checkpoint("bootstrap-verified-result").await?;
    Ok(())
}

pub async fn bootstrap_loader_is_centered_and_landscape_survives_both_themes(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    ctx.viewport().responsive(1280, 800, false).await?;
    let blocked = driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_01, vec![]).await?;
    ChromeDevTools::new(&driver).network_enabled().await?;
    ChromeDevTools::new(&driver)
        .blocked_urls(blocked.json())
        .await?;
    driver.goto(ctx.url("/")).await?;
    let artifacts = ctx.artifacts.clone();
    for (saved, name) in [(true, "dark"), (false, "light")] {
        zipeek_e2e::page_objects::browser::set_preference(
            &driver,
            "dark",
            serde_json::json!(saved),
        )
        .await?;
        driver.refresh().await?;
        let loader = driver
            .query_single("[data-loading='bootstrap'][role='status']")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(loader.text().await?, "Loading");
        zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_02, vec![]).await?.convert::<bool>()?);
        zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_03, vec![]).await?.convert::<bool>()?);
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_04,
                    vec![]
                )
                .await?
                .convert::<bool>()?,
            "HTML retains its default theme while WASM is blocked"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_05,
                    vec![]
                )
                .await?
                .convert::<bool>()?,
            "Preferences should only be restored by WASM"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_06,
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        driver
            .screenshot(&artifacts.join(format!("a30-bootstrap-{name}.png")))
            .await?;
    }
    ctx.viewport().responsive(360, 800, true).await?;
    zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_07, vec![]).await?.convert::<bool>()?);
    driver
        .screenshot(&artifacts.join("a30-bootstrap-mobile.png"))
        .await?;
    let reduced = driver
        .execute(
            zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_08,
            vec![],
        )
        .await?;
    ChromeDevTools::new(&driver).media(reduced.json()).await?;
    zipeek_e2e::e2e_ensure_eq!(driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_09, vec![]).await?.convert::<String>()?, "none");

    let clear = driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_10, vec![]).await?;
    ChromeDevTools::new(&driver)
        .blocked_urls(clear.json())
        .await?;
    driver.refresh().await?;
    zipeek_e2e::page_objects::shell::ready(&driver).await?;
    driver
        .query_single("html[data-appearance-ready='true']:not(.dark)")
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_11,
            vec![],
        )
        .await?;
    driver
        .find_css("[data-theme-toggle]")
        .await?
        .click()
        .await?;
    driver.query_single("html.dark").await?;
    zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::BOOTSTRAP_BOOTSTRAP_LOADER_IS_CENTERED_AND_LANDSCAPE_SURVIVES_BOTH_THEMES_12, vec![]).await?.convert::<bool>()?);
    ctx.open_route("/practice").await?;
    driver.query_single("[data-practice-path='quick']").await?;
    driver.query_single("[data-component='app-header']").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-component='practice-session-shell']")
            .await?
            .is_empty(),
        "Practice setup must not mount an exercise session without Start"
    );
    Ok(())
}

pub async fn capture_database_boundary(ctx: &ScenarioContext, state: &str) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let media = driver
        .execute(
            zipeek_e2e::js_scripts::platform::BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_01,
            vec![],
        )
        .await?;
    ChromeDevTools::new(&driver).media(media.json()).await?;
    for (dark, theme) in [(true, "dark"), (false, "light")] {
        let current = driver
            .execute(
                zipeek_e2e::js_scripts::platform::BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_02,
                vec![],
            )
            .await?
            .convert::<bool>()?;
        if current != dark {
            driver
                .find_css("[data-theme-toggle]")
                .await?
                .click()
                .await?;
        }
        for width in [400, 1280] {
            ctx.viewport().responsive(width, 800, width < 768).await?;
            driver
                .execute_async(
                    zipeek_e2e::js_scripts::platform::BOOTSTRAP_CAPTURE_DATABASE_BOUNDARY_03,
                    vec![],
                )
                .await?;
            zipeek_e2e::e2e_ensure!(
                zipeek_e2e::page_objects::browser::no_horizontal_overflow(driver).await?
            );
            driver
                .screenshot(&ctx.artifacts.join(format!("{state}-{theme}-{width}.png")))
                .await?;
        }
    }
    ctx.viewport().responsive(1280, 1000, false).await?;
    ctx.checkpoint("bootstrap-theme-recovered").await?;
    Ok(())
}
