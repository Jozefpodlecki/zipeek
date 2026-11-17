use std::time::Duration;

use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, js_scripts, page_objects::WebDriverActions,
};

const DRAGON: &str = "[data-learning-dragon]";
const RECORDING_TOGGLE: &str = "[data-personal-learning-history-toggle]";
const OTHER_TAB: &str = "[data-other-tab-warning]";

async fn recording_dragon_follows_preference(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/settings?section=data").await?;
    driver.query_single(RECORDING_TOGGLE).await?;
    let checkbox = driver.find_css(RECORDING_TOGGLE).await?;
    if checkbox.is_selected().await? {
        checkbox.click().await?;
    }
    zipeek_e2e::e2e_ensure!(driver.find_all_css(DRAGON).await?.is_empty());
    driver.find_css(RECORDING_TOGGLE).await?.click().await?;
    driver.query_single(DRAGON).await?;
    ctx.open_route("/practice").await?;
    driver.query_single(DRAGON).await?;
    driver.query_single("[data-component='app-header']").await?;
    driver
        .query_single("[data-component='navigation-drawer']")
        .await?;
    driver.refresh().await?;
    driver.query_single(DRAGON).await?;
    ctx.open_route("/settings?section=data").await?;
    driver.find_css(RECORDING_TOGGLE).await?.click().await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css(DRAGON).await?.is_empty());
    ctx.checkpoint("dragon-follows-learning-recording").await?;
    Ok(())
}

async fn practice_shells_follow_session_lifecycle(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver.query_single("[data-component='app-header']").await?;
    driver
        .query_single("[data-component='navigation-drawer']")
        .await?;
    let toggle = driver.query_single("[data-drawer-toggle]").await?;
    let before = toggle.attr("aria-expanded").await?;
    toggle.click().await?;
    let after = driver
        .query_single("[data-drawer-toggle]")
        .await?
        .attr("aria-expanded")
        .await?;
    zipeek_e2e::e2e_ensure!(
        before != after,
        "Practice setup must preserve the navigation drawer toggle"
    );
    driver.click("[data-practice-path='quick']").await?;
    driver.click("[data-practice-exercise='matching']").await?;
    driver
        .click("[data-practice-continue]:not([disabled])")
        .await?;
    driver
        .click("[data-practice-continue]:not([disabled])")
        .await?;
    driver
        .click("[data-practice-start]:not([disabled])")
        .await?;
    driver
        .query_single("[data-component='practice-session-shell']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-component='app-header'], [data-component='navigation-drawer'], [data-app-footer]")
            .await?
            .is_empty(),
        "A Practice session must not render the application chrome"
    );
    ctx.checkpoint("practice-session-is-distraction-free")
        .await?;
    Ok(())
}

async fn second_tab_presence_is_detected(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css(OTHER_TAB).await?.is_empty());
    let opened = driver
        .execute(
            js_scripts::platform::R421_OPEN_SECOND_ZIPEEK_TAB,
            vec![ctx.url("/#/practice").into()],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(opened, "Browser must allow opening a second test tab");
    let result: HarnessResult<()> = async {
        for _ in 0..20 {
            if !driver.find_all_css(OTHER_TAB).await?.is_empty() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Err("The first tab did not detect the second tab".into())
    }
    .await;
    driver
        .execute(js_scripts::platform::R421_CLOSE_SECOND_ZIPEEK_TAB, vec![])
        .await?;
    result?;
    for _ in 0..30 {
        if driver.find_all_css(OTHER_TAB).await?.is_empty() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err("The first tab did not clear the second-tab warning".into())
}

#[tokio::test]
async fn learning_header_and_browser_tabs() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("learning_dragon_pref", recording_dragon_follows_preference)
        .await?;
    harness
        .run(
            "practice_shell_lifecycle",
            practice_shells_follow_session_lifecycle,
        )
        .await?;
    harness
        .run("second_tab_presence", second_tab_presence_is_detected)
        .await?;
    harness.finish().await
}
