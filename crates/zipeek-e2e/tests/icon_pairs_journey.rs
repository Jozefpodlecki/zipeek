use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

async fn match_complete_round(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/icon-pairs").await?;
    driver
        .query_single("[aria-label='Icon Pairs setup']")
        .await?;
    driver
        .query_single("[aria-label='Vocabulary source'] button[aria-pressed='true']")
        .await?;
    driver
        .query_single("[aria-label='Icon Pairs setup'] > button")
        .await?
        .click()
        .await?;
    driver
        .query_single("[aria-label='Icon Pairs board']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-icon-pairs-tile]").await?.len(),
        12
    );
    zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-icon-choice]").await?.len(), 6);
    zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-hanzi-choice]").await?.len(), 6);
    driver
        .query_single("[data-icon-choice='0']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-hanzi-choice='1']")
        .await?
        .click()
        .await?;
    let mismatch = driver
        .query_single("[aria-label='Icon Pairs board'] [role='status']")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        mismatch.contains("Not a match"),
        "Incorrect pairs must provide feedback"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-icon-pairs-score]")
            .await?
            .text()
            .await?
            .contains("1 attempts"),
        "A mismatch must be counted without completing a pair"
    );
    for word_index in 0..6 {
        let icon_selector = format!("[data-icon-choice='{word_index}']");
        let hanzi_selector = format!("[data-hanzi-choice='{word_index}']");
        driver.query_single(&icon_selector).await?.click().await?;
        driver.query_single(&hanzi_selector).await?.click().await?;
        zipeek_e2e::e2e_ensure!(
            !driver
                .query_single(&icon_selector)
                .await?
                .is_enabled()
                .await?
        );
        zipeek_e2e::e2e_ensure!(
            !driver
                .query_single(&hanzi_selector)
                .await?
                .is_enabled()
                .await?
        );
    }
    let score = driver
        .query_single("[data-icon-pairs-score]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        score.contains("6 matched"),
        "All six pairs must match: {score}"
    );
    ctx.checkpoint("icon-pairs-complete-round").await?;
    driver
        .query_single("[aria-label='Icon Pairs board'] button")
        .await?;
    let next = driver
        .find_all_css("[aria-label='Icon Pairs board'] button")
        .await?;
    zipeek_e2e::e2e_ensure!(!next.is_empty());
    let mut advanced = false;
    for button in next {
        if button.is_enabled().await? && button.text().await?.contains("Next round") {
            button.click().await?;
            advanced = true;
            break;
        }
    }
    zipeek_e2e::e2e_ensure!(advanced, "Completed grid must expose Next round");
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[aria-label='Game progress']")
            .await?
            .text()
            .await?
            .contains("Round 2"),
        "New rounds must reset the grid and advance the counter"
    );
    Ok(())
}

#[tokio::test]
async fn icon_pairs_curated_grid_completes_and_advances() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    let result = harness
        .run("icon_pairs_full_round", match_complete_round)
        .await;
    let cleanup = harness.finish().await;
    result.and(cleanup)
}
