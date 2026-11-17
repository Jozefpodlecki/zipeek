use thirtyfour::prelude::*;
use zipeek_e2e::ChromeDevTools;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

use zipeek_e2e::harness::browser_scripts;
use zipeek_e2e::page_objects::matching::{self as match_page, Side};

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    match_page::ready(driver).await
}
async fn pairs(driver: &WebDriver) -> HarnessResult<Vec<usize>> {
    Ok(zipeek_e2e::page_objects::round::matching_pairs(driver).await?)
}
async fn match_all(driver: &WebDriver) -> HarnessResult<()> {
    ready(driver).await?;
    for (left, right) in pairs(driver).await?.into_iter().enumerate() {
        match_page::click_card(driver, Side::Left, left).await?;
        match_page::click_card(driver, Side::Right, right).await?;
    }
    driver.query_single(match_page::ENABLED_NEXT).await?;
    Ok(())
}
async fn wrong(driver: &WebDriver) -> HarnessResult<()> {
    let right = (pairs(driver).await?[0] + 1) % 4;
    match_page::click_card(driver, Side::Left, 0).await?;
    match_page::click_card(driver, Side::Right, right).await?;
    driver.query_first("[data-match-feedback='wrong']").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css("[data-match-feedback='wrong']")
            .await?
            .len(),
        2
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(browser_scripts::ASSERT_MATCHING_CARDS_DISABLED, vec![])
            .await?
            .convert::<bool>()?,
        "All matching cards must be disabled during incorrect-answer feedback"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .execute_async(browser_scripts::WAIT_FOR_MATCHING_FEEDBACK_TO_CLEAR, vec![])
            .await?
            .convert::<bool>()?,
        "Incorrect-answer feedback must eventually clear"
    );
    Ok(())
}
async fn next(driver: &WebDriver, offset: &str) -> HarnessResult<()> {
    match_page::continue_round(driver).await?;
    driver
        .query_single(format!(
            "[data-practice-offset='{offset}'] [data-match-count='0']"
        ))
        .await?;
    Ok(())
}
async fn summary(driver: &WebDriver) -> HarnessResult<()> {
    driver.query_single(match_page::SUMMARY).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .execute_async(browser_scripts::WAIT_FOR_MATCHING_SUMMARY_TO_SETTLE, vec![])
            .await?
            .convert::<bool>()?,
        "Matching summary must settle to the reported first-attempt score"
    );
    Ok(())
}

async fn presentation(ctx: &ScenarioContext, variant: &str) -> HarnessResult<()> {
    let driver = &ctx.driver;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-pinyin]").await?.len(),
        if variant == "hanzi-eng" { 4 } else { 0 }
    );
    for dark in [true, false] {
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_PRESENTATION_01,
                vec![dark.into()],
            )
            .await?;
        for width in [400, 1280, 1920] {
            if width == 1920 && (variant != "hanzi-eng" || !dark) {
                continue;
            }
            ctx.viewport()
                .responsive(width, if width >= 1280 { 720 } else { 900 }, width < 768)
                .await?;
            driver
                .find_css("[data-matching] h1")
                .await?
                .scroll_into_view()
                .await?;
            driver
                .execute_async(
                    zipeek_e2e::js_scripts::practice::MATCHING_PRESENTATION_02,
                    vec![],
                )
                .await?;
            zipeek_e2e::e2e_ensure!(
                driver
                    .execute(
                        zipeek_e2e::js_scripts::practice::ASSERT_MATCHING_LAYOUT,
                        vec![]
                    )
                    .await?
                    .convert::<bool>()?
            );
            if width >= 1280 {
                let overflow = driver
                    .execute(
                        zipeek_e2e::js_scripts::practice::MATCHING_PRESENTATION_04,
                        vec![],
                    )
                    .await?
                    .convert::<i64>()?;
                if overflow > 0 {
                    driver
                        .screenshot(&ctx.artifacts.join(format!(
                            "matching-overflow-{variant}-{}-{width}.png",
                            if dark { "dark" } else { "light" }
                        )))
                        .await?;
                }
                zipeek_e2e::e2e_ensure_eq!(
                    overflow,
                    0,
                    "Desktop matching page must fit without vertical scrolling; overflow: {overflow}px"
                );
            }
            if variant == "hanzi-eng" {
                zipeek_e2e::e2e_ensure!(
                    driver
                        .execute(
                            zipeek_e2e::js_scripts::practice::MATCHING_PRESENTATION_05,
                            vec![]
                        )
                        .await?
                        .convert::<bool>()?
                );
            }
            driver
                .screenshot(&ctx.artifacts.join(format!(
                    "fresh-{variant}-{}-{width}.png",
                    if dark { "dark" } else { "light" }
                )))
                .await?;
        }
    }
    Ok(())
}

