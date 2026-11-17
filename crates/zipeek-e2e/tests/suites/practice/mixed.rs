use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query_single(
            "[data-practice-state='ready'] :is([data-match-next],[data-mcq-continue]):disabled",
        )
        .await
}
async fn variant(driver: &WebDriver) -> HarnessResult<String> {
    ready(driver).await?;
    Ok(driver
        .find_css("[data-practice-variant]")
        .await?
        .attr("data-practice-variant")
        .await?
        .ok_or("No current type")?)
}
async fn round(driver: &WebDriver, make_wrong: bool) -> HarnessResult<()> {
    ready(driver).await?;
    let mcq = !driver.find_all_css("[data-mcq]").await?.is_empty();
    if mcq {
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css("[data-mcq-option][aria-pressed='true']")
                .await?
                .len(),
            0,
            "A new mixed MCQ round must not inherit an earlier answer"
        );
        let correct = zipeek_e2e::page_objects::round::mcq_correct_index(driver).await?;
        let chosen = if make_wrong {
            (correct + 1) % 3
        } else {
            correct
        };
        zipeek_e2e::page_objects::mcq::option(driver, chosen)
            .await?
            .click()
            .await?;
    } else {
        let pairs = zipeek_e2e::page_objects::round::matching_pairs(driver).await?;
        if make_wrong {
            driver
                .find_css("[data-match-side='left'][data-match-index='0']")
                .await?
                .click()
                .await?;
            driver
                .find_css(format!(
                    "[data-match-side='right'][data-match-index='{}']",
                    (pairs[0] + 1) % 4
                ))
                .await?
                .click()
                .await?;
            zipeek_e2e::e2e_ensure!(
                driver
                    .execute_async(
                        zipeek_e2e::harness::browser_scripts::WAIT_FOR_MATCHING_FEEDBACK_TO_CLEAR,
                        vec![],
                    )
                    .await?
                    .convert::<bool>()?,
                "Mixed-mode incorrect-answer feedback must clear",
            );
        }
        for (left, right) in pairs.into_iter().enumerate() {
            driver
                .find_css(format!(
                    "[data-match-side='left'][data-match-index='{left}']"
                ))
                .await?
                .click()
                .await?;
            driver
                .find_css(format!(
                    "[data-match-side='right'][data-match-index='{right}']"
                ))
                .await?
                .click()
                .await?;
        }
    }
    driver
        .query_single("[data-match-next]:not([disabled]),[data-mcq-continue]:not([disabled])")
        .await?
        .click()
        .await?;
    Ok(())
}
async fn summary(driver: &WebDriver) -> HarnessResult<()> {
    driver
        .query_single("[data-practice-summary] [data-summary-chart='ready']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .execute_async(
                zipeek_e2e::harness::browser_scripts::WAIT_FOR_MATCHING_SUMMARY_TO_SETTLE,
                vec![],
            )
            .await?
            .convert::<bool>()?,
        "Mixed-mode summary must settle to the reported score",
    );
    Ok(())
}

