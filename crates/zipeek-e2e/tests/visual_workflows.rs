use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn illustrated_courses_have_visible_artwork(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (route, image, tile) in [
        (
            "/courses/vocabulary",
            "vocabulary-peony-garden.png",
            "[data-vocabulary-region]",
        ),
        (
            "/courses/grammar",
            "grammar-autumn-bridge.png",
            "[data-mosaic-region]",
        ),
        (
            "/courses/pinyin",
            "pinyin-landscape.png",
            "[data-pinyin-region]",
        ),
    ] {
        ctx.open_route(route).await?;
        let source = driver.query_single("svg image").await?.attr("href").await?;
        zipeek_e2e::e2e_ensure!(
            source
                .as_deref()
                .is_some_and(|source| source.ends_with(image)),
            "Track {route} must use its own artwork: {source:?}"
        );
        let region = driver.query_single(tile).await?;
        let opacity = region
            .attr("fill-opacity")
            .await?
            .ok_or("Missing mosaic cover opacity")?
            .parse::<f32>()
            .map_err(|error| format!("Invalid mosaic cover opacity for {route}: {error}"))?;
        zipeek_e2e::e2e_ensure!(
            opacity < 0.9 && opacity > 0.5,
            "The initial illustration must be discernible without implying a completed region: {route} = {opacity}"
        );
        driver
            .query_single("[data-course-tracks] [aria-current='page']")
            .await?;
        ctx.checkpoint(&format!("illustrated-{}", route.replace('/', "-")))
            .await?;
    }
    Ok(())
}

async fn browse_and_practice_layout_contract(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='quick']").await?;
    driver
        .query_single("[data-practice-exercise-stage='choose']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .is_empty(),
        "Practice setup must not preselect exercises"
    );
    driver
        .query_single("[data-practice-exercise='matching']")
        .await?
        .click()
        .await?;
    driver
        .query_single(
            "[data-practice-exercise='matching'][aria-pressed='true'] [data-exercise-selected]",
        )
        .await?;
    ctx.checkpoint("practice-setup-selection-legibility")
        .await?;
    ctx.open_route("/browse?query=one").await?;
    driver
        .query_single("[data-component='browse-results-list']")
        .await?;
    driver.query_single("[data-browse-headword]").await?;
    ctx.checkpoint("browse-compact-headword-layout").await?;
    Ok(())
}

#[tokio::test]
async fn visual_workflows() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let courses = harness
        .run(
            "courses_visible_mosaic_art",
            illustrated_courses_have_visible_artwork,
        )
        .await;
    let layout = harness
        .run(
            "browse_and_practice_selection",
            browse_and_practice_layout_contract,
        )
        .await;
    let cleanup = harness.finish().await;
    courses.and(layout).and(cleanup)
}
