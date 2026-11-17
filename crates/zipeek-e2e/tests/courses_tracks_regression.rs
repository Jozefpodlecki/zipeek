use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn three_tracks_have_independent_mosaics(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (route, selector, count, label) in [
        (
            "/courses/vocabulary",
            "[data-vocabulary-region]",
            100,
            "Vocabulary",
        ),
        ("/courses/grammar", "[data-mosaic-region]", 100, "Grammar"),
        ("/courses/pinyin", "[data-pinyin-region]", 6, "Pinyin"),
    ] {
        ctx.open_route(route).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_all_css("[data-course-tracks] a").await?.len(),
            3,
            "Each course page must present the three track links",
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_all_css(selector).await?.len(),
            count,
            "Unexpected number of regions in {label}",
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css("[data-course-tracks] [aria-current='page']")
                .await?
                .len(),
            1,
            "Exactly one course track should be active",
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_all_css("[data-course-workspace]").await?.len(),
            1,
            "Every course track must use the shared workspace",
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_all_css("[data-course-explorer]").await?.len(),
            1,
            "Every track must provide a lower preview",
        );
        let tab_text = driver.query_single("[data-course-tracks]").await?.text().await?;
        zipeek_e2e::e2e_ensure!(
            !tab_text.contains("regions") && !tab_text.contains("lessons"),
            "Track tabs should show names only: {tab_text}",
        );
        ctx.checkpoint(&format!("courses-{}-track", label.to_lowercase()))
            .await?;
    }
    ctx.open_route("/courses").await?;
    driver.query_single("[data-component='vocabulary-courses']").await?;
    ctx.open_route("/courses/grammar").await?;
    driver.query_single("[data-component='courses-page']").await?;
    Ok(())
}

async fn vocabulary_selection_does_not_claim_mastery(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/vocabulary").await?;
    let explorer_before = driver.query_single("[data-vocabulary-explorer]").await?.rect().await?;
    driver
        .query_single("[data-vocabulary-region='vocabulary-2']")
        .await?
        .click()
        .await?;
    let explorer_after = driver.query_single("[data-vocabulary-explorer]").await?.rect().await?;
    zipeek_e2e::e2e_ensure!(
        (explorer_after.height - explorer_before.height).abs() < 2.0,
        "Changing Vocabulary regions must not resize the Explorer",
    );
    let details = driver
        .query_single("[data-vocabulary-details]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        details.contains("Family and kinship"),
        "The selected region must display its actual authored collection: {details}",
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-vocabulary-region='vocabulary-2'][aria-pressed='true']")
            .await?
            .len()
            == 1,
        "Selecting an authored vocabulary region must update selection without awarding mastery",
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-vocabulary-authored='true']")
            .await?
            .len(),
        100,
        "All Vocabulary regions must be authored",
    );
    driver
        .query_single("[data-vocabulary-region='vocabulary-100']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.query_single("[data-course-create-word-set]").await?.is_enabled().await?,
        "All 100 regions must allow Word Set acquisition",
    );
    let mosaic_frame = driver.query_single("[data-vocabulary-mosaic]").await?.rect().await?;
    let mosaic_svg = driver.query_single("[data-vocabulary-mosaic] svg").await?.rect().await?;
    zipeek_e2e::e2e_ensure!(
        (mosaic_frame.height - mosaic_svg.height).abs() < 3.0,
        "Vocabulary artwork must use the full allocated mosaic height",
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver.query_single("[data-vocabulary-mosaic] svg").await?.attr("preserveAspectRatio").await?.as_deref(),
        Some("none"),
        "The SVG region overlay must fill its viewport without empty bands",
    );
    let first_shape = driver.query_single("[data-vocabulary-region='vocabulary-1']").await?;
    zipeek_e2e::e2e_ensure!(
        first_shape.attr("d").await?.unwrap_or_default().contains(" Q "),
        "Vocabulary mosaic should use softly curved rather than rigid polygon edges",
    );
    let large = driver
        .query_single("[data-vocabulary-region='vocabulary-5']")
        .await?
        .rect()
        .await?;
    let small = driver
        .query_single("[data-vocabulary-region='vocabulary-48']")
        .await?
        .rect()
        .await?;
    zipeek_e2e::e2e_ensure!(
        large.width * large.height > 3.0 * small.width * small.height,
        "Larger Vocabulary collections must still occupy visibly more area under minimum-size limits",
    );
    ctx.checkpoint("vocabulary-all-regions-acquirable")
        .await?;
    Ok(())
}

