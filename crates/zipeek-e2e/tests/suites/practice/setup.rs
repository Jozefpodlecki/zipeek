use zipeek_e2e::page_objects::WebDriverActions;

use thirtyfour::prelude::*;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn current(driver: &WebDriver, step: &str) -> WebDriverResult<WebElement> {
    driver
        .query_single(format!(
            "[data-practice-progress] [data-practice-step='{step}'][aria-current='step']"
        ))
        .await
}

async fn continue_setup(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_css("[data-practice-continue]:not([disabled])")
        .await?
        .click()
        .await
}

async fn choose_stage(driver: &WebDriver, stage: &str) -> WebDriverResult<()> {
    driver
        .query_single(format!(
            "[data-practice-exercise-stage='{stage}'][aria-selected='true']"
        ))
        .await?;
    Ok(())
}

pub async fn setup_progress_mode_and_responsive_layout(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    ctx.checkpoint("setup-initial-state").await?;
    current(driver, "path").await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("[data-practice-step='summary']")
            .await?
            .is_enabled()
            .await?,
        "Practice Summary must remain unavailable before the learner completes setup"
    );

    driver
        .query_single("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    current(driver, "exercises").await?;
    choose_stage(driver, "choose").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-exercise]").await?.len(),
        8
    );
    driver
        .query_single("[data-practice-exercise='write-hanzi']")
        .await?;
    driver
        .query_single("[data-practice-exercise='listening-choice']")
        .await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-practice-exercise='sentence-translation'], [data-practice-exercise='spot-mistake']").await?.is_empty(),
        "Quick Practice must not expose AI or advanced grammar exercises");
    zipeek_e2e::e2e_ensure!(driver.execute(
        zipeek_e2e::js_scripts::practice::SETUP_SETUP_PROGRESS_MODE_AND_RESPONSIVE_LAYOUT_01,
        vec![],
    ).await?.convert::<bool>()?, "Every exercise tile must have a readable description");
    let matching = driver
        .find_css("[data-practice-exercise='matching']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        matching.attr("aria-pressed").await?.as_deref(),
        Some("false")
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .is_empty()
    );
    matching.click().await?;
    driver
        .query_single("[data-practice-exercise='matching'] [data-exercise-selected]")
        .await?;

    driver
        .find_css("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    choose_stage(driver, "configure").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise]")
            .await?
            .is_empty()
    );
    driver.query_single("[data-practice-config-tabs]").await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    driver
        .query_single("[data-practice-step-panel='vocabulary']")
        .await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .query_single("[data-mode-picker] [data-setup-choice='Fixed 5'][aria-pressed='true']")
        .await?;

    driver
        .find_css("[data-practice-step='exercises']")
        .await?
        .click()
        .await?;
    current(driver, "exercises").await?;
    driver
        .find_css("[data-practice-exercise-stage='choose']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-exercise='matching'][aria-pressed='true']")
        .await?;
    driver
        .find_css("[data-practice-step='summary']")
        .await?
        .click()
        .await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-mode-picker] [data-setup-choice='Fixed 10']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-mode-picker] [data-setup-choice='Fixed 10'][aria-pressed='true']")
        .await?;
    driver
        .find_css("[data-mode-picker] [data-setup-choice='Fixed 5']")
        .await?
        .click()
        .await?;

    for (width, name) in [(390, "mobile"), (1280, "desktop")] {
        ctx.viewport().responsive(width, 850, width < 600).await?;
        zipeek_e2e::e2e_ensure!(
            zipeek_e2e::page_objects::browser::horizontal_overflow_pixels(driver).await? <= 2,
            "Practice setup overflows at {width}px"
        );
        driver
            .screenshot(&ctx.artifacts.join(format!("practice-stepper-{name}.png")))
            .await?;
    }
    ctx.checkpoint_visible(
        "setup-summary-present",
        "[data-practice-step-panel='summary']",
    )
    .await?;
    ctx.checkpoint("setup-verified-result").await?;
    Ok(())
}

