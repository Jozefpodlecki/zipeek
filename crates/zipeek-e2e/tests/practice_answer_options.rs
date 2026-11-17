use zipeek_e2e::page_objects::{self, WebDriverActions, selectors::practice};

use thirtyfour::prelude::*;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

mod selectors {
    pub const QUICK: &str = super::practice::QUICK_PATH;
    pub const EXERCISE: &str = super::practice::EXERCISE;
    pub const CHARACTER_BUILDER: &str = super::practice::CHARACTER_BUILDER;
    pub const CONTINUE_SETUP: &str = super::practice::CONTINUE;
    pub const RETRY_OPTION: &str = "[data-practice-retry-incorrect]";
    pub const AUDIO_OPTION: &str = "[data-practice-show-audio]";
    pub const HINT_OPTION: &str = "[data-practice-pinyin-hint]";
    pub const AUTO_CHECK_OPTION: &str = "[data-practice-auto-check]";
    pub const START: &str = "[data-practice-start]:not([disabled])";
    pub const EXERCISE_PAGE: &str = "[data-component='vocabulary-exercise']";
    pub const AUDIO: &str = "[data-practice-audio] button";
    pub const AVAILABLE_READINGS: &str =
        "[aria-label='Available characters'] [data-character-reading]";
}

async fn advance(driver: &WebDriver) -> WebDriverResult<()> {
    driver.click(selectors::CONTINUE_SETUP).await
}

async fn practice_preferences_and_character_builder(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click(selectors::QUICK).await?;

    driver.click(selectors::CHARACTER_BUILDER).await?;
    let choices = driver.find_all_css(selectors::EXERCISE).await?;
    for choice in choices {
        let is_builder =
            choice.attr("data-practice-exercise").await?.as_deref() == Some("character-builder");
        if !is_builder && choice.attr("aria-pressed").await?.as_deref() == Some("true") {
            choice.click().await?;
        }
    }
    driver
        .query_single("[data-practice-exercise='character-builder'][aria-pressed='true']")
        .await?;
    driver
        .query_single("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;

    for selector in [selectors::RETRY_OPTION, selectors::AUDIO_OPTION] {
        let checkbox = driver.query_single(selector).await?;
        zipeek_e2e::e2e_ensure!(
            !checkbox.is_selected().await?,
            "Answer preference must default off: {selector}"
        );
        checkbox.click().await?;
        zipeek_e2e::e2e_ensure!(
            checkbox.is_selected().await?,
            "Answer preference must be configurable: {selector}"
        );
    }
    advance(driver).await?;
    advance(driver).await?;
    driver.query_single(selectors::START).await?.click().await?;
    driver.query_single(selectors::EXERCISE_PAGE).await?;
    driver.query_single(selectors::AUDIO).await?;

    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css(selectors::AVAILABLE_READINGS)
            .await?
            .len()
            >= 2,
        "Character builder must render pronunciation slots under available tiles"
    );
    driver
        .query_first("[aria-label='Available characters'] button:not([disabled])")
        .await?
        .click()
        .await?;
    let tile_style = driver.execute(
        zipeek_e2e::js_scripts::practice::PRACTICE_ANSWER_OPTIONS_PRACTICE_PREFERENCES_AND_CHARACTER_BUILDER_01,
        vec![],
    ).await?;
    zipeek_e2e::e2e_ensure!(
        tile_style
            .json()
            .get("sameFont")
            .and_then(serde_json::Value::as_bool)
            == Some(true),
        "Selected and available Hanzi must use one font family"
    );
    zipeek_e2e::e2e_ensure!(
        tile_style
            .json()
            .get("sameSize")
            .and_then(serde_json::Value::as_bool)
            == Some(true),
        "Selected and available Hanzi must use one size scale"
    );
    let text = driver
        .find_css(selectors::EXERCISE_PAGE)
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !text.contains("Skip this exercise"),
        "Ready exercise cannot expose Skip"
    );
    zipeek_e2e::e2e_ensure!(
        !text.contains("Exercise 1 of"),
        "Session wrapper must not repeat a technical heading"
    );

    driver.refresh().await?;
    driver.query_single(selectors::EXERCISE_PAGE).await?;
    driver.query_single(selectors::AUDIO).await?;
    ctx.checkpoint("character-builder-answer-options").await?;
    Ok(())
}

