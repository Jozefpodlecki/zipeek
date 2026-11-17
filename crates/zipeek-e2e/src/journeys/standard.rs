use super::capture_journey_step;
use crate::page_objects::{self, WebDriverActions};
use crate::{HarnessResult, ScenarioContext};

pub async fn exercise_five_scored_rounds(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    for choice in ["hanzi-to-meaning", "hanzi-eng"] {
        driver
            .find_css(format!("[data-setup-choice='{choice}']"))
            .await?
            .click()
            .await?;
    }
    driver
        .find_css("[data-setup-choice='Fixed']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-setup-choice='rounds-5']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-start]")
        .await?
        .click()
        .await?;

    let mut completed = 0_usize;
    for index in 0..5 {
        driver
            .query_single("[data-practice-state='ready'][data-practice-variant]")
            .await?;
        let variant = driver
            .find_css("[data-practice-variant]")
            .await?
            .attr("data-practice-variant")
            .await?
            .ok_or("Mixed round has no exercise type")?;
        match variant.as_str() {
            "hanzi-to-meaning" => {
                page_objects::mcq::ready(driver).await?;
                let correct = page_objects::round::mcq_correct_index(driver).await?;
                page_objects::mcq::option(driver, correct)
                    .await?
                    .click()
                    .await?;
                driver.query_single("[data-mcq-answer-feedback]").await?;
                if !driver
                    .find_all_css("[data-mcq-option]:not([disabled])")
                    .await?
                    .is_empty()
                {
                    return Err(
                        "Scored MCQ options stayed enabled after selecting a response".into(),
                    );
                }
                driver
                    .query_single("[data-mcq-continue]:not([disabled])")
                    .await?
                    .click()
                    .await?;
            }
            "hanzi-eng" => {
                driver.query_single("[data-matching]").await?;
                let pairs = page_objects::round::matching_pairs(driver).await?;
                if pairs.len() != 4 {
                    return Err(
                        format!("Matching round had {} rather than 4 pairs", pairs.len()).into(),
                    );
                }
                for (left, right) in pairs.into_iter().enumerate() {
                    page_objects::matching::click_card(
                        driver,
                        page_objects::matching::Side::Left,
                        left,
                    )
                    .await?;
                    page_objects::matching::click_card(
                        driver,
                        page_objects::matching::Side::Right,
                        right,
                    )
                    .await?;
                }
                page_objects::matching::continue_round(driver).await?;
            }
            other => return Err(format!("Unexpected scored exercise type: {other}").into()),
        }
        completed += 1;
        capture_journey_step(
            ctx,
            &format!("practice-scored-{completed:02}"),
            seed,
            "standard",
        )
        .await?;
        if index < 4 {
            driver
                .query_single("[data-practice-state='ready'][data-practice-variant]")
                .await?;
        }
    }
    driver.query_single("[data-practice-summary]").await?;
    let score = driver
        .query_single("[data-summary-first-attempt]")
        .await?
        .text()
        .await?;
    if score.trim() != "14 / 14" {
        return Err(
            format!("Five fully correct mixed rounds should score 14 / 14, got {score:?}").into(),
        );
    }
    let mistakes = driver
        .query_single("[data-summary-mistakes]")
        .await?
        .text()
        .await?;
    if mistakes.trim() != "0" {
        return Err(format!(
            "Five fully correct mixed rounds should have zero mistakes, got {mistakes:?}"
        )
        .into());
    }
    capture_journey_step(ctx, "practice-scored-summary", seed, "standard").await?;
    Ok(())
}

pub async fn interact_word_sets(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let name = format!("Full run {seed}");
    page_objects::word_sets::create(&ctx.driver, &name).await?;
    capture_journey_step(ctx, "wordset-created", seed, "standard").await
}

pub async fn interact_dictionary(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    page_objects::browse::set_search_query(&ctx.driver, "你好").await?;
    page_objects::browse::wait_for_search_query(&ctx.driver, "你好", Some("你好"), 1).await?;
    capture_journey_step(ctx, "dictionary-searched", seed, "standard").await
}

pub async fn interact_hanzi_writer(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let driver = &ctx.driver;
    page_objects::writing::select_source(driver, 1).await?;
    let search = driver.query_single(page_objects::writing::SEARCH).await?;
    search.send_keys("hello").await?;
    if search.prop("value").await?.as_deref() != Some("hello") {
        return Err("Hanzi writer search did not accept the text".into());
    }
    capture_journey_step(ctx, "hanzi-writer-search", seed, "standard").await
}

