use thirtyfour::prelude::*;

#[tokio::test]
async fn mounted_app_waits_for_dictionary_download() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    let intercept = driver.execute(r#"return {source: `
        const originalFetch = window.fetch.bind(window);
        window.fetch = (input, options) => {
            if (String(input).endsWith('hanzi.duckdb')) {
                return new Promise((resolve, reject) => {
                    window.releaseDictionary = () => originalFetch(input, options).then(resolve, reject);
                });
            }
            return originalFetch(input, options);
        };
    `};"#, vec![]).await?;
    driver
        .cdp()
        .send_raw("Page.addScriptToEvaluateOnNewDocument", intercept.json())
        .await?;
    driver.goto("http://127.0.0.1:1420/").await?;
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
    driver
        .execute_async(
            r#"
        const done = arguments[arguments.length - 1];
        function release() {
            if (window.releaseDictionary) { window.releaseDictionary(); done(true); }
            else { requestAnimationFrame(release); }
        }
        release();
    "#,
            vec![],
        )
        .await?;
    driver
        .query(By::Css("[data-database-state='ready'] [data-home]"))
        .single()
        .await?;
    driver.quit().await
}

#[tokio::test]
#[ignore = "Requires Pages build served at http://127.0.0.1:1421/zipeek/"]
async fn pages_dictionary_and_workers_use_the_zipeek_base_path() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    driver.goto("http://127.0.0.1:1421/zipeek/#/browse").await?;
    driver.query(By::Css("[data-database-state='ready'][data-database-rows='12100'][data-database-engine='v1.4.3']")).single().await?;
    assert!(driver.execute("const local = performance.getEntriesByType('resource').map(item => new URL(item.name)).filter(url => url.origin === location.origin); return local.some(url => url.pathname === '/zipeek/hanzi.duckdb') && local.some(url => url.pathname.includes('/zipeek/vendor/duckdb/')) && local.every(url => url.pathname.startsWith('/zipeek/'));", vec![]).await?.convert::<bool>()?);
    driver.quit().await
}

#[tokio::test]
async fn dictionary_failure_retry_and_real_read_only_queries() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    driver
        .set_script_timeout(std::time::Duration::from_secs(60))
        .await?;
    driver
        .cdp()
        .send_raw(
            "Network.enable",
            std::collections::BTreeMap::<String, String>::new(),
        )
        .await?;
    let blocked = driver
        .execute("return {urls: ['*hanzi.duckdb']};", vec![])
        .await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", blocked.json())
        .await?;
    driver.goto("http://127.0.0.1:1420/").await?;
    driver
        .query(By::Css("[data-database-state='failed']"))
        .single()
        .await?;
    assert!(driver.find_all(By::Css("[data-home]")).await?.is_empty());
    let clear = driver.execute("return {urls: []};", vec![]).await?;
    driver
        .cdp()
        .send_raw("Network.setBlockedURLs", clear.json())
        .await?;
    driver
        .find(By::Css("[data-database-retry]"))
        .await?
        .click()
        .await?;
    driver.query(By::Css("[data-database-state='ready'][data-database-rows='12100'][data-database-engine='v1.4.3']")).single().await?;
    let probe = driver.execute_async(r#"
        const done = arguments[arguments.length - 1];
        (async () => {
            const resource = performance.getEntriesByType('resource').find(item => item.name.endsWith('/src/services/database/bridge.js'));
            const bridge = await import(resource.name);
            const handle = await bridge.openDatabase();
            try {
                const rows = await bridge.queryDatabase(handle, 'SELECT * FROM words ORDER BY id ASC', []);
                const selected = await bridge.queryDatabase(handle, 'SELECT id FROM words WHERE id = ?', [17]);
                let readOnly = false;
                try { await bridge.queryDatabase(handle, 'DELETE FROM words WHERE id=1', []); }
                catch (error) { readOnly = /read.only/i.test(error.message); }
                done({ok: rows.length === 12100 && Object.keys(rows[0]).length === 10 && rows.every((row, index) => row.id === index + 1) && selected[0].id === 17 && readOnly});
            } finally { await bridge.closeDatabase(handle); }
        })().catch(error => done({ok: false, error: String(error)}));
    "#, vec![]).await?;
    assert!(
        driver
            .execute("return arguments[0].ok;", vec![probe.json().clone()])
            .await?
            .convert::<bool>()?,
        "{probe:?}"
    );
    driver.quit().await
}

#[tokio::test]
async fn practice_query_decoding_preserves_shared_url() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    let shared = "http://127.0.0.1:1420/#/practice/eng-pinyin?mode=fixed&rounds=5&seed=1&hsk=2.0&level=7&pos=verb%2Cnoun%2Cunknown&offset=40";
    driver.goto(shared).await?;
    let practice = driver.query(By::Css("[data-practice-variant='eng-pinyin'][data-practice-mode='fixed'][data-practice-level='1'][data-practice-offset='40'][data-practice-pos='noun,verb']")).single().await?;
    assert!(practice.is_displayed().await?);
    assert_eq!(driver.current_url().await?.as_str(), shared);
    driver
        .goto("http://127.0.0.1:1420/#/practice/pinyin-hanzi?mode=oops&offset=-1&pos=unknown")
        .await?;
    driver.query(By::Css("[data-practice-variant='pinyin-hanzi'][data-practice-mode='endless'][data-practice-offset='0'][data-practice-pos='']")).single().await?;
    driver.quit().await
}

#[tokio::test]
async fn bootstrap_loader_is_centered_and_landscape_survives_both_themes() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
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
    driver.goto("http://127.0.0.1:1420/").await?;
    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
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
    driver.query(By::Css("[data-home]")).single().await?;
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
    driver
        .goto("http://127.0.0.1:1420/#/practice/eng-pinyin")
        .await?;
    assert_eq!(
        driver
            .query(By::Css("[data-loading='yew'][role='status']"))
            .single()
            .await?
            .text()
            .await?,
        "Loading"
    );
    driver.quit().await?;
    Ok(())
}

#[tokio::test]
async fn landing_page_loads() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;

    driver.goto("http://127.0.0.1:1420/").await?;

    let heading = driver
        .query(By::Css("h1"))
        .desc("Landing page heading")
        .single()
        .await?;

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

    driver.quit().await?;

    Ok(())
}

#[tokio::test]
async fn responsive_shell_preserves_navigation_theme_and_keyboard_access() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 1000, 1.0, false)
        .await?;
    driver.goto("http://127.0.0.1:1420/").await?;
    driver.query(By::Css("[data-home]")).single().await?;

    let tiles = driver.find_all(By::Css(".navigation-tile")).await?;
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
            .find(By::Css(".repository-link"))
            .await?
            .attr("href")
            .await?
            .as_deref(),
        Some("https://github.com/Jozefpodlecki/zipeek")
    );
    assert!(driver.execute("return getComputedStyle(document.body).color !== getComputedStyle(document.body).backgroundColor;", vec![]).await?.convert::<bool>()?);

    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
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
                "return getComputedStyle(document.querySelector('.app-sidebar')).position;",
                vec![],
            )
            .await?
            .convert::<String>()?;
        assert_eq!(sidebar, if width <= 768 { "static" } else { "sticky" });
        if width == 360 {
            driver
                .cdp()
                .emulation()
                .set_device_metrics_override(360, 1500, 1.0, true)
                .await?;
            driver.find(By::Css(".brand-link")).await?.click().await?;
            driver
                .screenshot(&artifacts.join("a30-mobile-light.png"))
                .await?;
        }
    }

    driver.find_all(By::Css(".navigation-tile")).await?[0]
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css(".route-placeholder h1"))
            .single()
            .await?
            .text()
            .await?,
        "Practice"
    );
    assert!(
        driver
            .find(By::Css(".navigation-link[aria-current='page']"))
            .await?
            .text()
            .await?
            .contains("Practice")
    );
    driver
        .find(By::Css(".navigation-link[href='#/browse']"))
        .await?
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css(".route-placeholder h1"))
            .single()
            .await?
            .text()
            .await?,
        "Browse"
    );
    driver.find(By::Css(".brand-link")).await?.click().await?;
    driver.query(By::Css("[data-home]")).single().await?;
    driver.find_all(By::Css(".navigation-tile")).await?[2]
        .click()
        .await?;
    assert_eq!(
        driver
            .query(By::Css(".route-placeholder h1"))
            .single()
            .await?
            .text()
            .await?,
        "Random"
    );
    driver.find(By::Css(".brand-link")).await?.click().await?;
    driver.query(By::Css("[data-home]")).single().await?;
    let before_skip = driver.current_url().await?;
    driver
        .find(By::Css(".skip-link"))
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

    driver.quit().await?;
    Ok(())
}

#[tokio::test]
async fn drawer_collapses_and_compact_desktop_has_no_unnecessary_scrollbar() -> WebDriverResult<()>
{
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::managed(caps).await?;
    driver
        .cdp()
        .emulation()
        .set_device_metrics_override(1280, 720, 1.0, false)
        .await?;
    driver.goto("http://127.0.0.1:1420/").await?;
    driver.query(By::Css("[data-home]")).single().await?;
    assert!(driver.execute("return document.querySelector('.app-topbar').contains(document.querySelector('[data-theme-toggle]'));", vec![]).await?.convert::<bool>()?);
    assert!(driver.execute("const r = document.querySelector('[data-theme-toggle]').getBoundingClientRect(); return r.top >= 0 && r.bottom <= document.querySelector('.app-topbar').getBoundingClientRect().bottom && innerWidth - r.right <= 24;", vec![]).await?.convert::<bool>()?);
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
                "return document.querySelector('.app-main').getBoundingClientRect().left === 0;",
                vec![]
            )
            .await?
            .convert::<bool>()?
    );
    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target");
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
        .find(By::Css(".navigation-link[href='#/browse']"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css(
            ".navigation-link[aria-current='page'][href='#/browse']",
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
    driver.quit().await?;
    Ok(())
}