async fn per_exercise_configuration_is_isolated(ctx: ScenarioContext) -> HarnessResult<()> {
    use page_objects::practice_config as config;

    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single(config::QUICK_PATH)
        .await?
        .click()
        .await?;

    config::toggle_exercise(driver, "character-builder").await?;
    config::toggle_exercise(driver, "type-pinyin").await?;
    let selected = config::selected_exercises(driver).await?;
    zipeek_e2e::e2e_ensure!(selected.contains(&"character-builder".to_owned()));
    zipeek_e2e::e2e_ensure!(selected.contains(&"type-pinyin".to_owned()));
    driver
        .query_single(config::CONFIGURE_EXERCISES)
        .await?
        .click()
        .await?;

    config::select_tab(driver, config::TYPE_PINYIN_TAB).await?;
    zipeek_e2e::e2e_ensure!(!config::checkbox_checked(driver, config::RETRY).await?);
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css(config::AUDIO).await?.is_empty(),
        "Type pinyin does not support an audio-button preference"
    );
    config::set_checkbox(driver, config::RETRY, true).await?;
    config::set_checkbox(driver, selectors::HINT_OPTION, true).await?;

    config::select_tab(driver, config::CHARACTER_BUILDER_TAB).await?;
    zipeek_e2e::e2e_ensure!(
        !config::checkbox_checked(driver, config::RETRY).await?,
        "Type pinyin retry must not affect Character Builder"
    );
    zipeek_e2e::e2e_ensure!(!config::checkbox_checked(driver, config::AUDIO).await?);
    config::set_checkbox(driver, config::RETRY, true).await?;
    config::set_checkbox(driver, config::AUDIO, true).await?;
    config::set_checkbox(driver, selectors::AUTO_CHECK_OPTION, true).await?;

    config::select_tab(driver, config::TYPE_PINYIN_TAB).await?;
    zipeek_e2e::e2e_ensure!(
        config::checkbox_checked(driver, config::RETRY).await?,
        "Type pinyin retry preference should survive tab switching"
    );
    zipeek_e2e::e2e_ensure!(
        config::checkbox_checked(driver, selectors::HINT_OPTION).await?,
        "Pinyin hint preference must survive tab switching"
    );
    config::set_checkbox(driver, config::RETRY, false).await?;

    config::select_tab(driver, config::CHARACTER_BUILDER_TAB).await?;
    zipeek_e2e::e2e_ensure!(
        config::checkbox_checked(driver, config::RETRY).await?,
        "Character Builder retry must remain independent"
    );
    zipeek_e2e::e2e_ensure!(
        config::checkbox_checked(driver, config::AUDIO).await?,
        "Character Builder audio must survive tab switching"
    );
    zipeek_e2e::e2e_ensure!(
        config::checkbox_checked(driver, selectors::AUTO_CHECK_OPTION).await?,
        "Character Builder automatic check must survive tab switching"
    );
    ctx.checkpoint("per-exercise-configuration").await?;
    Ok(())
}

async fn retry_incorrect_pinyin_without_advancing(ctx: ScenarioContext) -> HarnessResult<()> {
    use page_objects::practice_config as config;

    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single(config::QUICK_PATH)
        .await?
        .click()
        .await?;
    let type_pinyin = driver
        .query_single("[data-practice-exercise='type-pinyin']")
        .await?;
    if type_pinyin.attr("aria-pressed").await?.as_deref() != Some("true") {
        type_pinyin.click().await?;
    }
    let choices = driver.find_all_css(selectors::EXERCISE).await?;
    for choice in choices {
        let exercise = choice.attr("data-practice-exercise").await?;
        let selected = choice.attr("aria-pressed").await?.as_deref() == Some("true");
        if exercise.as_deref() != Some("type-pinyin") && selected {
            choice.click().await?;
        }
    }
    driver
        .query_single(config::CONFIGURE_EXERCISES)
        .await?
        .click()
        .await?;
    config::select_tab(driver, config::TYPE_PINYIN_TAB).await?;
    config::set_checkbox(driver, config::RETRY, true).await?;
    config::set_checkbox(driver, selectors::HINT_OPTION, true).await?;
    advance(driver).await?;
    advance(driver).await?;
    driver.query_single(selectors::START).await?.click().await?;
    driver.query_single("[data-exercise='type-pinyin']").await?;

    let input = driver
        .query_single("[data-exercise='type-pinyin'] input[aria-label='Pinyin reading']")
        .await?;
    let hint = input.attr("placeholder").await?.unwrap_or_default();
    zipeek_e2e::e2e_ensure!(
        !hint.is_empty() && hint != "Type pinyin with tone marks or tone numbers",
        "Pinyin hint should provide the configured reading"
    );
    input.send_keys("zzzzzz-invalid-pinyin").await?;
    driver
        .query_single("[data-exercise='type-pinyin'] [data-exercise-check]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-exercise='type-pinyin'] [data-practice-retry-notice]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-exercise='type-pinyin'] [data-exercise-continue]")
            .await?
            .is_empty(),
        "Retry policy must not advance or display Continue after a wrong answer"
    );
    zipeek_e2e::e2e_ensure_eq!(
        input.prop("value").await?.as_deref(),
        Some(""),
        "Wrong typed answers must clear for another attempt"
    );
    driver.refresh().await?;
    let input = driver
        .query_single("[data-exercise='type-pinyin'] input[aria-label='Pinyin reading']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        input.attr("placeholder").await?.as_deref(),
        Some(hint.as_str())
    );
    ctx.checkpoint("type-pinyin-retry-without-advance").await?;
    Ok(())
}

