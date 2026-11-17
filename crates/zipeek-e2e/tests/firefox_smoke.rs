use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{BrowserKind, Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn browser_identity(ctx: ScenarioContext) -> HarnessResult<()> {
    let user_agent = ctx
        .driver
        .execute(
            zipeek_e2e::js_scripts::harness::E2E_BROWSER_USER_AGENT,
            vec![],
        )
        .await?
        .convert::<String>()?;
    let expected = match ctx.browser {
        BrowserKind::Chrome => "Chrome",
        BrowserKind::Firefox => "Firefox",
    };
    zipeek_e2e::e2e_ensure!(
        user_agent.contains(expected),
        "Selected {expected} browser did not match user agent: {user_agent}"
    );
    ctx.checkpoint_visible("browser-shell", "[data-component='app-shell']")
        .await
}

async fn firefox_navigation(ctx: ScenarioContext) -> HarnessResult<()> {
    for (path, marker) in [
        ("/browse", "[data-component='browse-page']"),
        ("/settings", "[data-component='settings-page']"),
        ("/games", "[data-component='games-gallery']"),
        ("/write-hanzi", "[data-component='write-hanzi-page']"),
        ("/practice", "[data-practice-setup]"),
    ] {
        ctx.open_route(path).await?;
        zipeek_e2e::e2e_ensure!(
            ctx.driver
                .query_single(marker)
                .await?
                .is_displayed()
                .await?,
            "Route {path} did not display {marker} on {}",
            ctx.browser.as_str()
        );
        ctx.checkpoint(&format!("firefox-{}", path.trim_start_matches('/')))
            .await?;
    }
    Ok(())
}

async fn firefox_history(ctx: ScenarioContext) -> HarnessResult<()> {
    ctx.open_route("/browse").await?;
    ctx.navigate_in_app(zipeek_e2e::AppRoute::new("/settings")?)
        .await?;
    ctx.driver
        .query_single("[data-component='settings-page']")
        .await?;
    ctx.driver.back().await?;
    ctx.driver
        .query_single("[data-component='browse-page']")
        .await?;
    ctx.driver.forward().await?;
    ctx.driver
        .query_single("[data-component='settings-page']")
        .await?;
    Ok(())
}

async fn firefox_responsive(ctx: ScenarioContext) -> HarnessResult<()> {
    ctx.open_route("/write-hanzi").await?;
    let measured = ctx.viewport().responsive(390, 844, true).await?;
    ctx.driver
        .query_single("[data-component='write-hanzi-page']")
        .await?;
    let viewport = ctx
        .driver
        .execute(
            zipeek_e2e::js_scripts::harness::E2E_BROWSER_VIEWPORT_WIDTH,
            vec![],
        )
        .await?
        .convert::<u64>()?;
    zipeek_e2e::e2e_ensure!(
        viewport == u64::from(measured.width) && viewport >= 320 && viewport < 640,
        "Firefox mobile viewport was not applied: {viewport}"
    );
    zipeek_e2e::e2e_ensure!(
        !zipeek_e2e::page_objects::navigation::has_horizontal_overflow(&ctx.driver).await?,
        "Firefox writing page overflows horizontally at measured {}px mobile viewport",
        measured.width
    );
    ctx.checkpoint("firefox-mobile-writing").await?;
    ctx.viewport().responsive(1280, 1000, false).await?;
    Ok(())
}

async fn browser_fixture_transport(ctx: ScenarioContext) -> HarnessResult<()> {
    ctx.configure_ai_fixture("malformed-generation").await?;
    let fixture = zipeek_e2e::harness::FixtureClient::from_ai_url(&ctx.ai_url)?;
    let state = fixture.requests().await?;
    zipeek_e2e::e2e_ensure_eq!(
        state["scenario"].as_str(),
        Some("malformed-generation"),
        "Fixture state is isolated from the browser transport"
    );
    ctx.configure_ai_fixture("success").await?;
    Ok(())
}

#[tokio::test]
async fn firefox_smoke() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness.run("browser-identity", browser_identity).await?;
    harness
        .run("browser-fixture-transport", browser_fixture_transport)
        .await?;
    harness
        .run("browser-feature-routes", firefox_navigation)
        .await?;
    harness.run("browser-history", firefox_history).await?;
    harness
        .run("browser-responsive", firefox_responsive)
        .await?;
    harness.finish().await
}
