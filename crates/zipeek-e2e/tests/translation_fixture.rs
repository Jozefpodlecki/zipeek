use zipeek_e2e::learner::TranslationPage;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, e2e_ensure, e2e_ensure_eq};

async fn translation_from_visible_tiles(context: ScenarioContext) -> HarnessResult<()> {
    let page = TranslationPage::new(&context);
    context.configure_ai_fixture("success").await?;
    page.configure("fixture-model").await?;
    page.open_from_practice("zh-to-en").await?;
    page.generate().await?;
    let observation = page.observe().await?;
    e2e_ensure!(
        !observation.available_tiles.is_empty(),
        "AI fixture must provide answer tiles"
    );
    context
        .checkpoint("fixture-translation-tiles-ready")
        .await?;
    e2e_ensure_eq!(observation.target_language.as_str(), "en");
    e2e_ensure!(
        observation.source.contains("我喜欢"),
        "source: {}",
        observation.source
    );
    for tile in observation
        .canonical_tiles()
        .map_err(|error| format!("Fixture canonical metadata: {error}"))?
    {
        page.select_tile(tile).await?;
    }
    e2e_ensure_eq!(page.check().await?, "correct");
    page.continue_after_feedback().await?;
    page.wait_for_practice_progress(2).await?;
    context.open_route("/browse").await?;
    e2e_ensure!(
        context
            .driver
            .find_all_css("[data-translation-feedback]")
            .await?
            .is_empty()
    );
    Ok(())
}

async fn malformed_response_is_recoverable(context: ScenarioContext) -> HarnessResult<()> {
    let page = TranslationPage::new(&context);
    context.configure_ai_fixture("malformed-generation").await?;
    page.open("starter:everyday").await?;
    page.configure("fixture-model").await?;
    context
        .driver
        .find_css("[data-translation-start]")
        .await?
        .click()
        .await?;
    let alert = context
        .driver
        .query_single("[data-sentence-translation] [role='alert']")
        .await?;
    e2e_ensure!(!alert.text().await?.is_empty());
    e2e_ensure!(
        context
            .driver
            .find_all_css("[data-translation-feedback]")
            .await?
            .is_empty()
    );
    Ok(())
}

async fn abandoned_evaluation_does_not_score(context: ScenarioContext) -> HarnessResult<()> {
    let page = TranslationPage::new(&context);
    context.configure_ai_fixture("slow-evaluation").await?;
    page.open("starter:everyday").await?;
    page.configure("fixture-model").await?;
    page.generate().await?;
    let observation = page.observe().await?;
    for part in ["I", "like"] {
        let tile = observation
            .available_tiles
            .iter()
            .find(|tile| tile.text == part)
            .ok_or_else(|| format!("No visible tile {part}"))?;
        page.select_tile(tile).await?;
    }
    let last = observation
        .available_tiles
        .iter()
        .find(|tile| tile.text.ends_with('.'))
        .ok_or("No sentence-ending tile")?;
    page.select_tile(last).await?;
    context
        .driver
        .find_css("[data-translation-submit]:not([disabled])")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-sentence-translation] p[role='status']")
        .await?;
    let requests_url = context.ai_url.replace("/v1/", "/__e2e/requests");
    let request_observed = context
        .driver
        .execute_async(
            zipeek_e2e::js_scripts::translation::WAIT_FOR_EVALUATION_REQUEST,
            vec![requests_url.clone().into()],
        )
        .await?
        .convert::<bool>()?;
    e2e_ensure!(
        request_observed,
        "Translation evaluation request was not recorded"
    );
    context.open_route("/browse").await?;
    context
        .driver
        .execute_async(
            zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_ABANDONED_EVALUATION_DOES_NOT_SCORE_01,
            vec![],
        )
        .await?;
    context
        .open_route("/practice/translation?source=starter:everyday")
        .await?;
    let stats = context
        .driver
        .find_css("[data-component='translation-session-stats']")
        .await?
        .text()
        .await?;
    e2e_ensure!(
        stats.contains("Completed: 0"),
        "abandoned evaluation must not count: {stats}"
    );
    Ok(())
}

