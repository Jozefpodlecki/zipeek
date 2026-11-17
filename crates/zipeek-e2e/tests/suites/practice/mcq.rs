use thirtyfour::prelude::*;
use zipeek_e2e::harness::browser_scripts;
use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    zipeek_e2e::page_objects::mcq::ready(driver).await
}

async fn correct_index(driver: &WebDriver) -> HarnessResult<usize> {
    Ok(zipeek_e2e::page_objects::round::mcq_correct_index(driver).await?)
}

async fn continue_to(driver: &WebDriver, offset: &str) -> HarnessResult<()> {
    driver
        .find_css("[data-mcq-continue]")
        .await?
        .send_keys(Key::Enter)
        .await?;
    driver
        .query_single(format!(
            "[data-practice-offset='{offset}'] [data-mcq-option='0']:not([disabled])"
        ))
        .await?;
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::focused_element_has_attribute(driver, "data-mcq-prompt",)
            .await?
    );
    Ok(())
}

pub async fn endless_mcq_answers_lock_and_continue(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for variant in ["hanzi-to-meaning", "meaning-to-hanzi", "meaning-to-pinyin"] {
        driver
            .goto(ctx.url(&format!(
                "/#/practice/mcq/{variant}?hsk=3.0&level=1&offset=0&exclude=1,2"
            )))
            .await?;
        ctx.checkpoint("mcq-initial-state").await?;
        ready(driver).await?;
        ctx.checkpoint_visible("mcq-options-ready", "[data-mcq-option]")
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_css("[data-mcq-continue]")
                .await?
                .is_displayed()
                .await?
        );
        zipeek_e2e::e2e_ensure!(
            !driver
                .find_css("[data-mcq-continue]")
                .await?
                .is_enabled()
                .await?
        );
        zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-mcq-option]").await?.len(), 3);
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_all_css("[data-mcq-option][aria-pressed='false']:not([disabled])")
                .await?
                .len(),
            3,
            "All unsubmitted MCQ options must be keyboard-accessible and unselected"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("progress, [role='progressbar']")
                .await?
                .is_empty()
        );
        driver
            .execute(browser_scripts::RESET_SPEECH_EVENTS, vec![])
            .await?;
        let listen = driver.find_all_css("[data-listen]").await?;
        zipeek_e2e::e2e_ensure_eq!(
            listen.len(),
            if variant == "hanzi-to-meaning" { 1 } else { 3 }
        );
        for button in listen {
            button.click().await?;
        }
        zipeek_e2e::e2e_ensure!(
            driver
                .execute(browser_scripts::ASSERT_CHINESE_SPEECH, vec![])
                .await?
                .convert::<bool>()?
        );
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MCQ_ENDLESS_MCQ_ANSWERS_LOCK_AND_CONTINUE_01,
                vec![],
            )
            .await?;
        driver.action_chain().send_keys("1").perform().await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-feedback='correct'], [data-feedback='wrong']")
                .await?
                .is_empty()
        );
        driver
            .execute(
                zipeek_e2e::js_scripts::practice::MCQ_ENDLESS_MCQ_ANSWERS_LOCK_AND_CONTINUE_02,
                vec![],
            )
            .await?;
        let wrong = (correct_index(driver).await? + 1) % 3;
        driver
            .action_chain()
            .send_keys((wrong + 1).to_string())
            .perform()
            .await?;
        driver.query_single("[data-feedback='wrong']").await?;
        zipeek_e2e::page_objects::mcq::assert_answer_locked(driver).await?;
        driver
            .action_chain()
            .send_keys((correct_index(driver).await? + 1).to_string())
            .perform()
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-correct]")
                .await?
                .attr("data-mcq-correct")
                .await?
                .as_deref(),
            Some("0")
        );
        continue_to(driver, "1").await?;
        let query = driver
            .current_url()
            .await?
            .fragment()
            .ok_or("Missing MCQ fragment")?
            .to_owned();
        zipeek_e2e::e2e_ensure!(
            query.contains("offset=1")
                && query.contains("exclude=1%2C2")
                && !query.contains("rounds=")
                && !query.contains("seed=")
        );
        for offset in 1..=3 {
            let correct = correct_index(driver).await?;
            driver
                .find_css(format!("[data-mcq-option='{correct}']"))
                .await?
                .send_keys(" ")
                .await?;
            driver.query_single("[data-feedback='correct']").await?;
            zipeek_e2e::e2e_ensure_eq!(
                driver
                    .find_css("[data-mcq-correct]")
                    .await?
                    .attr("data-mcq-correct")
                    .await?,
                Some(offset.to_string())
            );
            if offset == 3 {
                zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-mcq-flame]").await?.len(), 1);
            }
            continue_to(driver, &(offset + 1).to_string()).await?;
        }
        let wrong = (correct_index(driver).await? + 1) % 3;
        driver
            .find_css(format!("[data-mcq-option='{wrong}']"))
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-streak]")
                .await?
                .attr("data-mcq-streak")
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
            Some("3")
        );
        zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-mcq-flame]").await?.is_empty());
        driver.refresh().await?;
        ready(driver).await?;
        zipeek_e2e::e2e_ensure_eq!(
            driver
                .find_css("[data-mcq-correct]")
                .await?
                .attr("data-mcq-correct")
                .await?
                .as_deref(),
            Some("0")
        );
        zipeek_e2e::e2e_ensure!(driver.current_url().await?.as_str().contains("offset=4"));
    }
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-mcq-option]").await?.len() >= 2,
        "Replayed MCQ must retain at least two answer options"
    );
    ctx.checkpoint_visible("mcq-round-preserved", "[data-mcq]")
        .await?;
    ctx.checkpoint("mcq-verified-result").await?;
    Ok(())
}

