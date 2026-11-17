use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::page_objects::{WebElementQueryActions, selectors};
pub async fn first_visit_enters_app_before_database_readiness(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/")).await?;
    zipeek_e2e::page_objects::browser::remove_local_storage(driver, "zipeek.preferences.v1")
        .await?;
    zipeek_e2e::page_objects::browser::set_local_storage(
        driver,
        "zipeek.personal.learning_history.enabled.v1",
        "true",
    )
    .await?;
    driver.refresh().await?;
    let enter = driver.query_single("[data-welcome-enter]").await?;
    zipeek_e2e::e2e_ensure!(
        enter.is_displayed().await?,
        "First visit must display the entry action"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-database-state]")
            .await?
            .is_empty(),
        "Dictionary readiness cannot be assumed while landing is visible"
    );
    ctx.checkpoint_visible("landing-entry-present", "[data-welcome-enter]")
        .await?;
    enter.click().await?;
    driver.query_single("[data-onboarding-form]").await?;
    let consent = driver.query_single("[data-onboarding-recording]").await?;
    zipeek_e2e::e2e_ensure!(
        !consent.is_selected().await?,
        "Learning-history recording must start disabled even when an obsolete consent key says true",
    );
    let selected = driver
        .query_single("[data-onboarding-theme='jade']")
        .await?;
    selected.click().await?;
    driver
        .query_single("html[data-theme-preset='jade'] #app-background[src$='jade.webp']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::local_storage(driver, "zipeek.preferences.v1")
            .await?
            .is_none(),
        "Previewing a theme must not save onboarding choices",
    );
    driver
        .query_single("[data-onboarding-back]")
        .await?
        .click()
        .await?;
    driver
        .query_single("html[data-theme-preset='lavender'] [data-welcome-enter]")
        .await?;
    driver
        .query_single("[data-welcome-enter]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-onboarding-theme='jade']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-onboarding-finish]")
        .await?
        .click()
        .await?;
    let saved = zipeek_e2e::page_objects::browser::local_storage(driver, "zipeek.preferences.v1")
        .await?
        .ok_or("Onboarding did not save preferences")?;
    let preferences: serde_json::Value =
        serde_json::from_str(&saved).map_err(|error| error.to_string())?;
    zipeek_e2e::e2e_ensure_eq!(
        preferences["learning_history_enabled"],
        serde_json::Value::Bool(false),
        "Finishing first-visit onboarding must persist disabled recording consent",
    );
    let ready = driver.query_single("[data-database-state='ready']").await?;
    zipeek_e2e::e2e_ensure!(
        ready.is_displayed().await?,
        "Entering Zipeek must initialize the dictionary"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-welcome-enter]")
            .await?
            .is_empty(),
        "Landing entry should disappear after entering the app"
    );
    ctx.checkpoint_visible("landing-entry-completed", "[data-database-state='ready']")
        .await?;
    Ok(())
}

pub async fn onboarding_opt_in_persists_and_updates_header(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/")).await?;
    zipeek_e2e::page_objects::browser::remove_local_storage(driver, "zipeek.preferences.v1")
        .await?;
    driver.refresh().await?;
    driver
        .query_single("[data-welcome-enter]")
        .await?
        .click()
        .await?;
    let consent = driver.query_single("[data-onboarding-recording]").await?;
    zipeek_e2e::e2e_ensure!(!consent.is_selected().await?);
    consent.click().await?;
    driver
        .query_single("[data-onboarding-finish]")
        .await?
        .click()
        .await?;
    let saved = zipeek_e2e::page_objects::browser::local_storage(driver, "zipeek.preferences.v1")
        .await?
        .ok_or("Onboarding did not save preferences")?;
    let preferences: serde_json::Value =
        serde_json::from_str(&saved).map_err(|error| error.to_string())?;
    zipeek_e2e::e2e_ensure_eq!(
        preferences["learning_history_enabled"],
        serde_json::Value::Bool(true),
    );
    driver.query_single("[data-learning-dragon]").await?;
    driver.refresh().await?;
    driver.query_single("[data-learning-dragon]").await?;
    Ok(())
}