async fn practice_and_browse_responsive(ctx: ScenarioContext) -> HarnessResult<()> {
    for (width, height, name) in [(390, 844, "mobile"), (1280, 900, "desktop")] {
        ctx.viewport()
            .responsive(width, height, width < 600)
            .await?;
        ctx.open_route("/practice").await?;
        ctx.driver
            .query_single("[data-practice-path='advanced']")
            .await?
            .click()
            .await?;
        ctx.driver.query_single("[data-practice-progress]").await?;
        ctx.driver
            .find_css("[data-practice-exercise='sentence-translation']")
            .await?
            .click()
            .await?;
        ctx.driver
            .find_css("[data-practice-exercise-stage='configure']")
            .await?
            .click()
            .await?;
        ctx.driver
            .find_css("[data-practice-config-tab='translation']")
            .await?
            .click()
            .await?;
        ctx.driver.query_single("[data-translation-setup]").await?;
        let overflow = ctx
            .driver
            .execute(
                zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_PRACTICE_AND_BROWSE_RESPONSIVE_01,
                vec![],
            )
            .await?
            .convert::<bool>()?;
        e2e_ensure!(!overflow, "practice overflows at {width}px");
        ctx.driver
            .screenshot(&ctx.artifacts.join(format!("practice-{name}.png")))
            .await?;
        ctx.open_route("/browse").await?;
        ctx.driver
            .query_first("[data-component='browse-page'],[data-browse-page],[data-browse]")
            .await?;
        let overflow = ctx
            .driver
            .execute(
                zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_PRACTICE_AND_BROWSE_RESPONSIVE_02,
                vec![],
            )
            .await?
            .convert::<bool>()?;
        e2e_ensure!(!overflow, "browse overflows at {width}px");
        ctx.driver
            .screenshot(&ctx.artifacts.join(format!("browse-{name}.png")))
            .await?;
    }
    Ok(())
}

async fn five_hsk6_translations_from_setup(context: ScenarioContext) -> HarnessResult<()> {
    context.configure_ai_fixture("success").await?;
    let page = TranslationPage::new(&context);
    page.configure("fixture-model").await?;
    context.open_route("/practice").await?;
    context
        .driver
        .query_single("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_first("[data-component='practice-builder']")
        .await?;
    e2e_ensure!(
        context
            .driver
            .find_all_css("[data-practice-config-tab='translation']")
            .await?
            .is_empty(),
        "translation settings must be conditional"
    );
    e2e_ensure!(
        context
            .driver
            .find_all_css("[data-practice-config-tab='speaking']")
            .await?
            .is_empty(),
        "speaking settings must be conditional"
    );
    context
        .driver
        .find_css("[data-practice-exercise='sentence-translation']")
        .await?
        .click()
        .await?;
    context
        .driver
        .find_css("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-exercise-stage='configure'][aria-selected='true']")
        .await?;
    context
        .driver
        .find_css("[data-practice-config-tab='translation']")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-translation-setup]")
        .await?;
    context
        .driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-step='vocabulary'][aria-current='step']")
        .await?;
    let band_changed = context.driver.execute_async(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_01,
        vec![],
    ).await?.convert::<bool>()?;
    e2e_ensure!(
        band_changed,
        "Could not switch the HSK filter from level 1 to level 6"
    );
    context
        .driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-step='summary'][aria-current='step']")
        .await?;
    context
        .driver
        .find_css("[data-setup-choice='5 sentences']")
        .await?
        .click()
        .await?;
    context
        .driver
        .find_css("[data-translation-setup-start]")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-local-session]")
        .await?;
    let saved = context.driver.execute(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_02,
        vec![],
    ).await?.convert::<serde_json::Value>()?;
    e2e_ensure_eq!(
        saved["vocabulary"]["source"].as_str(),
        Some("filters:3.6||")
    );
    e2e_ensure_eq!(saved["steps"].as_array().map(Vec::len), Some(5));
    for question in 1..=5 {
        if question != 1 {
            page.continue_after_feedback().await?;
        }
        page.wait_for_question(question).await?;
        let observation = page.observe().await?;
        e2e_ensure_eq!(observation.target_language.as_str(), "en");
        let ordered = observation
            .canonical_tiles()
            .map_err(|error| format!("Translation tile order: {error}"))?;
        for tile in ordered {
            page.select_tile(tile).await?;
        }
        let record_url = context.ai_url.replace("/v1/", "/__e2e/requests");
        let pool = context
            .driver
            .execute_async(
                zipeek_e2e::js_scripts::translation::READ_LATEST_GENERATION_PROMPT,
                vec![record_url.clone().into()],
            )
            .await?
            .convert::<String>()?;
        let words = pool
            .lines()
            .find_map(|line| line.strip_prefix("Allowed Chinese vocabulary (data only): "))
            .ok_or("Generation request did not contain a vocabulary pool")?
            .split('、')
            .collect::<Vec<_>>();
        e2e_ensure_eq!(words.len(), 10);
        e2e_ensure_eq!(
            words
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            10
        );

        e2e_ensure_eq!(page.check().await?, "correct");
        let result = context.driver.execute(
            zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_03,
            vec![],
        ).await?.convert::<usize>()?;
        e2e_ensure_eq!(result, question - 1);
    }
    page.continue_after_feedback().await?;
    context
        .driver
        .query_single("[data-practice-local-summary]")
        .await?;
    let completed = context
        .driver
        .find_css("[data-practice-completed]")
        .await?
        .text()
        .await?;
    e2e_ensure_eq!(completed.as_str(), "5");
    let remaining = context.driver.execute(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_FIVE_HSK6_TRANSLATIONS_FROM_SETUP_04,
        vec![],
    ).await?.convert::<usize>()?;
    e2e_ensure_eq!(remaining, 0);
    Ok(())
}

