use zipeek_e2e::page_objects::{WebDriverActions, selectors::word_mosaics as mosaic};
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn complete_word_mosaics_round(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/word-mosaics?source=filters%3A3.1%7C%7C&rounds=1")
        .await?;
    driver.element(mosaic::ROOT).await?;
    let tiles = driver.elements(mosaic::TILE).await?;
    zipeek_e2e::e2e_ensure_eq!(tiles.len(), 16, "Word Mosaics should show eight pairs");

    ctx.checkpoint("word-mosaics-initial").await?;
    driver.click(&mosaic::tile(0, false)).await?;
    ctx.checkpoint("word-mosaics-first-selection").await?;
    driver.click(&mosaic::tile(1, true)).await?;
    ctx.checkpoint("word-mosaics-incorrect-pair").await?;
    let feedback = driver.element(mosaic::FEEDBACK).await?;
    zipeek_e2e::e2e_ensure!(
        feedback.text().await?.contains("Not a match"),
        "Mismatched tiles must provide feedback"
    );
    zipeek_e2e::e2e_ensure!(
        driver.elements(mosaic::CONTINUE).await?.is_empty(),
        "Continue must remain hidden before completing all pairs"
    );

    for pair in 0..8 {
        for meaning in [false, true] {
            driver.click(&mosaic::tile(pair, meaning)).await?;
        }
        if pair == 0 {
            ctx.checkpoint("word-mosaics-first-correct-pair").await?;
        }
        let disabled = driver.elements(&mosaic::matched_tiles(pair)).await?;
        zipeek_e2e::e2e_ensure_eq!(disabled.len(), 2, "Matched pair {pair} must be disabled");
    }

    zipeek_e2e::e2e_ensure!(
        driver
            .element(mosaic::PROGRESS)
            .await?
            .text()
            .await?
            .contains("8 of 8 pairs matched")
    );
    driver.click(mosaic::CONTINUE).await?;
    driver.element(mosaic::COMPLETED).await?;
    ctx.checkpoint("word-mosaics-round-complete").await?;
    Ok(())
}
