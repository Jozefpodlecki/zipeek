use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn ready(driver: &WebDriver, variant: &str) -> WebDriverResult<WebElement> {
    driver
        .query_single(format!(
            "[data-practice-state='ready'][data-practice-variant='{variant}']"
        ))
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
        ctx.checkpoint("practice-initial-state").await?;
        let page = ready(driver, variant).await?;
        zipeek_e2e::e2e_ensure!(
            page.is_displayed().await?,
            "The {variant} route must render a visible practice session"
        );
        zipeek_e2e::e2e_ensure_eq!(
            page.attr("data-practice-offset").await?.as_deref(),
            Some("8")
        );
        zipeek_e2e::e2e_ensure!(page.attr("data-practice-seed").await?.is_none());
        zipeek_e2e::e2e_ensure_eq!(driver.title().await?, format!("{title} · Zipeek"));
        zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), url);
        let round = page
            .attr("data-practice-round")
            .await?
            .ok_or("Missing debug round")?;
        driver.refresh().await?;
        zipeek_e2e::e2e_ensure_eq!(
            ready(driver, variant)
                .await?
                .attr("data-practice-round")
                .await?
                .as_deref(),
            Some(round.as_str())
        );
    }
    let seed = zipeek_e2e::page_objects::round::first_word_id(driver).await?;
    zipeek_e2e::page_objects::browser::set_local_storage(
        driver,
        "zipeek.excluded_words",
        &format!("[{seed}]"),
    )
    .await?;
    let shared = ctx.url(&format!("/#/practice/eng-pinyin?seed={seed}&offset=40"));
    driver.goto(&shared).await?;
    let page = ready(driver, "eng-pinyin").await?;
    zipeek_e2e::e2e_ensure_eq!(
        page.attr("data-practice-seed").await?.as_deref(),
        Some(seed.as_str())
    );
    zipeek_e2e::e2e_ensure_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("0")
    );
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), shared);
    let exclusions = ctx.url(&format!(
        "/#/practice/eng-pinyin?seed={seed}&exclude={seed},2147483647&offset=8"
    ));
    driver.goto(&exclusions).await?;
    let page = ready(driver, "eng-pinyin").await?;
    zipeek_e2e::e2e_ensure!(page.attr("data-practice-seed").await?.is_none());
    zipeek_e2e::e2e_ensure_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("8")
    );
    zipeek_e2e::e2e_ensure_eq!(
        page.attr("data-practice-exclusions").await?.as_deref(),
        Some(seed.as_str())
    );
    zipeek_e2e::e2e_ensure!(
        !zipeek_e2e::page_objects::round::round_contains_word(driver, &seed).await?
    );
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::local_storage(driver, "zipeek.excluded_words").await?,
        Some(format!("[{seed}]")),
    );
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.as_str(), exclusions);
    ctx.open_route("/practice/eng-pinyin?seed=2147483647&offset=12")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        ready(driver, "eng-pinyin")
            .await?
            .attr("data-practice-offset")
            .await?
            .as_deref(),
        Some("12")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-variant]").await?.len(),
        1,
        "Practice replay must not mount competing round hosts"
    );
    ctx.checkpoint_visible("practice-ready-after-replay", "[data-practice-variant]")
        .await?;
    ctx.checkpoint("practice-verified-result").await?;
    Ok(())
}

pub async fn late_queries_and_retry_keep_current_route(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .execute(
            include_str!("../../../fixtures/browser_probes/practice_queries.js"),
            vec![],
        )
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_01,
            vec![],
        )
        .await?;
    driver
        .execute_async(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_02,
            vec![],
        )
        .await?;
    driver
        .query_single("[data-practice-state='loading'] [role='status']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::focused_element_has_id(driver, "main-content").await?
    );
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_03,
            vec![],
        )
        .await?;
    ready(driver, "meaning-to-hanzi").await?;
    driver
        .execute_async(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_04,
            vec![],
        )
        .await?;
    ready(driver, "meaning-to-hanzi").await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_05,
            vec![],
        )
        .await?;
    driver
        .query_single("[data-practice-state='failed'] [data-practice-retry]")
        .await?
        .click()
        .await?;
    let page = ready(driver, "eng-pinyin").await?;
    zipeek_e2e::e2e_ensure_eq!(
        page.attr("data-practice-offset").await?.as_deref(),
        Some("12")
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .as_str()
            .ends_with("#/practice/eng-pinyin?offset=12")
    );
    driver
        .execute(
            zipeek_e2e::js_scripts::practice::PRACTICE_LATE_QUERIES_AND_RETRY_KEEP_CURRENT_ROUTE_06,
            vec![],
        )
        .await?;
    driver
        .query_single("[data-navigation-link][href='#/browse'][aria-current='page']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-state]")
            .await?
            .is_empty()
    );
    driver.back().await?;
    let restored = ready(driver, "hanzi-eng").await?;
    zipeek_e2e::e2e_ensure!(restored.is_displayed().await?);
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-variant]").await?.len(),
        1,
        "Back navigation must restore exactly one active practice host"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-state='failed']")
            .await?
            .is_empty(),
        "An earlier failed request must not replace the restored route"
    );
    zipeek_e2e::e2e_ensure_eq!(driver.title().await?, "Hanzi to English · Zipeek");
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::focused_element_has_id(driver, "main-content").await?
    );
    Ok(())
}

pub async fn planned_start_cancel_and_restart_preserves_finite_session(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .find_css("[data-practice-step='summary']")
        .await?
        .click()
        .await?;
    let fixed = driver.find_css("[data-setup-choice='Fixed 5']").await?;
    if fixed.attr("aria-pressed").await?.as_deref() != Some("true") {
        fixed.click().await?;
    }
    driver
        .find_css("[data-practice-step='summary']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?
        .click()
        .await?;
    driver.query_single("[data-practice-local-session]").await?;
    let session_url = driver.current_url().await?.to_string();
    zipeek_e2e::e2e_ensure!(session_url.contains("/practice/session/"));
    let before = driver
        .find_css("[data-practice-local-session] h1")
        .await?
        .text()
        .await?;
    driver
        .find_css("[data-practice-skip]")
        .await?
        .click()
        .await?;
    let after = driver
        .find_css("[data-practice-local-session] h1")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure_ne!(before, after);
    driver.refresh().await?;
    driver.query_single("[data-practice-local-session]").await?;
    zipeek_e2e::e2e_ensure_eq!(driver.current_url().await?.to_string(), session_url);
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-practice-local-session] h1")
            .await?
            .text()
            .await?,
        after
    );
    ctx.checkpoint_visible(
        "planned-session-route-restored",
        "[data-practice-local-session] h1",
    )
    .await?;
    Ok(())
}
