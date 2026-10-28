use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn navigation_and_rapid_theme_commands_preserve_dictionary(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.add_init_script(include_str!(
        "../../fixtures/browser_probes/provider_lifetime.js"
    ))
    .await?;
    driver.goto(ctx.url("/")).await?;
    driver
        .query(By::Css("[data-database-state='loading']"))
        .single()
        .await?;
    driver.execute_async("const done = arguments[arguments.length - 1]; function wait() { if (window.releaseDictionary) done(true); else requestAnimationFrame(wait); } wait();", vec![]).await?;
    let workers = driver
        .execute("return window.dictionaryWorkers;", vec![])
        .await?
        .convert::<u64>()?;
    assert!(workers > 0);
    for route in ["/practice", "/browse", "/"] {
        driver
            .find(By::Css(format!("[data-navigation-link][href='#{route}']")))
            .await?
            .click()
            .await?;
        driver
            .query(By::Css(format!(
                "[data-navigation-link][href='#{route}'][aria-current='page']"
            )))
            .single()
            .await?;
        driver
            .query(By::Css("[data-database-state='loading']"))
            .single()
            .await?;
    }
    assert!(
        driver
            .execute("return localStorage.getItem('dark') === null;", vec![])
            .await?
            .convert::<bool>()?
    );
    driver.execute("const toggle = document.querySelector('[data-theme-toggle]'); toggle.click(); toggle.click();", vec![]).await?;
    driver
        .query(By::Css(
            "html.dark body.dark [data-theme-toggle][aria-checked='true']",
        ))
        .single()
        .await?;
    assert!(
        driver
            .execute("return localStorage.getItem('dark') === 'true';", vec![])
            .await?
            .convert::<bool>()?
    );
    driver
        .execute_async(
            include_str!("../../fixtures/browser_probes/release_download.js"),
            vec![],
        )
        .await?;
    crate::page_objects::shell::ready(driver).await?;
    driver
        .find(By::Css("[data-navigation-link][href='#/browse']"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-route-placeholder] h1"))
        .single()
        .await?;
    driver.back().await?;
    driver.query(By::Css("[data-home]")).single().await?;
    driver.forward().await?;
    driver
        .query(By::Css(
            "[data-navigation-link][href='#/browse'][aria-current='page']",
        ))
        .single()
        .await?;
    driver
        .execute(
            "location.hash = '#/practice'; location.hash = '#/browse'; location.hash = '#/';",
            vec![],
        )
        .await?;
    driver.query(By::Css("[data-home]")).single().await?;
    crate::page_objects::shell::ready(driver).await?;
    assert_eq!(
        driver
            .execute("return window.dictionaryDownloads;", vec![])
            .await?
            .convert::<u64>()?,
        1
    );
    assert_eq!(
        driver
            .execute("return window.dictionaryWorkers;", vec![])
            .await?
            .convert::<u64>()?,
        workers
    );
    Ok(())
}
