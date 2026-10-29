use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css("[data-mcq] [data-mcq-option='0']:not([disabled])"))
        .single()
        .await
}

async fn correct_index(driver: &WebDriver) -> HarnessResult<usize> {
    Ok(driver.execute("const cards = JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound); return cards.slice(1).findIndex(card => card[0] === cards[0][0]);",vec![]).await?.convert::<usize>()?)
}

async fn continue_to(driver: &WebDriver, offset: &str) -> HarnessResult<()> {
    driver
        .find(By::Css("[data-mcq-continue]"))
        .await?
        .send_keys(Key::Enter)
        .await?;
    driver
        .query(By::Css(format!(
            "[data-practice-offset='{offset}'] [data-mcq-option='0']:not([disabled])"
        )))
        .single()
        .await?;
    assert!(
        driver
            .execute(
                "return document.activeElement.hasAttribute('data-mcq-prompt');",
                vec![]
            )
            .await?
            .convert::<bool>()?
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
        ready(driver).await?;
        assert!(
            driver
                .find(By::Css("[data-mcq-continue]"))
                .await?
                .is_displayed()
                .await?
        );
        assert!(
            !driver
                .find(By::Css("[data-mcq-continue]"))
                .await?
                .is_enabled()
                .await?
        );
        assert_eq!(
            driver.find_all(By::Css("[data-mcq-option]")).await?.len(),
            3
        );
        assert!(
            driver
                .find_all(By::Css("progress, [role='progressbar']"))
                .await?
                .is_empty()
        );
        driver.execute("window.spoken = []; Object.defineProperty(speechSynthesis, 'speak', {configurable:true, value: utterance => window.spoken.push([utterance.text, utterance.lang])}); Object.defineProperty(speechSynthesis, 'cancel', {configurable:true, value: () => {}});",vec![]).await?;
        let listen = driver.find_all(By::Css("[data-listen]")).await?;
        assert_eq!(
            listen.len(),
            if variant == "hanzi-to-meaning" { 1 } else { 3 }
        );
        for button in listen {
            button.click().await?;
        }
        assert!(driver.execute("return window.spoken.length > 0 && window.spoken.every(([text,lang]) => text.length > 0 && lang === 'zh-CN' && /[\\u3400-\\u9fff]/.test(text));",vec![]).await?.convert::<bool>()?);
        driver.execute("const input = document.createElement('input'); input.id = 'shortcut-test'; document.body.append(input); input.focus();",vec![]).await?;
        driver.action_chain().send_keys("1").perform().await?;
        assert!(
            driver
                .find_all(By::Css(
                    "[data-feedback='correct'], [data-feedback='wrong']"
                ))
                .await?
                .is_empty()
        );
        driver.execute("document.getElementById('shortcut-test').remove(); document.querySelector('[data-mcq-prompt]').focus();",vec![]).await?;
        let wrong = (correct_index(driver).await? + 1) % 3;
        driver
            .action_chain()
            .send_keys((wrong + 1).to_string())
            .perform()
            .await?;
        driver
            .query(By::Css("[data-feedback='wrong']"))
            .single()
            .await?;
        assert!(driver.execute("return [...document.querySelectorAll('[data-mcq-option]')].every(button => button.disabled) && [...document.querySelectorAll('[data-feedback=none]')].every(button => Number(getComputedStyle(button).opacity) === .5);",vec![]).await?.convert::<bool>()?);
        driver
            .action_chain()
            .send_keys((correct_index(driver).await? + 1).to_string())
            .perform()
            .await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-correct]"))
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
        assert!(
            query.contains("offset=1")
                && query.contains("exclude=1%2C2")
                && !query.contains("rounds=")
                && !query.contains("seed=")
        );
        for offset in 1..=3 {
            let correct = correct_index(driver).await?;
            driver
                .find(By::Css(format!("[data-mcq-option='{correct}']")))
                .await?
                .send_keys(" ")
                .await?;
            driver
                .query(By::Css("[data-feedback='correct']"))
                .single()
                .await?;
            assert_eq!(
                driver
                    .find(By::Css("[data-mcq-correct]"))
                    .await?
                    .attr("data-mcq-correct")
                    .await?,
                Some(offset.to_string())
            );
            if offset == 3 {
                assert_eq!(driver.find_all(By::Css("[data-mcq-flame]")).await?.len(), 1);
            }
            continue_to(driver, &(offset + 1).to_string()).await?;
        }
        let wrong = (correct_index(driver).await? + 1) % 3;
        driver
            .find(By::Css(format!("[data-mcq-option='{wrong}']")))
            .await?
            .click()
            .await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-streak]"))
                .await?
                .attr("data-mcq-streak")
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
            Some("3")
        );
        assert!(
            driver
                .find_all(By::Css("[data-mcq-flame]"))
                .await?
                .is_empty()
        );
        driver.refresh().await?;
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-mcq-correct]"))
                .await?
                .attr("data-mcq-correct")
                .await?
                .as_deref(),
            Some("0")
        );
        assert!(driver.current_url().await?.as_str().contains("offset=4"));
    }
    Ok(())
}

