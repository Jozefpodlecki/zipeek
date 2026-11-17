use thirtyfour::WebElement;
use zipeek_e2e::page_objects;
use zipeek_e2e::page_objects::{WebDriverActions, WebElementQueryActions};

use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind};

const ROCK_MUSIC: &str = "摇滚乐";
const ROCK_MUSIC_ROUTE: &str = "/#/browse?query=%E6%91%87%E6%BB%9A%E4%B9%90";

async fn open_rock_music(ctx: &ScenarioContext) -> HarnessResult<WebElement> {
    let driver = &ctx.driver;
    ctx.open_route(ROCK_MUSIC_ROUTE.trim_start_matches("/#"))
        .await?;
    let card = page_objects::browse::word_card(driver, ROCK_MUSIC).await?;
    zipeek_e2e::e2e_ensure!(card.text().await?.contains("rock music"));
    Ok(card)
}

async fn browse_expanded_meanings_keep_sense_metadata_scoped(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let card = open_rock_music(&ctx).await?;
    let panel = page_objects::browse::open_meanings(&card).await?;
    let counter = panel
        .query_single(page_objects::browse::SENSE_COUNTER)
        .await?;
    zipeek_e2e::e2e_ensure!(counter.text().await?.starts_with("1 of "));
    let first = panel
        .query_single("[data-spotlight-meaning]")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(!first.trim().is_empty());
    let navigator = panel.query_single("[data-meaning-navigator]").await?;
    zipeek_e2e::e2e_ensure!(
        navigator.rect().await?.width <= 400.0,
        "Meaning navigation must remain content-sized, not stretch across the word card"
    );
    let toggle = card
        .query_single(page_objects::browse::EXPLORE_MEANINGS)
        .await?;
    toggle.click().await?;
    zipeek_e2e::e2e_ensure!(
        card.find_all_css(page_objects::browse::SENSE_PANEL)
            .await?
            .is_empty()
    );
    ctx.checkpoint("browse-expanded-meanings").await?;
    Ok(())
}

async fn browse_without_selected_set_links_to_word_sets(ctx: ScenarioContext) -> HarnessResult<()> {
    let card = open_rock_music(&ctx).await?;
    zipeek_e2e::e2e_ensure!(
        card.find_all_css(page_objects::browse::SAVE_BUTTON)
            .await?
            .is_empty(),
        "Saving should not be offered without a selected Word Set"
    );
    let choose_set = card.query_single(page_objects::browse::CHOOSE_SET).await?;
    zipeek_e2e::e2e_ensure!(
        choose_set
            .attr("href")
            .await?
            .is_some_and(|href| href.contains("word-sets")),
        "The bookmark must navigate to the Word Sets page"
    );
    zipeek_e2e::e2e_ensure!(
        choose_set.inner_html().await?.contains("Choose a Word Set"),
        "The bookmark link must contain its accessible name"
    );
    choose_set.click().await?;
    ctx.driver
        .query_single("[data-component='word-sets-page']")
        .await?;
    Ok(())
}