use thirtyfour::prelude::*;
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn landing_page_loads(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();

    driver.goto(ctx.url("/")).await?;
    ctx.checkpoint("shell-initial-state").await?;

    let heading = zipeek_e2e::page_objects::home::brand(&driver).await?;
    ctx.checkpoint_visible("home-loaded", "[data-home]").await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::dictionary_row_count(&driver).await? > 0,
        "Dictionary metadata should be available after initialization"
    );

    zipeek_e2e::e2e_ensure_eq!(heading.text().await?, "Zipeek");

    let footer = driver.query_single("[data-app-footer]").await?;
    let footer_text = footer.text().await?;
    for notice in ["Jozef Podlecki", "CC-CEDICT", "CC BY-SA 4.0"] {
        zipeek_e2e::e2e_ensure!(
            footer_text.contains(notice),
            "The landing page footer is missing attribution {notice:?}: {footer_text}"
        );
    }

    zipeek_e2e::page_objects::browser::remove_local_storage(&driver, "zipeek.preferences.v1")
        .await?;
    driver.refresh().await?;
    let toggle = driver
        .query_single("[data-theme-toggle][aria-checked='true']")
        .await?;
    driver.query_single("html.dark body.dark").await?;
    toggle.send_keys(" ").await?;
    driver
        .query_single("[data-theme-toggle][aria-checked='false']")
        .await?;
    driver
        .query_single("html:not(.dark) body:not(.dark)")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::preference(&driver, "dark").await?,
        Some(serde_json::json!(false)),
    );

    driver.refresh().await?;
    driver
        .query_single("[data-theme-toggle][aria-checked='false']")
        .await?;
    driver
        .query_single("html:not(.dark) body:not(.dark)")
        .await?;

    zipeek_e2e::page_objects::browser::set_preference(
        &driver,
        "dark",
        serde_json::json!("invalid"),
    )
    .await?;
    driver.refresh().await?;
    let toggle = driver
        .query_single("[data-theme-toggle][aria-checked='true']")
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::platform::SHELL_LANDING_PAGE_LOADS_01,
            vec![],
        )
        .await?;
    toggle.click().await?;
    driver
        .query_single("[data-theme-toggle][aria-checked='false']")
        .await?;
    driver
        .query_single("html:not(.dark) body:not(.dark)")
        .await?;

    ctx.checkpoint_visible("shell-home-visible", "[data-home]")
        .await?;
    ctx.checkpoint("shell-verified-result").await?;
    Ok(())
}

