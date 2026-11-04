use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

async fn ready(driver: &WebDriver, variant: &str) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css(format!(
            "[data-practice-state='ready'][data-practice-variant='{variant}']"
        )))
        .single()
        .await
}

pub async fn practice_rounds_normalize_and_replay(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (variant, path, title) in [
        ("eng-pinyin", "/practice/eng-pinyin", "English to pinyin"),
        ("pinyin-hanzi", "/practice/pinyin-hanzi", "Pinyin to hanzi"),
        ("hanzi-eng", "/practice/hanzi-eng", "Hanzi to English"),
        (
            "hanzi-to-meaning",
            "/practice/mcq/hanzi-to-meaning",
            "Hanzi to meaning",
        ),
        (
            "meaning-to-hanzi",
            "/practice/mcq/meaning-to-hanzi",
            "Meaning to hanzi",
        ),
        (
            "meaning-to-pinyin",
            "/practice/mcq/meaning-to-pinyin",
            "Meaning to pinyin",
        ),
    ] {
        let url = ctx.url(&format!("/#{path}?mode=fixed&rounds=5&offset=8&seed=1"));
        driver.goto(&url).await?;
        let page = ready(driver, variant).await?;
        assert_eq!(
            page.attr("data-practice-offset").await?.as_deref(),
            Some("8")
        );
        assert!(page.attr("data-practice-seed").await?.is_none());
        assert_eq!(driver.title().await?, format!("{title} · Zipeek"));
        assert_eq!(driver.current_url().await?.as_str(), url);
        let round = page
            .attr("data-practice-round")
            .await?
            .ok_or("Missing debug round")?;
        driver.refresh().await?;
        assert_eq!(
            ready(driver, variant)
                .await?
                .attr("data-practice-round")
                .await?
                .as_deref(),
            Some(round.as_str())
        );
    }
    let seed = driver.execute("return JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound)[0][0].toString();", vec![]).await?.convert::<String>()?;
    driver
        .execute(
            format!("localStorage.setItem('zipeek.excluded_words', '[{seed}]');"),
            vec![],
        )
        .await?;
    let shared = ctx.url(&format!("/#/practice/eng-pinyin?seed={seed}&offset=40"));
    driver.goto(&shared).await?;
    let page = ready(driver, "eng-pinyin").await?;
    assert_eq!(
        page.attr("data-practice-seed").await?.as_deref(),
        Some(seed.as_str())
    );
    assert_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("0")
    );
    assert_eq!(driver.current_url().await?.as_str(), shared);
    let exclusions = ctx.url(&format!(
        "/#/practice/eng-pinyin?seed={seed}&exclude={seed},2147483647&offset=8"
    ));
    driver.goto(&exclusions).await?;
    let page = ready(driver, "eng-pinyin").await?;
    assert!(page.attr("data-practice-seed").await?.is_none());
    assert_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("8")
    );
    assert_eq!(
        page.attr("data-practice-exclusions").await?.as_deref(),
        Some(seed.as_str())
    );
    assert!(driver.execute(format!("return JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound).every(card => card[0] !== {seed}) && localStorage.getItem('zipeek.excluded_words') === '[{seed}]';"), vec![]).await?.convert::<bool>()?);
    assert_eq!(driver.current_url().await?.as_str(), exclusions);
    driver
        .goto(ctx.url("/#/practice/eng-pinyin?seed=2147483647&offset=12"))
        .await?;
    assert_eq!(
        ready(driver, "eng-pinyin")
            .await?
            .attr("data-practice-offset")
            .await?
            .as_deref(),
        Some("12")
    );
    Ok(())
}

pub async fn late_queries_and_retry_keep_current_route(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .execute(
            include_str!("../../fixtures/browser_probes/practice_queries.js"),
            vec![],
        )
        .await?;
    driver.execute("window.practiceQueryMode = 'hold-fail'; location.hash = '#/practice/eng-pinyin?offset=4';", vec![]).await?;
    driver.execute_async("const done = arguments[arguments.length - 1]; function wait() { if (window.releasePracticeQuery) done(true); else requestAnimationFrame(wait); } wait();", vec![]).await?;
    driver
        .query(By::Css("[data-practice-state='loading'] [role='status']"))
        .single()
        .await?;
    assert!(
        driver
            .execute(
                "return document.activeElement.id === 'main-content';",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    driver
        .execute(
            "location.hash = '#/practice/mcq/meaning-to-hanzi?offset=8';",
            vec![],
        )
        .await?;
    ready(driver, "meaning-to-hanzi").await?;
    driver.execute_async("const done = arguments[arguments.length - 1]; window.releasePracticeQuery(); requestAnimationFrame(() => requestAnimationFrame(() => done(true)));", vec![]).await?;
    ready(driver, "meaning-to-hanzi").await?;
    driver
        .execute(
            "window.practiceQueryMode = 'fail'; location.hash = '#/practice/eng-pinyin?offset=12';",
            vec![],
        )
        .await?;
    driver
        .query(By::Css(
            "[data-practice-state='failed'] [data-practice-retry]",
        ))
        .single()
        .await?
        .click()
        .await?;
    let page = ready(driver, "eng-pinyin").await?;
    assert_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("12")
    );
    assert!(
        driver
            .current_url()
            .await?
            .as_str()
            .ends_with("#/practice/eng-pinyin?offset=12")
    );
    driver.execute("location.hash = '#/practice/mcq/hanzi-to-meaning'; location.hash = '#/practice/hanzi-eng'; location.hash = '#/browse';", vec![]).await?;
    driver
        .query(By::Css(
            "[data-navigation-link][href='#/browse'][aria-current='page']",
        ))
        .single()
        .await?;
    assert!(
        driver
            .find_all(By::Css("[data-practice-state]"))
            .await?
            .is_empty()
    );
    driver.back().await?;
    ready(driver, "hanzi-eng").await?;
    assert_eq!(driver.title().await?, "Hanzi to English · Zipeek");
    assert!(
        driver
            .execute(
                "return document.activeElement.id === 'main-content';",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    Ok(())
}
