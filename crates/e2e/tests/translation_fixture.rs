use e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, e2e_ensure, e2e_ensure_eq};
use e2e::learner::TranslationPage;
use thirtyfour::prelude::*;

async fn translation_from_visible_tiles(context: ScenarioContext) -> HarnessResult<()> {
    let page = TranslationPage::new(&context);
    context.configure_ai_fixture("success").await?;
    page.open("starter:everyday").await?;
    page.configure("fixture-model").await?;
    page.generate().await?;
    let observation = page.observe().await?;
    e2e_ensure_eq!(observation.target_language.as_str(), "en");
    e2e_ensure!(observation.source.contains("我喜欢"), "source: {}", observation.source);
    let expected = ["I", "like"];
    for part in expected {
        let tile = observation.available_tiles.iter().find(|tile| tile.text == part)
            .ok_or_else(|| format!("No learner-visible tile for {part}"))?;
        page.select_tile(tile).await?;
    }
    let final_tile = observation.available_tiles.iter().find(|tile| tile.text.ends_with('.'))
        .ok_or("No learner-visible sentence-ending tile")?;
    page.select_tile(final_tile).await?;
    e2e_ensure_eq!(page.check().await?, "correct");
    context.driver.execute_async("const done=arguments[arguments.length-1];function ready(){const e=document.querySelector('[data-component=\'translation-session-stats\']'); if(e?.textContent.includes('Completed: 1')) done(true); else requestAnimationFrame(ready);}ready();",vec![]).await?;
    let stats = context.driver.find(By::Css("[data-component='translation-session-stats']")).await?.text().await?;
    e2e_ensure!(stats.contains("Completed: 1"), "stats: {stats}");
    context.driver.goto(context.url("/#/browse")).await?;
    e2e_ensure!(context.driver.find_all(By::Css("[data-translation-feedback]")).await?.is_empty());
    Ok(())
}

async fn malformed_response_is_recoverable(context: ScenarioContext) -> HarnessResult<()> {
    let page = TranslationPage::new(&context);
    context.configure_ai_fixture("malformed-generation").await?;
    page.open("starter:everyday").await?;
    page.configure("fixture-model").await?;
    context.driver.find(By::Css("[data-translation-start]")).await?.click().await?;
    let alert = context.driver.query(By::Css("[data-sentence-translation] [role='alert']")).single().await?;
    e2e_ensure!(!alert.text().await?.is_empty());
    e2e_ensure!(context.driver.find_all(By::Css("[data-translation-feedback]")).await?.is_empty());
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
        let tile = observation.available_tiles.iter().find(|tile| tile.text == part)
            .ok_or_else(|| format!("No visible tile {part}"))?;
        page.select_tile(tile).await?;
    }
    let last = observation.available_tiles.iter().find(|tile| tile.text.ends_with('.'))
        .ok_or("No sentence-ending tile")?;
    page.select_tile(last).await?;
    context.driver.find(By::Css("[data-translation-submit]:not([disabled])")).await?.click().await?;
    context.driver.query(By::Css("[data-sentence-translation] p[role='status']")).single().await?;
    let requests_url = context.ai_url.replace("/v1/", "/__e2e/requests");
    context.driver.execute_async(format!(
        "const done=arguments[arguments.length-1];async function wait(){{const res=await fetch('{requests_url}');const json=await res.json();if(json.records.some(record=>record.kind==='evaluation')) done(true); else setTimeout(wait,30);}}wait().catch(()=>done(false));"
    ),vec![]).await?;
    context.driver.goto(context.url("/#/browse")).await?;
    context.driver.execute_async(
        "const done = arguments[arguments.length - 1]; setTimeout(() => done(true), 1500);",
        vec![],
    ).await?;
    context.driver.goto(context.url("/#/practice/translation?source=starter:everyday")).await?;
    let stats = context.driver.find(By::Css("[data-component='translation-session-stats']")).await?.text().await?;
    e2e_ensure!(stats.contains("Completed: 0"), "abandoned evaluation must not count: {stats}");
    Ok(())
}

async fn practice_and_browse_responsive(ctx: ScenarioContext) -> HarnessResult<()> {
    for (width, height, name) in [(390, 844, "mobile"), (1280, 900, "desktop")] {
        ctx.driver.cdp().emulation().set_device_metrics_override(width, height, 1.0, width < 600).await?;
        ctx.driver.goto(ctx.url("/#/practice")).await?;
        ctx.driver.query(By::Css("[data-practice-tabs]")).single().await?;
        let translation = ctx.driver.find(By::Css("[data-practice-tab='translation']")).await?;
        translation.click().await?;
        ctx.driver.query(By::Css("[data-translation-setup]")).single().await?;
        let overflow = ctx.driver.execute("return document.documentElement.scrollWidth > innerWidth + 2", vec![]).await?.convert::<bool>()?;
        e2e_ensure!(!overflow, "practice overflows at {width}px");
        ctx.driver.screenshot(&ctx.artifacts.join(format!("practice-{name}.png"))).await?;
        ctx.driver.goto(ctx.url("/#/browse")).await?;
        ctx.driver.query(By::Css("[data-component='browse-page'],[data-browse-page],[data-browse]")).first().await?;
        let overflow = ctx.driver.execute("return document.documentElement.scrollWidth > innerWidth + 2", vec![]).await?.convert::<bool>()?;
        e2e_ensure!(!overflow, "browse overflows at {width}px");
        ctx.driver.screenshot(&ctx.artifacts.join(format!("browse-{name}.png"))).await?;
    }
    Ok(())
}