async fn browse_save_remove_persists_across_reload(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    page_objects::word_sets::create(driver, "Browse E2E").await?;
    zipeek_e2e::e2e_ensure!(
        page_objects::word_sets::selected_set_label(driver)
            .await?
            .contains("Browse E2E")
    );
    zipeek_e2e::e2e_ensure!(
        page_objects::word_sets::persisted_set_names(driver)
            .await?
            .contains(&"Browse E2E".to_owned())
    );

    let card = open_rock_music(&ctx).await?;
    let raw_word_id = card
        .attr("data-browse-word-id")
        .await?
        .ok_or("Browse result is missing its word ID")?;
    let word_id = raw_word_id
        .parse::<i32>()
        .map_err(|error| format!("Invalid Browse word ID {raw_word_id:?}: {error}"))?;
    let save = page_objects::browse::save_button(&card).await?;
    zipeek_e2e::e2e_ensure_eq!(
        save.attr("aria-label").await?.as_deref(),
        Some("Add to Word Set")
    );
    save.click().await?;
    card.query_single(page_objects::browse::SAVE_ACTIVE).await?;

    driver.refresh().await?;
    page_objects::browse::assert_word_visible(driver, ROCK_MUSIC).await?;
    let saved = driver
        .query_single(page_objects::browse::SAVE_ACTIVE)
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        saved.attr("aria-label").await?.as_deref(),
        Some("Remove from Word Set")
    );
    let sets = page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(
        sets.iter().any(|set| {
            set["name"] == "Browse E2E"
                && set["word_ids"]
                    .as_array()
                    .is_some_and(|ids| ids.iter().any(|id| id.as_i64() == Some(i64::from(word_id))))
        }),
        "Saved Word Set must contain the selected dictionary word"
    );

    saved.click().await?;
    driver
        .query_single(page_objects::browse::SAVE_INACTIVE)
        .await?;
    let sets = page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(
        sets.iter().all(|set| {
            set["word_ids"]
                .as_array()
                .is_some_and(|ids| ids.iter().all(|id| id.as_i64() != Some(i64::from(word_id))))
        }),
        "Removing a word must update persisted Word Set membership"
    );
    driver.refresh().await?;
    page_objects::browse::assert_word_visible(driver, ROCK_MUSIC).await?;
    driver
        .query_single(page_objects::browse::SAVE_INACTIVE)
        .await?;
    let restored = page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(
        restored.iter().all(|set| {
            set["word_ids"]
                .as_array()
                .is_some_and(|ids| ids.iter().all(|id| id.as_i64() != Some(i64::from(word_id))))
        }),
        "Removed dictionary word reappeared in a Word Set after reload"
    );
    ctx.checkpoint("browse-word-set-round-trip").await?;
    Ok(())
}

async fn browse_audio_is_accessible(ctx: ScenarioContext) -> HarnessResult<()> {
    let card = open_rock_music(&ctx).await?;
    let listen = card.query_single(page_objects::browse::LISTEN).await?;
    zipeek_e2e::e2e_ensure!(listen.is_displayed().await?);
    zipeek_e2e::e2e_ensure!(listen.is_enabled().await?);
    zipeek_e2e::e2e_ensure_eq!(listen.attr("aria-label").await?.as_deref(), Some("Listen"));
    zipeek_e2e::e2e_ensure_eq!(listen.tag_name().await?, "button");
    Ok(())
}

async fn browse_empty_results_are_not_errors(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse?query=zipeek-e2e-no-such-word-9999999")
        .await?;
    let status = driver.query_single(page_objects::browse::EMPTY).await?;
    zipeek_e2e::e2e_ensure!(status.text().await?.contains("No matching words"));
    page_objects::browse::assert_healthy(driver).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css(page_objects::browse::WORD_CARD)
            .await?
            .is_empty()
    );
    Ok(())
}

async fn browse_pagination_returns_to_first_page(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    page_objects::browse::ready(driver).await?;
    page_objects::browse::assert_healthy(driver).await?;
    page_objects::browse::wait_for_page(driver, 1).await?;
    let first_page_ids = page_objects::browse::visible_word_ids(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        first_page_ids.len(),
        5,
        "The first page must contain five distinct dictionary entries"
    );
    let next = driver.query_single(page_objects::browse::NEXT).await?;
    zipeek_e2e::e2e_ensure!(
        next.is_enabled().await?,
        "The expanded dictionary must have a second page"
    );
    next.click().await?;
    page_objects::browse::wait_for_page(driver, 2).await?;
    let second_page_ids = page_objects::browse::visible_word_ids(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        second_page_ids.len(),
        5,
        "The second page must contain five dictionary entries"
    );
    zipeek_e2e::e2e_ensure!(
        first_page_ids
            .iter()
            .all(|id| !second_page_ids.contains(id)),
        "Dictionary pagination repeated entries across pages: first={first_page_ids:?}, second={second_page_ids:?}"
    );
    let previous = driver.query_single(page_objects::browse::PREVIOUS).await?;
    zipeek_e2e::e2e_ensure!(previous.is_enabled().await?);
    previous.click().await?;
    page_objects::browse::wait_for_page(driver, 1).await?;
    zipeek_e2e::e2e_ensure_eq!(
        page_objects::browse::visible_word_ids(driver).await?,
        first_page_ids,
        "Returning to page one must restore the same dictionary entries in the same order"
    );
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_css(page_objects::browse::PREVIOUS)
            .await?
            .is_enabled()
            .await?,
        "Previous must be disabled on the first dictionary page"
    );
    Ok(())
}

