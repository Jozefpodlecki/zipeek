use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css(
            "[data-practice-state='ready'] :is([data-match-next],[data-mcq-continue]):disabled",
        ))
        .single()
        .await
}
async fn variant(driver: &WebDriver) -> HarnessResult<String> {
    ready(driver).await?;
    Ok(driver
        .find(By::Css("[data-practice-variant]"))
        .await?
        .attr("data-practice-variant")
        .await?
        .ok_or("No current type")?)
}
async fn round(driver: &WebDriver, make_wrong: bool) -> HarnessResult<()> {
    ready(driver).await?;
    let mcq = !driver.find_all(By::Css("[data-mcq]")).await?.is_empty();
    if mcq {
        let correct=driver.execute("const c=JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound);return c.slice(1).findIndex(x=>x[0]===c[0][0]);",vec![]).await?.convert::<usize>()?;
        driver
            .find(By::Css(format!(
                "[data-mcq-option='{}']",
                if make_wrong {
                    (correct + 1) % 3
                } else {
                    correct
                }
            )))
            .await?
            .click()
            .await?;
    } else {
        let pairs=driver.execute("const c=JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound);return c.slice(0,4).map(l=>c.slice(4).findIndex(r=>r[0]===l[0]));",vec![]).await?.convert::<Vec<usize>>()?;
        if make_wrong {
            driver
                .find(By::Css("[data-match-side='left'][data-match-index='0']"))
                .await?
                .click()
                .await?;
            driver
                .find(By::Css(format!(
                    "[data-match-side='right'][data-match-index='{}']",
                    (pairs[0] + 1) % 4
                )))
                .await?
                .click()
                .await?;
            driver.execute_async("const done=arguments[arguments.length-1];function wait(){if(!document.querySelector('[data-match-feedback=wrong]'))done(true);else requestAnimationFrame(wait);}wait();",vec![]).await?;
        }
        for (left, right) in pairs.into_iter().enumerate() {
            driver
                .find(By::Css(format!(
                    "[data-match-side='left'][data-match-index='{left}']"
                )))
                .await?
                .click()
                .await?;
            driver
                .find(By::Css(format!(
                    "[data-match-side='right'][data-match-index='{right}']"
                )))
                .await?
                .click()
                .await?;
        }
    }
    driver
        .query(By::Css(
            "[data-match-next]:not([disabled]),[data-mcq-continue]:not([disabled])",
        ))
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}
async fn summary(driver: &WebDriver) -> HarnessResult<()> {
    driver
        .query(By::Css(
            "[data-practice-summary] [data-summary-chart='ready']",
        ))
        .single()
        .await?;
    driver.execute_async("const done=arguments[arguments.length-1];function wait(){const s=document.querySelector('[data-summary-first-attempt]');if(s&&s.textContent===s.dataset.summaryFirstAttempt)done(true);else requestAnimationFrame(wait);}wait();",vec![]).await?;
    Ok(())
}

