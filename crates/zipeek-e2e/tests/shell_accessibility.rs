use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects};

async fn navigation_has_functional_skip_link(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    driver.query_single(page_objects::navigation::SKIP).await?;
    page_objects::shell::assert_navigation_ready(driver).await?;
    zipeek_e2e::e2e_ensure!(
        page_objects::navigation::skip_link_target_exists(driver).await?,
        "Skip link must target an existing main-content element"
    );
    zipeek_e2e::e2e_ensure!(
        !page_objects::navigation::has_horizontal_overflow(driver).await?,
        "Settings shell should not overflow at desktop width"
    );
    Ok(())
}

async fn navigation_titles_follow_routes(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    for (path, label) in [
        ("/settings", "Settings"),
        ("/browse", "Browse"),
        ("/practice", "Practice"),
        ("/courses/vocabulary", "Vocabulary courses"),
        ("/courses/grammar", "Grammar courses"),
    ] {
        page_objects::navigation::visit(driver, path).await?;
        page_objects::navigation::wait_for_title(driver, label).await?;
        zipeek_e2e::e2e_ensure!(
            page_objects::navigation::current_hash(driver)
                .await?
                .contains(path)
        );
    }
    ctx.checkpoint("navigation-route-titles").await?;
    Ok(())
}

async fn navigation_does_not_overflow_at_mobile_width(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.set_window_rect(0, 0, 390, 844).await?;
    for path in ["/settings", "/browse", "/practice", "/courses"] {
        ctx.open_route(path).await?;
        driver.query_single("[data-component='app-shell']").await?;
        zipeek_e2e::e2e_ensure!(
            !page_objects::navigation::has_horizontal_overflow(driver).await?,
            "Horizontal overflow on mobile at {path}"
        );
    }
    ctx.checkpoint("mobile-shell-overflow").await?;
    Ok(())
}

#[tokio::test]
async fn shell_accessibility() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "navigation_skip_link_targets_content",
            navigation_has_functional_skip_link,
        )
        .await?;
    harness
        .run(
            "navigation_titles_follow_routes",
            navigation_titles_follow_routes,
        )
        .await?;
    harness
        .run(
            "navigation_mobile_no_horizontal_overflow",
            navigation_does_not_overflow_at_mobile_width,
        )
        .await?;
    harness.finish().await
}
