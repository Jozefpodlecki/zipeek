use zipeek_e2e::{HarnessResult, ScenarioContext, page_objects::WebDriverActions};

pub async fn play(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/market-bargain").await?;
    driver
        .query_single("[data-game-setup='Market Bargain']")
        .await?;
    driver
        .query_single("[data-market-difficulty='hard']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-market-support='reading']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-market-hanzi]").await?;
    zipeek_e2e::e2e_ensure_eq!(driver.is_present("[data-market-pinyin]").await?, false);
    driver
        .query_single("[data-market-reveal]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-market-pinyin]").await?;
    driver
        .query_single("[data-market-settings]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-market-support='listening']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-market-hidden]").await?;
    zipeek_e2e::e2e_ensure_eq!(driver.is_present("[data-market-hanzi]").await?, false);
    driver
        .query_single("[data-market-reveal]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-market-hanzi]").await?;
    driver.query_single("[data-market-pinyin]").await?;
    driver.query_single("[data-market-bargain]").await?;
    driver
        .query_single("[data-market-settings][aria-label='Change settings']")
        .await?;
    driver
        .query_single("[role='progressbar'][aria-valuemax='8']")
        .await?;
    let price = driver
        .query_single("[data-market-choice='0']")
        .await?
        .text()
        .await?;
    assert_eq!(price.split('.').nth(1).map(str::len), Some(2));
    let round_label = driver
        .query_single("[data-game-progress]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(round_label, "Round 1 of 8");
    let mut saw_sub_yuan_price = false;
    for round in 0..8 {
        for answer in 0..3 {
            let choice_text = driver
                .query_single(&format!("[data-market-choice='{answer}']"))
                .await?
                .text()
                .await?;
            if choice_text.starts_with("¥0.") {
                saw_sub_yuan_price = true;
            }
            let marker = driver
                .query_single("[data-game-progress]")
                .await?
                .text()
                .await?;
            if marker != format!("Round {} of 8", round + 1) {
                break;
            }
            let choice = driver
                .query_single(&format!("[data-market-choice='{answer}']"))
                .await?;
            choice.click().await?;
            if driver.is_present("[data-market-continue]").await? {
                driver.query_single("[data-market-review]").await?;
                driver
                    .query_single("[data-market-continue]")
                    .await?
                    .click()
                    .await?;
                break;
            }
        }
    }
    zipeek_e2e::e2e_ensure_eq!(saw_sub_yuan_price, true);
    driver.query_single("[data-game-complete]").await?;
    driver.query_single("[data-market-assisted-count]").await?;
    driver.query_single("[data-market-bargain] button").await?;
    ctx.checkpoint("market-bargain-complete").await?;
    Ok(())
}
