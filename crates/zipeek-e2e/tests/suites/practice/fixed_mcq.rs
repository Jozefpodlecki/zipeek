use thirtyfour::prelude::*;
use zipeek_e2e::ChromeDevTools;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    zipeek_e2e::page_objects::mcq::ready(driver).await
}

async fn answer(driver: &WebDriver, correct: bool) -> HarnessResult<()> {
    zipeek_e2e::page_objects::mcq::answer(driver, correct).await
}

async fn summary(driver: &WebDriver) -> HarnessResult<WebElement> {
    let page = driver.query_single("[data-practice-summary]").await?;
    driver.query_single("[data-summary-chart='ready']").await?;
    driver
        .execute_async(
            zipeek_e2e::js_scripts::practice::FIXED_MCQ_SUMMARY_01,
            vec![],
        )
        .await?;
    Ok(page)
}

async fn motion(driver: &WebDriver, value: &str) -> HarnessResult<()> {
    ChromeDevTools::new(driver)
        .reduced_motion(value == "reduce")
        .await?;
    Ok(())
}

pub async fn fixed_mcq_counts_stats_and_restart(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.add_init_script("window.fixedClock=100000;Date.now=()=>window.fixedClock;")
        .await?;
    ctx.checkpoint("fixed-mcq-initial-state").await?;
    for (variant, total) in [
        ("hanzi-to-meaning", 5),
        ("meaning-to-hanzi", 10),
        ("meaning-to-pinyin", 20),
    ] {
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_01,
                vec![],
            )
            .await?;
        driver.goto(ctx.url(&format!("/#/practice/mcq/{variant}?mode=fixed&rounds={total}&hsk=3.0&level=1&offset=18446744073709551615&exclude=1,2"))).await?;
        for index in 0..total {
            ready(driver).await?;
            ctx.checkpoint_visible("fixed-mcq-progress", "[data-mcq-progress]")
                .await?;
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_all_css("[data-mcq-option][aria-pressed='true']")
                    .await?
                    .len(),
                0,
                "Every fixed-session MCQ round must begin with no selected answer"
            );
            zipeek_e2e::e2e_ensure!(
                driver
                    .find_all_css("[data-practice-summary]")
                    .await?
                    .is_empty()
            );
            let progress = driver.find_css("[data-mcq-progress]").await?;
            zipeek_e2e::e2e_ensure_eq!(progress.attr("value").await?, Some(index.to_string()));
            zipeek_e2e::e2e_ensure_eq!(progress.attr("max").await?, Some(total.to_string()));
            zipeek_e2e::e2e_ensure!(
                driver
                    .find_css("[data-mcq-options]")
                    .await?
                    .is_displayed()
                    .await?,
                "Fixed MCQ must retain a visible answer region until the final round"
            );
            if index == total - 1 {
                driver.execute(zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_02,vec![]).await?;
            }
            answer(driver, index >= 2).await?;
        }
        let page = summary(driver).await?;
        zipeek_e2e::e2e_ensure!(page.text().await?.contains("First-attempt"));
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-summary-first-attempt]")
                .await?
                .text()
                .await?,
            format!("{} / {total}", total - 2)
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
            (total - 2).to_string()
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver.find_css("[data-summary-time]").await?.text().await?,
            "1m 40s"
        );
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_03,
                vec![],
            )
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-summary-time]")
                .await?
                .attr("data-summary-time")
                .await?
                .as_deref(),
            Some("100000")
        );
        let old = driver.current_url().await?.to_string();
        driver
            .find_css("[data-summary-restart]")
            .await?
            .send_keys(Key::Enter)
            .await?;
        ready(driver).await?;
        let new = driver.current_url().await?.to_string();
        zipeek_e2e::e2e_ensure_ne!(old, new);
        zipeek_e2e::e2e_ensure!(
            new.contains(&format!("rounds={total}"))
                && new.contains("mode=fixed")
                && new.contains("hsk=3.0")
                && new.contains("level=1")
                && new.contains("exclude=1%2C2")
                && !new.contains("seed=")
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-progress]")
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-correct]")
                .await?
                .attr("data-mcq-correct")
                .await?
                .as_deref(),
            Some("0")
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(
                    zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_MCQ_COUNTS_STATS_AND_RESTART_04,
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        answer(driver, true).await?;
        ready(driver).await?;
        driver.refresh().await?;
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-progress]")
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
        zipeek_e2e::page_objects::browser::navigate_hash(driver, "/practice").await?;
        driver
            .query_single("[data-practice-start]:not([disabled])")
            .await?;
        driver.back().await?;
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-progress]")
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
    }
    driver
        .execute(
            include_str!("../../../fixtures/browser_probes/empty_strict_pool.js"),
            vec![],
        )
        .await?;
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        "/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5&offset=0",
    )
    .await?;
    for index in 0..5 {
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-progress]")
                .await?
                .attr("value")
                .await?,
            Some(index.to_string())
        );
        answer(driver, true).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    driver
        .find_css("[data-summary-restart]")
        .await?
        .click()
        .await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-progress]")
            .await?
            .attr("value")
            .await?
            .as_deref(),
        Some("0")
    );
    ctx.checkpoint_visible("fixed-mcq-final-state", "[data-mcq]")
        .await?;
    ctx.checkpoint("fixed-mcq-verified-result").await?;
    Ok(())
}

