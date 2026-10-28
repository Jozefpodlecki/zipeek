use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn landing_page_loads(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = ctx.driver.clone();

    driver.goto(ctx.url("/")).await?;

    let heading = crate::page_objects::home::brand(&driver).await?;

    assert_eq!(heading.text().await?, "Zipeek");

    let footer = driver.query(By::Css("[data-app-footer]")).single().await?;
    assert!(footer.text().await?.contains("Jozef Podlecki"));
    assert!(footer.text().await?.contains("CC-CEDICT"));
    assert!(footer.text().await?.contains("CC BY-SA 4.0"));

    driver
        .execute("localStorage.removeItem('dark');", vec![])
        .await?;
    driver.refresh().await?;
    let toggle = driver
        .query(By::Css("[data-theme-toggle][aria-checked='true']"))
        .single()
        .await?;
    driver
        .query(By::Css("html.dark body.dark"))
        .single()
        .await?;
    toggle.send_keys(" ").await?;
    driver
        .query(By::Css("[data-theme-toggle][aria-checked='false']"))
        .single()
        .await?;
    driver
        .query(By::Css("html:not(.dark) body:not(.dark)"))
        .single()
        .await?;
    assert!(
        driver
            .execute("return localStorage.getItem('dark') === 'false';", vec![])
            .await?
            .convert::<bool>()?
    );

    driver.refresh().await?;
    driver
        .query(By::Css("[data-theme-toggle][aria-checked='false']"))
        .single()
        .await?;
    driver
        .query(By::Css("html:not(.dark) body:not(.dark)"))
        .single()
        .await?;

    driver
        .execute("localStorage.setItem('dark', 'invalid');", vec![])
        .await?;
    driver.refresh().await?;
    let toggle = driver
        .query(By::Css("[data-theme-toggle][aria-checked='true']"))
        .single()
        .await?;
    driver.execute("Storage.prototype.setItem = function() { throw new DOMException('blocked', 'SecurityError'); };", vec![]).await?;
    toggle.click().await?;
    driver
        .query(By::Css("[data-theme-toggle][aria-checked='false']"))
        .single()
        .await?;
    driver
        .query(By::Css("html:not(.dark) body:not(.dark)"))
        .single()
        .await?;

    Ok(())
}

pub async fn responsive_shell_preserves_navigation_theme_and_keyboard_access(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 1000, 1.0, false)
        .await?;
    driver.goto(ctx.url("/")).await?;
    driver.query(By::Css("[data-home]")).single().await?;

    let tiles = driver
        .find_all(By::Css("[data-navigation-tiles] > a"))
        .await?;
    assert_eq!(tiles.len(), 3);
    for (tile, expected) in tiles.iter().zip(["Practice", "Browse", "Random"]) {
        assert_eq!(tile.find(By::Css("h2")).await?.text().await?, expected);
    }
    assert_eq!(
        driver
            .find(By::Css("[data-daily-word]"))
            .await?
            .text()
            .await?
            .matches("No word available")
            .count(),
        1
    );
    assert_eq!(
        driver
            .find(By::Css("[aria-label='Zipeek repository on GitHub']"))
            .await?
            .attr("href")
            .await?
            .as_deref(),
        Some("https://github.com/Jozefpodlecki/zipeek")
    );
    assert!(driver.execute("return getComputedStyle(document.body).color !== getComputedStyle(document.body).backgroundColor;", vec![]).await?.convert::<bool>()?);

    let artifacts = ctx.artifacts.clone();
    driver
        .screenshot(&artifacts.join("a30-desktop-dark.png"))
        .await?;
    driver
        .find(By::Css("[data-theme-toggle]"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("html:not(.dark)")).single().await?;
    driver
        .screenshot(&artifacts.join("a30-desktop-light.png"))
        .await?;

    for width in [360, 768, 1024, 1280] {
        driver
            .cdp()
            .emulation()
            .set_device_metrics_override(width, 1000, 1.0, width <= 768)
            .await?;
        assert_eq!(
            driver
                .execute("return window.innerWidth;", vec![])
                .await?
                .convert::<u32>()?,
            width
        );
        assert!(
            driver
                .execute(
                    "return document.documentElement.scrollWidth <= window.innerWidth;",
                    vec![]
                )
                .await?
                .convert::<bool>()?,
            "horizontal overflow at {width}"
        );
        let sidebar = driver
            .execute(
                "return getComputedStyle(document.querySelector('#app-drawer')).position;",
                vec![],
            )
            .await?
            .convert::<String>()?;
        assert_eq!(sidebar, if width < 768 { "static" } else { "sticky" });
        if width == 360 {
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(360, 1500, 1.0, true)
                .await?;
            driver
                .find(By::Css("header a[href='#/']"))
                .await?
                .click()
                .await?;
            driver
                .screenshot(&artifacts.join("a30-mobile-light.png"))
                .await?;
        }
    }

    driver
        .find_all(By::Css("[data-navigation-tiles] > a"))
        .await?[0]
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css("[data-practice-setup] h1"))
            .single()
            .await?
            .text()
            .await?,
        "Practice"
    );
    assert!(
        driver
            .find(By::Css("[data-navigation-link][aria-current='page']"))
            .await?
            .text()
            .await?
            .contains("Practice")
    );
    driver
        .find(By::Css("[data-navigation-link][href='#/browse']"))
        .await?
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css("[data-route-placeholder] h1"))
            .single()
            .await?
            .text()
            .await?,
        "Browse"
    );
    driver
        .find(By::Css("header a[href='#/']"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("[data-home]")).single().await?;
    driver
        .find_all(By::Css("[data-navigation-tiles] > a"))
        .await?[2]
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css("[data-route-placeholder] h1"))
            .single()
            .await?
            .text()
            .await?,
        "Random"
    );
    driver
        .find(By::Css("header a[href='#/']"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("[data-home]")).single().await?;
    let before_skip = driver.current_url().await?;
    driver
        .find(By::Css("[data-skip-link]"))
        .await?
        .send_keys(Key::Enter)
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
    assert_eq!(driver.current_url().await?, before_skip);
    assert_eq!(driver.current_url().await?.path(), "/");

    Ok(())
}