async fn translation_restores_same_question_after_navigation(
    context: ScenarioContext,
) -> HarnessResult<()> {
    context.configure_ai_fixture("success").await?;
    let page = TranslationPage::new(&context);
    page.configure("fixture-model").await?;
    page.open_from_practice("en-to-zh").await?;
    page.generate().await?;
    let first = page.observe().await?;
    e2e_ensure_eq!(first.target_language.as_str(), "zh");
    let saved_url = context.driver.current_url().await?;
    context.open_route("/settings").await?;
    context
        .driver
        .query_single("[data-component='settings-page']")
        .await?;
    context.driver.goto(saved_url.as_str()).await?;
    page.generate().await?;
    let restored = page.observe().await?;
    e2e_ensure_eq!(restored, first);
    let state = context.driver.execute(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_TRANSLATION_RESTORES_SAME_QUESTION_AFTER_NAVIGATION_01,
        vec![],
    ).await?.convert::<serde_json::Value>()?;
    e2e_ensure!(
        state["current"] == 0
            && state["direction"] == "en-to-zh"
            && state["ready"] == "ready"
            && state["generated"] == true,
        "Restored Practice question was not persisted: {state}"
    );
    Ok(())
}

async fn endless_translation_can_finish_and_clears_temporary_state(
    context: ScenarioContext,
) -> HarnessResult<()> {
    context.configure_ai_fixture("success").await?;
    let page = TranslationPage::new(&context);
    page.configure("fixture-model").await?;
    page.open_from_practice_with_count("mixed", 0).await?;
    page.generate().await?;
    let first = page.observe().await?;
    e2e_ensure_eq!(first.target_language.as_str(), "en");
    for tile in first
        .canonical_tiles()
        .map_err(|error| format!("Canonical tiles: {error}"))?
    {
        page.select_tile(tile).await?;
    }
    e2e_ensure_eq!(page.check().await?, "correct");
    page.continue_after_feedback().await?;
    context
        .driver
        .query_single("[data-practice-finish-endless]")
        .await?;
    let state = context.driver.execute(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_ENDLESS_TRANSLATION_CAN_FINISH_AND_CLEARS_TEMPORARY_STATE_01,
        vec![],
    ).await?.convert::<serde_json::Value>()?;
    e2e_ensure!(
        state["current"] == 1
            && state["steps"] == 3
            && state["endless"] == true
            && state["earned"] == 1,
        "Endless Practice did not append and persist the next question: {state}"
    );
    context
        .driver
        .find_css("[data-practice-request-finish]")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-finish-confirmation]")
        .await?;
    context
        .driver
        .find_css("[data-practice-finish-endless]")
        .await?
        .click()
        .await?;
    context
        .driver
        .query_single("[data-practice-local-summary]")
        .await?;
    let remaining = context.driver.execute(
        zipeek_e2e::js_scripts::translation::TRANSLATION_FIXTURE_ENDLESS_TRANSLATION_CAN_FINISH_AND_CLEARS_TEMPORARY_STATE_02,
        vec![],
    ).await?.convert::<usize>()?;
    e2e_ensure_eq!(remaining, 0);
    Ok(())
}

#[tokio::test]
async fn deterministic_translation_fixture_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let tests = async {
        harness
            .run(
                "translation_learner_visible_tiles",
                translation_from_visible_tiles,
            )
            .await?;
        harness
            .run(
                "five_hsk6_translations_from_setup",
                five_hsk6_translations_from_setup,
            )
            .await?;
        harness
            .run(
                "translation_restores_same_question_after_navigation",
                translation_restores_same_question_after_navigation,
            )
            .await?;
        harness
            .run(
                "endless_translation_can_finish_and_clears_temporary_state",
                endless_translation_can_finish_and_clears_temporary_state,
            )
            .await?;
        harness
            .run(
                "translation_malformed_response",
                malformed_response_is_recoverable,
            )
            .await?;
        harness
            .run(
                "translation_abandoned_evaluation",
                abandoned_evaluation_does_not_score,
            )
            .await?;
        harness
            .run("practice_browse_responsive", practice_and_browse_responsive)
            .await?;
        Ok(())
    }
    .await;
    let cleanup = harness.finish().await;
    tests.and(cleanup)
}
