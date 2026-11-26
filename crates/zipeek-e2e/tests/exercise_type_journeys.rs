use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::{WebDriverActions, practice_config};
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

const EXERCISE_TABS: [(&str, &str); 16] = [
    ("write-hanzi", "write-hanzi"),
    ("matching", "matching"),
    ("multiple-choice", "multiple-choice"),
    ("sentence-translation", "translation"),
    ("speaking", "speaking"),
    ("listening-choice", "listening"),
    ("type-pinyin", "type-pinyin"),
    ("character-builder", "character-builder"),
    ("word-mosaics", "word-mosaics"),
    ("character-hunt", "hunt"),
    ("fill-blank", "fill-blank"),
    ("audio-dictation", "audio-dictation"),
    ("sentence-builder", "sentence-builder"),
    ("sentence-transformation", "sentence-transformation"),
    ("spot-mistake", "spot-mistake"),
    ("image-vocabulary", "image"),
];

async fn choose_only(driver: &WebDriver, id: &str) -> HarnessResult<()> {
    for tile in driver.find_all_css("[data-practice-exercise]").await? {
        let selected = tile.attr("aria-pressed").await?.as_deref() == Some("true");
        let target = tile.attr("data-practice-exercise").await?.as_deref() == Some(id);
        if selected != target {
            tile.click().await?;
        }
    }
    driver
        .query_single(&format!(
            "[data-practice-exercise='{id}'][aria-pressed='true']"
        ))
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .len(),
        1,
        "Selecting {id} must not leave another exercise selected"
    );
    Ok(())
}

async fn all_exercise_types_are_selectable_and_configurable(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-exercise]").await?.len(),
        EXERCISE_TABS.len(),
        "Advanced Practice must expose every registered exercise type"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                r#"
            const main = document.querySelector('[data-component="main-content"]');
            const scroller = document.querySelector('[data-practice-exercise-scroll]');
            const last = document.querySelector('[data-practice-exercise="image-vocabulary"]');
            if (!main || !scroller || !last ||
                !['auto', 'scroll'].includes(getComputedStyle(scroller).overflowY) ||
                getComputedStyle(main).overflowY !== 'hidden') return false;
            const setup = document.querySelector('[data-practice-setup]');
            if (!setup || setup.getBoundingClientRect().bottom > innerHeight + 2 ||
                main.scrollHeight > main.clientHeight + 2) return false;
            const needsScroll = scroller.scrollHeight > scroller.clientHeight;
            scroller.scrollTop = scroller.scrollHeight;
            const boundary = scroller.getBoundingClientRect();
            const tile = last.getBoundingClientRect();
            return (!needsScroll || scroller.scrollTop > 0) &&
                tile.bottom <= boundary.bottom + 2 && tile.top >= boundary.top - 2 &&
                main.scrollTop === 0;
            "#,
                vec![],
            )
            .await?
            .convert::<bool>()?,
        "Advanced exercise grid must scroll to the final tile inside its own panel while the main viewport stays fixed"
    );
    driver.query_single("[data-practice-exercise='sentence-translation'][data-external-services='ai'] [data-exercise-service='ai']").await?;
    driver.query_single("[data-practice-exercise='image-vocabulary'][data-external-services='ai,image-generation'] [data-exercise-service='image-generation']").await?;
    for id in [
        "fill-blank",
        "sentence-builder",
        "sentence-transformation",
        "spot-mistake",
    ] {
        driver.query_single(&format!("[data-practice-exercise='{id}'][data-external-services='ai'] [data-exercise-service='ai']")).await?;
    }
    driver
        .query_single("[data-practice-exercise='speaking']")
        .await?;
    driver
        .query_single("[data-practice-exercise='matching'][data-external-services='none']")
        .await?;
    for (id, tab) in EXERCISE_TABS {
        let tile = driver
            .query_single(&format!("[data-practice-exercise='{id}']"))
            .await?;
        let description_id = format!("practice-exercise-description-{id}");
        zipeek_e2e::e2e_ensure_eq!(
            tile.attr("aria-describedby").await?.as_deref(),
            Some(description_id.as_str()),
            "{id} must identify its exercise description"
        );
        zipeek_e2e::e2e_ensure!(
            !driver
                .query_single(&format!("[id='{description_id}']"))
                .await?
                .text()
                .await?
                .trim()
                .is_empty(),
            "{id} must have a nonempty learner-facing description"
        );
        choose_only(driver, id).await?;
        driver
            .query_single("[data-practice-exercise-stage='configure']")
            .await?
            .click()
            .await?;
        let config_tab = driver
            .query_single(&format!("[data-practice-config-tab='{tab}']"))
            .await?;
        config_tab.click().await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .query_single(&format!("[data-practice-config-tab='{tab}']"))
                .await?
                .attr("aria-selected")
                .await?
                .as_deref(),
            Some("true"),
            "{id} configuration tab must become selected"
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css("[data-practice-config-tab][aria-selected='true']")
                .await?
                .len(),
            1,
            "{id} must have exactly one active configuration tab"
        );
        driver
            .query_single("[data-practice-exercise-stage='choose']")
            .await?
            .click()
            .await?;
        driver
            .query_single(&format!(
                "[data-practice-exercise='{id}'][aria-pressed='true']"
            ))
            .await?;
    }
    ctx.checkpoint("all-sixteen-exercise-types-configured")
        .await?;
    Ok(())
}