pub async fn setup_selection_guard_and_history(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    current(driver, "exercises").await?;

    zipeek_e2e::e2e_ensure!(
        driver
            .query_first("[data-practice-exercise][aria-pressed='true']")
            .await
            .is_err()
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("[data-practice-continue]")
            .await?
            .is_enabled()
            .await?
    );
    driver
        .query_single("[data-practice-step-panel='exercises'] [role='alert']")
        .await?;

    driver
        .find_css("[data-practice-exercise='matching']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-continue]:not([disabled])")
        .await?;
    driver
        .find_css("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    choose_stage(driver, "configure").await?;
    driver
        .query_single("[data-practice-config-tab='matching']")
        .await?;
    driver
        .find_css("[data-pair-count='7']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-pair-count='7'][aria-pressed='true']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-config-tab='multiple-choice']")
            .await?
            .is_empty()
    );

    driver
        .find_css("[data-practice-step-navigation] button:not([data-practice-continue])")
        .await?
        .click()
        .await?;
    choose_stage(driver, "choose").await?;
    driver
        .query_single("[data-practice-exercise='matching'][aria-pressed='true']")
        .await?;
    driver
        .query_single("[data-practice-exercise='multiple-choice'][aria-pressed='false']")
        .await?;
    driver
        .find_css("[data-practice-step='path']")
        .await?
        .click()
        .await?;
    current(driver, "path").await?;
    driver
        .find_css("[data-practice-step='exercises']")
        .await?
        .click()
        .await?;
    current(driver, "exercises").await?;
    driver
        .query_single("[data-practice-exercise='matching'][aria-pressed='true']")
        .await?;
    Ok(())
}

pub async fn standalone_practice_playlist_selection(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-continue]")
            .await?
            .is_empty()
    );
    driver
        .query_single("[data-practice-path='quick']")
        .await?
        .click()
        .await?;

    for id in ["word-mosaics", "character-hunt", "character-builder"] {
        driver
            .find_css(format!("[data-practice-exercise='{id}']"))
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .len(),
        3,
    );
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    driver
        .find_css("[data-practice-start]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-component='vocabulary-exercise']")
        .await?;
    Ok(())
}

pub async fn authored_practice_continue_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='fill-blank']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-exercise='fill-blank'][aria-pressed='true']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-selected-exercise-count]")
            .await?
            .is_empty()
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise-guidance]")
            .await?
            .is_empty()
    );
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-authored-exercise]")
        .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    driver.refresh().await?;
    driver
        .query_single("[data-practice-authored-exercise]")
        .await?;
    driver
        .find_css("#authored-hanzi-response")
        .await?
        .send_keys("错误")
        .await?;
    driver
        .find_css("[data-practice-authored-exercise] button[type='submit']")
        .await?
        .click()
        .await?;
    driver.query_single("[data-practice-feedback]").await?;
    driver
        .find_css("[data-practice-feedback] button")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-authored-exercise]")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
    Ok(())
}

pub async fn sentence_builder_session_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='sentence-builder']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-sentence-builder]")
        .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    driver.refresh().await?;
    driver
        .query_single("[data-practice-sentence-builder]")
        .await?;
    let tiles = driver
        .find_all_css("[aria-label='Available Chinese phrase tiles'] button")
        .await?;
    zipeek_e2e::e2e_ensure!(tiles.len() >= 2);
    zipeek_e2e::page_objects::sentence_builder::choose_correct(driver).await?;
    driver
        .find_css("[data-sentence-builder-check]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-sentence-builder] [data-practice-feedback]")
        .await?;
    driver
        .find_css("[data-sentence-builder-continue]")
        .await?
        .click()
        .await?;
    Ok(())
}

pub async fn spot_mistake_identify_repair_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='spot-mistake']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver.query_single("[data-practice-spot-mistake]").await?;
    driver.refresh().await?;
    driver.query_single("[data-practice-spot-mistake]").await?;
    let spans = driver
        .find_all_css("[aria-label='Select the incorrect Chinese phrase'] button")
        .await?;
    zipeek_e2e::e2e_ensure!(spans.len() >= 3);
    spans[2].click().await?;
    driver
        .find_css("#spot-mistake-repair")
        .await?
        .send_keys("去")
        .await?;
    driver
        .find_css("[data-spot-mistake-check]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-spot-mistake] [data-practice-feedback]")
        .await?;
    driver
        .find_css("[data-spot-mistake-continue]")
        .await?
        .click()
        .await?;
    Ok(())
}

