use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::{WebDriverActions, selectors};
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn crossword_click_to_place_and_reset(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/crosswords").await?;
    ctx.checkpoint("recent-routes-initial-state").await?;
    driver
        .query_single("[data-component='hanzi-crosswords']")
        .await?;
    zipeek_e2e::page_objects::crossword::open_setup(driver).await?;
    zipeek_e2e::page_objects::crossword::start(driver).await?;

    let rendered = driver
        .execute(
            zipeek_e2e::js_scripts::platform::RECENT_ROUTES_CROSSWORD_CLICK_TO_PLACE_AND_RESET_01,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        rendered,
        "Dictionary-based crossword must be visibly sized and playable"
    );

    let tile = driver
        .query_first(zipeek_e2e::page_objects::crossword::TILE)
        .await?;
    tile.click().await?;
    driver
        .query_first(zipeek_e2e::page_objects::crossword::SELECTED_TILE)
        .await?;

    ctx.checkpoint_visible(
        "crossword-selected-tile",
        zipeek_e2e::page_objects::crossword::SELECTED_TILE,
    )
    .await?;
    zipeek_e2e::page_objects::crossword::new_game(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::crossword::score(driver).await?,
        "0",
        "A newly generated crossword must reset its displayed score"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .as_str()
            .contains("/#/games/crosswords"),
        "Resetting Crossword must not navigate away from the game"
    );
    ctx.checkpoint_visible(
        "crossword-after-placement",
        zipeek_e2e::page_objects::crossword::TILE,
    )
    .await?;
    ctx.checkpoint("recent-routes-verified-result").await?;
    Ok(())
}

pub async fn settings_font_and_theme_are_independent(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    zipeek_e2e::page_objects::browser::set_preference(driver, "dark", serde_json::json!(true))
        .await?;
    driver.refresh().await?;
    driver
        .query_single("html.dark [data-settings-tab='appearance']")
        .await?;

    let presets = driver
        .find_all_css("[data-settings-appearance] button[aria-pressed]")
        .await?;
    let mut found = false;
    for preset in presets {
        if preset.text().await?.contains("Jade Light") {
            preset.click().await?;
            found = true;
            break;
        }
    }
    zipeek_e2e::e2e_ensure!(found, "Jade Light preset must be available");
    driver.query_single("html.dark").await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::preference(driver, "dark").await?
            == Some(serde_json::json!(true))
    );

    driver
        .find_css("[data-settings-tab-button='characters']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-hanzi-font-option='serif']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-hanzi-font-option='serif'][aria-pressed='true']")
        .await?;
    driver.refresh().await?;
    driver
        .find_css("[data-settings-tab-button='characters']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-hanzi-font-option='serif'][aria-pressed='true']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::preference(driver, "font").await?,
        Some(serde_json::json!("serif")),
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("html")
            .await?
            .attr("data-hanzi-font")
            .await?
            .as_deref(),
        Some("serif"),
    );
    Ok(())
}

pub async fn dictionary_autocomplete_keyboard_and_pagination(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    driver
        .query_single("[data-component='browse-search-form']")
        .await?;
    let input = driver.find_css("[data-browse-filter-autocomplete]").await?;
    input.send_keys("noun").await?;
    driver
        .query_single("[role='listbox'] [role='option']")
        .await?;
    input.send_keys(Key::Enter).await?;
    driver
        .query_single(selectors::browse::REMOVE_NOUN_FILTER)
        .await?;
    driver
        .find_css("[data-browse-clear]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("button[aria-label='Remove PoS: noun']")
            .await?
            .is_empty()
    );
    driver
        .query_single("[data-component='browse-pagination']")
        .await?;
    zipeek_e2e::e2e_ensure!(driver
        .execute(
            zipeek_e2e::js_scripts::platform::RECENT_ROUTES_DICTIONARY_AUTOCOMPLETE_KEYBOARD_AND_PAGINATION_01,
            vec![],
        )
        .await?
        .convert::<bool>()?);
    Ok(())
}

pub async fn background_presets_persist_without_changing_display_mode(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    driver.query_single("[data-settings-appearance]").await?;
    for preset in ["jade", "sunset", "midnight", "ink", "maple", "lavender"] {
        driver
            .find_css(format!("[data-background-preset='{preset}']"))
            .await?
            .click()
            .await?;
        driver
            .query_single(format!(
                "html[data-theme-preset='{preset}'] #app-background[src$='{preset}.webp']"
            ))
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            zipeek_e2e::page_objects::browser::preference(driver, "preset").await?,
            Some(serde_json::json!(preset)),
            "Saved wallpaper must match the visible selection"
        );
        let image = driver.find_css("#app-background").await?;
        zipeek_e2e::e2e_ensure!(
            image
                .attr("src")
                .await?
                .is_some_and(|source| source.ends_with(&format!("/{preset}.webp"))),
            "The selected background image must be installed"
        );
    }
    driver
        .find_css("[data-background-preset='jade']")
        .await?
        .click()
        .await?;
    driver.refresh().await?;
    driver
        .query_single("html[data-theme-preset='jade'] #app-background[src$='jade.webp']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::preference(driver, "dark").await?,
        Some(serde_json::json!(true)),
        "Background selection must not change display mode"
    );
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::preference(driver, "preset").await?,
        Some(serde_json::json!("jade")),
        "Preset must survive a browser reload"
    );
    ctx.checkpoint_visible(
        "background-preset-restored",
        "html[data-theme-preset='jade'] #app-background[src$='jade.webp']",
    )
    .await?;
    Ok(())
}
