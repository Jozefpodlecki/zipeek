use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn matching_session_is_focused_and_interactive(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='quick']").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .is_empty()
    );
    driver.click("[data-practice-exercise='matching']").await?;
    driver
        .query_single(
            "[data-practice-exercise='matching'][aria-pressed='true'] [data-exercise-selected]",
        )
        .await?;
    driver
        .click("[data-practice-exercise-stage='configure']")
        .await?;
    driver.click("[data-pair-count='2']").await?;
    driver
        .query_single("[data-pair-count='2'][aria-pressed='true']")
        .await?;
    driver
        .click("[data-practice-continue]:not([disabled])")
        .await?;
    driver
        .click("[data-practice-continue]:not([disabled])")
        .await?;
    driver
        .click("[data-practice-start]:not([disabled])")
        .await?;
    driver
        .query_single("[data-component='practice-session-shell']")
        .await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-component='app-shell'], [data-component='navigation-drawer'], [data-app-footer], [data-component='app-header']").await?.is_empty());
    driver
        .query_single("[data-shell-mode='practice-session']")
        .await?;
    driver
        .query_single("[data-practice-theme-background] img")
        .await?;
    driver
        .query_single("[data-creature-progress] [role='progressbar']")
        .await?;
    driver
        .query_single("[data-creature-progress] .practice-creature-underlay")
        .await?;
    let creature_loaded = driver
        .execute_async(
            "const done = arguments[arguments.length - 1]; const image = document.querySelector('[data-creature-progress] .practice-creature-underlay'); if (!image) return done(false); image.decode().then(() => done(image.naturalWidth > 0)).catch(() => done(false));",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        creature_loaded,
        "Theme-specific creature must decode in the browser"
    );
    let orientation = driver.execute("const theme = document.documentElement.getAttribute('data-theme-preset'); const expected = (theme === 'midnight' || theme === 'maple') ? 'horizontal' : 'vertical'; return document.querySelector('[data-practice-progress-layout]')?.dataset.practiceProgressLayout === expected && document.querySelector('[data-creature-progress]')?.dataset.creatureOrientation === expected;", vec![]).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        orientation,
        "Practice layout must place horizontal creatures below the exercise"
    );
    driver.query_single("[data-practice-leave] svg").await?;
    driver
        .query_single("[data-practice-leave] .sr-only")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-step-count]")
            .await?
            .is_empty()
    );
    let quiz = driver.query_single("[data-local-quiz]").await?;
    let first_column = quiz
        .find_all(By::Css("[aria-label='Prompts'] button"))
        .await?;
    let second_column = quiz
        .find_all(By::Css("[aria-label='Answers'] button"))
        .await?;
    zipeek_e2e::e2e_ensure_eq!(first_column.len(), 2);
    zipeek_e2e::e2e_ensure_eq!(second_column.len(), 2);
    first_column[0].click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        first_column[0].attr("aria-pressed").await?.as_deref(),
        Some("true")
    );
    driver.query_single("[data-matching-selected]").await?;
    zipeek_e2e::e2e_ensure!(quiz.text().await?.contains("Selected:"));
    let mut successful_pair = false;
    for answer in &second_column {
        answer.click().await?;
        if !driver
            .find_all_css("[data-matching-matched]")
            .await?
            .is_empty()
        {
            successful_pair = true;
            break;
        }
        first_column[0].click().await?;
    }
    zipeek_e2e::e2e_ensure!(successful_pair);
    zipeek_e2e::e2e_ensure!(quiz.text().await?.contains("1 of 2 pairs matched"));
    driver
        .query_single("a[href*='/practice']:not([href*='/session/'])")
        .await?;
    ctx.checkpoint("matching-two-pair-session-ui").await?;
    Ok(())
}

#[tokio::test]
async fn matching_session_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run(
            "matching_two_pair_session",
            matching_session_is_focused_and_interactive,
        )
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
