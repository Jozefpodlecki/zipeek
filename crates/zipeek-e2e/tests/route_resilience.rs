use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::page_objects::WebElementQueryActions;
use zipeek_e2e::{AppRoute, Harness, HarnessResult, ScenarioContext, SuiteKind};

async fn direct_deep_links_survive_reload(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let cases = [
        ("/#/browse", "[data-component='browse-page']"),
        ("/#/word-sets", "[data-component='word-sets-page']"),
        ("/#/settings", "[data-component='settings-page']"),
        (
            "/#/courses/pinyin/tones",
            "[data-component='course-lesson']",
        ),
        ("/#/games/cloud-match", "[data-component='cloud-match']"),
        ("/#/practice", "[data-practice-setup]"),
    ];
    for (path, marker) in cases {
        ctx.open_route(path.trim_start_matches("/#")).await?;
        driver.query_single(marker).await?;
        driver.refresh().await?;
        driver.query_single(marker).await?;
        zipeek_e2e::e2e_ensure!(
            driver.current_url().await?.as_str().contains(path),
            "Deep link changed after reload: {path}"
        );
    }
    ctx.checkpoint("deep-links-reload").await?;
    Ok(())
}

async fn back_forward_navigation_restores_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    driver
        .query_single("[data-component='browse-page']")
        .await?;
    ctx.navigate_in_app(AppRoute::new("/settings")?).await?;
    driver
        .query_single("[data-component='settings-page']")
        .await?;
    driver.back().await?;
    driver
        .query_single("[data-component='browse-page']")
        .await?;
    zipeek_e2e::e2e_ensure!(driver.current_url().await?.to_string().contains("#/browse"));
    driver.forward().await?;
    driver
        .query_single("[data-component='settings-page']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .to_string()
            .contains("#/settings")
    );
    Ok(())
}

async fn unknown_route_provides_recovery(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/definitely-not-a-real-route").await?;
    let main = driver
        .query_single("[data-component='main-content']")
        .await?;
    zipeek_e2e::e2e_ensure!(main.is_displayed().await?);
    zipeek_e2e::e2e_ensure!(
        !main.text().await?.trim().is_empty(),
        "Unknown route must explain navigation failure"
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_all_css("[data-component='app-header']")
            .await?
            .is_empty(),
        "Unknown routes must retain shell navigation"
    );
    driver.refresh().await?;
    driver.query_single("[data-route-placeholder]").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .current_url()
            .await?
            .as_str()
            .contains("#/definitely-not-a-real-route"),
        "Unknown routes must remain stable across a browser reload"
    );
    let recovery = driver.query_single("[data-route-placeholder] a").await?;
    zipeek_e2e::e2e_ensure_eq!(
        recovery.text().await?.trim(),
        "Back to home",
        "Unknown routes must offer a clearly named recovery action"
    );
    recovery.click().await?;
    driver
        .query_single("[data-component='main-content']")
        .await?;
    zipeek_e2e::page_objects::browser::wait_for_hash_route(driver, "/").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-route-placeholder]")
            .await?
            .is_empty(),
        "The recovery link must leave the not-found page"
    );
    ctx.checkpoint("unknown-route-home-recovery").await?;
    Ok(())
}

async fn explicit_not_found_route_offers_recovery(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/#/404")).await?;
    let placeholder = driver.query_single("[data-route-placeholder]").await?;
    zipeek_e2e::e2e_ensure!(placeholder.is_displayed().await?);
    zipeek_e2e::e2e_ensure!(
        placeholder.text().await?.contains("Page not found"),
        "The explicit /404 route should render the same recovery UI"
    );
    let link = placeholder.find_css("a").await?;
    link.click().await?;
    zipeek_e2e::page_objects::browser::wait_for_hash_route(driver, "/").await?;
    Ok(())
}

#[tokio::test]
async fn route_resilience() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "route-deep-links-survive-reload",
            direct_deep_links_survive_reload,
        )
        .await?;
    harness
        .run(
            "route-browser-back-forward",
            back_forward_navigation_restores_routes,
        )
        .await?;
    harness
        .run("route-not-found-recovery", unknown_route_provides_recovery)
        .await?;
    harness
        .run(
            "explicit-404-route-recovery",
            explicit_not_found_route_offers_recovery,
        )
        .await?;
    harness.finish().await
}
