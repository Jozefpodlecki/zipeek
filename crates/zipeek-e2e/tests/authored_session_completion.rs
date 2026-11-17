use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

#[derive(Clone, Copy)]
enum Exercise {
    FillBlank,
    Dictation,
    SentenceBuilder,
    Transformation,
    SpotMistake,
}

impl Exercise {
    fn id(self) -> &'static str {
        match self {
            Self::FillBlank => "fill-blank",
            Self::Dictation => "audio-dictation",
            Self::SentenceBuilder => "sentence-builder",
            Self::Transformation => "sentence-transformation",
            Self::SpotMistake => "spot-mistake",
        }
    }
}

async fn launch(ctx: &ScenarioContext, exercise: Exercise) -> HarnessResult<usize> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.click("[data-practice-path='advanced']").await?;
    for tile in driver.find_all_css("[data-practice-exercise]").await? {
        let is_target =
            tile.attr("data-practice-exercise").await?.as_deref() == Some(exercise.id());
        let is_selected = tile.attr("aria-pressed").await?.as_deref() == Some("true");
        if is_target != is_selected {
            tile.click().await?;
        }
    }
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-practice-exercise][aria-pressed='true']")
            .await?
            .len(),
        1,
        "A completion journey must contain exactly one exercise type"
    );
    driver
        .click("[data-practice-exercise-stage='configure']")
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
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-component='navigation-drawer'], [data-app-footer], [data-component='shell-header']").await?.is_empty(),
        "Practice sessions must run without the application navigation or footer");
    let progress = driver
        .query_single("[data-practice-step-progress]")
        .await?
        .text()
        .await?;
    let count = progress
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|words| {
            (words[0] == "of")
                .then(|| words[1].parse::<usize>().ok())
                .flatten()
        })
        .ok_or_else(|| format!("Missing bounded exercise count in {progress:?}"))?;
    zipeek_e2e::e2e_ensure!(
        (1..=50).contains(&count),
        "Unexpected exercise count: {count}"
    );
    Ok(count)
}

async fn add_tiles(driver: &WebDriver, root: &str, words: &[&str]) -> HarnessResult<()> {
    for word in words {
        driver
            .click(&format!(
                "{root} button[aria-label='Add {word}']:not([disabled])"
            ))
            .await?;
    }
    Ok(())
}

async fn answer(driver: &WebDriver, exercise: Exercise) -> HarnessResult<()> {
    match exercise {
        Exercise::FillBlank => {
            let prompt = driver
                .query_single("[data-practice-authored-exercise]")
                .await?
                .text()
                .await?;
            let word = if prompt.contains("I bought a book yesterday") {
                "買"
            } else if prompt.contains("I study Chinese every day") {
                "學習"
            } else if prompt.contains("This cup of tea is too hot") {
                "熱"
            } else {
                return Err(format!("Unknown cloze prompt: {prompt}").into());
            };
            driver
                .query_single("#authored-hanzi-response")
                .await?
                .send_keys(word)
                .await?;
            driver
                .click("[data-practice-authored-exercise] button[type='submit']:not([disabled])")
                .await?;
            driver
                .query_with_text(
                    "[data-practice-authored-exercise] [data-practice-feedback]",
                    "Correct",
                )
                .await?;
            driver
                .click("[data-practice-authored-exercise] [data-practice-feedback] button")
                .await?;
        }
        Exercise::Dictation => {
            zipeek_e2e::e2e_ensure!(
                driver
                    .query_single("[data-practice-authored-exercise]")
                    .await?
                    .text()
                    .await?
                    .contains("No transcript is shown until you check"),
                "Dictation must not expose the answer before grading"
            );
            driver
                .query_single("#authored-hanzi-response")
                .await?
                .send_keys("not-a-mandarin-transcript")
                .await?;
            zipeek_e2e::e2e_ensure!(
                !driver
                    .query_single("[data-practice-authored-exercise] button[type='submit']")
                    .await?
                    .is_enabled()
                    .await?,
                "Each dictation round must require fresh playback"
            );
            driver.query(By::XPath("//section[@data-practice-authored-exercise]//button[normalize-space()='Play Mandarin audio']"))
                .single().await?.click().await?;
            driver
                .click("[data-practice-authored-exercise] button[type='submit']:not([disabled])")
                .await?;
            driver
                .query_with_text(
                    "[data-practice-authored-exercise] [data-practice-feedback]",
                    "Not quite",
                )
                .await?;
            driver
                .click("[data-practice-authored-exercise] [data-practice-feedback] button")
                .await?;
        }
        Exercise::SentenceBuilder => {
            let prompt = driver
                .query_single("[data-practice-sentence-builder]")
                .await?
                .text()
                .await?;
            let order: &[&str] = if prompt.contains("I study Chinese every day") {
                &["我", "每天", "学习", "中文"]
            } else if prompt.contains("Tomorrow I will go to school") {
                &["我", "明天", "去", "学校"]
            } else if prompt.contains("We drink tea at home") {
                &["我们", "在家", "喝茶"]
            } else {
                return Err(format!("Unknown Sentence Builder prompt: {prompt}").into());
            };
            add_tiles(driver, "[data-practice-sentence-builder]", order).await?;
            driver
                .click("[data-sentence-builder-check]:not([disabled])")
                .await?;
            driver
                .query_with_text(
                    "[data-practice-sentence-builder] [data-practice-feedback]",
                    "Correct",
                )
                .await?;
            driver.click("[data-sentence-builder-continue]").await?;
        }
        Exercise::Transformation => {
            let id = driver
                .query_single("[data-practice-sentence-transformation]")
                .await?
                .attr("data-item-id")
                .await?
                .ok_or("Transformation item has no ID")?;
            let order: &[&str] = match id.as_str() {
                "shi-de-time-01" => &["我", "是", "昨天", "买", "这本书", "的"],
                "ba-disposal-01" => &["我", "把", "门", "关上", "了"],
                "bi-comparison-01" => &["哥哥", "比", "弟弟", "高"],
                other => return Err(format!("Unknown transformation item {other}").into()),
            };
            add_tiles(driver, "[data-practice-sentence-transformation]", order).await?;
            driver
                .click("[data-sentence-transformation-check]:not([disabled])")
                .await?;
            driver
                .query_with_text(
                    "[data-practice-sentence-transformation] [data-practice-feedback]",
                    "Correct",
                )
                .await?;
            driver
                .click("[data-sentence-transformation-continue]")
                .await?;
        }
        Exercise::SpotMistake => {
            let prompt = driver
                .query_single("[data-practice-spot-mistake]")
                .await?
                .text()
                .await?;
            let (wrong_index, replacement) = if prompt.contains("I will go to school tomorrow") {
                (2, "会去")
            } else if prompt.contains("I have three books") {
                (3, "书")
            } else if prompt.contains("She drinks tea every day") {
                (3, "茶")
            } else {
                return Err(format!("Unknown Spot the Mistake prompt: {prompt}").into());
            };
            let choices = driver
                .find_all_css("[aria-label='Select the incorrect Chinese phrase'] button")
                .await?;
            choices
                .get(wrong_index)
                .ok_or("Missing incorrect phrase choice")?
                .click()
                .await?;
            driver
                .query_single("#spot-mistake-repair")
                .await?
                .send_keys(replacement)
                .await?;
            driver
                .click("[data-spot-mistake-check]:not([disabled])")
                .await?;
            let review = driver
                .query_single("[data-practice-spot-mistake] [data-practice-feedback]")
                .await?
                .text()
                .await?;
            zipeek_e2e::e2e_ensure!(
                review.contains("Mistake identified correctly")
                    && review.contains("Correction accepted"),
                "Both detection and repair must be graded independently: {review}"
            );
            driver.click("[data-spot-mistake-continue]").await?;
        }
    }
    Ok(())
}

