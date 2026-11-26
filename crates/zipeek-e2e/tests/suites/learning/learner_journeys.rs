use zipeek_e2e::page_objects::WebElementQueryActions;

use zipeek_e2e::page_objects::{WebDriverActions, selectors};

use thirtyfour::prelude::*;
use zipeek_e2e::{HarnessResult, ScenarioContext};

async fn active_step(driver: &WebDriver, step: &str) -> WebDriverResult<WebElement> {
    driver
        .query_single(format!(
            "[data-practice-progress] [data-practice-step='{step}'][aria-current='step']"
        ))
        .await
}

async fn choose_stage(driver: &WebDriver, stage: &str) -> WebDriverResult<WebElement> {
    driver
        .query_single(format!(
            "[data-practice-exercise-stage='{stage}'][aria-selected='true']"
        ))
        .await
}

pub async fn rapid_hsk_selection_remains_responsive(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    ctx.checkpoint("learner-journeys-initial-state").await?;
    driver
        .find_css("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    active_step(driver, "vocabulary").await?;
    ctx.checkpoint_visible(
        "practice-vocabulary-step",
        "[data-practice-exercise-stage=\"vocabulary\"][aria-selected=\"true\"]",
    )
    .await?;

    driver.execute(
        zipeek_e2e::js_scripts::learning::LEARNER_JOURNEYS_RAPID_HSK_SELECTION_REMAINS_RESPONSIVE_01,
        vec![],
    ).await?;
    driver
        .query_single("[data-hsk-band='3:4'][aria-pressed='true']")
        .await?;
    driver
        .query_single("[data-hsk-band='3:5'][aria-pressed='true']")
        .await?;
    driver
        .query_single("[data-hsk-band='3:2'][aria-pressed='false']")
        .await?;
    driver
        .query_single("[data-hsk-band='3:3'][aria-pressed='false']")
        .await?;

    driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    active_step(driver, "summary").await?;

    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-vocabulary-preflight-status]")
            .await?
            .is_empty()
    );
    driver
        .query_single("[data-practice-vocabulary-validation='on-launch']")
        .await?;
    driver
        .query_single("[data-practice-summary-actions]")
        .await?;
    driver.query_single("[data-practice-summary-back]").await?;
    driver.query_single("[data-practice-start]").await?;
    driver
        .query_single("[data-practice-start]:not([disabled])")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-practice-start]")
            .await?
            .is_enabled()
            .await?,
        "A valid configured vocabulary selection must enable the Start action"
    );
    ctx.checkpoint("learner-journeys-verified-result").await?;
    Ok(())
}

pub async fn practice_progress_and_configuration_journey(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;

    active_step(driver, "path").await?;
    for step in ["exercises", "vocabulary", "summary"] {
        zipeek_e2e::e2e_ensure!(
            driver
                .find_css(format!("[data-practice-step='{step}']"))
                .await?
                .is_enabled()
                .await?
                == false,
            "Unvisited step {step} must not be navigable"
        );
    }

    driver
        .find_css("[data-practice-path='advanced']")
        .await?
        .click()
        .await?;
    active_step(driver, "exercises").await?;
    choose_stage(driver, "choose").await?;
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css("[data-practice-exercise]").await?.len(),
        15
    );

    driver
        .find_css("[data-practice-exercise='speaking']")
        .await?
        .click()
        .await?;
    driver
        .query_single(
            "[data-practice-exercise='speaking'][aria-pressed='true'][data-selected='true']",
        )
        .await?;
    driver
        .find_css("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    choose_stage(driver, "configure").await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-exercise]")
            .await?
            .is_empty(),
        "Configure must not repeat the exercise selection grid"
    );

    driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    active_step(driver, "vocabulary").await?;
    driver
        .query_single("[data-practice-step-panel='vocabulary']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-practice-step='summary']")
            .await?
            .is_enabled()
            .await?
            == false
    );

    driver
        .find_css("[data-practice-step='path']")
        .await?
        .click()
        .await?;
    active_step(driver, "path").await?;
    driver
        .find_css("[data-practice-step='exercises']")
        .await?
        .click()
        .await?;
    active_step(driver, "exercises").await?;

    driver
        .find_css("[data-practice-exercise-stage='choose']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-exercise='speaking'][aria-pressed='true']")
        .await?;
    Ok(())
}