pub async fn mixed_setup_progress_summary(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    ctx.checkpoint_visible(
        "mixed-activity-options",
        "[data-setup-choice='hanzi-to-meaning']",
    )
    .await?;
    driver
        .find_css("[data-setup-choice='hanzi-to-meaning']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-activity-picker] [aria-pressed='true']")
            .await?
            .len(),
        1
    );
    driver
        .find_css("[data-setup-choice='hanzi-eng']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-activity-picker] [aria-pressed='true']")
            .await?
            .len(),
        2
    );
    driver
        .screenshot(&ctx.artifacts.join("selected-types.png"))
        .await?;
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
        .execute(
            zipeek_e2e::js_scripts::practice::MIXED_MIXED_SETUP_PROGRESS_SUMMARY_01,
            vec![],
        )
        .await?;
    driver
        .find_css("[data-practice-start]")
        .await?
        .send_keys(Key::Enter)
        .await?;
    let mut previous = String::new();
    let mut offset = 0;
    for index in 0..5 {
        let current = variant(driver).await?;
        zipeek_e2e::e2e_ensure_ne!(current, previous);
        zipeek_e2e::e2e_ensure_eq!(
            current,
            if index % 2 == 0 {
                "hanzi-eng"
            } else {
                "hanzi-to-meaning"
            }
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-practice-offset]")
                .await?
                .attr("data-practice-offset")
                .await?,
            Some(offset.to_string())
        );
        let progress = driver
            .find_css("[data-match-progress], [data-mcq-progress]")
            .await?;
        let value = if current == "hanzi-eng" {
            progress.attr("data-match-progress").await?
        } else {
            progress.attr("value").await?
        };
        zipeek_e2e::e2e_ensure_eq!(value, Some(index.to_string()));
        zipeek_e2e::e2e_ensure!(
            driver
                .current_url()
                .await?
                .as_str()
                .contains("types=hanzi-to-meaning%2Chanzi-eng")
        );
        if index == 4 {
            driver
                .execute(
                    zipeek_e2e::js_scripts::practice::MIXED_MIXED_SETUP_PROGRESS_SUMMARY_02,
                    vec![],
                )
                .await?;
        }
        offset += if current == "hanzi-eng" { 4 } else { 1 };
        previous = current;
        round(driver, index < 2).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "11 / 14"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-mistakes]")
            .await?
            .text()
            .await?,
        "2"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-highest-streak]")
            .await?
            .text()
            .await?,
        "9"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_css("[data-summary-time]").await?.text().await?,
        "1m 40s"
    );
    driver
        .screenshot(&ctx.artifacts.join("mixed-summary.png"))
        .await?;
    driver
        .find_css("[data-summary-restart]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(variant(driver).await?, "hanzi-to-meaning");
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-correct]")
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("0")
    );
    for _ in 0..5 {
        round(driver, false).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "11 / 11"
    );
    driver
        .find_css("[data-summary-setup]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-activity-picker] [aria-pressed='true']")
            .await?
            .len(),
        1
    );
    Ok(())
}

pub async fn mixed_urls_replay_and_seed(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let url = ctx.url(
        "/#/practice/eng-pinyin?types=eng-pinyin,hanzi-to-meaning,eng-pinyin,invalid&offset=0",
    );
    driver.goto(&url).await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), url);
    let before = driver
        .find_css("[data-practice-round]")
        .await?
        .attr("data-practice-round")
        .await?;
    let seed = zipeek_e2e::page_objects::round::first_word_id(driver).await?;
    driver.refresh().await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-practice-round]")
            .await?
            .attr("data-practice-round")
            .await?,
        before
    );
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        &format!("/practice/eng-pinyin?types=eng-pinyin,hanzi-to-meaning&seed={seed}&offset=40"),
    )
    .await?;
    driver
        .query_single("[data-practice-seed] [data-match-next]")
        .await?;
    round(driver, false).await?;
    zipeek_e2e::e2e_ensure_eq!(variant(driver).await?, "hanzi-to-meaning");
    zipeek_e2e::e2e_ensure!(!driver.current_url().await?.as_str().contains("seed="));
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-practice-offset]")
            .await?
            .attr("data-practice-offset")
            .await?
            .as_deref(),
        Some("4")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-correct]")
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("4")
    );
    let mut previous = String::from("hanzi-to-meaning");
    for _ in 0..6 {
        round(driver, false).await?;
        let current = variant(driver).await?;
        zipeek_e2e::e2e_ensure_ne!(current, previous);
        previous = current;
    }
    driver.refresh().await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-match-correct], [data-mcq-correct]")
            .await?
            .text()
            .await?,
        "Correct: 0"
    );
    let invalid = ctx.url("/#/practice/mcq/meaning-to-pinyin?types=eng-pinyin,unknown&offset=8");
    driver.goto(&invalid).await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), invalid);
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-practice-types]")
            .await?
            .attr("data-practice-types")
            .await?
            .as_deref(),
        Some("meaning-to-pinyin")
    );
    round(driver, false).await?;
    zipeek_e2e::e2e_ensure_eq!(variant(driver).await?, "meaning-to-pinyin");
    zipeek_e2e::e2e_ensure!(!driver.current_url().await?.as_str().contains("types="));
    ctx.checkpoint_visible(
        "mixed-session-playable-after-replay",
        "[data-mcq], [data-matching]",
    )
    .await?;
    ctx.checkpoint("mixed-session-url-restored").await?;
    Ok(())
}