async fn quick_mode_hides_advanced_exercises(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single(practice_config::QUICK_PATH)
        .await?
        .click()
        .await?;
    let quick = [
        "write-hanzi",
        "matching",
        "multiple-choice",
        "listening-choice",
        "type-pinyin",
        "character-builder",
        "word-mosaics",
        "character-hunt",
    ];
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-exercise]").await?.len(),
        quick.len()
    );
    for (id, _) in EXERCISE_TABS {
        let visible = !driver
            .find_all_css(&format!("[data-practice-exercise='{id}']"))
            .await?
            .is_empty();
        zipeek_e2e::e2e_ensure_eq!(
            visible,
            quick.contains(&id),
            "Quick mode visibility mismatch for {id}"
        );
    }
    ctx.checkpoint("quick-mode-eight-exercise-types").await?;
    Ok(())
}

async fn start_authored(ctx: ScenarioContext, id: &str, root: &str) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    choose_only(driver, id).await?;
    driver
        .query_single("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-continue]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-continue]:not([disabled])")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver.query_single(root).await?;
    Ok(())
}

async fn fill_blank_incorrect_answer_is_reviewed(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "fill-blank", "[data-practice-authored-exercise]").await?;
    let submit = driver
        .query_single("[data-practice-authored-exercise] button[type='submit']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !submit.is_enabled().await?,
        "An empty cloze must not submit"
    );
    driver
        .query_single("#authored-hanzi-response")
        .await?
        .send_keys("zz-invalid-answer")
        .await?;
    driver
        .query_single("[data-practice-authored-exercise] button[type='submit']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-authored-exercise] [data-practice-feedback]")
            .await?
            .text()
            .await?
            .contains("Not quite"),
        "Incorrect fill-in-the-blank response must produce review feedback"
    );
    driver
        .query_single("[data-practice-authored-exercise] input[disabled]")
        .await?;
    driver
        .query_single("[data-practice-authored-exercise] [data-practice-feedback] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-authored-exercise] [data-practice-feedback]")
            .await?
            .is_empty(),
        "Continuing must dismiss the reviewed cloze question"
    );
    Ok(())
}

async fn audio_dictation_requires_playback(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "audio-dictation", "[data-practice-authored-exercise]").await?;
    driver
        .query_single("#authored-hanzi-response")
        .await?
        .send_keys("你好")
        .await?;
    let submit = driver
        .query_single("[data-practice-authored-exercise] button[type='submit']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !submit.is_enabled().await?,
        "Dictation must require audio before checking"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-authored-exercise]")
            .await?
            .text()
            .await?
            .contains("Play Mandarin audio"),
        "Dictation must provide a playback control"
    );
    Ok(())
}