pub async fn interact_games(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .query_single(page_objects::games::PROVINCE_FEATURED_CARD)
        .await?
        .click()
        .await?;
    driver
        .query_single(page_objects::games::PROVINCE_SETUP)
        .await?;
    capture_journey_step(ctx, "games-province-setup", seed, "standard").await?;
    let levels = driver
        .find_all_css("[data-game-setup='Province Quest'] button[aria-pressed]")
        .await?;
    if levels.len() != 6 {
        return Err(format!(
            "Province Quest difficulty and support controls changed: {}",
            levels.len()
        )
        .into());
    }
    levels[0].click().await?;
    levels[4].click().await?;
    driver
        .query_single("[data-game-setup='Province Quest'] button:not([aria-pressed])")
        .await?
        .click()
        .await?;
    for round in 0..10 {
        if round % 2 == 0 {
            driver
                .query_first("[data-province-map] [data-region]")
                .await?
                .click()
                .await?;
        } else {
            let highlighted = driver
                .find_all_css("[data-province-map] path[fill='#eab308']")
                .await?;
            if highlighted.len() != 1 {
                return Err(format!(
                    "Province Quest round {} has {} highlighted regions",
                    round + 1,
                    highlighted.len()
                )
                .into());
            }
            let correct = highlighted[0]
                .attr("data-region")
                .await?
                .ok_or("Highlighted region has no ID")?;
            let choices = driver
                .find_all_css("[data-province-choices] button")
                .await?;
            if choices.len() != 3 {
                return Err(format!(
                    "Province Quest expected three Hanzi choices, got {}",
                    choices.len()
                )
                .into());
            }
            let mut answered = false;
            for choice in choices {
                if choice.attr("data-province-choice").await?.as_deref() != Some(correct.as_str()) {
                    choice.click().await?;
                    answered = true;
                    break;
                }
            }
            if !answered {
                return Err("Province Quest has no incorrect answer to assess".into());
            }
        }
        let review = driver.query_single("[data-province-review]").await?;
        if round % 2 == 1 && !review.text().await?.contains("Not quite") {
            return Err(format!(
                "Province Quest did not grade wrong answer in round {}",
                round + 1
            )
            .into());
        }
        driver
            .query_single("[data-province-review] button")
            .await?
            .click()
            .await?;
        if round == 4 {
            capture_journey_step(ctx, "games-province-midpoint", seed, "standard").await?;
        }
    }
    let completed = driver
        .query_single("[data-province-quest] h1")
        .await?
        .text()
        .await?;
    if completed != "Province Quest complete" {
        return Err(format!("Province Quest did not complete: {completed}").into());
    }
    capture_journey_step(ctx, "games-province-scored-complete", seed, "standard").await?;
    Ok(())
}

pub async fn interact_courses(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    page_objects::courses::open_lesson(&ctx.driver, "initials").await?;
    capture_journey_step(ctx, "courses-pinyin-lesson", seed, "standard").await?;
    Ok(())
}

pub async fn interact_settings(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .query_single("[data-settings-tab-button='images']")
        .await?
        .click()
        .await?;
    let image = driver.query_single("[data-image-server-url]").await?;
    image.clear().await?;
    image.send_keys(&ctx.image_url).await?;
    driver
        .query_single("[data-settings-image-server] [data-action='save']")
        .await?
        .click()
        .await?;
    let status = driver
        .query_single("[data-settings-image-server] [role='status']")
        .await?
        .text()
        .await?;
    if !status.contains("saved") {
        return Err(format!("Image endpoint was not saved from Settings: {status}").into());
    }
    capture_journey_step(ctx, "settings-image-saved", seed, "standard").await?;

    driver
        .query_single("[data-settings-tab-button='ai']")
        .await?
        .click()
        .await?;
    let endpoint = driver.query_single("[data-ai-base-url]").await?;
    endpoint.clear().await?;
    endpoint.send_keys(&ctx.ai_url).await?;
    let model = driver.query_single("[data-ai-model-setting]").await?;
    model.clear().await?;
    model.send_keys("fixture-model").await?;
    driver.query_single("[data-ai-save]").await?.click().await?;
    capture_journey_step(ctx, "settings-ai-saved", seed, "standard").await?;

    ctx.open_route("/settings").await?;
    driver.refresh().await?;
    driver
        .query_single("[data-settings-tab-button='images']")
        .await?
        .click()
        .await?;
    let saved_image = driver
        .query_single("[data-image-server-url]")
        .await?
        .prop("value")
        .await?;
    if saved_image.as_deref() != Some(ctx.image_url.as_str()) {
        return Err(format!("Image endpoint did not persist: {saved_image:?}").into());
    }
    driver
        .query_single("[data-settings-tab-button='ai']")
        .await?
        .click()
        .await?;
    let saved_model = driver
        .query_single("[data-ai-model-setting]")
        .await?
        .prop("value")
        .await?;
    if saved_model.as_deref() != Some("fixture-model") {
        return Err(format!("AI model did not persist: {saved_model:?}").into());
    }
    let saved_url = driver
        .query_single("[data-ai-base-url]")
        .await?
        .prop("value")
        .await?;
    if saved_url.as_deref() != Some(ctx.ai_url.as_str()) {
        return Err(format!("AI endpoint did not persist: {saved_url:?}").into());
    }
    capture_journey_step(ctx, "settings-provider-persistence", seed, "standard").await?;
    Ok(())
}