pub async fn quick_write_hanzi_summary(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='write-hanzi']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .query_single("[data-practice-summary-configuration]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-practice-summary-configuration]")
            .await?
            .text()
            .await?
            .contains("Write Hanzi"),
        "Quick Practice must name Write Hanzi in the session summary"
    );
    Ok(())
}

pub async fn listening_choice_session_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='listening-choice']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    for _ in 0..5 {
        driver.query_single("[data-practice-local-session]").await?;
        if !driver
            .find_all_css("[data-listening-session]")
            .await?
            .is_empty()
        {
            driver.refresh().await?;
            driver.query_single("[data-listening-session]").await?;
            driver.click("[data-listening-play]").await?;
            zipeek_e2e::e2e_ensure!(
                zipeek_e2e::page_objects::audio_probes::has_mandarin_speak(
                    &zipeek_e2e::page_objects::audio_probes::speech_events(driver).await?
                ),
                "Listening playback must invoke the Mandarin speech service"
            );
            driver.click("[data-listening-play]").await?;
            let plays = zipeek_e2e::page_objects::audio_probes::mandarin_speaks(
                &zipeek_e2e::page_objects::audio_probes::speech_events(driver).await?,
            )
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
            zipeek_e2e::e2e_ensure!(
                plays.len() >= 2 && plays[plays.len() - 1] == plays[plays.len() - 2],
                "Listening replay must invoke speech with the same Mandarin question"
            );
            let choices = driver
                .find_all_css("[aria-label='Choose the word you heard'] button")
                .await?;
            zipeek_e2e::e2e_ensure_eq!(choices.len(), 3);
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_all_css("[data-listening-choice-id] .text-accent")
                    .await?
                    .len(),
                0,
            );
            choices[0].click().await?;
            driver
                .query_single("[data-listening-session] [role='status']")
                .await?;
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_all_css("[data-listening-choice-id] .text-accent")
                    .await?
                    .len(),
                3,
            );
            driver
                .find_css("[data-listening-continue]")
                .await?
                .click()
                .await?;
            zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
            return Ok(());
        }
        driver
            .find_css("[data-practice-skip]")
            .await?
            .click()
            .await?;
    }
    panic!("Mixed session did not schedule Listening Choice");
}

pub async fn character_hunt_uuid_restore_and_skip(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='character-hunt']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    for _ in 0..5 {
        driver.query_single("[data-practice-local-session]").await?;
        if !driver
            .find_all_css("[data-resumable-character-hunt]")
            .await?
            .is_empty()
        {
            driver.refresh().await?;
            driver
                .query_single("[data-resumable-character-hunt]")
                .await?;
            driver
                .find_all_css("[data-hunt-grid-size] [data-hunt-tile]")
                .await?;
            driver
                .find_css("[data-practice-skip]")
                .await?
                .click()
                .await?;
            zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
            return Ok(());
        }
        driver
            .find_css("[data-practice-skip]")
            .await?
            .click()
            .await?;
    }
    panic!("Mixed session did not schedule Character Hunt");
}

pub async fn ai_practice_requires_model_before_launch(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::SETUP_AI_PRACTICE_REQUIRES_MODEL_BEFORE_LAUNCH_01,
            vec![],
        )
        .await?;
    driver.refresh().await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='sentence-translation']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    let start = driver.find_css("[data-practice-start]").await?;
    zipeek_e2e::e2e_ensure!(start.is_enabled().await? == false);
    driver.query_single("[role='status']").await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .current_url()
            .await?
            .as_str()
            .contains("/practice/session/")
    );
    Ok(())
}