async fn fill_blank_correct_hanzi_is_accepted(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "fill-blank", "[data-practice-authored-exercise]").await?;
    let prompt = driver
        .query_single("[data-practice-authored-exercise]")
        .await?
        .text()
        .await?;
    let answer = if prompt.contains("I bought a book yesterday") {
        "買"
    } else if prompt.contains("I study Chinese every day") {
        "學習"
    } else if prompt.contains("This cup of tea is too hot") {
        "熱"
    } else {
        return Err(format!("Unrecognized authored cloze prompt: {prompt}").into());
    };
    driver
        .query_single("#authored-hanzi-response")
        .await?
        .send_keys(answer)
        .await?;
    driver
        .query_single("[data-practice-authored-exercise] button[type='submit']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-practice-authored-exercise] [data-practice-feedback]")
            .await?
            .text()
            .await?
            .contains("Correct"),
        "Accepted traditional Hanzi spelling must be graded correct"
    );
    driver
        .query_single("[data-practice-authored-exercise] [data-practice-feedback] button")
        .await?
        .click()
        .await?;
    Ok(())
}

async fn audio_dictation_plays_grades_and_continues(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "audio-dictation", "[data-practice-authored-exercise]").await?;
    driver
        .query_single("#authored-hanzi-response")
        .await?
        .send_keys("你好")
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-practice-authored-exercise] button[type='submit']")
            .await?
            .is_enabled()
            .await?,
        "Typing before audio must not bypass the playback prerequisite"
    );
    driver.query(By::XPath("//section[@data-practice-authored-exercise]//button[normalize-space()='Play Mandarin audio']"))
        .single().await?.click().await?;
    driver
        .query_single("[data-practice-authored-exercise] button[type='submit']:not([disabled])")
        .await?
        .click()
        .await?;
    let feedback = driver
        .query_single("[data-practice-authored-exercise] [data-practice-feedback]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        feedback.contains("Correct") || feedback.contains("Not quite"),
        "Dictation must grade the answer after audio playback"
    );
    driver
        .query_single("[data-practice-authored-exercise] [data-practice-feedback] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-authored-exercise] [data-practice-feedback]")
            .await?
            .is_empty(),
        "Continuing must dismiss the dictation review"
    );
    Ok(())
}

async fn sentence_builder_undo_clear_and_grade(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "sentence-builder", "[data-practice-sentence-builder]").await?;
    let check = driver.query_single("[data-sentence-builder-check]").await?;
    zipeek_e2e::e2e_ensure!(
        !check.is_enabled().await?,
        "Empty sentence cannot be graded"
    );
    driver
        .query_first("[data-practice-sentence-builder] button[aria-label^='Add ']")
        .await?
        .click()
        .await?;
    let undo = driver
        .query(By::XPath(
            "//section[@data-practice-sentence-builder]//button[normalize-space()='Undo']",
        ))
        .single()
        .await?;
    zipeek_e2e::e2e_ensure!(
        undo.is_enabled().await?,
        "Undo must enable after choosing a tile"
    );
    undo.click().await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-sentence-builder-check]")
            .await?
            .is_enabled()
            .await?
    );
    for tile in driver
        .find_all_css("[data-practice-sentence-builder] button[aria-label^='Add ']")
        .await?
    {
        tile.click().await?;
    }
    driver
        .query_single("[data-sentence-builder-check]:not([disabled])")
        .await?
        .click()
        .await?;
    let feedback = driver
        .query_single("[data-practice-sentence-builder] [data-practice-feedback]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        feedback.contains("Correct") || feedback.contains("Not quite"),
        "Sentence Builder must grade a complete tile arrangement"
    );
    driver
        .query_single("[data-sentence-builder-continue]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-sentence-builder] [data-practice-feedback]")
            .await?
            .is_empty(),
        "Continuing must dismiss Sentence Builder review"
    );
    Ok(())
}

