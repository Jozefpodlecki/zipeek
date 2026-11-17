use zipeek_e2e::page_objects::WebDriverActions;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects};

async fn absent_session_does_not_show_reminder(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings").await?;
    driver
        .query_single("[data-component='settings-page']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css(page_objects::session::REMINDER)
            .await?
            .is_empty(),
        "A fresh browser must not invent an unfinished Practice session"
    );
    Ok(())
}

async fn navigating_outside_practice_keeps_shell_usable(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.query_single("[data-component='app-shell']").await?;
    for (route, marker) in [
        ("/settings", "[data-component='settings-page']"),
        ("/browse", "[data-component='browse-page']"),
        ("/word-sets", "[data-component='word-sets-page']"),
    ] {
        page_objects::navigation::visit(driver, route).await?;
        driver.query_single(marker).await?;
        let main = driver
            .query_single("[data-component='main-content']")
            .await?;
        zipeek_e2e::e2e_ensure!(
            main.is_displayed().await?,
            "Navigation blocked by a session overlay on {route}"
        );
        zipeek_e2e::e2e_ensure!(
            page_objects::navigation::current_hash(driver)
                .await?
                .contains(route)
        );
    }
    Ok(())
}

#[tokio::test]
async fn session_reminder_contract() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "no_phantom_unfinished_session",
            absent_session_does_not_show_reminder,
        )
        .await?;
    harness
        .run(
            "navigation_outside_practice_remains_usable",
            navigating_outside_practice_keeps_shell_usable,
        )
        .await?;
    harness.finish().await
}