async fn five_hsk6_translations_from_setup(context: ScenarioContext) -> HarnessResult<()> {
    context.configure_ai_fixture("success").await?;
    context.driver.goto(context.url("/#/practice")).await?;
    context.driver.query(By::Css("[data-component='practice-builder']")).single().await?;
    e2e_ensure!(context.driver.find_all(By::Css("[data-practice-config-tab='translation']")).await?.is_empty(), "translation settings must be conditional");
    e2e_ensure!(context.driver.find_all(By::Css("[data-practice-config-tab='speaking']")).await?.is_empty(), "speaking settings must be conditional");
    context.driver.query(By::Css("[data-practice-tab='translation']")).single().await?.click().await?;
    context.driver.query(By::Css("[data-practice-config-tab='translation']")).single().await?;
    context.driver.query(By::Css("[data-translation-setup]")).single().await?;
    context.driver.execute_async(
        "const done=arguments[arguments.length-1]; document.querySelector('[data-hsk-band=\"3:1\"]').click(); function after(){if(document.querySelector('[data-hsk-band=\"3:1\"]').getAttribute('aria-pressed')==='false'){ document.querySelector('[data-hsk-band=\"3:6\"]').click(); done(true); }else{ requestAnimationFrame(after); }} after();",
        vec![],
    ).await?;
    context.driver.execute(
        "const count=document.querySelector('[data-translation-setup-count]'); count.value='5'; count.dispatchEvent(new Event('change',{bubbles:true}));",
        vec![],
    ).await?;
    context.driver.find(By::Css("[data-translation-setup-start]")).await?.click().await?;
    context.driver.query(By::Css("[data-sentence-translation][data-translation-mode='fixed']")).single().await?;
    let source = context.driver.execute(
        "return document.querySelector('[data-translation-source]').value",
        vec![],
    ).await?.convert::<String>()?;
    e2e_ensure_eq!(source.as_str(), "filters:3.6||");
    let page = TranslationPage::new(&context);
    page.configure("fixture-model").await?;
    for question in 1..=5 {
        if question == 1 {
            context.driver.find(By::Css("[data-translation-start]")).await?.click().await?;
        } else {
            page.continue_after_feedback().await?;
        }
        page.wait_for_question(question).await?;
        let observation = page.observe().await?;
        e2e_ensure_eq!(observation.target_language.as_str(), "en");
        let mut ordered = observation.available_tiles.clone();
        ordered.sort_by_key(|tile| {
            if tile.text == "I" { 0 } else if tile.text == "like" { 1 } else { 2 }
        });
        for tile in &ordered {
            page.select_tile(tile).await?;
        }
        e2e_ensure_eq!(page.check().await?, "correct");
        let progress = context.driver.find(By::Css("[data-translation-progress]")).await?.text().await?;
        e2e_ensure!(progress.contains(&format!("{question} / 5")), "progress: {progress}");
    }
    page.continue_after_feedback().await?;
    context.driver.query(By::Css("[data-translation-complete]")).single().await?;
    e2e_ensure!(context.driver.find_all(By::Css("[data-translation-start], [data-translation-submit], [data-feedback-continue]")).await?.is_empty(), "No sixth exercise may start");
    let stats = context.driver.find(By::Css("[data-component='translation-session-stats']")).await?.text().await?;
    e2e_ensure!(stats.contains("Completed: 5"), "stats: {stats}");
    Ok(())
}

#[tokio::test]
async fn deterministic_translation_fixture_scenarios() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let tests = async {
        harness.run("translation_learner_visible_tiles", translation_from_visible_tiles).await?;
        harness.run("five_hsk6_translations_from_setup", five_hsk6_translations_from_setup).await?;
        harness.run("translation_malformed_response", malformed_response_is_recoverable).await?;
        harness.run("translation_abandoned_evaluation", abandoned_evaluation_does_not_score).await?;
        harness.run("practice_browse_responsive", practice_and_browse_responsive).await?;
        Ok(())
    }.await;
    let cleanup = harness.finish().await;
    tests.and(cleanup)
}
