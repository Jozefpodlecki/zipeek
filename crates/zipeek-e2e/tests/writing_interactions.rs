use std::time::Duration;
use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::writing};

async fn writing_source_switches_are_exclusive(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/write-hanzi").await?;
    driver.query_single(writing::PAGE).await?;
    let choices = driver.find_all_css(writing::SOURCES).await?;
    zipeek_e2e::e2e_ensure_eq!(
        choices.len(),
        2,
        "Writing vocabulary must have two source modes"
    );
    for index in 0..choices.len() {
        writing::select_source(driver, index).await?;
        let active = driver
            .find_all_css("[aria-label='Vocabulary source'] button[aria-pressed='true']")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            active.len(),
            1,
            "Exactly one writing source must be selected"
        );
    }
    ctx.checkpoint("writing-vocabulary-mode").await?;
    Ok(())
}

async fn writing_search_is_available_without_word_sets(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/write-hanzi").await?;
    driver.query_single(writing::PAGE).await?;
    let search = driver.query(By::XPath("//div[@role='group' and @aria-label='Vocabulary source']//button[normalize-space()='Search vocabulary']")).single().await?;
    search.click().await?;
    let field = driver.query_single(writing::SEARCH).await?;
    zipeek_e2e::e2e_ensure!(
        field.is_displayed().await?,
        "Vocabulary search must be available with no saved Word Sets"
    );
    zipeek_e2e::e2e_ensure!(
        !field
            .attr("placeholder")
            .await?
            .unwrap_or_default()
            .is_empty()
    );
    field.send_keys("hello").await?;
    zipeek_e2e::e2e_ensure_eq!(
        field.prop("value").await?.as_deref(),
        Some("hello"),
        "Vocabulary search must accept input without a saved Word Set"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css(writing::ACTIVE_SOURCE).await?.len(),
        1,
        "Searching must not change the selected vocabulary source"
    );
    Ok(())
}

async fn writing_layout_uses_responsive_tailwind(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.set_window_rect(0, 0, 1280, 800).await?;
    ctx.open_route("/write-hanzi").await?;
    driver.query_single(writing::PAGE).await?;
    let desktop = driver
        .execute(
            zipeek_e2e::js_scripts::learning::WRITING_TAILWIND_DESKTOP_LAYOUT,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        desktop,
        "Wide writing workspace must have bounded independently scrolling panels"
    );
    let mobile_size = ctx.viewport().responsive(390, 844, true).await?;
    let mobile_result: HarnessResult<()> = async {
        driver.query_single(writing::PAGE).await?;
        let width = driver
            .execute(
                zipeek_e2e::js_scripts::learning::WRITING_VIEWPORT_WIDTH,
                vec![],
            )
            .await?
            .convert::<u64>()?;
        zipeek_e2e::e2e_ensure_eq!(
            width,
            u64::from(mobile_size.width),
            "Measured mobile viewport width"
        );
        let mobile = driver
            .execute(
                zipeek_e2e::js_scripts::learning::WRITING_TAILWIND_MOBILE_LAYOUT,
                vec![],
            )
            .await?
            .convert::<bool>()?;
        zipeek_e2e::e2e_ensure!(mobile, "Narrow writing workspace must remain scrollable");
        ctx.checkpoint("writing-tailwind-responsive-layout").await?;
        Ok(())
    }
    .await;
    let restore = ctx.viewport().viewport(1280, 1000, false).await;
    restore?;
    mobile_result
}

async fn hanzi_training_never_accepts_without_reference(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/hanzi-writing").await?;
    driver.query_single(writing::HANZI_TRAINING).await?;
    let assessment = driver
        .query_wait_first(
            writing::HANZI_ASSESSMENT_RESOLVED,
            Duration::from_secs(15),
            Duration::from_millis(200),
        )
        .await?;
    let status = driver
        .query_single(writing::HANZI_TRAINING_STATUS)
        .await?
        .text()
        .await?;
    let phase = assessment
        .attr("data-hanzi-assessment")
        .await?
        .unwrap_or_default();
    match phase.as_str() {
        "ready" => {
            zipeek_e2e::e2e_ensure!(
                driver.is_present(writing::HANZI_NEXT_STROKE).await?,
                "Available training references must display the next start point"
            );
            zipeek_e2e::e2e_ensure!(status.contains("Stroke") || status.contains("stroke"));
        }
        "unavailable" => {
            zipeek_e2e::e2e_ensure!(
                status.contains("reference") || status.contains("stroke geometry"),
                "Missing stroke references must be explained: {status}"
            );
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .query_single(writing::HANZI_TRAINING)
                    .await?
                    .attr("data-hanzi-strokes-completed")
                    .await?
                    .as_deref(),
                Some("0"),
                "Missing references may not advance stroke progress"
            );
        }
        other => return Err(format!("Unexpected writing phase: {other}").into()),
    }
    ctx.checkpoint("hanzi-training-reference-state").await?;
    Ok(())
}

