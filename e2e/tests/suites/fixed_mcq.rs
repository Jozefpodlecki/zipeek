use e2e::{HarnessResult, ScenarioContext};
use std::collections::BTreeMap;
use thirtyfour::prelude::*;

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css("[data-mcq-option='0']:not([disabled])"))
        .single()
        .await
}

async fn answer(driver: &WebDriver, correct: bool) -> HarnessResult<()> {
    ready(driver).await?;
    let index=driver.execute("const c=JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound); return c.slice(1).findIndex(x=>x[0]===c[0][0]);",vec![]).await?.convert::<usize>()?;
    let index = if correct { index } else { (index + 1) % 3 };
    driver
        .find(By::Css(format!("[data-mcq-option='{index}']")))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-mcq-continue]:not([disabled])"))
        .single()
        .await?
        .send_keys(Key::Enter)
        .await?;
    Ok(())
}

async fn summary(driver: &WebDriver) -> HarnessResult<WebElement> {
    let page = driver
        .query(By::Css("[data-practice-summary]"))
        .single()
        .await?;
    driver
        .query(By::Css("[data-summary-chart='ready']"))
        .single()
        .await?;
    driver.execute_async("const done=arguments[arguments.length-1]; function wait(){const s=document.querySelector('[data-summary-first-attempt]');if(s && s.textContent===s.dataset.summaryFirstAttempt)done(true);else requestAnimationFrame(wait);} wait();",vec![]).await?;
    Ok(page)
}

async fn motion(driver: &WebDriver, value: &str) -> HarnessResult<()> {
    driver
        .cdp()
        .send_raw(
            "Emulation.setEmulatedMedia",
            BTreeMap::from([(
                "features",
                vec![BTreeMap::from([
                    ("name", "prefers-reduced-motion"),
                    ("value", value),
                ])],
            )]),
        )
        .await?;
    Ok(())
}

pub async fn fixed_mcq_counts_stats_and_restart(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.add_init_script("window.fixedClock=100000;Date.now=()=>window.fixedClock;")
        .await?;
    for (variant, total) in [
        ("hanzi-to-meaning", 5),
        ("meaning-to-hanzi", 10),
        ("meaning-to-pinyin", 20),
    ] {
        driver
            .execute(
                "window.fixedClock=100000;Date.now=()=>window.fixedClock;",
                vec![],
            )
            .await?;
        driver.goto(ctx.url(&format!("/#/practice/mcq/{variant}?mode=fixed&rounds={total}&hsk=3.0&level=1&offset=18446744073709551615&exclude=1,2"))).await?;
        for index in 0..total {
            ready(driver).await?;
            assert!(
                driver
                    .find_all(By::Css("[data-practice-summary]"))
                    .await?
                    .is_empty()
            );
            let progress = driver.find(By::Css("[data-mcq-progress]")).await?;
            assert_eq!(progress.attr("value").await?, Some(index.to_string()));
            assert_eq!(progress.attr("max").await?, Some(total.to_string()));
            if index == total - 1 {
                driver.execute("window.fixedClock=200000;document.dispatchEvent(new Event('visibilitychange'));",vec![]).await?;
            }
            answer(driver, index >= 2).await?;
        }
        let page = summary(driver).await?;
        assert!(page.text().await?.contains("First-attempt"));
        assert_eq!(
            driver
                .find(By::Css("[data-summary-first-attempt]"))
                .await?
                .text()
                .await?,
            format!("{} / {total}", total - 2)
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-mistakes]"))
                .await?
                .text()
                .await?,
            "2"
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-highest-streak]"))
                .await?
                .text()
                .await?,
            (total - 2).to_string()
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-time]"))
                .await?
                .text()
                .await?,
            "1m 40s"
        );
        driver.execute("window.fixedClock=900000;window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');",vec![]).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-summary-time]"))
                .await?
                .attr("data-summary-time")
                .await?
                .as_deref(),
            Some("100000")
        );
        let old = driver.current_url().await?.to_string();
        driver
            .find(By::Css("[data-summary-restart]"))
            .await?
            .send_keys(Key::Enter)
            .await?;
        ready(driver).await?;
        let new = driver.current_url().await?.to_string();
        assert_ne!(old, new);
        assert!(
            new.contains(&format!("rounds={total}"))
                && new.contains("mode=fixed")
                && new.contains("hsk=3.0")
                && new.contains("level=1")
                && new.contains("exclude=1%2C2")
                && !new.contains("seed=")
        );
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-progress]"))
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-correct]"))
                .await?
                .attr("data-mcq-correct")
                .await?
                .as_deref(),
            Some("0")
        );
        assert!(
            driver
                .execute(
                    "return window.oldSummaryCanvas.dataset.summaryChart==='destroyed';",
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        answer(driver, true).await?;
        ready(driver).await?;
        driver.refresh().await?;
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-progress]"))
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
        driver
            .execute("location.hash='#/practice';", vec![])
            .await?;
        driver
            .query(By::Css("[data-practice-start]:not([disabled])"))
            .single()
            .await?;
        driver.back().await?;
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-progress]"))
                .await?
                .attr("value")
                .await?
                .as_deref(),
            Some("0")
        );
    }
    driver
        .execute(
            include_str!("../../fixtures/browser_probes/empty_strict_pool.js"),
            vec![],
        )
        .await?;
    driver
        .execute(
            "location.hash='#/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5&offset=0';",
            vec![],
        )
        .await?;
    for index in 0..5 {
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-progress]"))
                .await?
                .attr("value")
                .await?,
            Some(index.to_string())
        );
        answer(driver, true).await?;
    }
    summary(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    driver
        .find(By::Css("[data-summary-restart]"))
        .await?
        .click()
        .await?;
    ready(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-progress]"))
            .await?
            .attr("value")
            .await?
            .as_deref(),
        Some("0")
    );
    Ok(())
}