pub async fn endless_mcq_seed_wrap_and_responsive_layout(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .goto(ctx.url("/#/practice/mcq/meaning-to-hanzi?offset=18446744073709551615"))
        .await?;
    ready(driver).await?;
    let correct = correct_index(driver).await?;
    driver
        .find(By::Css(format!("[data-mcq-option='{correct}']")))
        .await?
        .click()
        .await?;
    continue_to(driver, "0").await?;
    assert!(
        driver
            .find(By::Css("[data-mcq]"))
            .await?
            .text()
            .await?
            .contains("pool exhausted, looping")
    );
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-correct]"))
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("1")
    );
    let seed=driver.execute("return JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound)[0][0].toString();",vec![]).await?.convert::<String>()?;
    driver
        .execute(
            format!("location.hash = '#/practice/mcq/hanzi-to-meaning?seed={seed}&offset=44';"),
            vec![],
        )
        .await?;
    driver.query(By::Css("[data-practice-variant='hanzi-to-meaning'][data-practice-offset='0'] [data-mcq-option='0']:not([disabled])")).single().await?;
    let correct = correct_index(driver).await?;
    driver
        .find(By::Css(format!("[data-mcq-option='{correct}']")))
        .await?
        .click()
        .await?;
    continue_to(driver, "1").await?;
    assert!(!driver.current_url().await?.as_str().contains("seed="));
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-correct]"))
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("1")
    );
    for variant in ["hanzi-to-meaning", "meaning-to-hanzi", "meaning-to-pinyin"] {
        driver
            .execute(
                format!("location.hash = '#/practice/mcq/{variant}';"),
                vec![],
            )
            .await?;
        driver
            .query(By::Css(format!(
                "[data-practice-variant='{variant}'] [data-mcq-option='0']:not([disabled])"
            )))
            .single()
            .await?;
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
                    .set_device_metrics_override(width, 700, 1.0, width < 768)
                    .await?;
                driver
                    .find(By::Css("[data-mcq-prompt]"))
                    .await?
                    .scroll_into_view()
                    .await?;
                assert!(driver.execute("const button = document.querySelector('[data-mcq-continue]').getBoundingClientRect(); return document.documentElement.scrollWidth <= innerWidth && button.top >= 0 && button.bottom <= innerHeight;",vec![]).await?.convert::<bool>()?);
                driver
                    .screenshot(&ctx.artifacts.join(format!(
                        "{variant}-{}-{width}.png",
                        if dark { "dark" } else { "light" }
                    )))
                    .await?;
            }
        }
    }
    driver
        .execute("location.hash = '#/practice';", vec![])
        .await?;
    driver
        .query(By::Css("[data-practice-start]:not([disabled])"))
        .single()
        .await?
        .send_keys(Key::Enter)
        .await?;
    ready(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-correct]"))
            .await?
            .attr("data-mcq-correct")
            .await?
            .as_deref(),
        Some("0")
    );
    Ok(())
}