pub async fn homepage_tiles_navigate_to_product_areas(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (label, destination, selector) in [
        ("Practice", "/practice", "[data-practice-progress]"),
        ("Browse", "/browse", "[data-component='browse-page']"),
        (
            "Word Sets",
            "/word-sets",
            "[data-component='word-sets-page']",
        ),
        ("Random", "/random", "[data-component='random-page']"),
    ] {
        driver.goto(ctx.url("/")).await?;
        driver.query_single("[data-home]").await?;
        let tile = driver
            .query_with_text("[data-navigation-tiles] a", label)
            .await?;
        tile.click().await?;
        driver.query_single(selector).await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .current_url()
                .await?
                .as_str()
                .contains(&format!("#{destination}")),
            "{label} tile did not navigate to {destination}"
        );
    }
    Ok(())
}

pub async fn secondary_product_routes_render(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (route, selector) in [
        ("/write-hanzi", "[data-component='write-hanzi-page']"),
        (
            "/translate-sentence",
            "[data-component='translate-sentence-page']",
        ),
        ("/games", "[data-component='games-gallery']"),
        ("/settings", "[data-component='settings-page']"),
        ("/docs", "[data-component='docs-page']"),
    ] {
        driver
            .goto(ctx.url(&format!("/#/{}", route.trim_start_matches('/'))))
            .await?;
        driver.query_single(selector).await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-route-placeholder]")
                .await?
                .is_empty()
        );
    }
    Ok(())
}

pub async fn word_set_rename_switch_and_delete(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    driver
        .query_single("[data-component='word-sets-page']")
        .await?;

    for name in ["E2E first", "E2E second"] {
        driver
            .find_css("[data-new-word-set]")
            .await?
            .send_keys(name)
            .await?;
        driver
            .find_css("[data-create-word-set]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            zipeek_e2e::page_objects::browser::saved_word_sets(driver)
                .await?
                .iter()
                .any(|set| set["name"] == name)
        );
    }
    driver
        .query_with_text("[data-word-set-option]", "E2E first")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-word-set-rename-tools] summary")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-rename-word-set]")
        .await?
        .send_keys("E2E renamed")
        .await?;
    driver
        .find_css("[data-confirm-rename-word-set]")
        .await?
        .click()
        .await?;
    driver
        .query_with_text("[data-target-word-set]", "E2E renamed")
        .await?;
    driver
        .query_with_text("[data-word-set-option]", "E2E second")
        .await?
        .click()
        .await?;
    driver.click(selectors::word_sets::DELETE).await?;
    driver.click(selectors::word_sets::DELETE).await?;
    let sets = zipeek_e2e::page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(sets.iter().any(|set| set["name"] == "E2E renamed"));
    zipeek_e2e::e2e_ensure!(!sets.iter().any(|set| set["name"] == "E2E second"));
    driver.refresh().await?;
    driver
        .query_with_text("[data-target-word-set]", "E2E renamed")
        .await?;
    Ok(())
}

