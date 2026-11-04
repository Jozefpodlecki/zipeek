use e2e::{HarnessResult, ScenarioContext};
use std::collections::BTreeMap;
use thirtyfour::prelude::*;

async fn ready(driver: &WebDriver) -> WebDriverResult<WebElement> {
    driver
        .query(By::Css("[data-matching] [data-match-next]"))
        .single()
        .await
}
async fn card(driver: &WebDriver, side: &str, index: usize) -> WebDriverResult<WebElement> {
    driver
        .find(By::Css(format!(
            "[data-match-side='{side}'][data-match-index='{index}']"
        )))
        .await
}
async fn pairs(driver: &WebDriver) -> HarnessResult<Vec<usize>> {
    Ok(driver.execute("const c=JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound);return c.slice(0,4).map(left=>c.slice(4).findIndex(right=>right[0]===left[0]));",vec![]).await?.convert::<Vec<usize>>()?)
}
async fn match_all(driver: &WebDriver) -> HarnessResult<()> {
    ready(driver).await?;
    for (left, right) in pairs(driver).await?.into_iter().enumerate() {
        card(driver, "left", left).await?.click().await?;
        card(driver, "right", right).await?.click().await?;
    }
    driver
        .query(By::Css("[data-match-next]:not([disabled])"))
        .single()
        .await?;
    Ok(())
}
async fn wrong(driver: &WebDriver) -> HarnessResult<()> {
    let right = (pairs(driver).await?[0] + 1) % 4;
    card(driver, "left", 0).await?.click().await?;
    card(driver, "right", right).await?.click().await?;
    driver
        .query(By::Css("[data-match-feedback='wrong']"))
        .first()
        .await?;
    assert_eq!(
        driver
            .find_all(By::Css("[data-match-feedback='wrong']"))
            .await?
            .len(),
        2
    );
    assert!(driver.execute("return [...document.querySelectorAll('[data-match-side]')].every(card=>card.disabled);",vec![]).await?.convert::<bool>()?);
    driver.execute_async("const done=arguments[arguments.length-1];function wait(){if(!document.querySelector('[data-match-feedback=wrong]'))done(true);else requestAnimationFrame(wait);}wait();",vec![]).await?;
    Ok(())
}
async fn next(driver: &WebDriver, offset: &str) -> HarnessResult<()> {
    driver
        .find(By::Css("[data-match-next]"))
        .await?
        .send_keys(Key::Enter)
        .await?;
    driver
        .query(By::Css(format!(
            "[data-practice-offset='{offset}'] [data-match-count='0']"
        )))
        .single()
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

async fn presentation(ctx: &ScenarioContext, variant: &str) -> HarnessResult<()> {
    let driver = &ctx.driver;
    assert_eq!(
        driver
            .find_all(By::Css("[data-practice-pinyin]"))
            .await?
            .len(),
        if variant == "hanzi-eng" { 4 } else { 0 }
    );
    for dark in [true, false] {
        driver
            .execute(
                "document.documentElement.classList.toggle('dark',arguments[0]);",
                vec![dark.into()],
            )
            .await?;
        for width in [400, 1280, 1920] {
            if width == 1920 && (variant != "hanzi-eng" || !dark) {
                continue;
            }
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(
                    width,
                    if width >= 1280 { 720 } else { 900 },
                    1.0,
                    width < 768,
                )
                .await?;
            driver
                .find(By::Css("[data-matching] h1"))
                .await?
                .scroll_into_view()
                .await?;
            driver.execute_async("const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));",vec![]).await?;
            assert!(driver.execute("const board=document.querySelector('[data-matching]');const r=board.getBoundingClientRect();const cards=[...document.querySelectorAll('[data-match-card]')];return r.width<=897 && document.documentElement.scrollWidth<=innerWidth && !getComputedStyle(board).backgroundColor.includes('rgba') && cards.every((c,i)=>{const b=c.getBoundingClientRect(),other=cards[i^1].getBoundingClientRect(),audio=c.querySelector('[data-listen]');return Math.abs(b.height-other.height)<1 && (!audio || (audio.getBoundingClientRect().bottom<=b.bottom && audio.getBoundingClientRect().top>=b.top));});",vec![]).await?.convert::<bool>()?);
            if width >= 1280 {
                assert!(
                    driver
                        .execute(
                            "return document.documentElement.scrollHeight<=innerHeight;",
                            vec![]
                        )
                        .await?
                        .convert::<bool>()?,
                    "Desktop practice should fit without vertical scroll"
                );
            }
            if variant == "hanzi-eng" {
                assert!(driver.execute("return [...document.querySelectorAll('[data-practice-pinyin]')].every(p=>p.textContent.trim() && p.getBoundingClientRect().right<=p.closest('[data-match-card]').querySelector('[data-listen]').getBoundingClientRect().left);",vec![]).await?.convert::<bool>()?);
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
        assert_eq!(
            driver.find_all(By::Css("[data-match-side]")).await?.len(),
            8
        );
        assert!(
            !driver
                .find(By::Css("[data-match-next]"))
                .await?
                .is_enabled()
                .await?
        );
        assert!(
            driver
                .find_all(By::Css("[data-match-progress]"))
                .await?
                .is_empty()
        );
        driver.execute("window.matchNodes=[...document.querySelectorAll('[data-match-side]')];window.spoken=[];Object.defineProperty(speechSynthesis,'speak',{configurable:true,value:u=>window.spoken.push([u.text,u.lang])});Object.defineProperty(speechSynthesis,'cancel',{configurable:true,value:()=>{}});",vec![]).await?;
        presentation(&ctx, variant).await?;
        let listens = driver.find_all(By::Css("[data-listen]")).await?;
        assert_eq!(listens.len(), if variant == "pinyin-hanzi" { 8 } else { 4 });
        listens[0].click().await?;
        assert!(driver.execute("return window.spoken.length===1 && window.spoken[0][1]==='zh-CN' && /[\\u3400-\\u9fff]/.test(window.spoken[0][0]) && !document.querySelector('[data-match-selected=true]');",vec![]).await?.convert::<bool>()?);
        card(driver, "left", 0).await?.send_keys(" ").await?;
        card(driver, "left", 1).await?.click().await?;
        assert_eq!(
            card(driver, "left", 0)
                .await?
                .attr("data-match-selected")
                .await?
                .as_deref(),
            Some("true")
        );
        card(driver, "left", 0).await?.click().await?;
        assert!(
            driver
                .find_all(By::Css("[data-match-selected='true']"))
                .await?
                .is_empty()
        );
        card(driver, "right", 0).await?.click().await?;
        driver
            .action_chain()
            .send_keys(Key::Escape)
            .perform()
            .await?;
        assert!(
            driver
                .find_all(By::Css("[data-match-selected='true']"))
                .await?
                .is_empty()
        );
        wrong(driver).await?;
        for (left, right) in pairs(driver).await?.into_iter().enumerate() {
            card(driver, "right", right).await?.click().await?;
            card(driver, "left", left)
                .await?
                .send_keys(Key::Enter)
                .await?;
            assert_eq!(
                driver
                    .find(By::Css("[data-match-count]"))
                    .await?
                    .attr("data-match-count")
                    .await?,
                Some((left + 1).to_string())
            );
            assert_eq!(
                driver
                    .find(By::Css("[data-match-next]"))
                    .await?
                    .is_enabled()
                    .await?,
                left == 3
            );
        }
        assert_eq!(
            driver
                .find(By::Css("[data-match-correct]"))
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("4")
        );
        assert_eq!(
            driver.find_all(By::Css("[data-match-flame]")).await?.len(),
            1
        );
        assert!(driver.execute("return [...document.querySelectorAll('[data-match-side]')].every((node,index)=>node===window.matchNodes[index]);",vec![]).await?.convert::<bool>()?);
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
                    .set_device_metrics_override(
                        width,
                        if width >= 1280 { 720 } else { 900 },
                        1.0,
                        width < 768,
                    )
                    .await?;
                driver
                    .find(By::Css("[data-matching] h1"))
                    .await?
                    .scroll_into_view()
                    .await?;
                assert!(driver.execute("const next=document.querySelector('[data-match-next]').getBoundingClientRect();return document.documentElement.scrollWidth<=innerWidth && next.top>=0 && next.bottom<=innerHeight;",vec![]).await?.convert::<bool>()?);
                driver
                    .screenshot(&ctx.artifacts.join(format!(
                        "matching-{variant}-{}-{width}.png",
                        if dark { "dark" } else { "light" }
                    )))
                    .await?;
            }
        }
        next(driver, "4").await?;
        assert_eq!(
            driver
                .find(By::Css("[data-match-correct]"))
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("4")
        );
        assert!(
            driver
                .current_url()
                .await?
                .as_str()
                .contains("exclude=1%2C2")
        );
        wrong(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-match-streak]"))
                .await?
                .attr("data-match-streak")
                .await?
                .as_deref(),
            Some("0")
        );
        driver.refresh().await?;
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-match-correct]"))
                .await?
                .attr("data-match-correct")
                .await?
                .as_deref(),
            Some("0")
        );
    }
    driver
        .goto(ctx.url("/#/practice/mcq/hanzi-to-meaning?offset=0"))
        .await?;
    let pinyin = driver
        .query(By::Css("[data-mcq] [data-practice-pinyin]"))
        .single()
        .await?;
    assert!(!pinyin.text().await?.is_empty());
    assert!(driver.execute("const p=document.querySelector('[data-practice-pinyin]'),a=document.querySelector('[data-listen]');return p.getBoundingClientRect().bottom<=a.getBoundingClientRect().top;",vec![]).await?.convert::<bool>()?);
    for (width, dark) in [(400, false), (1280, true)] {
        driver
            .execute(
                "document.documentElement.classList.toggle('dark',arguments[0]);",
                vec![dark.into()],
            )
            .await?;
        driver
            .cdp()
            .emulation()
            .set_device_metrics_override(
                width,
                if width >= 1280 { 720 } else { 900 },
                1.0,
                width < 768,
            )
            .await?;
        driver
            .find(By::Css("[data-mcq] h1"))
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
                "window.matchClock=100000;Date.now=()=>window.matchClock;",
                vec![],
            )
            .await?;
        driver
            .goto(ctx.url(&format!(
                "/#/practice/{variant}?mode=fixed&rounds={total}&offset=18446744073709551615"
            )))
            .await?;
        for round in 0..total {
            ready(driver).await?;
            assert_eq!(
                driver
                    .find(By::Css("[data-match-progress]"))
                    .await?
                    .attr("data-match-progress")
                    .await?,
                Some(round.to_string())
            );
            if round == 0 {
                wrong(driver).await?;
            }
            match_all(driver).await?;
            assert!(
                driver
                    .find_all(By::Css("[data-practice-summary]"))
                    .await?
                    .is_empty()
            );
            if round == total - 1 {
                driver.execute("window.matchClock=200000;", vec![]).await?;
            }
            driver
                .find(By::Css("[data-match-next]"))
                .await?
                .click()
                .await?;
        }
        summary(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-summary-first-attempt]"))
                .await?
                .text()
                .await?,
            format!("{} / {}", total * 4 - 2, total * 4)
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-mistakes]"))
                .await?
                .text()
                .await?,
            "1"
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-highest-streak]"))
                .await?
                .text()
                .await?,
            (total * 4).to_string()
        );
        assert_eq!(
            driver
                .find(By::Css("[data-summary-time]"))
                .await?
                .text()
                .await?,
            "1m 40s"
        );
        driver.execute("window.oldMatchChart=document.querySelector('[data-summary-ring]');window.matchClock=900000;",vec![]).await?;
        let old = driver.current_url().await?.to_string();
        driver
            .find(By::Css("[data-summary-restart]"))
            .await?
            .click()
            .await?;
        ready(driver).await?;
        assert_ne!(old, driver.current_url().await?.to_string());
        assert_eq!(
            driver
                .find(By::Css("[data-match-progress]"))
                .await?
                .attr("data-match-progress")
                .await?
                .as_deref(),
            Some("0")
        );
        assert!(
            driver
                .execute(
                    "return window.oldMatchChart.dataset.summaryChart==='destroyed';",
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        driver
            .execute("location.hash='#/practice';", vec![])
            .await?;
        driver
            .query(By::Css("[data-practice-start]:not([disabled])"))
            .single()
            .await?;
    }
    Ok(())
}