pub async fn mixed_setup_progress_summary(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/#/practice")).await?;
    driver
        .query(By::Css("[data-practice-start]:not([disabled])"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-setup-choice='hanzi-to-meaning']"))
        .await?
        .click()
        .await?;
    assert_eq!(
        driver
            .find_all(By::Css("[data-activity-picker] [aria-pressed='true']"))
            .await?
            .len(),
        1
    );
    driver
        .find(By::Css("[data-setup-choice='hanzi-eng']"))
        .await?
        .click()
        .await?;
    assert_eq!(
        driver
            .find_all(By::Css("[data-activity-picker] [aria-pressed='true']"))
            .await?
            .len(),
        2
    );
    driver
        .screenshot(&ctx.artifacts.join("selected-types.png"))
        .await?;
    driver
        .find(By::Css("[data-setup-choice='Fixed']"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-setup-choice='rounds-5']"))
        .await?
        .click()
        .await?;
    driver.execute("window.mixedClock=100000;Date.now=()=>window.mixedClock;Object.defineProperty(crypto,'getRandomValues',{configurable:true,value:bytes=>{bytes.fill(0);bytes[0]=1;return bytes;}});",vec![]).await?;
    driver
        .find(By::Css("[data-practice-start]"))
        .await?
        .send_keys(Key::Enter)
        .await?;
    let mut previous = String::new();
    let mut offset = 0;
    for index in 0..5 {
        let current = variant(driver).await?;
        assert_ne!(current, previous);
        assert_eq!(
            current,
            if index % 2 == 0 {
                "hanzi-eng"
            } else {
                "hanzi-to-meaning"
            }
        );
        assert_eq!(
            driver
                .find(By::Css("[data-practice-offset]"))
                .await?
                .attr("data-practice-offset")
                .await?,
            Some(offset.to_string())
        );
        let progress = driver
            .find(By::Css("[data-match-progress], [data-mcq-progress]"))
            .await?;
        let value = if current == "hanzi-eng" {
            progress.attr("data-match-progress").await?
        } else {
            progress.attr("value").await?
        };
        assert_eq!(value, Some(index.to_string()));
        assert!(
            driver
                .current_url()
                .await?
                .as_str()
                .contains("types=hanzi-to-meaning%2Chanzi-eng")
        );
        if index == 4 {
            driver.execute("window.mixedClock=200000;", vec![]).await?;
        }
        offset += if current == "hanzi-eng" { 4 } else { 1 };
        previous = current;
        round(driver, index < 2).await?;
    }
    summary(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "11 / 14"
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
        "9"
    );
    assert_eq!(
        driver
            .find(By::Css("[data-summary-time]"))
            .await?
            .text()
            .await?,
        "1m 40s"
    );
    driver
        .screenshot(&ctx.artifacts.join("mixed-summary.png"))
        .await?;
    driver
        .find(By::Css("[data-summary-restart]"))
        .await?
        .click()
        .await?;
    assert_eq!(variant(driver).await?, "hanzi-to-meaning");
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-correct]"))
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
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "11 / 11"
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
    assert_eq!(
        driver
            .find_all(By::Css("[data-activity-picker] [aria-pressed='true']"))
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
    assert_eq!(driver.current_url().await?.as_str(), url);
    let before = driver
        .find(By::Css("[data-practice-round]"))
        .await?
        .attr("data-practice-round")
        .await?;
    let seed=driver.execute("return JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound)[0][0].toString();",vec![]).await?.convert::<String>()?;
    driver.refresh().await?;
    ready(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-practice-round]"))
            .await?
            .attr("data-practice-round")
            .await?,
        before
    );
    driver.execute(format!("location.hash='#/practice/eng-pinyin?types=eng-pinyin,hanzi-to-meaning&seed={seed}&offset=40';"),vec![]).await?;
    driver
        .query(By::Css("[data-practice-seed] [data-match-next]"))
        .single()
        .await?;
    round(driver, false).await?;
    assert_eq!(variant(driver).await?, "hanzi-to-meaning");
    assert!(!driver.current_url().await?.as_str().contains("seed="));
    assert_eq!(
        driver
            .find(By::Css("[data-practice-offset]"))
            .await?
            .attr("data-practice-offset")
            .await?
            .as_deref(),
        Some("4")
    );
    assert_eq!(
        driver
            .find(By::Css("[data-mcq-correct]"))
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
        assert_ne!(current, previous);
        previous = current;
    }
    driver.refresh().await?;
    ready(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-match-correct], [data-mcq-correct]"))
            .await?
            .text()
            .await?,
        "Correct: 0"
    );
    let invalid = ctx.url("/#/practice/mcq/meaning-to-pinyin?types=eng-pinyin,unknown&offset=8");
    driver.goto(&invalid).await?;
    ready(driver).await?;
    assert_eq!(driver.current_url().await?.as_str(), invalid);
    assert_eq!(
        driver
            .find(By::Css("[data-practice-types]"))
            .await?
            .attr("data-practice-types")
            .await?
            .as_deref(),
        Some("meaning-to-pinyin")
    );
    round(driver, false).await?;
    assert_eq!(variant(driver).await?, "meaning-to-pinyin");
    assert!(!driver.current_url().await?.as_str().contains("types="));
    Ok(())
}