pub async fn drawer_collapses_and_compact_desktop_has_no_unnecessary_scrollbar(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = ctx.driver.clone();
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 720, 1.0, false)
        .await?;
    driver.goto(ctx.url("/")).await?;
    driver.query(By::Css("[data-home]")).single().await?;
    assert!(driver.execute("return document.querySelector('[data-topbar]').contains(document.querySelector('[data-theme-toggle]'));", vec![]).await?.convert::<bool>()?);
    assert!(driver.execute("const r = document.querySelector('[data-theme-toggle]').getBoundingClientRect(); return r.top >= 0 && r.bottom <= document.querySelector('[data-topbar]').getBoundingClientRect().bottom && innerWidth - r.right <= 24;", vec![]).await?.convert::<bool>()?);
    assert!(
        !driver
            .find(By::Css("body"))
            .await?
            .text()
            .await?
            .contains("Chinese vocabulary")
    );

    for (width, height) in [(1024, 768), (1280, 720), (1366, 768)] {
        driver
            .cdp()
            .emulation()
            .set_device_metrics_override(width, height, 1.0, false)
            .await?;
        assert!(driver.execute("return document.documentElement.scrollHeight <= innerHeight && document.documentElement.scrollWidth <= innerWidth;", vec![]).await?.convert::<bool>()?, "unnecessary scrolling at {width}x{height}");
    }
    let toggle = driver.find(By::Css("[data-drawer-toggle]")).await?;
    toggle.send_keys(" ").await?;
    driver
        .query(By::Css("[data-drawer-toggle][aria-expanded='false']"))
        .single()
        .await?;
    assert!(
        !driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );
    assert!(
        driver
            .execute(
                "return document.querySelector('#main-content').getBoundingClientRect().left === 0;",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    let artifacts = ctx.artifacts.clone();
    driver
        .screenshot(&artifacts.join("a30-collapsed.png"))
        .await?;
    toggle.click().await?;
    driver
        .query(By::Css("[data-drawer-toggle][aria-expanded='true']"))
        .single()
        .await?;
    assert!(
        driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );

    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(360, 800, 1.0, true)
        .await?;
    toggle.click().await?;
    driver
        .query(By::Css("[data-drawer-toggle][aria-expanded='false']"))
        .single()
        .await?;
    assert!(
        !driver
            .find(By::Id("app-drawer"))
            .await?
            .is_displayed()
            .await?
    );
    assert!(
        driver
            .execute(
                "return document.documentElement.scrollHeight > innerHeight;",
                vec![]
            )
            .await?
            .convert::<bool>()?,
        "mobile content must remain scrollable when it does not fit"
    );
    toggle.click().await?;
    driver
        .query(By::Css("[data-drawer-toggle][aria-expanded='true']"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-navigation-link][href='#/browse']"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css(
            "[data-navigation-link][aria-current='page'][href='#/browse']",
        ))
        .single()
        .await?;
    assert!(
        !driver
            .find(By::Css("body"))
            .await?
            .text()
            .await?
            .contains("Follow your curiosity")
    );
    Ok(())
}
