use page_objects::settings::{self, Section};
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects};

async fn settings_sections_have_matching_panels(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    for section in Section::ALL {
        settings::select_section(driver, section).await?;
        let selected = format!("[data-settings-tab-button='{}']", section.key());
        let button = driver.query_single(&selected).await?;
        zipeek_e2e::e2e_ensure_eq!(button.attr("aria-current").await?.as_deref(), Some("true"));
        zipeek_e2e::e2e_ensure_eq!(
            button.attr("aria-controls").await?.as_deref(),
            Some("settings-active-panel")
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css("[data-settings-tab-button][aria-current='true']",)
                .await?
                .len(),
            1,
            "Exactly one Settings section must be current"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single(settings::ACTIVE_PANEL)
                .await?
                .is_displayed()
                .await?
        );
        let current = driver
            .find_css(settings::PAGE)
            .await?
            .attr("data-settings-tab")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(current.as_deref(), Some(section.key()));
    }
    ctx.checkpoint("settings-all-panels").await?;
    Ok(())
}

async fn font_choices_are_not_inert(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    for key in [
        "system",
        "sans",
        "serif",
        "calligraphy",
        "lxgw-wenkai",
        "ma-shan-zheng",
        "zcool-xiaowei",
        "long-cang",
        "noto-serif-sc",
        "zcool-kuaile",
    ] {
        settings::choose_font(driver, key).await?;
        let actual = driver
            .execute(
                zipeek_e2e::js_scripts::settings::SETTINGS_INTERACTIONS_FONT_CHOICES_ARE_NOT_INERT_01,
                vec![],
            )
            .await?
            .convert::<String>()?;
        zipeek_e2e::e2e_ensure_eq!(actual, key, "Font choice was displayed but not applied");
    }
    for size in ["normal", "large", "extra-large"] {
        settings::choose_character_size(driver, size).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::settings::SETTINGS_INTERACTIONS_FONT_CHOICES_ARE_NOT_INERT_02,
                    vec![]
                )
                .await?
                .convert::<String>()?,
            size,
            "Selected Character Builder size must be applied to the document"
        );
    }
    settings::choose_character_size(driver, "large").await?;
    driver.refresh().await?;
    settings::select_section(driver, Section::Characters).await?;
    driver
        .query_single("[data-character-size-option='large'][aria-pressed='true']")
        .await?;
    ctx.checkpoint("settings-font-and-size-reload").await?;
    Ok(())
}

async fn browser_service_availability_is_read_only(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (section, name) in [(Section::Ai, "ai"), (Section::Images, "image")] {
        ctx.open_route("/settings").await?;
        settings::select_section(driver, section).await?;
        let selector = format!("[data-native-service='{name}']");
        let panel = driver.query_single(&selector).await?;
        zipeek_e2e::e2e_ensure!(panel.is_displayed().await?);
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css(&format!("{selector} button"))
                .await?
                .len()
                == 1,
            "Browser service monitoring must offer Refresh without native Start/Stop controls"
        );
        driver
            .find_css(&format!("{selector} button"))
            .await?
            .click()
            .await?;
    }
    ctx.checkpoint("browser-service-availability").await?;
    Ok(())
}

#[tokio::test]
async fn settings_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "settings_sections_have_matching_panels",
            settings_sections_have_matching_panels,
        )
        .await?;
    harness
        .run(
            "settings_font_choices_apply_and_persist",
            font_choices_are_not_inert,
        )
        .await?;
    harness
        .run(
            "browser_service_availability_is_read_only",
            browser_service_availability_is_read_only,
        )
        .await?;
    harness.finish().await
}
