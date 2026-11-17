use thirtyfour::components::SelectElement;
use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::WebDriverActions,
};

const SETUP: &str = "[data-game-setup='Province Quest']";
const MAP: &str = "[data-province-map]";
const REGIONS: &str = "[data-province-map] [data-region]";
const CHOICES: &str = "[data-province-choices] button";
const REVIEW: &str = "[data-province-review]";

async fn complete_game(ctx: ScenarioContext, difficulty: usize) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/province-quest").await?;
    driver.query_single(SETUP).await?;

    let difficulty_buttons = driver
        .find_all_css("[data-game-setup='Province Quest'] button[aria-pressed]")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(difficulty_buttons.len(), 6);
    difficulty_buttons[difficulty].click().await?;

    difficulty_buttons[4].click().await?;
    driver
        .query_single("[data-game-setup='Province Quest'] button:not([aria-pressed])")
        .await?
        .click()
        .await?;

    for round in 0..10 {
        if round % 2 == 0 {
            if difficulty == 2 {
                let province = driver.query_single("[data-province-quest] select").await?;

                SelectElement::new(&province)
                    .await?
                    .select_by_value("44")
                    .await?;
            }
            driver.query_first(REGIONS).await?;
            let regions = driver.find_all_css(REGIONS).await?;
            if difficulty != 2 {
                zipeek_e2e::e2e_ensure_eq!(regions.len(), 34);
            } else {
                zipeek_e2e::e2e_ensure!(!regions.is_empty());
            }
            zipeek_e2e::e2e_ensure_eq!(
                regions[0].attr("aria-label").await?.as_deref(),
                Some("Map region 1"),
                "Pre-answer map controls must not reveal region names"
            );
            if round == 0 {
                let help = driver.query_single("[data-province-prompt] button").await?;
                help.click().await?;
            }

            driver.query_first(REGIONS).await?.click().await?;
        } else {
            driver.query_single(MAP).await?;
            let highlighted = driver
                .find_all_css("[data-province-map] path[fill='#eab308']")
                .await?;
            zipeek_e2e::e2e_ensure_eq!(
                highlighted.len(),
                1,
                "Choice question highlights one region"
            );
            zipeek_e2e::e2e_ensure!(
                highlighted[0]
                    .attr("aria-label")
                    .await?
                    .unwrap_or_default()
                    .starts_with("Highlighted map region "),
                "Highlighted region must be announced without revealing its name"
            );
            let highlighted_id = highlighted[0].attr("data-region").await?;
            let choices = driver.find_all_css(CHOICES).await?;
            zipeek_e2e::e2e_ensure_eq!(choices.len(), 3);
            let mut selected_wrong = false;
            for choice in choices {
                if choice.attr("data-province-choice").await? != highlighted_id {
                    choice.click().await?;
                    selected_wrong = true;
                    break;
                }
            }
            zipeek_e2e::e2e_ensure!(selected_wrong, "A distractor must be available");
        }
        let review = driver.query_single(REVIEW).await?;
        if round % 2 == 1 {
            zipeek_e2e::e2e_ensure!(
                review.text().await?.contains("Not quite"),
                "A deliberately wrong answer must be scored as incorrect"
            );
        }
        let after_answer = driver.find_all_css(CHOICES).await?;
        if round % 2 == 1 {
            zipeek_e2e::e2e_ensure_eq!(after_answer.len(), 3);
            for choice in after_answer {
                zipeek_e2e::e2e_ensure_eq!(choice.is_enabled().await?, false);
            }
        }
        driver
            .query_single("[data-province-review] button")
            .await?
            .click()
            .await?;
    }
    let completed = driver
        .query_single("[data-province-quest] h1")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure_eq!(completed.as_str(), "Province Quest complete");
    driver
        .query_single("[data-province-quest] button")
        .await?
        .click()
        .await?;
    driver.query_single(SETUP).await?;
    Ok(())
}

#[tokio::test]
async fn province_quest_all_difficulties_complete_both_modes() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run("province_quest_easy", |ctx| complete_game(ctx, 0))
        .await?;
    harness
        .run("province_quest_medium", |ctx| complete_game(ctx, 1))
        .await?;
    harness
        .run("province_quest_hard", |ctx| complete_game(ctx, 2))
        .await?;
    harness.finish().await
}
