use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn finish_time_weaver(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/time-weaver").await?;
    driver.query_single("[data-time-weaver-setup]").await?;
    driver
        .query_single("[data-time-weaver-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-time-weaver-game]").await?;
    let mut saw_time_round = false;
    let mut saw_build_round = false;
    for _round in 0..15 {
        let prompt = driver
            .query_single("[data-time-weaver-prompt]")
            .await?
            .text()
            .await?;
        if prompt.contains(':') {
            saw_time_round = true;
        }
        let tiles = driver
            .find_all_css("[data-time-weaver-build] [data-time-weaver-tile]")
            .await?;
        if !tiles.is_empty() {
            zipeek_e2e::e2e_ensure!(tiles.len() >= 3);
            if !saw_build_round {
                driver
                    .query_single("[data-time-weaver-tile='0']")
                    .await?
                    .click()
                    .await?;
                driver
                    .query_single("[data-time-weaver-tile='1']")
                    .await?
                    .click()
                    .await?;
                zipeek_e2e::e2e_ensure_eq!(
                    driver
                        .find_all_css("[data-time-weaver-assembled] button")
                        .await?
                        .len(),
                    2
                );
                driver
                    .query_single("[aria-label='Undo last tile']")
                    .await?
                    .click()
                    .await?;
                zipeek_e2e::e2e_ensure_eq!(
                    driver
                        .find_all_css("[data-time-weaver-assembled] button")
                        .await?
                        .len(),
                    1
                );
                driver
                    .query_single("[aria-label='Reset answer']")
                    .await?
                    .click()
                    .await?;
                zipeek_e2e::e2e_ensure_eq!(
                    driver
                        .find_all_css("[data-time-weaver-assembled] button")
                        .await?
                        .len(),
                    0
                );
                saw_build_round = true;
            }
            for tile in 0..tiles.len() {
                driver
                    .query_single(&format!("[data-time-weaver-tile='{tile}']"))
                    .await?
                    .click()
                    .await?;
            }
            driver
                .query_single("[data-time-weaver-submit]")
                .await?
                .click()
                .await?;
        } else {
            let choices = driver
                .find_all_css("[data-time-weaver-choices] button")
                .await?;
            zipeek_e2e::e2e_ensure_eq!(choices.len(), 3);
            choices[0].click().await?;
            for button in driver
                .find_all_css("[data-time-weaver-choices] button")
                .await?
            {
                zipeek_e2e::e2e_ensure!(!button.is_enabled().await?);
            }
        }
        let review = driver.query_single("[data-time-weaver-review]").await?;
        zipeek_e2e::e2e_ensure!(!review.text().await?.is_empty());
        driver
            .query_single("[data-time-weaver-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(saw_time_round);
    zipeek_e2e::e2e_ensure!(saw_build_round);
    driver.query_single("[data-time-weaver-finished]").await?;
    driver.query_single("[data-time-weaver-score]").await?;
    Ok(())
}

#[tokio::test]
async fn time_weaver_builds_and_recognizes_chinese_dates_weekdays_and_times() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("time_weaver_gameplay", finish_time_weaver)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