pub async fn matching_interaction_and_layout(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for variant in ["eng-pinyin", "pinyin-hanzi", "hanzi-eng"] {
        driver
            .goto(ctx.url(&format!("/#/practice/{variant}?offset=0&exclude=1,2")))
            .await?;
        ready(driver).await?;
        ctx.checkpoint_visible("matching-board-loaded", "[data-matching] [data-match-next]")
            .await?;
        zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-match-side]").await?.len(), 8);
        zipeek_e2e::e2e_ensure!(
            !driver
                .find_css("[data-match-next]")
                .await?
                .is_enabled()
                .await?
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-match-progress]")
                .await?
                .is_empty()
        );
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_01,
                vec![],
            )
            .await?;
        driver
            .execute(browser_scripts::RESET_SPEECH_EVENTS, vec![])
            .await?;
        presentation(&ctx, variant).await?;
        let listens = driver.find_all_css("[data-listen]").await?;
        zipeek_e2e::e2e_ensure_eq!(listens.len(), if variant == "pinyin-hanzi" { 8 } else { 4 });
        listens[0].click().await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(browser_scripts::ASSERT_MATCHING_LISTEN, vec![])
                .await?
                .convert::<bool>()?
        );
        match_page::card(driver, Side::Left, 0)
            .await?
            .send_keys(" ")
            .await?;
        match_page::click_card(driver, Side::Left, 1).await?;
        zipeek_e2e::e2e_ensure_eq!(
            match_page::card(driver, Side::Left, 0)
                .await?
                .attr("data-match-selected")
                .await?
                .as_deref(),
            Some("true")
        );
        zipeek_e2e::e2e_ensure_eq!(
            match_page::card(driver, Side::Left, 0)
                .await?
                .attr("aria-pressed")
                .await?
                .as_deref(),
            Some("true"),
            "Selected matching cards must expose the pressed state to assistive technology"
        );
        match_page::click_card(driver, Side::Left, 0).await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-match-selected='true']")
                .await?
                .is_empty()
        );
        match_page::click_card(driver, Side::Right, 0).await?;
        driver
            .action_chain()
            .send_keys(Key::Escape)
            .perform()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-match-selected='true']")
                .await?
                .is_empty()
        );
        wrong(driver).await?;
        for (left, right) in pairs(driver).await?.into_iter().enumerate() {
            match_page::click_card(driver, Side::Right, right).await?;
            match_page::card(driver, Side::Left, left)
                .await?
                .send_keys(Key::Enter)
                .await?;
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_css("[data-match-count]")
                    .await?
                    .attr("data-match-count")
                    .await?,
                Some((left + 1).to_string())
            );
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_css("[data-match-next]")
                    .await?
                    .is_enabled()
                    .await?,
                left == 3
            );
        }
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-correct]")
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("4")
        );
        zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-match-flame]").await?.len(), 1);
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_02,
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        for dark in [true, false] {
            let current = driver
                .execute(
                    zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_03,
                    vec![],
                )
                .await?
                .convert::<bool>()?;
            if current != dark {
                driver
                    .find_css("[data-theme-toggle]")
                    .await?
                    .click()
                    .await?;
            }
            for width in [400, 768, 1280] {
                ctx.viewport()
                    .responsive(width, if width >= 1280 { 720 } else { 900 }, width < 768)
                    .await?;
                driver
                    .find_css("[data-matching] h1")
                    .await?
                    .scroll_into_view()
                    .await?;
                zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_04,vec![]).await?.convert::<bool>()?);
                driver
                    .screenshot(&ctx.artifacts.join(format!(
                        "matching-{variant}-{}-{width}.png",
                        if dark { "dark" } else { "light" }
                    )))
                    .await?;
            }
        }
        next(driver, "4").await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-correct]")
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("4")
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .current_url()
                .await?
                .as_str()
                .contains("exclude=1%2C2")
        );
        wrong(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-streak]")
                .await?
                .attr("data-match-streak")
                .await?
                .as_deref(),
            Some("0")
        );
        driver.refresh().await?;
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-correct]")
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("0")
        );
    }
    ctx.open_route("/practice/mcq/hanzi-to-meaning?offset=0")
        .await?;
    let pinyin = driver
        .query_single("[data-mcq] [data-practice-pinyin]")
        .await?;
    zipeek_e2e::e2e_ensure!(!pinyin.text().await?.is_empty());
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_05,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    for (width, dark) in [(400, false), (1280, true)] {
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_INTERACTION_AND_LAYOUT_06,
                vec![dark.into()],
            )
            .await?;
        ctx.viewport()
            .responsive(width, if width >= 1280 { 720 } else { 900 }, width < 768)
            .await?;
        driver
            .find_css("[data-mcq] h1")
            .await?
            .scroll_into_view()
            .await?;
        driver
            .screenshot(&ctx.artifacts.join(format!("mcq-pinyin-{width}.png")))
            .await?;
    }
    Ok(())
}