pub async fn browse_filter_listen_and_save_word(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    driver
        .find_css("[data-new-word-set]")
        .await?
        .send_keys("E2E browsing vocabulary")
        .await?;
    driver
        .find_css("[data-create-word-set]")
        .await?
        .click()
        .await?;
    driver
        .query_single(selectors::word_sets::SELECTABLE_OPTION)
        .await?;
    ctx.open_route("/browse").await?;

    let category = driver.find_css("[data-browse-filter-autocomplete]").await?;
    category.send_keys("noun").await?;
    driver
        .query_single("[role='listbox'] [role='option']")
        .await?;
    category.send_keys(Key::Enter).await?;
    driver
        .query_single(selectors::browse::REMOVE_NOUN_FILTER)
        .await?;

    let first = driver
        .query_first("[data-component='browse-result']")
        .await?;
    let word_id = first
        .attr("data-browse-word-id")
        .await?
        .ok_or("Browse result missing stable word ID")?;
    first.find_css("[data-listen]").await?.click().await?;
    first
        .find_css("[data-toggle-saved-word]")
        .await?
        .click()
        .await?;
    driver
        .query_with_text(
            format!("[data-browse-word-id='{word_id}'] [data-toggle-saved-word]"),
            "Remove",
        )
        .await?;

    ctx.open_route("/word-sets").await?;
    driver
        .query_with_text("[data-component='word-sets-page'] li button", "Remove")
        .await?;
    let search = driver.find_css("[data-word-set-search]").await?;
    search.send_keys("unmatched-vocabulary-search").await?;
    driver
        .query_with_text(
            "[data-word-set-vocabulary] [role='status']",
            "No words match your search.",
        )
        .await?;
    search.clear().await?;
    driver.query_single("[data-word-set-word-id]").await?;
    let saved_sets = zipeek_e2e::page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(saved_sets.iter().any(|set| {
        set["name"] == "E2E browsing vocabulary"
            && set["word_ids"]
                .as_array()
                .is_some_and(|words| words.len() == 1)
    }));
    Ok(())
}

pub async fn quick_practice_summary_defaults_to_fixed_five(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .find_css("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    active_step(driver, "exercises").await?;
    for _ in 0..3 {
        driver
            .find_css("[data-practice-continue]")
            .await?
            .click()
            .await?;
    }
    active_step(driver, "summary").await?;
    driver
        .query_single("[data-mode-picker] [data-setup-choice='Fixed 5'][aria-pressed='true']")
        .await?;

    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-practice-continue]")
            .await?
            .is_empty()
    );
    Ok(())
}

pub async fn character_hunt_easy_session_launch(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice").await?;
    driver
        .find_css("[data-practice-path='quick']")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-practice-exercise='character-hunt']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-practice-exercise='character-hunt'][data-selected='true']")
        .await?;
    driver
        .find_css("[data-practice-exercise-stage='configure']")
        .await?
        .click()
        .await?;
    choose_stage(driver, "configure").await?;
    driver
        .query_single("[data-character-hunt-settings]")
        .await?;
    driver
        .find_css("[data-hunt-difficulty='easy']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-hunt-difficulty='easy'][aria-pressed='true']")
        .await?;
    driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    active_step(driver, "vocabulary").await?;
    driver
        .find_css("[data-hsk-band='3:2']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-hsk-band='3:2'][aria-pressed='true']")
        .await?;
    driver
        .find_css("[data-practice-continue]")
        .await?
        .click()
        .await?;
    active_step(driver, "summary").await?;
    driver
        .query_single("[data-character-hunt-difficulty]")
        .await?;
    driver
        .find_css("[data-practice-start]")
        .await?
        .click()
        .await?;
    let mut url = driver.current_url().await?;
    let route_query = url
        .fragment()
        .and_then(|fragment| fragment.split_once('?'))
        .map(|(_, query)| query.to_owned())
        .expect("Character Hunt route must carry a vocabulary query");
    url.set_fragment(None);
    url.set_query(Some(&route_query));
    let source = url
        .query_pairs()
        .find(|(key, _)| key == "source")
        .map(|(_, value)| value.into_owned());
    zipeek_e2e::e2e_ensure!(
        source
            .as_deref()
            .is_some_and(|value| value.starts_with("filters:3.1,3.2|")),
        "Character Hunt must preserve multiple selected HSK bands: {source:?}",
    );
    driver
        .query_single("[data-component='character-hunt'] [data-hunt-grid-size='2']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(driver.find_all_css("[data-hunt-tile]").await?.len(), 4);
    let hint = driver.find_css("[data-hunt-hint]").await?;
    zipeek_e2e::e2e_ensure!(
        hint.is_enabled().await?,
        "Hint is available before selecting a tile"
    );
    zipeek_e2e::e2e_ensure!(
        hint.attr("data-action").await?.is_none(),
        "Hunt controls should dispatch typed actions, not DOM action strings"
    );
    driver.query_single("[data-hunt-progress]").await?;

    let tile_labels = driver.find_all_css("[data-hunt-tile][aria-label]").await?;
    zipeek_e2e::e2e_ensure_eq!(tile_labels.len(), 4);
    zipeek_e2e::e2e_ensure!(
        !driver.find_all_css("[data-hunt-pinyin]").await?.is_empty(),
        "At least one playable tile should have a dictionary-aligned reading",
    );

    driver.find_css("[data-hunt-exit]").await?.click().await?;
    driver.query_single("[data-hunt-exit-dialog]").await?;
    driver
        .find_css("[data-hunt-exit-dialog] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-hunt-exit-dialog]")
            .await?
            .is_empty()
    );
    for round in 1..=5_u16 {
        complete_easy_hunt_round(driver).await?;
        if round == 5 {
            let summary = driver
                .query_single("[data-hunt-session-summary]")
                .await?
                .text()
                .await?;
            zipeek_e2e::e2e_ensure!(summary.contains("5 rounds completed"), "{summary}");
            zipeek_e2e::e2e_ensure!(summary.contains("words found"), "{summary}");
            ctx.checkpoint_visible(
                "character-hunt-board-completed",
                "[data-hunt-session-summary]",
            )
            .await?;
        } else {
            driver
                .query_single("[data-hunt-next]")
                .await?
                .click()
                .await?;
            driver.query_single("[data-hunt-grid-size='2']").await?;
        }
    }
    Ok(())
}