pub async fn ai_sentence_uuid_generates_restores_and_continues(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.configure_ai_fixture("success").await?;
    ctx.open_route("/practice").await?;
    let endpoint = serde_json::to_string(&ctx.ai_url).map_err(|error| error.to_string())?;
    let configure = format!(
        "const settings=JSON.parse(localStorage.getItem('zipeek.preferences.v1')||'{{}}');settings.ai_url={endpoint};settings.ai_model='fixture-model';localStorage.setItem('zipeek.preferences.v1',JSON.stringify(settings));"
    );
    driver.execute(&configure, vec![]).await?;
    driver.refresh().await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='multiple-choice']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='sentence-translation']")
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver.query_single("[data-generated-translation]").await?;
    ctx.checkpoint("ai-uuid-generated").await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    driver.refresh().await?;
    driver.query_single("[data-generated-translation]").await?;
    ctx.checkpoint("ai-uuid-restored").await?;
    let tiles = driver
        .find_all_css("[aria-label='Available translation tiles'] button:not([disabled])")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !tiles.is_empty(),
        "Generated translation must supply selectable tiles"
    );
    for tile in tiles {
        tile.click().await?;
    }
    driver
        .find_css("[data-generated-translation] button:not([disabled])")
        .await?;
    driver
        .find(By::XPath("//button[normalize-space()='Check answer']"))
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-generated-translation] [role='status']")
        .await?;
    ctx.checkpoint("ai-uuid-feedback").await?;
    driver
        .find(By::XPath("//button[normalize-space()='Continue']"))
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
    ctx.checkpoint_visible(
        "ai-translation-feedback-continued",
        "[data-practice-local-session]",
    )
    .await?;
    Ok(())
}

pub async fn sentence_transformation_answer_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    start_single_authored_exercise(
        driver,
        "sentence-transformation",
        "[data-practice-sentence-transformation]",
    )
    .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(
        session_url.as_str().contains("/practice/session/"),
        "Sentence Transformation must use a resumable session"
    );
    let check = driver
        .find_css("[data-sentence-transformation-check]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !check.is_enabled().await?,
        "Cannot submit Sentence Transformation before selecting a tile"
    );
    let tile = driver
        .find_css("[data-transformation-bank-tile]:not([disabled])")
        .await?;
    tile.click().await?;
    driver
        .query_single("[data-sentence-transformation-check]:not([disabled])")
        .await?;
    driver.refresh().await?;
    driver
        .query_single("[data-practice-sentence-transformation]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("[data-sentence-transformation-check]")
            .await?
            .is_enabled()
            .await?,
        "An unsubmitted answer must reset after restoring the session"
    );
    driver
        .find_css("[data-transformation-bank-tile]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-sentence-transformation-check]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-sentence-transformation] [data-practice-feedback]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css("[data-transformation-bank-tile]")
            .await?
            .is_enabled()
            .await?,
        "Submitted Sentence Transformation choices must remain locked"
    );
    driver
        .find_css("[data-sentence-transformation-continue]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
    Ok(())
}

pub async fn audio_dictation_play_and_restore(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    start_single_authored_exercise(
        driver,
        "audio-dictation",
        "[data-practice-authored-exercise]",
    )
    .await?;
    let session_url = driver.current_url().await?;
    zipeek_e2e::e2e_ensure!(session_url.as_str().contains("/practice/session/"));
    let root = driver.find_css("[data-practice-authored-exercise]").await?;
    zipeek_e2e::e2e_ensure!(root.text().await?.contains("Audio Dictation"));
    let response = driver.find_css("#authored-hanzi-response").await?;
    response.send_keys("错误").await?;
    let submit = driver
        .find_css("[data-practice-authored-exercise] button[type='submit']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !submit.is_enabled().await?,
        "Dictation cannot be submitted before hearing the prompt"
    );
    driver.refresh().await?;
    driver
        .query_single("[data-practice-authored-exercise]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("#authored-hanzi-response")
            .await?
            .prop("value")
            .await?
            .unwrap_or_default()
            .is_empty(),
        "Unsubmitted dictation must not survive session restoration"
    );
    driver
        .find(By::XPath(
            "//button[normalize-space()='Play Mandarin audio']",
        ))
        .await?
        .click()
        .await?;
    driver
        .find_css("#authored-hanzi-response")
        .await?
        .send_keys("错误")
        .await?;
    driver
        .find_css("[data-practice-authored-exercise] button[type='submit']:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-authored-exercise] [data-practice-feedback]")
        .await?;
    driver
        .find_css("[data-practice-feedback] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?, session_url);
    Ok(())
}

async fn start_single_authored_exercise(
    driver: &WebDriver,
    exercise_id: &str,
    expected_root: &str,
) -> HarnessResult<()> {
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    driver
        .find_css(format!("[data-practice-exercise='{exercise_id}']"))
        .await?
        .click()
        .await?;
    continue_setup(driver).await?;
    current(driver, "vocabulary").await?;
    continue_setup(driver).await?;
    current(driver, "summary").await?;
    driver
        .find_css("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver.query_single(expected_root).await?;
    Ok(())
}
