use zipeek_e2e::page_objects::{WebDriverActions, selectors};
use zipeek_e2e::{HarnessResult, ScenarioContext};

pub async fn personal_sets_persist_across_navigation_and_backup_rejects_invalid_data(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    ctx.checkpoint("word-sets-initial-state").await?;
    driver
        .query_single("[data-component='word-sets-page'] [data-new-word-set]")
        .await?
        .send_keys("E2E saved vocabulary")
        .await?;
    driver
        .find_css("[data-create-word-set]")
        .await?
        .click()
        .await?;
    driver
        .query_single(selectors::word_sets::SELECTABLE_OPTION)
        .await?;
    ctx.checkpoint_visible("word-set-created", "[data-target-word-set]")
        .await?;
    let saved_sets = zipeek_e2e::page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(saved_sets.iter().any(|set| {
        set["id"]
            .as_str()
            .is_some_and(|id| id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()))
            && set["name"] == "E2E saved vocabulary"
            && set["word_ids"]
                .as_array()
                .is_some_and(|words| words.is_empty())
    }));

    let stable_id = saved_sets
        .iter()
        .find(|set| set["name"] == "E2E saved vocabulary")
        .and_then(|set| set["id"].as_str())
        .ok_or("New Word Set did not receive a stable ID")?
        .to_owned();

    ctx.open_route("/practice").await?;
    driver
        .find_css("[data-vocabulary-personal]")
        .await?
        .click()
        .await?;
    driver
        .query_single(
            "[data-component='practice-setup-page'] [data-vocabulary-set] option:not([value=''])",
        )
        .await?;

    ctx.open_route("/word-sets").await?;
    driver.refresh().await?;
    let reloaded = zipeek_e2e::page_objects::browser::saved_word_sets(driver).await?;
    zipeek_e2e::e2e_ensure!(
        reloaded.iter().any(|set| set["id"] == stable_id),
        "Word Set identity changed on reload"
    );
    driver
        .query_single(selectors::word_sets::SELECTABLE_OPTION)
        .await?;
    driver
        .find_css("[data-component='word-set-backup'] summary")
        .await?
        .click()
        .await?;
    driver
        .find_css("[data-export-word-sets]")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(driver
        .execute(
            zipeek_e2e::js_scripts::learning::WORD_SETS_PERSONAL_SETS_PERSIST_ACROSS_NAVIGATION_AND_BACKUP_REJECTS_INVALID_DATA_01,
            vec![],
        )
        .await?
        .convert::<bool>()?);

    let backup = driver.find_css("[data-word-set-backup-json]").await?;
    backup.clear().await?;
    backup.send_keys("not valid json").await?;
    driver
        .find_css("[data-import-word-sets]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-component='word-set-panel'] [role='alert']")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(
        zipeek_e2e::page_objects::browser::saved_word_sets(driver).await?,
        saved_sets,
        "Importing invalid JSON must preserve the existing saved vocabulary, not just its count"
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_css("[data-database-state='ready']")
            .await?
            .is_displayed()
            .await?,
        "Personal Word Set edits must not invalidate shared dictionary availability"
    );
    ctx.open_route("/browse").await?;
    ctx.checkpoint_visible(
        "word-set-navigation-restored",
        "[data-component='browse-page']",
    )
    .await?;
    ctx.checkpoint("word-sets-verified-result").await?;
    Ok(())
}

pub async fn dictionary_search_is_independent_of_saved_word_sets(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    driver
        .find_css("[data-new-word-set]")
        .await?
        .send_keys("Separate collection")
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
    driver
        .query_single("[data-component='browse-search-form']")
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-browse-word-set]")
            .await?
            .is_empty()
    );
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-target-word-set]")
            .await?
            .is_empty()
    );
    driver
        .query_single("[data-browse-filter-autocomplete]")
        .await?;
    Ok(())
}

pub async fn deleting_personal_set_persists_without_touching_dictionary(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/word-sets").await?;
    driver
        .find_css("[data-new-word-set]")
        .await?
        .send_keys("Removal guard")
        .await?;
    driver
        .find_css("[data-create-word-set]")
        .await?
        .click()
        .await?;
    driver
        .query_single(selectors::word_sets::SELECTABLE_OPTION)
        .await?;
    driver.click(selectors::word_sets::DELETE).await?;
    driver
        .click_with_text(
            selectors::word_sets::DELETE,
            selectors::word_sets::CONFIRM_DELETION,
        )
        .await?;
    driver.refresh().await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-target-word-set][data-selected-word-set-id]:not([data-selected-word-set-id=''])")
            .await?
            .is_empty()
    );
    zipeek_e2e::e2e_ensure!(
        zipeek_e2e::page_objects::browser::saved_word_sets(driver)
            .await?
            .is_empty()
    );
    ctx.open_route("/browse").await?;
    driver
        .query_single("[data-component='browse-page']")
        .await?;
    ctx.checkpoint_visible(
        "word-set-deletion-persisted",
        "[data-component='browse-page']",
    )
    .await?;
    Ok(())
}
