use crate::{HarnessResult, page_objects::WebDriverActions};
use std::collections::BTreeMap;
use thirtyfour::WebDriver;

pub const ROOT: &str = "[data-practice-sentence-builder]";

pub async fn answer(driver: &WebDriver) -> HarnessResult<Vec<String>> {
    let meaning = driver
        .query_single(&format!("{ROOT} > p"))
        .await?
        .text()
        .await?;
    let answers: BTreeMap<String, Vec<String>> =
        serde_json::from_str(include_str!("../../fixtures/sentence-builder-answers.json"))
            .map_err(|error| format!("Invalid Sentence Builder test answers: {error}"))?;
    answers
        .get(&meaning)
        .cloned()
        .ok_or_else(|| format!("Unknown Sentence Builder test prompt: {meaning}").into())
}

pub async fn choose(driver: &WebDriver, words: &[String]) -> HarnessResult<()> {
    let chosen = driver.execute(r#"
        const bank = [...document.querySelectorAll('[data-practice-sentence-builder] button[aria-label^="Add "]')];
        const used = new Set();
        for (const word of arguments[0]) {
            const tile = bank.find(button => !button.disabled && !used.has(button) && button.getAttribute('aria-label') === `Add ${word}`);
            if (!tile) throw new Error(`Missing available tile: ${word}`);
            used.add(tile);
            tile.click();
        }
        return true;
    "#, vec![serde_json::json!(words)]).await?.convert::<bool>()?;
    crate::e2e_ensure!(chosen);
    Ok(())
}

pub async fn choose_correct(driver: &WebDriver) -> HarnessResult<()> {
    choose(driver, &answer(driver).await?).await
}