async fn vocabulary_word_set_snapshot_survives_reload(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/vocabulary").await?;
    driver
        .query_single("[data-vocabulary-region='vocabulary-1']")
        .await?
        .click()
        .await?;
    let roadmap = driver.query_single("[aria-label='Word Set acquisition roadmap']").await?;
    zipeek_e2e::e2e_ensure!(
        !roadmap.find_all_css("li").await?.is_empty(),
        "Selected regions must expose the source-backed Word Set roadmap",
    );
    let explorer = driver.query_single("[data-vocabulary-explorer]").await?;
    zipeek_e2e::e2e_ensure_eq!(
        explorer.find_all_css("article").await?.len(),
        4,
        "Explorer must paginate to four vocabulary cards instead of overflowing vertically",
    );
    driver.query_single("[aria-label='Next vocabulary page']").await?.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        explorer.find_all_css("article").await?.len(),
        4,
        "Explorer pagination must preserve its bounded card count",
    );
    driver.query_single("[data-vocabulary-region='vocabulary-2']").await?.click().await?;
    zipeek_e2e::e2e_ensure!(
        driver.query_single("[data-vocabulary-explorer]").await?.text().await?.contains("1/"),
        "Changing regions must reset Explorer paging to its first page",
    );
    driver.query_single("[data-vocabulary-region='vocabulary-1']").await?.click().await?;
    let add = driver.query_single("[data-course-create-word-set]").await?;
    zipeek_e2e::e2e_ensure!(add.is_enabled().await?);
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-vocabulary-details] ul[aria-label='Course vocabulary preview']").await?.is_empty(),
        "The selected-region panel must not include a duplicate long word list",
    );
    add.click().await?;
    driver
        .query_single("[data-course-word-set-existing]")
        .await?;
    driver.refresh().await?;
    driver
        .query_single("[data-course-word-set-existing]")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-course-create-word-set]")
            .await?
            .is_empty(),
        "An existing course-derived Word Set must not be offered a second time",
    );
    ctx.checkpoint("vocabulary-word-set-snapshot-persisted")
        .await?;
    Ok(())
}

async fn grammar_course_has_single_entry(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/grammar").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-mosaic-region]").await?.len(),
        100,
        "Grammar must render exactly 100 grammar-only regions",
    );
    let grammar_shape = driver.query_single("[data-mosaic-region='grammar-1']").await?;
    zipeek_e2e::e2e_ensure!(
        grammar_shape.attr("d").await?.unwrap_or_default().contains(" Q "),
        "Grammar cells must use shared gently curved mosaic boundaries",
    );
    zipeek_e2e::e2e_ensure_eq!(
        grammar_shape.attr("fill-opacity").await?.as_deref(),
        Some("0.27"),
        "The selected grammar shape must retain an artwork preview",
    );
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-mosaic-region] foreignObject, [data-mosaic-region] use").await?.is_empty(),
        "Grammar mosaic shapes must not contain icons",
    );
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-grammar-play]").await?.is_empty(),
        "The redundant inline Grammar player must be absent",
    );
    driver.query_single("[data-mosaic-region='grammar-2']").await?.click().await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-course-details] a").await?.len(),
        1,
        "Each published Grammar region has a single entry link",
    );
    driver.query_single("[data-course-details] a").await?.click().await?;
    driver.query_single("[data-component='course-lesson-shell']").await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-component='app-shell']").await?.is_empty(),
        "The focused lesson must not render global navigation chrome",
    );
    driver.query_single("[data-grammar-point-id]").await?;
    ctx.checkpoint("grammar-single-entry-focused-shell").await?;
    Ok(())
}

#[tokio::test]
async fn courses_tracks_regression() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let tracks = harness
        .run(
            "courses_three_tracks",
            three_tracks_have_independent_mosaics,
        )
        .await;
    let authored = harness
        .run(
            "vocabulary_authored_unpublished",
            vocabulary_selection_does_not_claim_mastery,
        )
        .await;
    let snapshot = harness
        .run(
            "vocabulary_word_set_persistence",
            vocabulary_word_set_snapshot_survives_reload,
        )
        .await;
    let grammar = harness
        .run(
            "grammar_single_entry",
            grammar_course_has_single_entry,
        )
        .await;
    let cleanup = harness.finish().await;
    tracks.and(authored).and(snapshot).and(grammar).and(cleanup)
}