pub async fn matching_fixed_summary_and_restart(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (variant, total) in [("eng-pinyin", 5), ("pinyin-hanzi", 10), ("hanzi-eng", 20)] {
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_01,
                vec![],
            )
            .await?;
        ctx.viewport().responsive(1280, 900, false).await?;
        driver
            .goto(ctx.url(&format!(
                "/#/practice/{variant}?mode=fixed&rounds={total}&offset=18446744073709551615"
            )))
            .await?;
        for round in 0..total {
            ready(driver).await?;
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_css("[data-match-progress]")
                    .await?
                    .attr("data-match-progress")
                    .await?,
                Some(round.to_string())
            );
            if round == 0 {
                wrong(driver).await?;
            }
            match_all(driver).await?;
            zipeek_e2e::e2e_ensure!(
                driver
                    .find_all_css("[data-practice-summary]")
                    .await?
                    .is_empty()
            );
            if round == total - 1 {
                driver.execute(zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_02, vec![]).await?;
            }
            match_page::continue_round(driver).await?;
        }
        summary(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-summary-first-attempt]")
                .await?
                .text()
                .await?,
            format!("{} / {}", total * 4 - 2, total * 4)
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-summary-mistakes]")
                .await?
                .text()
                .await?,
            "1"
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-summary-highest-streak]")
                .await?
                .text()
                .await?,
            (total * 4).to_string()
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_css("[data-summary-time]").await?.text().await?,
            "1m 40s"
        );
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_03,
                vec![],
            )
            .await?;
        let old = driver.current_url().await?.to_string();
        driver
            .find_css("[data-summary-restart]")
            .await?
            .click()
            .await?;
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_ne!(old, driver.current_url().await?.to_string());
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-progress]")
                .await?
                .attr("data-match-progress")
                .await?
                .as_deref(),
            Some("0")
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_FIXED_SUMMARY_AND_RESTART_04,
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        zipeek_e2e::page_objects::browser::navigate_hash(driver, "/practice").await?;
        driver
            .query_single(
                "[data-practice-progress] [data-practice-step='path'][aria-current='step']",
            )
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-practice-start]")
                .await?
                .is_empty(),
            "Returning to setup should show the first step, not a previously completed summary"
        );
    }
    Ok(())
}

pub async fn matching_recovery_seed_and_same_url(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ChromeDevTools::new(driver).reduced_motion(true).await?;
    ctx.open_route("/practice/pinyin-hanzi?offset=18446744073709551615")
        .await?;
    ready(driver).await?;
    match_all(driver).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_RECOVERY_SEED_AND_SAME_URL_01,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    let seed = zipeek_e2e::page_objects::round::first_word_id(driver).await?;
    next(driver, "0").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-match-correct]")
            .await?
            .attr("data-match-correct")
            .await?
            .as_deref(),
        Some("4")
    );
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        &format!("/practice/pinyin-hanzi?seed={seed}&offset=40"),
    )
    .await?;
    driver
        .query_single("[data-practice-seed] [data-match-next]")
        .await?;
    match_all(driver).await?;
    next(driver, "4").await?;
    zipeek_e2e::e2e_ensure!(!driver.current_url().await?.as_str().contains("seed="));
    driver
        .execute(
            include_str!("../../../fixtures/browser_probes/empty_strict_pool.js"),
            vec![],
        )
        .await?;
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        "/practice/hanzi-eng?offset=0&mode=fixed&rounds=5",
    )
    .await?;
    for round in 0..5 {
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-match-progress]")
                .await?
                .attr("data-match-progress")
                .await?,
            Some(round.to_string())
        );
        match_all(driver).await?;
        match_page::continue_round(driver).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "20 / 20"
    );
    driver
        .find_css("[data-summary-setup]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-progress] [data-practice-step='path'][aria-current='step']")
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::MATCHING_MATCHING_RECOVERY_SEED_AND_SAME_URL_02,
            vec![],
        )
        .await?;
    driver
        .query_single("[data-practice-state='failed'] [data-match-setup]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-progress] [data-practice-step='path'][aria-current='step']")
        .await?;
    ctx.checkpoint_visible(
        "matching-recovery-navigation",
        "[data-practice-progress] [data-practice-step='path'][aria-current='step']",
    )
    .await?;
    Ok(())
}