pub async fn matching_recovery_seed_and_same_url(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver
        .cdp()
        .send_raw(
            "Emulation.setEmulatedMedia",
            BTreeMap::from([(
                "features",
                vec![BTreeMap::from([
                    ("name", "prefers-reduced-motion"),
                    ("value", "reduce"),
                ])],
            )]),
        )
        .await?;
    driver
        .goto(ctx.url("/#/practice/pinyin-hanzi?offset=18446744073709551615"))
        .await?;
    ready(driver).await?;
    match_all(driver).await?;
    assert!(driver.execute("return [...document.querySelectorAll('[data-match-feedback=correct]')].every(card=>getComputedStyle(card).animationName==='none' && Number(getComputedStyle(card).opacity)===1 && card.querySelector('[aria-label=Correct]')!==null);",vec![]).await?.convert::<bool>()?);
    let seed=driver.execute("return JSON.parse(document.querySelector('[data-practice-round]').dataset.practiceRound)[0][0].toString();",vec![]).await?.convert::<String>()?;
    next(driver, "0").await?;
    assert_eq!(
        driver
            .find(By::Css("[data-match-correct]"))
            .await?
            .attr("data-match-correct")
            .await?
            .as_deref(),
        Some("4")
    );
    driver
        .execute(
            format!("location.hash='#/practice/pinyin-hanzi?seed={seed}&offset=40';"),
            vec![],
        )
        .await?;
    driver
        .query(By::Css("[data-practice-seed] [data-match-next]"))
        .single()
        .await?;
    match_all(driver).await?;
    next(driver, "4").await?;
    assert!(!driver.current_url().await?.as_str().contains("seed="));
    driver
        .execute(
            include_str!("../../fixtures/browser_probes/empty_strict_pool.js"),
            vec![],
        )
        .await?;
    driver
        .execute(
            "location.hash='#/practice/hanzi-eng?offset=0&mode=fixed&rounds=5';",
            vec![],
        )
        .await?;
    for round in 0..5 {
        ready(driver).await?;
        assert_eq!(
            driver
                .find(By::Css("[data-match-progress]"))
                .await?
                .attr("data-match-progress")
                .await?,
            Some(round.to_string())
        );
        match_all(driver).await?;
        driver
            .find(By::Css("[data-match-next]"))
            .await?
            .click()
            .await?;
    }
    summary(driver).await?;
    assert_eq!(
        driver
            .find(By::Css("[data-summary-first-attempt]"))
            .await?
            .text()
            .await?,
        "20 / 20"
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
    driver.execute("const post=Worker.prototype.postMessage;Worker.prototype.postMessage=function(message,...rest){if(message.type==='CREATE_PREPARED' && message.data[1].startsWith('SELECT * FROM words WHERE hsk3_level BETWEEN'))message={...message,data:[message.data[0],message.data[1].replace(' ORDER BY id ASC',' AND FALSE ORDER BY id ASC')]};return post.call(this,message,...rest)};location.hash='#/practice/eng-pinyin?offset=8';",vec![]).await?;
    driver
        .query(By::Css("[data-practice-state='failed'] [data-match-setup]"))
        .single()
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-practice-start]:not([disabled])"))
        .single()
        .await?;
    Ok(())
}