async fn hanzi_guide_toggle_and_character_switch(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/hanzi-writing").await?;
    driver.query_single(writing::HANZI_TRAINING).await?;
    let canvas = driver.query_single(writing::HANZI_CANVAS).await?;
    let original = canvas
        .attr("data-hanzi-character")
        .await?
        .unwrap_or_default();
    let toggle = driver.query_single(writing::HANZI_GUIDE_TOGGLE).await?;
    zipeek_e2e::e2e_ensure_eq!(toggle.attr("aria-pressed").await?.as_deref(), Some("true"));
    toggle.click().await?;
    zipeek_e2e::e2e_ensure_eq!(toggle.attr("aria-pressed").await?.as_deref(), Some("false"));
    zipeek_e2e::e2e_ensure!(
        !driver.is_present(writing::HANZI_GUIDE).await?,
        "Guide must be hidden after toggling it off"
    );
    toggle.click().await?;
    zipeek_e2e::e2e_ensure_eq!(toggle.attr("aria-pressed").await?.as_deref(), Some("true"));
    let targets = driver.find_all_css(writing::HANZI_CHARACTER_CHOICE).await?;
    zipeek_e2e::e2e_ensure!(
        targets.len() >= 2,
        "Need at least two selectable Hanzi targets"
    );
    targets[1].click().await?;
    let selected = driver
        .query_single(writing::HANZI_CANVAS)
        .await?
        .attr("data-hanzi-character")
        .await?
        .unwrap_or_default();
    zipeek_e2e::e2e_ensure!(
        selected != original,
        "Target switch must replace canvas character"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single(writing::HANZI_TRAINING)
            .await?
            .attr("data-hanzi-strokes-completed")
            .await?
            .as_deref(),
        Some("0"),
        "Switching targets must reset completed strokes"
    );
    ctx.checkpoint("hanzi-guide-and-target-switch").await?;
    Ok(())
}

async fn hanzi_rejection_and_reset(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/hanzi-writing").await?;
    let assessment = driver
        .query_wait_first(
            writing::HANZI_ASSESSMENT_RESOLVED,
            Duration::from_secs(15),
            Duration::from_millis(200),
        )
        .await?;
    let phase = assessment
        .attr("data-hanzi-assessment")
        .await?
        .unwrap_or_default();
    if phase == "unavailable" {
        zipeek_e2e::e2e_ensure!(
            !driver.is_present(writing::HANZI_FEEDBACK).await?,
            "No grading feedback may appear without reference data"
        );
        zipeek_e2e::e2e_ensure!(
            driver.is_present(writing::HANZI_RETRY).await?,
            "Failed references must offer retry"
        );
        driver
            .query_single(writing::HANZI_RETRY)
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .query_single(writing::HANZI_TRAINING)
                .await?
                .attr("data-hanzi-strokes-completed")
                .await?
                .as_deref(),
            Some("0"),
            "Retry may not advance progress"
        );
        return Ok(());
    }
    zipeek_e2e::e2e_ensure_eq!(phase.as_str(), "ready");
    driver
        .query_single(writing::HANZI_REPLAY)
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.is_present("[data-hanzi-replay]").await?,
        "Replay should expose stroke animation"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[aria-label='Accepted stroke progress']")
            .await?
            .attr("aria-disabled")
            .await?
            .as_deref(),
        Some("true"),
        "Replay should suspend learner progress presentation"
    );

    let canvas = driver.query_single(writing::HANZI_CANVAS).await?;
    driver
        .action_chain()
        .move_to_element_center(&canvas)
        .click()
        .perform()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[aria-label='Accepted stroke progress']")
            .await?
            .attr("aria-disabled")
            .await?
            .as_deref(),
        Some("false"),
        "Drawing should cancel replay and restore learner progress"
    );
    let feedback = driver
        .query_wait_first(
            writing::HANZI_FEEDBACK,
            Duration::from_secs(5),
            Duration::from_millis(100),
        )
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        feedback.attr("data-hanzi-feedback").await?.as_deref(),
        Some("rejected")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single(writing::HANZI_TRAINING)
            .await?
            .attr("data-hanzi-strokes-completed")
            .await?
            .as_deref(),
        Some("0"),
        "Rejected stroke cannot advance progress"
    );
    driver
        .query_single(writing::HANZI_RESET)
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver.is_present(writing::HANZI_FEEDBACK).await?,
        "Clearing must reset visible grading feedback"
    );
    ctx.checkpoint("hanzi-reject-and-clear").await?;
    Ok(())
}