async fn interrupted_practice_is_resumable(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click(selectors::QUICK).await?;
    driver.click(selectors::CHARACTER_BUILDER).await?;
    let choices = driver.find_all_css(selectors::EXERCISE).await?;
    for choice in choices {
        let is_builder =
            choice.attr("data-practice-exercise").await?.as_deref() == Some("character-builder");
        if !is_builder && choice.attr("aria-pressed").await?.as_deref() == Some("true") {
            choice.click().await?;
        }
    }
    advance(driver).await?;
    advance(driver).await?;
    driver.query_single(selectors::START).await?.click().await?;
    driver
        .query_single("[data-exercise='character-builder']")
        .await?;

    ctx.open_route("/practice").await?;
    let blocked = driver
        .query_single("[data-practice-session-blocked]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        blocked
            .text()
            .await?
            .contains("Continue your saved Practice"),
        "Existing UUID session must block a second session from Practice setup"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-start]")
            .await?
            .is_empty(),
        "Blocked Practice setup must not offer Start"
    );
    driver
        .query_first("[data-practice-session-blocked] a[href*='/practice/session/']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-exercise='character-builder']")
        .await?;

    ctx.open_route("/settings").await?;
    let reminder = page_objects::session::reminder(driver).await?;
    zipeek_e2e::e2e_ensure!(
        reminder
            .text()
            .await?
            .contains("Practice session in progress")
    );
    page_objects::session::dismiss(driver).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css(page_objects::session::REMINDER)
            .await?
            .is_empty(),
        "Dismiss must hide the panel without ending the saved session"
    );
    ctx.open_route("/browse").await?;
    page_objects::session::reminder(driver).await?;
    page_objects::session::resume(driver).await?;
    driver
        .query_single("[data-exercise='character-builder']")
        .await?;

    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-session-blocked]")
        .await?;
    page_objects::session::end(driver).await?;
    driver.query_single("[data-practice-setup]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-session-blocked]")
            .await?
            .is_empty(),
        "Practice must refresh immediately when the saved session is discarded"
    );
    driver.refresh().await?;
    driver.query_single("[data-practice-setup]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-unfinished-session]")
            .await?
            .is_empty()
    );
    ctx.checkpoint("practice-reminder-resume-and-end").await?;
    Ok(())
}

async fn saved_practice_profile_round_trip(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click(selectors::QUICK).await?;
    advance(driver).await?;
    advance(driver).await?;
    driver
        .query_single("#practice-profile-name")
        .await?
        .send_keys("Daily Mandarin")
        .await?;
    driver
        .query_single("[data-practice-profile-save]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-profile-saved]")
            .await?
            .text()
            .await?
            .contains("Daily Mandarin"),
        "Saving must provide a visible confirmation"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-profile-save]")
            .await?
            .is_empty(),
        "Saved unchanged configuration must not invite another save"
    );

    driver.refresh().await?;
    ctx.open_route("/practice").await?;
    let profiles = driver
        .query_single("[data-practice-saved-profiles]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        profiles.text().await?.contains("Daily Mandarin"),
        "Saved profile must survive a browser refresh"
    );
    driver
        .query_single("[data-practice-profile-load]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-step-panel='summary']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-summary-configuration]")
            .await?
            .text()
            .await?
            .contains("Matching"),
        "Loaded profile must restore selected exercise types"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-profile-active]")
            .await?
            .text()
            .await?
            .contains("Daily Mandarin"),
        "Loaded configuration must identify its saved profile"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-profile-save]")
            .await?
            .is_empty(),
        "Loading an unchanged profile must not require saving again"
    );

    driver
        .query_single("[data-practice-step='path']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-profile-remove]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-saved-profiles]")
            .await?
            .text()
            .await?
            .contains("Remove this saved profile?"),
        "Profile removal must require confirmation"
    );
    driver
        .query_single("[data-practice-saved-profiles] .border-red-500")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-profile-load]")
            .await?
            .is_empty(),
        "Deleted profile must not remain selectable"
    );
    ctx.checkpoint("practice-profile-save-load-remove").await?;
    Ok(())
}

