use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn mcq_session_feedback(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='quick']").await?;
    driver
        .click("[data-practice-exercise='multiple-choice']")
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
        .query_single("[data-practice-theme-background] img")
        .await?;
    driver
        .query_single("[data-creature-progress] .practice-creature-illuminated")
        .await?;
    let image_decoded = driver
        .execute_async(
            "const done = arguments[arguments.length - 1]; const image = document.querySelector('[data-creature-progress] .practice-creature-illuminated'); if (!image) return done(false); image.decode().then(() => done(image.naturalWidth > 0)).catch(() => done(false));",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        image_decoded,
        "Bundled creature illustration must decode in Chrome"
    );
    let animated_fill = driver
        .execute(
            "const rail = document.querySelector('[data-creature-progress][data-animation-engine=animato]'); const fill = rail?.querySelector('[data-creature-reveal]'); const image = rail?.querySelector('.practice-creature-illuminated'); return !!(fill && image && image.getAttribute('src')?.includes('/practice/') && image.complete && image.naturalWidth > 0 && image.dataset.creatureArtwork?.endsWith('.webp') && getComputedStyle(fill).maskImage.includes('linear-gradient') && getComputedStyle(fill).clipPath === 'none');",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        animated_fill,
        "Theme creature requires a feathered Animato-driven reveal without a hard clip"
    );
    let no_rail_line = driver
        .execute(
            "const rail = document.querySelector('[data-creature-progress]'); return !!(rail && !rail.querySelector('.practice-creature-ornament') && rail.querySelector('[data-reveal-treatment=feathered]'));",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        no_rail_line,
        "Portrait and landscape creatures must not have an artificial center line"
    );
    let viewport_is_bounded = driver
        .execute(
            "return document.documentElement.scrollHeight <= document.documentElement.clientHeight + 1 && document.body.scrollHeight <= window.innerHeight + 1;",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        viewport_is_bounded,
        "Practice session must not introduce document-level vertical scrolling"
    );
    driver.query_single("[data-mcq-prompt]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-local-quiz] h2")
            .await?
            .is_empty()
    );
    let theme_creature_matches = driver
        .execute(
            "const theme = document.documentElement.getAttribute('data-theme-preset'); const expected = {lavender:'dragon', jade:'qilin', sunset:'phoenix', midnight:'white-tiger', ink:'crane', maple:'deer'}; const rail = document.querySelector('[data-creature-progress]'); const art = rail?.querySelector('.practice-creature-underlay'); const prefix = location.pathname.startsWith('/zipeek/') ? '/zipeek/practice/' : '/practice/'; return !!(theme && rail && art && rail.getAttribute('data-progress-creature') === expected[theme] && art.complete && art.naturalWidth > 0 && art.getAttribute('src')?.startsWith(prefix) && art.dataset.creatureArtwork === expected[theme] + '.webp');",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        theme_creature_matches,
        "Progress creature must match the active appearance preset"
    );
    let orientation_is_correct = driver
        .execute(
            "const theme = document.documentElement.getAttribute('data-theme-preset'); const isHorizontal = theme === 'midnight' || theme === 'maple'; const rail = document.querySelector('[data-creature-progress]'); const layout = document.querySelector('[data-practice-progress-layout]'); const art = rail?.querySelector('.practice-creature-underlay'); const mask = rail?.querySelector('[data-creature-reveal]'); return !!(theme && art && mask && rail?.dataset.creatureOrientation === (isHorizontal ? 'horizontal' : 'vertical') && layout?.dataset.practiceProgressLayout === (isHorizontal ? 'horizontal' : 'vertical') && (isHorizontal ? art.naturalWidth > art.naturalHeight * 2 : art.naturalHeight > art.naturalWidth * 2) && getComputedStyle(mask).maskImage.includes(isHorizontal ? 'to right' : 'to top') && getComputedStyle(mask).clipPath === 'none');",
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        orientation_is_correct,
        "Creature artwork, layout and reveal must follow the active theme orientation"
    );
    let answers = driver
        .find_all_css("[data-mcq-exercise] [role='group'] button")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(answers.len(), 4);
    answers[0].click().await?;
    driver.query_single("[data-mcq-feedback]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-mcq-exercise] [role='group'] button svg")
            .await?
            .is_empty()
    );
    driver.query_single("[data-mcq-correct]").await?;
    driver.query_single("[data-local-quiz-continue]").await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-mcq-incorrect]").await?.len() <= 1);
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-step-count]")
            .await?
            .is_empty()
    );
    ctx.checkpoint("practice-mcq-feedback-and-dragon-progress")
        .await?;
    Ok(())
}

#[tokio::test]
async fn mcq_session_uses_typed_answer_feedback() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("practice_mcq_answer_feedback", mcq_session_feedback)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
