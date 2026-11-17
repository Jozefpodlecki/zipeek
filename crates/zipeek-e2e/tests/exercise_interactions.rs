#[path = "suites/practice/word_mosaics.rs"]
mod word_mosaics_suite;

use thirtyfour::prelude::*;
use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, exercises},
};

async fn character_hunt_hint_progress(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/character-hunt?source=hsk&standard=3&level=1&rounds=1")
        .await?;
    driver.element(exercises::HUNT_PAGE).await?;
    let game = driver.element(exercises::HUNT_SESSION).await?;
    zipeek_e2e::e2e_ensure!(game.is_displayed().await?);
    let size = exercises::hunt_dimension(driver).await?;
    zipeek_e2e::e2e_ensure!(
        (3..=12).contains(&size),
        "Invalid hunt board dimension: {size}"
    );
    let hint = driver.element(exercises::HUNT_HINT).await?;
    zipeek_e2e::e2e_ensure!(
        hint.is_enabled().await?,
        "Initial Character Hunt should offer a hint"
    );
    hint.click().await?;
    driver.query_single("[data-hunt-progress-panel]").await?;
    ctx.checkpoint("character-hunt-hint").await?;
    Ok(())
}

async fn image_vocabulary_selection_is_exclusive(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/image-vocabulary").await?;
    driver.element(exercises::IMAGE).await?;
    for mode in [
        exercises::ImageAnswerMode::OneOfThree,
        exercises::ImageAnswerMode::SelectAll,
        exercises::ImageAnswerMode::OneOfThree,
    ] {
        exercises::choose_image_mode(driver, mode).await?;
    }
    ctx.checkpoint("image-vocabulary-selection-mode").await?;
    Ok(())
}

async fn image_vocabulary_is_selectable_and_image_fixture_is_reachable(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='advanced']").await?;
    let tile = driver
        .query_single("[data-practice-exercise='image-vocabulary']")
        .await?;
    tile.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-practice-exercise='image-vocabulary']")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true"),
        "Picture Vocabulary must be selectable from the Practice setup"
    );
    ctx.configure_image_fixture().await?;
    ctx.assert_image_fixture_reachable().await?;
    ctx.assert_image_fixture_generates_image().await?;
    ctx.checkpoint("picture-vocabulary-image-server").await?;
    Ok(())
}

async fn image_vocabulary_completes_and_regenerates(ctx: ScenarioContext) -> HarnessResult<()> {
    use zipeek_e2e::page_objects::image_vocabulary;

    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    ctx.configure_image_exercise_fixtures().await?;
    ctx.open_route("/practice/image-vocabulary?standard=3&level=1")
        .await?;
    driver.element(exercises::IMAGE).await?;
    image_vocabulary::wait_for_review(driver).await?;
    ctx.assert_ai_request_kind("image-scene").await?;
    image_vocabulary::approve(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css(image_vocabulary::ANSWERS).await?.len(),
        3,
        "One-of-three mode must present three answers"
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single(image_vocabulary::CHECK)
            .await?
            .is_enabled()
            .await?,
        "Checking an empty answer must be disabled"
    );
    zipeek_e2e::e2e_ensure_eq!(image_vocabulary::select_options(driver, 1).await?, 3);
    let result = image_vocabulary::check(driver).await?;
    zipeek_e2e::e2e_ensure!(
        result.contains("Correct!") || result.contains("Not quite."),
        "Picture Vocabulary must display grading feedback, got {result:?}"
    );
    zipeek_e2e::e2e_ensure!(
        image_vocabulary::all_options_disabled(driver).await?,
        "Checking an answer must lock the options"
    );
    ctx.checkpoint("picture-vocabulary-answer-graded").await?;
    image_vocabulary::regenerate(driver).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-image-answer-choices]")
            .await?
            .is_empty(),
        "Regeneration must require reviewing and approving a fresh image"
    );
    ctx.checkpoint("picture-vocabulary-regenerated").await?;

    exercises::choose_image_mode(driver, exercises::ImageAnswerMode::SelectAll).await?;
    image_vocabulary::wait_for_review(driver).await?;
    image_vocabulary::approve(driver).await?;
    let count = image_vocabulary::select_options(driver, 2).await?;
    zipeek_e2e::e2e_ensure!(
        (4..=6).contains(&count),
        "Select-all mode must present four to six choices, got {count}"
    );
    let result = image_vocabulary::check(driver).await?;
    zipeek_e2e::e2e_ensure!(
        result.contains("Correct!") || result.contains("Not quite."),
        "Select-all mode must display grading feedback, got {result:?}"
    );
    ctx.checkpoint("picture-vocabulary-select-all-graded")
        .await?;
    Ok(())
}

async fn listening_requires_sufficient_word_set(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/listening").await?;
    let root = driver.element(exercises::LISTENING).await?;
    zipeek_e2e::e2e_ensure!(root.is_displayed().await?);
    let start = driver
        .query(By::XPath(
            "//section[@data-listening-choice]//button[normalize-space()='Start listening']",
        ))
        .single()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !start.is_enabled().await?,
        "An empty browser has no playable listening Word Set"
    );
    zipeek_e2e::e2e_ensure!(root.text().await?.contains("Create a Word Set"));
    ctx.checkpoint("listening-empty-word-set").await?;
    Ok(())
}

#[tokio::test]
async fn exercise_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("character-hunt-hint", character_hunt_hint_progress)
        .await?;
    harness
        .run(
            "image-vocabulary-selection",
            image_vocabulary_selection_is_exclusive,
        )
        .await?;
    harness
        .run(
            "picture-vocabulary-server-reachable",
            image_vocabulary_is_selectable_and_image_fixture_is_reachable,
        )
        .await?;
    harness
        .run(
            "picture-vocabulary-playable-completion",
            image_vocabulary_completes_and_regenerates,
        )
        .await?;
    harness
        .run(
            "listening-requires-vocabulary",
            listening_requires_sufficient_word_set,
        )
        .await?;
    harness
        .run(
            "word-mosaics-round-complete",
            word_mosaics_suite::complete_word_mosaics_round,
        )
        .await?;
    harness.finish().await
}