pub async fn final_provider_exercises(ctx: &ScenarioContext, seed: u64) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.configure_image_exercise_fixtures().await?;
    ctx.open_route("/practice/image-vocabulary?standard=3&level=1")
        .await?;
    driver.query_single("[data-image-vocabulary]").await?;
    page_objects::image_vocabulary::wait_for_review(driver).await?;
    page_objects::image_vocabulary::approve(driver).await?;
    let choices = page_objects::image_vocabulary::select_options(driver, 1).await?;
    if choices != 3 {
        return Err(format!("Image practice produced {choices} rather than 3 options").into());
    }
    let feedback = page_objects::image_vocabulary::check(driver).await?;
    if !feedback.contains("Correct!") && !feedback.contains("Not quite.") {
        return Err(format!("Image practice did not score the answer: {feedback}").into());
    }
    ctx.assert_ai_request_kind("image-scene").await?;
    capture_journey_step(ctx, "practice-image-scored", seed, "standard").await?;

    let translation = crate::learner::TranslationPage::new(ctx);
    ctx.configure_ai_fixture("success").await?;
    translation.configure("fixture-model").await?;
    translation
        .open_from_practice_with_count("zh-to-en", 5)
        .await?;
    translation.generate().await?;
    let observation = translation.observe().await?;
    for tile in observation
        .canonical_tiles()
        .map_err(|error| format!("Invalid translation tile order: {error}"))?
    {
        translation.select_tile(tile).await?;
    }
    if translation.check().await? != "correct" {
        return Err("Translation fixture did not score a correct response".into());
    }
    capture_journey_step(ctx, "practice-translation-scored", seed, "standard").await?;
    Ok(())
}

pub async fn assert_no_recognition_opfs_evidence(
    ctx: &ScenarioContext,
    seed: u64,
) -> HarnessResult<()> {
    let script = r#"
        const done = arguments[arguments.length - 1];
        (async () => {
            if (!navigator.storage || typeof navigator.storage.getDirectory !== 'function') {
                throw new Error('OPFS inspection unavailable; cannot attest tracking-off privacy');
            }
            const root = await navigator.storage.getDirectory();
            const files = [];
            async function inspect(directory, prefix) {
                for await (const [name, handle] of directory.entries()) {
                    const path = `${prefix}/${name}`;
                    if (handle.kind === 'directory') {
                        await inspect(handle, path);
                    } else {
                        files.push(path);
                    }
                }
            }
            await inspect(root, '');
            done({files});
        })().catch(error => done({error: String(error)}));
    "#;
    let result: serde_json::Value = ctx
        .driver
        .execute_async(script, Vec::new())
        .await?
        .convert()?;
    if let Some(error) = result.get("error").and_then(serde_json::Value::as_str) {
        return Err(format!("Cannot verify opted-out OPFS privacy: {error}").into());
    }
    let files = result
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or("OPFS inspection did not produce a file listing")?;
    if !files.is_empty() {
        return Err(format!(
            "Cannot prove recognition_evidence is absent: OPFS contains {} files. Inspect the personal DuckDB contents through a read-only database connection before accepting this run. Paths: {files:?}",
            files.len()
        ).into());
    }
    capture_journey_step(ctx, "privacy-opfs-empty", seed, "standard").await?;
    Ok(())
}
