use thirtyfour::prelude::*;
use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, practice_config, sentence_builder},
};

async fn launch(ctx: &ScenarioContext, automatic: bool, retry: bool) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='advanced']").await?;
    for tile in driver.find_all_css("[data-practice-exercise]").await? {
        let target =
            tile.attr("data-practice-exercise").await?.as_deref() == Some("sentence-builder");
        if (tile.attr("aria-pressed").await?.as_deref() == Some("true")) != target {
            tile.click().await?;
        }
    }
    driver
        .click("[data-practice-exercise-stage='configure']")
        .await?;
    practice_config::set_checkbox(
        driver,
        "[data-sentence-builder-settings] label:nth-child(1) input",
        automatic,
    )
    .await?;
    practice_config::set_checkbox(
        driver,
        "[data-sentence-builder-settings] label:nth-child(2) input",
        retry,
    )
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
    driver.query_single(sentence_builder::ROOT).await?;
    Ok(())
}

async fn selection(driver: &WebDriver) -> HarnessResult<Vec<String>> {
    let mut words = Vec::new();
    for tile in driver
        .find_all_css("[data-sentence-builder-selection] button[aria-label^='Remove ']")
        .await?
    {
        words.push(tile.text().await?);
    }
    Ok(words)
}

async fn drag(driver: &WebDriver, source: &str, target: &str) -> HarnessResult<()> {
    let applied = driver.execute(r#"
        const source = document.querySelector(arguments[0]);
        const target = document.querySelector(arguments[1]);
        if (!source || !target) return false;
        const dataTransfer = new DataTransfer();
        source.dispatchEvent(new DragEvent('dragstart', {bubbles:true, cancelable:true, dataTransfer}));
        target.dispatchEvent(new DragEvent('dragover', {bubbles:true, cancelable:true, dataTransfer}));
        target.dispatchEvent(new DragEvent('drop', {bubbles:true, cancelable:true, dataTransfer}));
        source.dispatchEvent(new DragEvent('dragend', {bubbles:true, dataTransfer}));
        return true;
    "#, vec![source.into(), target.into()]).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(applied);
    Ok(())
}

async fn grade(driver: &WebDriver, automatic: bool) -> HarnessResult<()> {
    if !automatic {
        driver
            .click("[data-sentence-builder-check]:not([disabled])")
            .await?;
    }
    driver
        .query_single("[data-practice-sentence-builder] [data-practice-feedback]")
        .await?;
    Ok(())
}

async fn session_position(driver: &WebDriver) -> HarnessResult<usize> {
    let value = driver
        .execute(
            r#"
        const match = location.hash.match(/practice\/session\/([^/?]+)/);
        if (!match) throw new Error('Missing session route');
        return JSON.parse(localStorage.getItem('zipeek.practice.session.v1.' + match[1])).current;
    "#,
            vec![],
        )
        .await?
        .convert::<usize>()?;
    Ok(value)
}

async fn advance_once(driver: &WebDriver) -> HarnessResult<()> {
    let current = session_position(driver).await?;
    driver.execute("const button = document.querySelector('[data-sentence-builder-continue]'); button.click(); button.click();", vec![]).await?;
    driver
        .query_single("[data-sentence-builder-selection]:not(:has(button))")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(session_position(driver).await?, current + 1);
    zipeek_e2e::e2e_ensure!(
        driver
            .query(By::Css(
                "[data-practice-sentence-builder] [data-practice-feedback]"
            ))
            .not_exists()
            .await?
    );
    Ok(())
}

async fn interactions(ctx: ScenarioContext, automatic: bool, retry: bool) -> HarnessResult<()> {
    let driver = &ctx.driver;
    launch(&ctx, automatic, retry).await?;
    let root = driver.query_single(sentence_builder::ROOT).await?;
    zipeek_e2e::e2e_ensure_eq!(
        root.attr("data-auto-check").await?.as_deref(),
        Some(if automatic { "true" } else { "false" })
    );
    zipeek_e2e::e2e_ensure_eq!(
        root.attr("data-retry-incorrect").await?.as_deref(),
        Some(if retry { "true" } else { "false" })
    );
    drag(
        driver,
        "[data-practice-sentence-builder] button[aria-label^='Add ']:not([disabled])",
        "[data-sentence-builder-selection]",
    )
    .await?;
    driver
        .query_single("[data-sentence-builder-selection] button[aria-label^='Remove ']")
        .await?;
    drag(
        driver,
        "[data-practice-sentence-builder] button[aria-label^='Add ']:not([disabled])",
        "[data-sentence-builder-selection] > div",
    )
    .await?;
    driver
        .query_single("[data-sentence-builder-selection] > div:nth-child(2)")
        .await?;
    let before = selection(driver).await?;
    driver.query_single("[data-sentence-builder-selection] button[aria-label^='Move '][title='Move right']:not([disabled])").await?.send_keys(Key::Enter).await?;
    let mut reversed = before.clone();
    reversed.reverse();
    zipeek_e2e::e2e_ensure_eq!(selection(driver).await?, reversed);
    drag(
        driver,
        "[data-sentence-builder-selection] > div",
        "[data-sentence-builder-selection]",
    )
    .await?;
    zipeek_e2e::e2e_ensure_eq!(selection(driver).await?, before);
    driver
        .click("[data-practice-sentence-builder] button[aria-label='Undo']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(selection(driver).await?.len(), 1);
    driver
        .click("[data-practice-sentence-builder] button[aria-label='Clear']")
        .await?;
    let mut wrong = sentence_builder::answer(driver).await?;
    wrong.reverse();
    sentence_builder::choose(driver, &wrong).await?;
    grade(driver, automatic).await?;
    driver
        .query_with_text(
            "[data-practice-sentence-builder] [data-practice-feedback]",
            "Try another order",
        )
        .await?;
    if retry {
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-sentence-builder-continue]")
                .await?
                .is_empty()
        );
        driver
            .click_with_text("[data-practice-sentence-builder] button", "Try again")
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query(By::Css(
                    "[data-practice-sentence-builder] [data-practice-feedback]"
                ))
                .not_exists()
                .await?
        );
        zipeek_e2e::e2e_ensure_eq!(selection(driver).await?, wrong);
        driver
            .click("[data-practice-sentence-builder] button[aria-label='Clear']")
            .await?;
        sentence_builder::choose_correct(driver).await?;
        grade(driver, automatic).await?;
        driver
            .query_with_text(
                "[data-practice-sentence-builder] [data-practice-feedback]",
                "Correct",
            )
            .await?;
    }
    let checked = selection(driver).await?;
    drag(
        driver,
        "[data-practice-sentence-builder] button[aria-label^='Add ']",
        "[data-sentence-builder-selection]",
    )
    .await?;
    drag(
        driver,
        "[data-sentence-builder-selection] > div",
        "[data-sentence-builder-selection] > div:last-child",
    )
    .await?;
    zipeek_e2e::e2e_ensure_eq!(selection(driver).await?, checked);
    ctx.checkpoint("sentence-builder-locked-feedback").await?;
    advance_once(driver).await?;
    driver.refresh().await?;
    let resumed = driver.query_single(sentence_builder::ROOT).await?;
    zipeek_e2e::e2e_ensure_eq!(
        resumed.attr("data-auto-check").await?.as_deref(),
        Some(if automatic { "true" } else { "false" })
    );
    zipeek_e2e::e2e_ensure_eq!(
        resumed.attr("data-retry-incorrect").await?.as_deref(),
        Some(if retry { "true" } else { "false" })
    );
    sentence_builder::choose_correct(driver).await?;
    grade(driver, automatic).await?;
    driver
        .query_with_text(
            "[data-practice-sentence-builder] [data-practice-feedback]",
            "Correct",
        )
        .await?;
    ctx.checkpoint("sentence-builder-resumed-options").await?;
    Ok(())
}

async fn run_suite(kind: SuiteKind) -> HarnessResult<()> {
    let mut harness = Harness::start(kind).await?;
    for automatic in [false, true] {
        for retry in [false, true] {
            harness
                .run(
                    &format!("sentence-builder-auto-{automatic}-retry-{retry}"),
                    move |ctx| interactions(ctx, automatic, retry),
                )
                .await?;
        }
    }
    harness.finish().await
}

#[tokio::test]
async fn sentence_builder_at_root() -> HarnessResult<()> {
    run_suite(SuiteKind::Root).await
}

#[tokio::test]
async fn sentence_builder_at_pages_base() -> HarnessResult<()> {
    run_suite(SuiteKind::Pages).await
}