async fn hanzi_replay_restores_practice_progress(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/hanzi-writing").await?;
    driver.query_single(writing::HANZI_TRAINING).await?;
    let phase = writing::assessment_phase(driver).await?;
    if phase == "unavailable" {
        zipeek_e2e::e2e_ensure!(
            driver.is_present(writing::HANZI_RETRY).await?,
            "An unavailable reference must provide recovery rather than replay"
        );
        return Ok(());
    }
    zipeek_e2e::e2e_ensure_eq!(phase, "ready", "Unexpected reference readiness state");
    let progress_before = driver
        .query_single(writing::HANZI_TRAINING)
        .await?
        .attr("data-hanzi-strokes-completed")
        .await?
        .unwrap_or_default();
    driver
        .query_single(writing::HANZI_REPLAY)
        .await?
        .click()
        .await?;
    driver.query_single("[data-hanzi-replay]").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[aria-label='Accepted stroke progress']")
            .await?
            .attr("aria-disabled")
            .await?
            .as_deref(),
        Some("true"),
    );
    writing::wait_for_replay_to_finish(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[aria-label='Accepted stroke progress']")
            .await?
            .attr("aria-disabled")
            .await?
            .as_deref(),
        Some("false"),
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single(writing::HANZI_TRAINING)
            .await?
            .attr("data-hanzi-strokes-completed")
            .await?
            .unwrap_or_default(),
        progress_before,
        "Watching strokes must not change accepted practice strokes"
    );
    driver
        .query_single(writing::HANZI_REPLAY)
        .await?
        .click()
        .await?;
    driver.query_single("[data-hanzi-replay]").await?;
    driver
        .query_single(writing::HANZI_RESET)
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver.is_present("[data-hanzi-replay]").await?,
        "Clear must cancel replay without waiting for the animation timer"
    );
    ctx.checkpoint("hanzi-replay-completes-and-cancels").await?;
    Ok(())
}

async fn write_hanzi_is_selectable_as_practice_exercise(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='advanced']").await?;
    let tile = driver
        .query_single("[data-practice-exercise='write-hanzi']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(tile.attr("aria-pressed").await?.as_deref(), Some("false"));
    tile.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-practice-exercise='write-hanzi']")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true"),
        "Write Hanzi must participate in the shared Practice exercise selector"
    );
    ctx.checkpoint("write-hanzi-practice-selection").await?;
    Ok(())
}

#[tokio::test]
async fn writing_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "writing-source-selection",
            writing_source_switches_are_exclusive,
        )
        .await?;
    harness
        .run(
            "writing-search-without-word-sets",
            writing_search_is_available_without_word_sets,
        )
        .await?;
    harness
        .run(
            "writing-tailwind-responsive-layout",
            writing_layout_uses_responsive_tailwind,
        )
        .await?;
    harness
        .run(
            "hanzi-training-reference-state",
            hanzi_training_never_accepts_without_reference,
        )
        .await?;
    harness
        .run(
            "hanzi-guide-and-target-switch",
            hanzi_guide_toggle_and_character_switch,
        )
        .await?;
    harness
        .run("hanzi-reject-and-clear", hanzi_rejection_and_reset)
        .await?;
    harness
        .run(
            "hanzi-replay-completes-and-cancels",
            hanzi_replay_restores_practice_progress,
        )
        .await?;
    harness
        .run(
            "write-hanzi-practice-selection",
            write_hanzi_is_selectable_as_practice_exercise,
        )
        .await?;
    harness.finish().await
}
