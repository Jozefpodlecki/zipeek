use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn mounted_app_waits_for_dictionary_download(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    ctx.add_init_script(include_str!(
        "../../fixtures/browser_probes/hold_download.js"
    ))
    .await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query(By::Css("[data-database-state='loading'] [role='status']"))
        .single()
        .await?;
    assert!(driver.find_all(By::Css("[data-home]")).await?.is_empty());
    assert!(
        driver
            .find(By::Css("[data-theme-toggle]"))
            .await?
            .is_displayed()
            .await?
    );
    capture_database_boundary(&ctx, "loading").await?;
    driver
        .execute_async(
            include_str!("../../fixtures/browser_probes/release_download.js"),
            vec![],
        )
        .await?;
    driver
        .query(By::Css("[data-database-state='ready'] [data-home]"))
        .single()
        .await?;
    Ok(())
}

pub async fn bootstrap_loader_is_centered_and_landscape_survives_both_themes(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 800, 1.0, false)
        .await?;
    let blocked = driver.execute("return {urls: ['*.wasm']};", vec![]).await?;
    driver
        .cdp()
        .send_raw(
            "Network.enable",
            std::collections::BTreeMap::<String, String>::new(),
        )
        .await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", blocked.json())
        .await?;
    driver.goto(ctx.url("/")).await?;
    let artifacts = ctx.artifacts.clone();
    for (saved, name) in [("true", "dark"), ("false", "light")] {
        driver
            .execute(format!("localStorage.setItem('dark', '{saved}');"), vec![])
            .await?;
        driver.refresh().await?;
        let loader = driver
            .query(By::Css("[data-loading='bootstrap'][role='status']"))
            .single()
            .await?;
        assert_eq!(loader.text().await?, "Loading");
        assert!(driver.execute("const r = document.querySelector('.loading-panel').getBoundingClientRect(); return Math.abs(r.left + r.width / 2 - innerWidth / 2) < 1 && Math.abs(r.top + r.height / 2 - innerHeight / 2) < 1;", vec![]).await?.convert::<bool>()?);
        assert!(driver.execute("const i = document.querySelector('.app-backdrop img'); return i.complete && i.naturalWidth > 0 && !document.querySelector('#root').contains(i) && getComputedStyle(i).display !== 'none';", vec![]).await?.convert::<bool>()?);
        assert_eq!(
            driver
                .execute(
                    "return document.documentElement.classList.contains('dark');",
                    vec![]
                )
                .await?
                .convert::<bool>()?,
            saved == "true"
        );
        assert!(
            driver
                .execute(
                    "return document.documentElement.scrollHeight <= innerHeight;",
                    vec![]
                )
                .await?
                .convert::<bool>()?
        );
        driver
            .screenshot(&artifacts.join(format!("a30-bootstrap-{name}.png")))
            .await?;
    }
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(360, 800, 1.0, true)
        .await?;
    assert!(driver.execute("const r = document.querySelector('.loading-panel').getBoundingClientRect(); return r.top >= 0 && r.bottom <= innerHeight && r.left >= 0 && r.right <= innerWidth && Math.abs(r.top + r.height / 2 - innerHeight / 2) < 1;", vec![]).await?.convert::<bool>()?);
    driver
        .screenshot(&artifacts.join("a30-bootstrap-mobile.png"))
        .await?;
    let reduced = driver
        .execute(
            "return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};",
            vec![],
        )
        .await?;
    driver
        .cdp()
        .send_raw("Emulation.setEmulatedMedia", reduced.json())
        .await?;
    assert_eq!(driver.execute("return getComputedStyle(document.querySelector('.loading-spinner')).animationName;", vec![]).await?.convert::<String>()?, "none");

    let clear = driver.execute("return {urls: []};", vec![]).await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", clear.json())
        .await?;
    driver.refresh().await?;
    crate::page_objects::shell::ready(&driver).await?;
    driver
        .execute(
            "window.backdropBeforeToggle = document.querySelector('.app-backdrop');",
            vec![],
        )
        .await?;
    driver
        .find(By::Css("[data-theme-toggle]"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("html.dark")).single().await?;
    assert!(driver.execute("return window.backdropBeforeToggle === document.querySelector('.app-backdrop') && !document.querySelector('#root').contains(window.backdropBeforeToggle);", vec![]).await?.convert::<bool>()?);
    driver.goto(ctx.url("/#/practice/eng-pinyin")).await?;
    driver
        .query(By::Css("[data-practice-state='ready'] [data-matching]"))
        .single()
        .await?;
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
    Ok(())
}

pub async fn capture_database_boundary(ctx: &ScenarioContext, state: &str) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let media = driver
        .execute(
            "return {features: [{name: 'prefers-reduced-motion', value: 'reduce'}]};",
            vec![],
        )
        .await?;
    driver
        .cdp()
        .send_raw("Emulation.setEmulatedMedia", media.json())
        .await?;
    for (dark, theme) in [(true, "dark"), (false, "light")] {
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
        for width in [400, 1280] {
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(width, 800, 1.0, width < 768)
                .await?;
            driver.execute_async("const done = arguments[arguments.length - 1]; requestAnimationFrame(() => requestAnimationFrame(() => done(true)));", vec![]).await?;
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
                .screenshot(&ctx.artifacts.join(format!("{state}-{theme}-{width}.png")))
                .await?;
        }
    }
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 1000, 1.0, false)
        .await?;
    Ok(())
}
