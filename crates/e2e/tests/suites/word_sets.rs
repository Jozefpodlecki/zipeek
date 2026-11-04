use e2e::{HarnessResult, ScenarioContext};
use thirtyfour::prelude::*;

pub async fn personal_sets_persist_across_navigation_and_backup_rejects_invalid_data(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/")).await?;
    driver
        .query(By::Css("[data-database-state='ready']"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-navigation-link][href='#/browse']"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-component='browse-page'] [data-new-word-set]"))
        .single()
        .await?
        .send_keys("E2E saved vocabulary")
        .await?;
    driver
        .find(By::Css("[data-create-word-set]"))
        .await?
        .click()
        .await?;
    assert!(driver
        .execute(
            "return JSON.parse(localStorage.getItem('zipeek.user_word_sets.v1')).some(s => s.name === 'E2E saved vocabulary' && s.word_ids.length === 0);",
            vec![],
        )
        .await?
        .convert::<bool>()?);

    driver
        .find(By::Css("[data-navigation-link][href='#/practice']"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-vocabulary-personal]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-component='practice-setup-page'] [data-vocabulary-set] option[value='0']"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-navigation-link][href='#/browse']"))
        .await?
        .click()
        .await?;
    driver.refresh().await?;
    driver
        .query(By::Css("[data-component='browse-page'] [data-target-word-set] option[value='0']"))
        .single()
        .await?;
    driver
        .find(By::Css("[data-component='word-set-backup'] summary"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-export-word-sets]"))
        .await?
        .click()
        .await?;
    assert!(driver
        .find(By::Css("[data-word-set-backup-json]"))
        .await?
        .attr("value")
        .await?
        .unwrap_or_default()
        .contains("E2E saved vocabulary")
        || driver
            .execute(
                "return document.querySelector('[data-word-set-backup-json]').value.includes('E2E saved vocabulary');",
                vec![],
            )
            .await?
            .convert::<bool>()?);

    let backup = driver
        .find(By::Css("[data-word-set-backup-json]"))
        .await?;
    backup.clear().await?;
    backup.send_keys("not valid json").await?;
    driver
        .find(By::Css("[data-import-word-sets]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-component='browse-page'] [role='alert']"))
        .single()
        .await?;
    assert!(driver
        .execute(
            "return JSON.parse(localStorage.getItem('zipeek.user_word_sets.v1')).length === 1;",
            vec![],
        )
        .await?
        .convert::<bool>()?);
    Ok(())
}

pub async fn browse_retry_keeps_invalid_personal_set_filter_blocked(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/#/browse")).await?;
    driver
        .query(By::Css("[data-component='browse-page'] [data-new-word-set]"))
        .single()
        .await?
        .send_keys("Retry guard")
        .await?;
    driver
        .find(By::Css("[data-create-word-set]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-browse-word-set] option[value='saved:0']"))
        .single()
        .await?;
    driver
        .execute(
            "const select = document.querySelector('[data-browse-word-set]'); const missing = document.createElement('option'); missing.value = 'saved:999'; missing.textContent = 'Missing set'; select.appendChild(missing); select.value = missing.value; select.dispatchEvent(new Event('change', { bubbles: true }));",
            vec![],
        )
        .await?;
    let error = driver
        .query(By::Css("[data-browse-error]"))
        .single()
        .await?;
    assert!(error.text().await?.contains("Selected Word Set no longer exists"));
    assert!(error.find_all(By::Css("button")).await?.is_empty());
    assert!(driver.find_all(By::Css("[data-browse-results]")).await?.is_empty());
    driver.find(By::Css("[data-browse-refresh]")).await?.click().await?;
    let error = driver
        .query(By::Css("[data-browse-error]"))
        .single()
        .await?;
    assert!(error.text().await?.contains("Selected Word Set no longer exists"));
    Ok(())
}

pub async fn deleting_personal_set_auto_refreshes(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    driver.goto(ctx.url("/#/browse")).await?;
    driver
        .query(By::Css("[data-component='browse-page'] [data-new-word-set]"))
        .single()
        .await?
        .send_keys("Removal guard")
        .await?;
    driver
        .find(By::Css("[data-create-word-set]"))
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-target-word-set] option[value='0']"))
        .single()
        .await?;
    driver
        .execute(
            "for (const [selector, value] of [['[data-browse-word-set]', 'saved:0'], ['[data-target-word-set]', '0']]) { const element = document.querySelector(selector); element.value = value; element.dispatchEvent(new Event('change', { bubbles: true })); }",
            vec![],
        )
        .await?;
    driver
        .query(By::Css("[data-browse-empty]"))
        .single()
        .await?;
    let delete = driver
        .query(By::Css("[data-component='word-set-manager'] [data-delete-word-set]"))
        .single()
        .await?;
    delete.click().await?;
    driver
        .query(By::Css("[data-delete-word-set]"))
        .single()
        .await?
        .click()
        .await?;
    driver
        .query(By::Css("[data-browse-results]"))
        .single()
        .await?;
    assert!(driver.find_all(By::Css("[data-browse-error]")).await?.is_empty());
    assert!(driver.execute(
        "return document.querySelector('[data-browse-word-set]').value === '';",
        vec![],
    ).await?.convert::<bool>()?);
    driver
        .find(By::Css("[data-new-word-set]"))
        .await?
        .send_keys("Backup guard")
        .await?;
    driver
        .find(By::Css("[data-create-word-set]"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-component='word-set-backup'] summary"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-export-word-sets]"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-import-word-sets]"))
        .await?
        .click()
        .await?;
    let error = driver
        .query(By::Css("[data-browse-error]"))
        .single()
        .await?;
    assert!(error.text().await?.contains("Word Sets changed"));
    assert!(error.find_all(By::Css("button")).await?.is_empty());
    assert!(driver.find_all(By::Css("[data-browse-results]")).await?.is_empty());
    Ok(())
}