pub async fn responsive_shell_preserves_navigation_theme_and_keyboard_access(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    ctx.viewport().responsive(1280, 1000, false).await?;
    driver.goto(ctx.url("/")).await?;
    driver.query_single("[data-home]").await?;

    let tiles = driver
        .find_all_css(selectors::shell::NAVIGATION_TILES)
        .await?;
    zipeek_e2e::e2e_ensure_eq!(tiles.len(), 4);
    for (tile, expected) in tiles
        .iter()
        .zip(["Practice", "Browse", "Word Sets", "Random"])
    {
        zipeek_e2e::e2e_ensure_eq!(tile.find_css("h2").await?.text().await?, expected);
    }
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-daily-word]")
            .await?
            .text()
            .await?
            .matches("No word available")
            .count(),
        1
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[aria-label='Zipeek repository on GitHub']")
            .await?
            .attr("href")
            .await?
            .as_deref(),
        Some("https://github.com/Jozefpodlecki/zipeek")
    );
    zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_01, vec![]).await?.convert::<bool>()?);

    let artifacts = ctx.artifacts.clone();
    driver
        .screenshot(&artifacts.join("a30-desktop-dark.png"))
        .await?;
    driver
        .find_css("[data-theme-toggle]")
        .await?
        .click()
        .await?;
    driver.query_single("html:not(.dark)").await?;
    driver
        .screenshot(&artifacts.join("a30-desktop-light.png"))
        .await?;

    for width in [360, 768, 1024, 1280] {
        ctx.viewport().responsive(width, 1000, width <= 768).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .execute(zipeek_e2e::js_scripts::platform::SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_02, vec![])
                .await?
                .convert::<u32>()?,
            width
        );
        zipeek_e2e::e2e_ensure!(
            zipeek_e2e::page_objects::browser::no_horizontal_overflow(&driver).await?,
            "horizontal overflow at {width}"
        );
        let sidebar = driver
            .execute(
                zipeek_e2e::js_scripts::platform::SHELL_RESPONSIVE_SHELL_PRESERVES_NAVIGATION_THEME_AND_KEYBOARD_ACCESS_03,
                vec![],
            )
            .await?
            .convert::<String>()?;
        zipeek_e2e::e2e_ensure_eq!(sidebar, if width < 768 { "static" } else { "sticky" });
        if width == 360 {
            ctx.viewport().responsive(360, 1500, true).await?;
            driver
                .find_css(selectors::shell::BRAND_LINK)
                .await?
                .click()
                .await?;
            driver
                .screenshot(&artifacts.join("a30-mobile-light.png"))
                .await?;
        }
    }

    driver
        .find_all_css(selectors::shell::NAVIGATION_TILES)
        .await?[0]
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single(selectors::shell::PRACTICE_PROGRESS)
            .await?
            .is_displayed()
            .await?
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-navigation-link][aria-current='page']")
            .await?
            .text()
            .await?
            .contains("Practice")
    );
    driver
        .find_css("[data-navigation-link][href='#/browse']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single(selectors::shell::BROWSE_QUERY)
            .await?
            .is_displayed()
            .await?
    );
    driver
        .find_css(selectors::shell::BRAND_LINK)
        .await?
        .click()
        .await?;
    driver.query_single("[data-home]").await?;
    driver
        .find_all_css(selectors::shell::NAVIGATION_TILES)
        .await?[3]
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single(selectors::shell::RANDOM_PAGE)
            .await?
            .is_displayed()
            .await?
    );
    driver
        .find_css(selectors::shell::BRAND_LINK)
        .await?
        .click()
        .await?;
    driver.query_single("[data-home]").await?;
    let before_skip = driver.current_url().await?;
    driver
        .find_css("[data-skip-link]")
        .await?
        .send_keys(Key::Enter)
        .await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::focused_element_has_id(&driver, "main-content").await?
    );
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, before_skip);
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.path(), "/");

    Ok(())
}

pub async fn drawer_collapses_and_compact_desktop_has_no_unnecessary_scrollbar(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    ctx.viewport().responsive(1280, 720, false).await?;
    driver.goto(ctx.url("/")).await?;
    driver.query_single("[data-home]").await?;
    zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_01, vec![]).await?.convert::<bool>()?);
    zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_02, vec![]).await?.convert::<bool>()?);
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("body")
            .await?
            .text()
            .await?
            .contains("Chinese vocabulary")
    );

    for (width, height) in [(1024, 768), (1280, 720), (1366, 768)] {
        ctx.viewport().responsive(width, height, false).await?;
        zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::platform::SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_03, vec![]).await?.convert::<bool>()?, "unnecessary scrolling at {width}x{height}");
    }
    let toggle = driver.find_css("[data-drawer-toggle]").await?;
    toggle.send_keys(" ").await?;
    driver
        .query_single("[data-drawer-toggle][aria-expanded='false']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::platform::SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_04,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    let artifacts = ctx.artifacts.clone();
    driver
        .screenshot(&artifacts.join("a30-collapsed.png"))
        .await?;
    toggle.click().await?;
    driver
        .query_single("[data-drawer-toggle][aria-expanded='true']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );

    ctx.viewport().responsive(360, 800, true).await?;
    toggle.click().await?;
    driver
        .query_single("[data-drawer-toggle][aria-expanded='false']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::platform::SHELL_DRAWER_COLLAPSES_AND_COMPACT_DESKTOP_HAS_NO_UNNECESSARY_SCROLLBAR_05,
                vec![]
            )
            .await?
            .convert::<bool>()?,
        "mobile content must remain scrollable when it does not fit"
    );
    toggle.click().await?;
    driver
        .query_single("[data-drawer-toggle][aria-expanded='true']")
        .await?;
    driver
        .find_css("[data-navigation-link][href='#/browse']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-navigation-link][aria-current='page'][href='#/browse']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("body")
            .await?
            .text()
            .await?
            .contains("Follow your curiosity")
    );
    ctx.checkpoint_visible(
        "responsive-navigation-verified",
        "[data-navigation-link][aria-current='page'][href='#/browse']",
    )
    .await?;
    Ok(())
}