pub async fn fixed_summary_motion_layout_and_setup(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    motion(driver, "reduce").await?;
    driver
        .goto(ctx.url("/#/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5&level=1&hsk=3.0"))
        .await?;
    for _ in 0..5 {
        answer(driver, true).await?;
    }
    summary(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-ring]"))
            .await?
            .attr("data-chart-animation")
            .await?
            .as_deref(),
        Some("0")
    );
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    for dark in [true, false] {
        let current = driver
            .execute(
                "return document.documentElement.classList.contains('dark');",
                vec![],
            )
            .await?
            .convert::<bool>()?;
        if current != dark {
            driver
                .find(By::Css("[data-theme-toggle]"))
                .await?
                .click()
                .await?;
        }
        for width in [400, 768, 1280] {
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(width, 900, 1.0, width < 768)
                .await?;
            assert!(driver.execute("return document.documentElement.scrollWidth<=innerWidth && document.querySelector('[data-summary-ring]').width>0;",vec![]).await?.convert::<bool>()?);
            driver
                .screenshot(&ctx.artifacts.join(format!(
                    "summary-{}-{width}.png",
                    if dark { "dark" } else { "light" }
                )))
                .await?;
        }
    }
    driver.execute("window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');window.entropy=0;Object.defineProperty(crypto,'getRandomValues',{configurable:true,value:bytes=>{bytes.fill(0);bytes[0]=window.entropy++;return bytes;}});",vec![]).await?;
    driver
        .find(By::Css("[data-summary-restart]"))
        .await?
        .click()
        .await?;
    ready(driver).await?;
    let first_restart = driver.current_url().await?.to_string();
    assert!(
        driver
            .execute(
                "return window.oldSummaryCanvas.dataset.summaryChart==='destroyed';",
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
    assert_eq!(
        driver
            .find(By::Css("[data-summary-ring]"))
            .await?
            .attr("data-chart-animation")
            .await?
            .as_deref(),
        Some("300")
    );
    motion(driver, "reduce").await?;
    driver
        .query(By::Css("[data-summary-ring][data-chart-animation='0']"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-summary-restart]"))
        .await?
        .click()
        .await?;
    ready(driver).await?;
    assert_ne!(first_restart, driver.current_url().await?.to_string());
    for _ in 0..5 {
        answer(driver, false).await?;
    }
    summary(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "0 / 5"
    );
    driver
        .execute(
            "window.oldSummaryCanvas=document.querySelector('[data-summary-ring]');",
            vec![],
        )
        .await?;
    driver
        .find(By::Css("[data-summary-setup]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-practice-start]:not([disabled])"))
        .single()
        .await?;
    assert_eq!(driver.current_url().await?.fragment(), Some("/practice"));
    assert_eq!(
        driver
            .find(By::Css("[data-mode-picker] [aria-pressed='true']"))
            .await?
            .text()
            .await?,
        "Endless"
    );
    assert!(
        driver
            .find_all(By::Css("[data-round-counts]"))
            .await?
            .is_empty()
    );
    assert!(
        driver
            .execute(
                "return window.oldSummaryCanvas.dataset.summaryChart==='destroyed';",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    ctx.block_urls(&["*vendor/summary/summary.mjs*"]).await?;
    driver
        .execute(
            "location.hash='#/practice/mcq/hanzi-to-meaning?mode=fixed&rounds=5';",
            vec![],
        )
        .await?;
    driver.refresh().await?;
    for _ in 0..5 {
        answer(driver, true).await?;
    }
    driver
        .query(By::Css("[data-summary-chart-error]"))
        .single()
        .await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "5 / 5"
    );
    assert!(
        driver
            .find(By::Css("[data-summary-restart]"))
            .await?
            .is_enabled()
            .await?
    );
    driver
        .find(By::Css("[data-summary-setup]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-practice-start]:not([disabled])"))
        .single()
        .await?;
    Ok(())
}