async fn complete(ctx: ScenarioContext, exercise: Exercise) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let total = launch(&ctx, exercise).await?;
    let url = driver.current_url().await?;
    for index in 1..=total {
        driver
            .query_with_text(
                "[data-practice-step-progress]",
                &format!("Exercise {index} of {total}"),
            )
            .await?;
        answer(driver, exercise).await?;
    }
    let summary = driver.query_single("[data-practice-local-summary]").await?;
    zipeek_e2e::e2e_ensure!(summary.text().await?.contains("Excellent work!"));
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-practice-completion-count]")
            .await?
            .text()
            .await?,
        format!("{total} of {total} exercises")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-practice-completed]")
            .await?
            .text()
            .await?,
        total.to_string()
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[data-practice-skipped]")
            .await?
            .text()
            .await?,
        "0"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .query_single("[aria-label='Exercises completed']")
            .await?
            .attr("aria-valuenow")
            .await?
            .as_deref(),
        Some("100")
    );
    ctx.checkpoint(&format!("{}-practice-finished", exercise.id()))
        .await?;
    driver
        .query_with_text(
            "[data-practice-history-notice]",
            "Learning history is off",
        )
        .await?;
    driver.refresh().await?;
    driver
        .query_single("[data-component='practice-session-unavailable'][data-session-error='not-found']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.current_url().await?,
        url,
        "Reload must not silently start a new session"
    );
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-practice-local-summary], [data-practice-authored-exercise], [data-practice-sentence-builder], [data-practice-sentence-transformation], [data-practice-spot-mistake]").await?.is_empty(),
        "A finalized temporary session must not reappear after history handling removed it"
    );
    driver
        .click_with_text("a", "Back to Practice setup")
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    Ok(())
}

async fn cloze_session(ctx: ScenarioContext) -> HarnessResult<()> {
    complete(ctx, Exercise::FillBlank).await
}
async fn dictation_session(ctx: ScenarioContext) -> HarnessResult<()> {
    complete(ctx, Exercise::Dictation).await
}
async fn builder_session(ctx: ScenarioContext) -> HarnessResult<()> {
    complete(ctx, Exercise::SentenceBuilder).await
}
async fn transformation_session(ctx: ScenarioContext) -> HarnessResult<()> {
    complete(ctx, Exercise::Transformation).await
}
async fn mistake_session(ctx: ScenarioContext) -> HarnessResult<()> {
    complete(ctx, Exercise::SpotMistake).await
}

#[tokio::test]
async fn authored_session_completion() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("cloze-all-rounds-cleanup", cloze_session)
        .await?;
    harness
        .run("dictation-all-rounds-cleanup", dictation_session)
        .await?;
    harness
        .run("sentence-builder-all-rounds-cleanup", builder_session)
        .await?;
    harness
        .run(
            "sentence-transformation-all-rounds-cleanup",
            transformation_session,
        )
        .await?;
    harness
        .run("spot-mistake-all-rounds-cleanup", mistake_session)
        .await?;
    harness.finish().await
}
