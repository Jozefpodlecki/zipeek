use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

async fn choice(driver: &WebDriver, value: &str) -> WebDriverResult<WebElement> {
    driver
        .find(By::Css(format!("[data-setup-choice='{value}']")))
        .await
}

async fn setup(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css(
            "[data-practice-setup] [data-practice-start]:not([disabled])",
        ))
        .single()
        .await
}

pub async fn setup_controls_generate_all_six_practice_urls(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.execute("window.savedSetupExclusions = '[1,2,2,\"bad\",0,2147483647]'; localStorage.setItem('zipeek.excluded_words', window.savedSetupExclusions); location.hash = '#/practice';", vec![]).await?;
    setup(driver).await?;
    assert_eq!(
        driver
            .find_all(By::Css("[data-activity-picker] button"))
            .await?
            .len(),
        6
    );
    assert_eq!(
        driver
            .find(By::Css("[data-exclusions-count]"))
            .await?
            .attr("data-exclusions-count")
            .await?
            .as_deref(),
        Some("2")
    );
    assert_eq!(
        choice(driver, "hanzi-to-meaning")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    assert!(
        driver
            .find_all(By::Css("[data-round-counts]"))
            .await?
            .is_empty()
    );
    choice(driver, "Fixed").await?.send_keys(" ").await?;
    assert_eq!(
        choice(driver, "rounds-10")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    choice(driver, "rounds-5").await?.click().await?;
    choice(driver, "Endless").await?.click().await?;
    choice(driver, "Fixed").await?.click().await?;
    assert_eq!(
        choice(driver, "rounds-5")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    choice(driver, "level-7").await?.click().await?;
    assert_eq!(choice(driver, "level-7").await?.text().await?, "HSK 7–9");
    choice(driver, "HSK 2.0").await?.click().await?;
    assert!(
        driver
            .find_all(By::Css("[data-setup-choice='level-7']"))
            .await?
            .is_empty()
    );
    assert_eq!(
        choice(driver, "level-1")
            .await?
            .attr("aria-pressed")
            .await?
            .as_deref(),
        Some("true")
    );
    for (variant, path) in [
        ("hanzi-to-meaning", "/practice/mcq/hanzi-to-meaning"),
        ("meaning-to-hanzi", "/practice/mcq/meaning-to-hanzi"),
        ("meaning-to-pinyin", "/practice/mcq/meaning-to-pinyin"),
        ("eng-pinyin", "/practice/eng-pinyin"),
        ("pinyin-hanzi", "/practice/pinyin-hanzi"),
        ("hanzi-eng", "/practice/hanzi-eng"),
    ] {
        setup(driver).await?;
        choice(driver, variant).await?.click().await?;
        choice(driver, "HSK 2.0").await?.click().await?;
        choice(driver, "level-3").await?.click().await?;
        choice(driver, "Fixed").await?.click().await?;
        choice(driver, "rounds-20").await?.click().await?;
        driver
            .find(By::Css("[data-practice-start]"))
            .await?
            .send_keys(Key::Enter)
            .await?;
        driver
            .query(By::Css(format!(
                "[data-practice-state='ready'][data-practice-variant='{variant}']"
            )))
            .single()
            .await?;
        let url = driver.current_url().await?;
        assert!(
            url.as_str().starts_with(&ctx.url(&format!("/#{path}?"))),
            "{url}"
        );
        assert!(driver.execute("const q = new URLSearchParams(location.hash.split('?')[1]); return q.get('mode') === 'fixed' && q.get('rounds') === '20' && q.get('hsk') === '2.0' && q.get('level') === '3' && q.get('offset') === '0' && q.get('exclude') === '1,2' && !q.has('seed') && !q.has('pos') && !location.hash.includes('%252C') && localStorage.getItem('zipeek.excluded_words') === window.savedSetupExclusions;",vec![]).await?.convert::<bool>()?);
        driver.back().await?;
    }
    setup(driver).await?;
    driver
        .find(By::Css("[data-exclusions-reset]"))
        .await?
        .click()
        .await?;
    assert!(
        driver
            .find_all(By::Css("[data-exclusions-count]"))
            .await?
            .is_empty()
    );
    assert!(driver.execute("return localStorage.getItem('zipeek.excluded_words') === null && location.hash === '#/practice';",vec![]).await?.convert::<bool>()?);
    for (theme, dark) in [("dark", true), ("light", false)] {
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
            assert!(
                driver
                    .execute(
                        "return document.documentElement.scrollWidth <= innerWidth;",
                        vec![]
                    )
                    .await?
                    .convert::<bool>()?
            );
            driver
                .screenshot(&ctx.artifacts.join(format!("setup-{theme}-{width}.png")))
                .await?;
        }
    }
    setup(driver).await?.send_keys(Key::Enter).await?;
    driver
        .query(By::Css(
            "[data-practice-state='ready'][data-practice-variant='hanzi-to-meaning']",
        ))
        .single()
        .await?;
    assert!(driver.execute("const q = new URLSearchParams(location.hash.split('?')[1]); return q.get('mode') === 'endless' && q.get('hsk') === '3.0' && q.get('level') === '1' && q.get('offset') === '0' && !q.has('exclude') && !q.has('rounds') && !q.has('seed');",vec![]).await?.convert::<bool>()?);
    Ok(())
}

pub async fn setup_exclusion_failure_retry_and_reset_error(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .execute(
            include_str!("../../fixtures/browser_probes/practice_queries.js"),
            vec![],
        )
        .await?;
    driver.execute("localStorage.setItem('zipeek.excluded_words', '[1]'); window.practiceQueryMode = 'hold-fail'; location.hash = '#/practice';",vec![]).await?;
    driver.execute_async("const done = arguments[arguments.length - 1]; function wait() {if (window.releasePracticeQuery) done(true); else requestAnimationFrame(wait);} wait();",vec![]).await?;
    assert!(
        !driver
            .find(By::Css("[data-practice-start]"))
            .await?
            .is_enabled()
            .await?
    );
    driver
        .execute("window.releasePracticeQuery();", vec![])
        .await?;
    driver
        .query(By::Css("[data-exclusions-retry]"))
        .single()
        .await?
        .click()
        .await?;
    setup(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-exclusions-count]"))
            .await?
            .attr("data-exclusions-count")
            .await?
            .as_deref(),
        Some("1")
    );
    driver.execute("window.originalRemoveItem = Storage.prototype.removeItem; Storage.prototype.removeItem = function() { throw new DOMException('blocked', 'SecurityError'); };",vec![]).await?;
    driver
        .find(By::Css("[data-exclusions-reset]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-practice-setup] [role='alert']"))
        .single()
        .await?;
    assert!(
        driver
            .execute(
                "return localStorage.getItem('zipeek.excluded_words') === '[1]';",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    driver
        .execute(
            "Storage.prototype.removeItem = window.originalRemoveItem;",
            vec![],
        )
        .await?;
    driver
        .find(By::Css("[data-exclusions-reset]"))
        .await?
        .click()
        .await?;
    assert!(
        driver
            .find_all(By::Css("[data-exclusions-count]"))
            .await?
            .is_empty()
    );
    driver
        .execute(
            "localStorage.setItem('zipeek.excluded_words', '{invalid');",
            vec![],
        )
        .await?;
    driver.refresh().await?;
    setup(driver).await?;
    assert!(
        driver
            .find_all(By::Css("[data-exclusions-count]"))
            .await?
            .is_empty()
    );
    assert!(
        driver
            .execute(
                "return localStorage.getItem('zipeek.excluded_words') === '{invalid';",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    Ok(())
}
