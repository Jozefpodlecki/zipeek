use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn configure_game(ctx: &ScenarioContext, route: &str) -> HarnessResult<()> {
    ctx.configure_ai_fixture("success").await?;
    ctx.open_route(route).await?;
    ctx.configure_image_exercise_fixtures().await?;
    ctx.driver
        .query_single("[data-game-start]:not([disabled])")
        .await?
        .click()
        .await?;
    Ok(())
}

async fn sentence_relay_answers_all_rounds(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    configure_game(&ctx, "/games/sentence-relay").await?;
    driver
        .query_single("[data-ai-sentence-game] [data-sentence-tile]")
        .await?;
    ctx.assert_ai_request_kind("sentence-relay").await?;

    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-sentence-check]")
            .await?
            .is_enabled()
            .await?,
        "A sentence cannot be checked without choosing all its tiles"
    );
    for index in [0, 1, 2] {
        driver
            .query_single(format!("[data-sentence-tile='{index}']"))
            .await?
            .click()
            .await?;
    }
    driver
        .query_single("[data-sentence-check]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-ai-sentence-game]")
            .await?
            .text()
            .await?
            .contains("Not quite"),
        "Wrong sentence order must show review feedback"
    );
    driver
        .query(By::XPath(
            "//section[@data-ai-sentence-game]//button[normalize-space()='Clear']",
        ))
        .single()
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-sentence-check]")
            .await?
            .is_enabled()
            .await?,
        "Clearing a rejected attempt must restore the empty-answer gate"
    );
    for round in 0..4 {
        for index in [1, 2, 0] {
            driver
                .query_single(format!("[data-sentence-tile='{index}']:not([disabled])"))
                .await?
                .click()
                .await?;
        }
        driver
            .query_single("[data-sentence-check]:not([disabled])")
            .await?
            .click()
            .await?;
        let expected = format!("{} / 4 sentences", round + 1);
        driver
            .query_with_text("[data-ai-sentence-game] [data-game-progress]", &expected)
            .await?;
    }
    driver
        .query_single("[data-ai-sentence-game] [data-game-complete]")
        .await?;
    ctx.checkpoint("sentence-relay-all-four-rounds").await?;
    driver
        .query(By::XPath(
            "//section[@data-ai-sentence-game]//button[normalize-space()='Play again']",
        ))
        .single()
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-ai-sentence-game] [data-sentence-tile]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-ai-sentence-game] [data-game-complete]")
            .await?
            .is_empty(),
        "Restart must return Sentence Relay to a playable question"
    );
    Ok(())
}

async fn dialogue_quest_answers_all_rounds(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    configure_game(&ctx, "/games/dialogue-quest").await?;
    driver
        .query_single("[data-dialogue-quest] [data-dialogue-choice]")
        .await?;
    ctx.assert_ai_request_kind("dialogue-quest").await?;
    driver
        .query_single("[data-dialogue-choice='1']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-dialogue-quest]")
            .await?
            .text()
            .await?
            .contains("Try another response"),
        "Wrong dialogue choice must show retry feedback"
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single("[data-dialogue-choice='1']")
            .await?
            .is_enabled()
            .await?,
        "Rejected dialogue choice must be disabled until the learner advances"
    );
    for round in 0..4 {
        driver
            .query_single("[data-dialogue-choice='0']:not([disabled])")
            .await?
            .click()
            .await?;
        let expected = format!("{} / 4 conversations", round + 1);
        driver
            .query_with_text("[data-dialogue-quest] [data-game-progress]", &expected)
            .await?;
    }
    driver
        .query_single("[data-dialogue-quest] [data-game-complete]")
        .await?;
    ctx.checkpoint("dialogue-quest-all-four-rounds").await?;
    driver
        .query(By::XPath(
            "//section[@data-dialogue-quest]//button[normalize-space()='Play again']",
        ))
        .single()
        .await?
        .click()
        .await?;
    driver.query_single("[data-dialogue-choice='0']").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-dialogue-quest] [data-game-complete]")
            .await?
            .is_empty(),
        "Restart must return Dialogue Quest to a playable conversation"
    );
    Ok(())
}

async fn ai_game_service_failure_is_visible(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.configure_ai_fixture("server-error").await?;
    ctx.open_route("/games/dialogue-quest").await?;
    ctx.configure_image_exercise_fixtures().await?;
    driver
        .query_single("[data-game-start]:not([disabled])")
        .await?
        .click()
        .await?;
    let alert = driver.query_single("[role='alert']").await?;
    zipeek_e2e::e2e_ensure!(
        !alert.text().await?.trim().is_empty(),
        "AI game service failure must be visible to the learner"
    );
    driver
        .query_single("[data-game-setup='Dialogue Quest']")
        .await?;
    ctx.checkpoint("dialogue-quest-ai-error-recovery").await?;
    Ok(())
}

async fn sentence_relay_invalid_generation_is_visible(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.configure_ai_fixture("malformed-generation").await?;
    ctx.open_route("/games/sentence-relay").await?;
    ctx.configure_image_exercise_fixtures().await?;
    driver
        .query_single("[data-game-start]:not([disabled])")
        .await?
        .click()
        .await?;
    let error = driver.query_single("[role='alert']").await?.text().await?;
    zipeek_e2e::e2e_ensure!(
        error.contains("invalid JSON") || error.contains("required sentence"),
        "Invalid AI sentence JSON must show actionable generation feedback: {error}"
    );
    driver
        .query_single("[data-game-setup='Sentence Relay']")
        .await?;
    ctx.checkpoint("sentence-relay-invalid-generation").await?;
    Ok(())
}

#[tokio::test]
async fn ai_game_journeys() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "sentence-relay-graded-completion-restart",
            sentence_relay_answers_all_rounds,
        )
        .await?;
    harness
        .run(
            "dialogue-quest-wrong-answer-completion-restart",
            dialogue_quest_answers_all_rounds,
        )
        .await?;
    harness
        .run(
            "dialogue-quest-service-error",
            ai_game_service_failure_is_visible,
        )
        .await?;
    harness
        .run(
            "sentence-relay-malformed-generation",
            sentence_relay_invalid_generation_is_visible,
        )
        .await?;
    harness.finish().await
}