pub async fn fixed_summary_motion_layout_and_setup(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    motion(driver, "reduce").await?;
    ctx.open_route("/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5&level=1&hsk=3.0")
        .await?;
    for _ in 0..5 {
        answer(driver, true).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-ring]")
            .await?
            .attr("data-chart-animation")
            .await?
            .as_deref(),
        Some("0")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    for dark in [true, false] {
        let current = driver
            .execute(
                zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_01,
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
            ctx.viewport().responsive(width, 900, width < 768).await?;
            zipeek_e2e::e2e_ensure!(driver.execute(zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_02,vec![]).await?.convert::<bool>()?);
            driver
                .screenshot(&ctx.artifacts.join(format!(
                    "summary-{}-{width}.png",
                    if dark { "dark" } else { "light" }
                )))
                .await?;
        }
    }
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_03,
            vec![],
        )
        .await?;
    driver
        .find_css("[data-summary-restart]")
        .await?
        .click()
        .await?;
    ready(driver).await?;
    let first_restart = driver.current_url().await?.to_string();
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_04,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    motion(driver, "no-preference").await?;
    for _ in 0..5 {
        answer(driver, true).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-ring]")
            .await?
            .attr("data-chart-animation")
            .await?
            .as_deref(),
        Some("300")
    );
    motion(driver, "reduce").await?;
    driver
        .query_single("[data-summary-ring][data-chart-animation='0']")
        .await?;
    driver
        .find_css("[data-summary-restart]")
        .await?
        .click()
        .await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_ne!(first_restart, driver.current_url().await?.to_string());
    for _ in 0..5 {
        answer(driver, false).await?;
    }
    summary(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "0 / 5"
    );
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_05,
            vec![],
        )
        .await?;
    driver
        .find_css("[data-summary-setup]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.fragment(), Some("/practice"));
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mode-picker] [aria-pressed='true']")
            .await?
            .text()
            .await?,
        "Endless"
    );
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-round-counts]").await?.is_empty());
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::FIXED_MCQ_FIXED_SUMMARY_MOTION_LAYOUT_AND_SETUP_06,
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    ctx.block_urls(&["*vendor/summary/summary.mjs*"]).await?;
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        "/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5",
    )
    .await?;
    driver.refresh().await?;
    for _ in 0..5 {
        answer(driver, true).await?;
    }
    driver.query_single("[data-summary-chart-error]").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-summary-first-attempt]")
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-summary-restart]")
            .await?
            .is_enabled()
            .await?
    );
    driver
        .find_css("[data-summary-setup]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    ctx.checkpoint_visible(
        "fixed-session-summary-navigation",
        "[data-practice-start]:not([disabled])",
    )
    .await?;
    Ok(())
}