async fn complete_easy_hunt_round(driver: &WebDriver) -> HarnessResult<()> {
    for _ in 0..4 {
        if !driver
            .find_all_css("[data-hunt-next], [data-hunt-finished]")
            .await?
            .is_empty()
        {
            return Ok(());
        }
        let mut progress_made = false;
        let tiles = driver
            .find_all_css("[data-hunt-tile]:not([disabled])")
            .await?;
        let mut active = Vec::new();
        for tile in &tiles {
            if let Some(index) = tile
                .attr("data-hunt-tile")
                .await?
                .and_then(|value| value.parse::<usize>().ok())
            {
                active.push(index);
            }
        }
        let previous = driver
            .find_all_css("[data-hunt-tile][data-completed='true']")
            .await?
            .len();
        'combinations: for first in &active {
            for second in &active {
                if first == second {
                    continue;
                }
                for third in std::iter::once(None).chain(
                    active
                        .iter()
                        .filter(|third| *third != first && *third != second)
                        .map(Some),
                ) {
                    let selected = driver
                        .find_all_css("[data-hunt-tile][aria-pressed='true']")
                        .await?;
                    for tile in selected {
                        tile.click().await?;
                    }
                    let mut choice = vec![*first, *second];
                    if let Some(third) = third {
                        choice.push(*third);
                    }
                    for index in choice {
                        driver
                            .find_css(format!("[data-hunt-tile='{index}']"))
                            .await?
                            .click()
                            .await?;
                    }
                    let found = driver
                        .find_all_css("[data-hunt-tile][data-completed='true']")
                        .await?
                        .len();
                    if found > previous {
                        let word_count = driver.find_all_css("[data-hunt-found-word]").await?.len();
                        let meaning_count = driver
                            .find_all_css("[data-hunt-found-meaning]")
                            .await?
                            .len();
                        zipeek_e2e::e2e_ensure_eq!(
                            word_count,
                            meaning_count,
                            "Every completed word needs a definition field"
                        );
                        zipeek_e2e::e2e_ensure!(word_count > 0);
                        progress_made = true;
                        break 'combinations;
                    }
                }
            }
        }
        zipeek_e2e::e2e_ensure!(
            progress_made,
            "Could not find a valid word on a 2x2 Character Hunt board"
        );
    }
    zipeek_e2e::e2e_ensure!(
        !driver
            .find_all_css("[data-hunt-next], [data-hunt-finished]")
            .await?
            .is_empty(),
        "Easy board did not complete"
    );
    Ok(())
}