pub async fn endless_mcq_seed_wrap_and_responsive_layout(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/mcq/meaning-to-hanzi?offset=18446744073709551615")
        .await?;
    ready(driver).await?;
    let correct = correct_index(driver).await?;
    driver
        .find_css(format!("[data-mcq-option='{correct}']"))
        .await?
        .click()
        .await?;
    continue_to(driver, "0").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-mcq]")
            .await?
            .text()
            .await?
            .contains("pool exhausted, looping")
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-correct]")
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("1")
    );
    let seed = zipeek_e2e::page_objects::round::first_word_id(driver).await?;
    zipeek_e2e::page_objects::browser::navigate_hash(
        driver,
        &format!("/practice/mcq/hanzi-to-meaning?seed={seed}&offset=44"),
    )
    .await?;
    driver.query_single("[data-practice-variant='hanzi-to-meaning'][data-practice-offset='0'] [data-mcq-option='0']:not([disabled])").await?;
    let correct = correct_index(driver).await?;
    driver
        .find_css(format!("[data-mcq-option='{correct}']"))
        .await?
        .click()
        .await?;
    continue_to(driver, "1").await?;
    zipeek_e2e::e2e_ensure!(!driver.current_url().await?.as_str().contains("seed="));
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-correct]")
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("1")
    );
    for variant in ["hanzi-to-meaning", "meaning-to-hanzi", "meaning-to-pinyin"] {
        zipeek_e2e::page_objects::browser::navigate_hash(
            driver,
            &format!("/practice/mcq/{variant}"),
        )
        .await?;
        driver
            .query_single(format!(
                "[data-practice-variant='{variant}'] [data-mcq-option='0']:not([disabled])"
            ))
            .await?;
        for dark in [true, false] {
            let current = driver
                .execute(
                    zipeek_e2e::js_scripts::practice::MCQ_ENDLESS_MCQ_SEED_WRAP_AND_RESPONSIVE_LAYOUT_01,
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
                ctx.viewport().responsive(width, 700, width < 768).await?;
                driver
                    .find_css("[data-mcq-prompt]")
                    .await?
                    .scroll_into_view()
                    .await?;
                zipeek_e2e::e2e_ensure!(
                    zipeek_e2e::page_objects::browser::no_horizontal_overflow(driver).await?
                );
                zipeek_e2e::e2e_ensure!(
                    zipeek_e2e::page_objects::browser::element_inside_viewport(
                        driver,
                        "[data-mcq-continue]",
                    )
                    .await?
                );
                driver
                    .screenshot(&ctx.artifacts.join(format!(
                        "{variant}-{}-{width}.png",
                        if dark { "dark" } else { "light" }
                    )))
                    .await?;
            }
        }
    }
    zipeek_e2e::page_objects::browser::navigate_hash(driver, "/practice").await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?
        .send_keys(Key::Enter)
        .await?;
    ready(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_css("[data-mcq-correct]")
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("0")
    );
    ctx.checkpoint_visible("endless-mcq-round-replayed", "[data-mcq-correct]")
        .await?;
    Ok(())
}
