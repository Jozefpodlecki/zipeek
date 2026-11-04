use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn shell_and_card_styling(ctx: ScenarioContext) -> HarnessResult<()> {
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
    for (saved, theme) in [("true", "dark"), ("false", "light")] {
        driver
            .execute(format!("localStorage.setItem('dark', '{saved}');"), vec![])
            .await?;
        driver.refresh().await?;
        driver
            .query(By::Css("[data-database-state='ready']"))
            .single()
            .await?;
        for width in [400, 640, 768, 1024, 1280, 1920] {
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(width, 1000, 1.0, width < 768)
                .await?;
            driver.action_chain().move_to(0, 0).perform().await?;
            driver.execute_async("const done = arguments[arguments.length - 1]; document.fonts.ready.then(() => requestAnimationFrame(() => requestAnimationFrame(() => done(true))));", vec![]).await?;
            let failures = driver
                .execute(
                    include_str!("../../fixtures/browser_probes/styling_acceptance.js"),
                    vec![],
                )
                .await?
                .convert::<Vec<String>>()?;
            assert!(failures.is_empty(), "{theme} at {width}px: {failures:?}");
            let metrics = driver
                .execute(
                    include_str!("../../fixtures/browser_probes/styling.js"),
                    vec![],
                )
                .await?
                .convert::<String>()?;
            std::fs::write(ctx.artifacts.join(format!("{theme}-{width}.json")), metrics)?;
            driver
                .screenshot(&ctx.artifacts.join(format!("{theme}-{width}.png")))
                .await?;
        }
    }
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(400, 800, 1.0, true)
        .await?;
    driver
        .execute(
            "window.scrollTo(0, document.documentElement.scrollHeight);",
            vec![],
        )
        .await?;
    driver
        .screenshot(&ctx.artifacts.join("light-400-footer.png"))
        .await?;
    let toggle = driver.find(By::Css("[data-theme-toggle]")).await?;
    toggle.send_keys(" ").await?;
    assert!(driver.execute("const s = getComputedStyle(document.activeElement); return document.activeElement.matches('[data-theme-toggle]:focus-visible') && s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) >= 3 && s.outlineOffset === '1px';", vec![]).await?.convert::<bool>()?);
    let tile = driver.find(By::Css("[data-navigation-tiles] > a")).await?;
    driver
        .action_chain()
        .move_to_element_center(&tile)
        .perform()
        .await?;
    let motion = driver.execute("const s = getComputedStyle(document.querySelector('[data-navigation-tiles] > a')); return [matchMedia('(prefers-reduced-motion: reduce)').matches, s.translate, s.transform, s.transitionProperty];", vec![]).await?.convert::<(bool, String, String, String)>()?;
    assert!(
        motion.0 && motion.1 == "none" && motion.2 == "none" && motion.3 == "none",
        "Reduced-motion state: {motion:?}"
    );
    Ok(())
}
