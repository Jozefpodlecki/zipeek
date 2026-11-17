use zipeek_e2e::learner::TranslationPage;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, e2e_ensure};

async fn live_learner(context: ScenarioContext) -> HarnessResult<()> {
    let endpoint = std::env::var("ZIPEEK_E2E_AI_ENDPOINT")
        .map_err(|_| "Set ZIPEEK_E2E_AI_ENDPOINT to your LM Studio /v1/ base URL")?;
    let model = std::env::var("ZIPEEK_E2E_AI_MODEL")
        .map_err(|_| "Set ZIPEEK_E2E_AI_MODEL to the loaded LM Studio model ID")?;
    let page = TranslationPage::new(&context);
    println!("AI translation: configuring endpoint and model in Settings");
    page.configure_endpoint(&model, &endpoint).await?;
    println!("AI translation: selecting Sentence translation in Practice");
    page.open_from_practice("zh-to-en").await?;
    println!("AI translation: exercise launched from Practice with saved Settings configuration");
    println!("AI translation: generating sentence with {model}");
    page.generate().await?;
    println!("AI translation: word bank ready");
    context.checkpoint("ai-generated-question").await?;
    let observation = page.observe().await?;
    e2e_ensure!(
        !observation.available_tiles.is_empty(),
        "Live AI produced no playable translation tiles"
    );
    let mut tile_indices = std::collections::BTreeSet::new();
    for tile in &observation.available_tiles {
        e2e_ensure!(
            tile_indices.insert(tile.index),
            "Live AI returned a repeated tile index"
        );
    }

    let ordered = observation
        .canonical_tiles()
        .map_err(|error| format!("Invalid canonical tile metadata: {error}"))?;
    let script = context.artifacts.join("learner-observation.txt");
    let visible = observation
        .available_tiles
        .iter()
        .map(|tile| {
            format!(
                "{}: {} (solution: {:?})",
                tile.index, tile.text, tile.solution_order
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(
        &script,
        format!(
            "source: {}\ntarget: {}\ntiles:\n{visible}",
            observation.source, observation.target_language
        ),
    )?;
    println!(
        "AI translation: assembling {} canonical answer tiles",
        ordered.len()
    );
    for tile in ordered {
        page.select_tile(tile).await?;
    }
    println!("AI translation: submitting translation for evaluation");
    let verdict = page.check().await?;
    println!("AI translation: evaluation returned {verdict}");
    context.checkpoint("ai-answer-feedback").await?;
    e2e_ensure!(
        verdict == "correct" || verdict == "incorrect",
        "verdict: {verdict}"
    );
    page.continue_after_feedback().await?;
    page.wait_for_practice_progress(2).await?;
    let recorded = context
        .driver
        .execute(
            zipeek_e2e::js_scripts::translation::AI_LEARNER_LIVE_LEARNER_01,
            vec![],
        )
        .await?
        .convert::<serde_json::Value>()?;
    e2e_ensure!(
        recorded["current"] == 1
            && recorded["outcome"]["scored"]["possible"] == 1
            && recorded["direction"] == "zh-to-en",
        "Translation result was not recorded in the resumable Practice plan: {recorded}"
    );
    println!("AI translation: first exercise scored and persisted; continuing to exercise two");
    Ok(())
}

#[tokio::test]
#[ignore = "Requires an explicit running LM Studio model; not part of deterministic CI"]
async fn ai_completes_translation_through_browser() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness.run("live_ai_translation", live_learner).await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