async fn exercise_selection_keeps_other_choices(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click(selectors::QUICK).await?;
    let matching = driver
        .query_single("[data-practice-exercise='matching']")
        .await?;
    let multiple_choice = driver
        .query_single("[data-practice-exercise='multiple-choice']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        matching.attr("aria-pressed").await?.as_deref(),
        Some("true")
    );
    zipeek_e2e::e2e_ensure_eq!(
        multiple_choice.attr("aria-pressed").await?.as_deref(),
        Some("true")
    );
    matching.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        matching.attr("aria-pressed").await?.as_deref(),
        Some("false")
    );
    zipeek_e2e::e2e_ensure_eq!(
        multiple_choice.attr("aria-pressed").await?.as_deref(),
        Some("true"),
        "Deselecting Matching must not clear Multiple Choice"
    );
    driver.click(selectors::CHARACTER_BUILDER).await?;
    zipeek_e2e::e2e_ensure_eq!(
        multiple_choice.attr("aria-pressed").await?.as_deref(),
        Some("true"),
        "Adding Character Builder must preserve existing selection"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single(selectors::CHARACTER_BUILDER)
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    multiple_choice.click().await?;
    driver.click(selectors::CHARACTER_BUILDER).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css(selectors::CONTINUE_SETUP)
            .await?
            .is_empty(),
        "Empty exercise selection must not allow setup to advance"
    );
    ctx.checkpoint("practice-exercise-selection-consistency")
        .await?;
    Ok(())
}

async fn character_builder_automatic_check(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click(selectors::QUICK).await?;
    driver.click(selectors::CHARACTER_BUILDER).await?;
    let choices = driver.find_all_css(selectors::EXERCISE).await?;
    for choice in choices {
        let selected = choice.attr("aria-pressed").await?.as_deref() == Some("true");
        let builder =
            choice.attr("data-practice-exercise").await?.as_deref() == Some("character-builder");
        if selected && !builder {
            choice.click().await?;
        }
    }
    driver
        .query_single("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    driver
        .query_single(selectors::RETRY_OPTION)
        .await?
        .click()
        .await?;
    driver
        .query_single(selectors::AUTO_CHECK_OPTION)
        .await?
        .click()
        .await?;
    advance(driver).await?;
    advance(driver).await?;
    driver.query_single(selectors::START).await?.click().await?;
    driver
        .query_single("[data-exercise='character-builder'][data-auto-check='true']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-exercise='character-builder'] [data-exercise-check]")
            .await?
            .is_empty(),
        "Automatic checking must not require the manual Check Answer button"
    );
    let slots = driver
        .find_all_css("[aria-label='Selected characters'] button")
        .await?
        .len();
    for _ in 0..slots {
        driver
            .query_first("[aria-label='Available characters'] button:not([disabled])")
            .await?
            .click()
            .await?;
    }
    driver
        .query_first(
            "[data-exercise='character-builder'] [data-practice-answer-feedback] [role='status']",
        )
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-exercise='character-builder'] [data-practice-retry-notice]")
            .await?
            .is_empty(),
        "Automatic checking must not display a separate Try again notice"
    );
    driver.refresh().await?;
    driver
        .query_single("[data-exercise='character-builder'][data-auto-check='true']")
        .await?;
    ctx.checkpoint("character-builder-automatic-check").await?;
    Ok(())
}

#[tokio::test]
async fn practice_answer_options() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "practice_exercise_selection_consistency",
            exercise_selection_keeps_other_choices,
        )
        .await?;
    harness
        .run(
            "character_builder_answer_options",
            practice_preferences_and_character_builder,
        )
        .await?;
    harness
        .run(
            "character_builder_automatic_check",
            character_builder_automatic_check,
        )
        .await?;
    harness
        .run(
            "exercise_options_are_independent",
            per_exercise_configuration_is_isolated,
        )
        .await?;
    harness
        .run(
            "type_pinyin_retry_stays_on_round",
            retry_incorrect_pinyin_without_advancing,
        )
        .await?;
    harness
        .run(
            "unfinished_session_resume_and_end",
            interrupted_practice_is_resumable,
        )
        .await?;
    harness
        .run(
            "practice_profile_save_load_remove",
            saved_practice_profile_round_trip,
        )
        .await?;
    harness.finish().await
}