async fn browse_live_query_round_trip(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/browse").await?;
    page_objects::browse::wait_for_search_query(driver, "", None, 5).await?;

    page_objects::browse::set_search_query(driver, ROCK_MUSIC).await?;
    let matching =
        page_objects::browse::wait_for_search_query(driver, ROCK_MUSIC, Some(ROCK_MUSIC), 1)
            .await?;
    page_objects::browse::set_search_query(driver, "").await?;
    let restored = page_objects::browse::wait_for_search_query(driver, "", None, 5).await?;
    zipeek_e2e::e2e_ensure_eq!(restored["query"].as_str(), Some(""));
    zipeek_e2e::e2e_ensure_ne!(
        restored["ids"],
        matching["ids"],
        "Clearing the search must restore the unfiltered word IDs"
    );
    ctx.checkpoint("browse-query-round-trip").await?;
    Ok(())
}

async fn word_discovery_meaning_navigation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/random").await?;
    let current = driver.query_single("[data-meaning-counter]").await?;
    let label = current.text().await?;
    zipeek_e2e::e2e_ensure!(
        label.starts_with("1 of "),
        "Unexpected initial meaning: {label}"
    );
    let count = label
        .split_whitespace()
        .nth(2)
        .ok_or("Meaning counter is missing a total")?
        .parse::<usize>()
        .map_err(|error| format!("Invalid meaning count: {error}"))?;
    zipeek_e2e::e2e_ensure!(count > 0);
    let previous = driver.query_single("[data-previous-meaning]").await?;
    zipeek_e2e::e2e_ensure!(!previous.is_enabled().await?);
    let next = driver.query_single("[data-next-meaning]").await?;
    zipeek_e2e::e2e_ensure_eq!(next.is_enabled().await?, count > 1);
    if count > 1 {
        next.click().await?;
        driver
            .query_with_text("[data-meaning-counter]", "2 of ")
            .await?;
        previous.click().await?;
    }
    driver
        .query_single("[data-next-random-word]")
        .await?
        .click()
        .await?;
    let refreshed = driver
        .query_with_text("[data-meaning-counter]", "1 of ")
        .await?
        .text()
        .await?;
    zipeek_e2e::e2e_ensure!(
        refreshed.starts_with("1 of "),
        "New word must reset meaning selection"
    );
    ctx.checkpoint("word-discovery-meaning-navigation").await?;
    Ok(())
}

#[tokio::test]
async fn browse_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "browse_word_set_discovery",
            browse_without_selected_set_links_to_word_sets,
        )
        .await?;
    harness
        .run(
            "browse_word_set_membership",
            browse_save_remove_persists_across_reload,
        )
        .await?;
    harness
        .run("browse_listen_accessibility", browse_audio_is_accessible)
        .await?;
    harness
        .run("browse_empty_search", browse_empty_results_are_not_errors)
        .await?;
    harness
        .run("browse_live_query_round_trip", browse_live_query_round_trip)
        .await?;
    harness
        .run(
            "browse_pagination_round_trip",
            browse_pagination_returns_to_first_page,
        )
        .await?;
    harness
        .run(
            "browse_expanded_meanings",
            browse_expanded_meanings_keep_sense_metadata_scoped,
        )
        .await?;
    harness
        .run(
            "word_discovery_meaning_navigation",
            word_discovery_meaning_navigation,
        )
        .await?;
    harness.finish().await
}