async fn sentence_transformation_tile_recovery(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(
        ctx,
        "sentence-transformation",
        "[data-practice-sentence-transformation]",
    )
    .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-sentence-transformation-check]")
            .await?
            .is_enabled()
            .await?,
        "Transformation must not grade an empty tile selection"
    );
    driver
        .query_first("[data-transformation-bank-tile]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-sentence-transformation-check]")
            .await?
            .is_enabled()
            .await?
    );
    driver
        .query_first("[data-transformation-selected-tile]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-sentence-transformation-check]")
            .await?
            .is_enabled()
            .await?,
        "Removing the only transformation tile must disable grading"
    );
    driver
        .query_first("[data-transformation-bank-tile]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-sentence-transformation-check]")
        .await?
        .click()
        .await?;
    let feedback = driver
        .query_single("[data-practice-sentence-transformation] [data-practice-feedback]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        feedback.contains("Accepted answer:")
            && (feedback.contains("Correct") || feedback.contains("Not quite")),
        "Transformation review must show grading and its accepted answer"
    );
    driver
        .query_single("[data-sentence-transformation-continue]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-sentence-transformation] [data-practice-feedback]")
            .await?
            .is_empty(),
        "Continuing must dismiss Transformation review"
    );
    Ok(())
}

async fn spot_mistake_requires_detection_and_repair(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    start_authored(ctx, "spot-mistake", "[data-practice-spot-mistake]").await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-spot-mistake-check]")
            .await?
            .is_enabled()
            .await?,
        "Spot the Mistake must require both a selected span and a replacement"
    );
    driver.query_first("[data-practice-spot-mistake] [aria-label='Select the incorrect Chinese phrase'] button").await?.click().await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-spot-mistake-check]")
            .await?
            .is_enabled()
            .await?,
        "Selecting a span alone must not enable the correction check"
    );
    driver
        .query_single("#spot-mistake-repair")
        .await?
        .send_keys("错")
        .await?;
    driver
        .query_single("[data-spot-mistake-check]")
        .await?
        .click()
        .await?;
    let feedback = driver
        .query_single("[data-practice-spot-mistake] [data-practice-feedback]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        feedback.contains("Mistake") && feedback.contains("Correction"),
        "Feedback must distinguish identifying the incorrect span from repairing it"
    );
    driver
        .query_single("[data-practice-spot-mistake] input[disabled]")
        .await?;
    driver
        .query_single("[data-spot-mistake-continue]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-spot-mistake] [data-practice-feedback]")
            .await?
            .is_empty(),
        "Continuing must dismiss Spot the Mistake review"
    );
    Ok(())
}

#[tokio::test]
async fn exercise_type_journeys() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "all-exercise-types-select-configure",
            all_exercise_types_are_selectable_and_configurable,
        )
        .await?;
    harness
        .run(
            "quick-mode-exercise-catalog",
            quick_mode_hides_advanced_exercises,
        )
        .await?;
    harness
        .run(
            "fill-blank-incorrect-feedback",
            fill_blank_incorrect_answer_is_reviewed,
        )
        .await?;
    harness
        .run(
            "fill-blank-accepted-traditional-hanzi",
            fill_blank_correct_hanzi_is_accepted,
        )
        .await?;
    harness
        .run(
            "audio-dictation-playback-gate",
            audio_dictation_requires_playback,
        )
        .await?;
    harness
        .run(
            "audio-dictation-playback-grade-continue",
            audio_dictation_plays_grades_and_continues,
        )
        .await?;
    harness
        .run(
            "sentence-builder-undo",
            sentence_builder_undo_clear_and_grade,
        )
        .await?;
    harness
        .run(
            "sentence-transformation-tile-recovery",
            sentence_transformation_tile_recovery,
        )
        .await?;
    harness
        .run(
            "spot-mistake-detection-repair",
            spot_mistake_requires_detection_and_repair,
        )
        .await?;
    harness.finish().await
}
