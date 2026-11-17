use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub async fn set_local_storage(driver: &WebDriver, key: &str, value: &str) -> WebDriverResult<()> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_SET_LOCAL_STORAGE_01,
            vec![key.into(), value.into()],
        )
        .await?;
    Ok(())
}

pub async fn seed_default_preferences(driver: &WebDriver) -> WebDriverResult<()> {
    set_local_storage(
        driver,
        "zipeek.preferences.v1",
        include_str!("../../fixtures/preferences-default.json"),
    )
    .await
}

pub async fn remove_local_storage(driver: &WebDriver, key: &str) -> WebDriverResult<()> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_REMOVE_LOCAL_STORAGE_01,
            vec![key.into()],
        )
        .await?;
    Ok(())
}

pub async fn local_storage(driver: &WebDriver, key: &str) -> WebDriverResult<Option<String>> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_LOCAL_STORAGE_01,
            vec![key.into()],
        )
        .await?
        .convert::<Option<String>>()
}

pub async fn focused_element_has_attribute(
    driver: &WebDriver,
    attribute: &str,
) -> WebDriverResult<bool> {
    Ok(driver
        .active_element()
        .await?
        .attr(attribute)
        .await?
        .is_some())
}

pub async fn focused_element_has_id(driver: &WebDriver, id: &str) -> WebDriverResult<bool> {
    Ok(driver.active_element().await?.attr("id").await?.as_deref() == Some(id))
}

pub async fn no_horizontal_overflow(driver: &WebDriver) -> WebDriverResult<bool> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_NO_HORIZONTAL_OVERFLOW_01,
            vec![],
        )
        .await?
        .convert::<bool>()
}

pub async fn navigate_hash(driver: &WebDriver, route: &str) -> crate::HarnessResult<()> {
    let target = crate::harness::AppRoute::from_deep_link(route)?.hash_path();
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_NAVIGATE_HASH_01,
            vec![format!("#{target}").into()],
        )
        .await?;
    wait_for_hash_route(driver, &target).await?;
    Ok(())
}

pub async fn element_inside_viewport(driver: &WebDriver, selector: &str) -> WebDriverResult<bool> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_ELEMENT_INSIDE_VIEWPORT_01,
            vec![selector.into()],
        )
        .await?
        .convert::<bool>()
}

pub async fn saved_word_sets(driver: &WebDriver) -> crate::HarnessResult<Vec<serde_json::Value>> {
    let raw = local_storage(driver, "zipeek.user_word_sets.v1")
        .await?
        .ok_or("Word Sets were not saved to browser storage")?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("Saved Word Sets contain invalid JSON: {error}").into())
}

pub async fn horizontal_overflow_pixels(driver: &WebDriver) -> WebDriverResult<i64> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_HORIZONTAL_OVERFLOW_PIXELS_01,
            vec![],
        )
        .await?
        .convert::<i64>()
}

pub async fn set_preference(
    driver: &WebDriver,
    field: &str,
    value: serde_json::Value,
) -> WebDriverResult<()> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_SET_PREFERENCE_01,
            vec![field.into(), value],
        )
        .await?;
    Ok(())
}

pub async fn preference(
    driver: &WebDriver,
    field: &str,
) -> WebDriverResult<Option<serde_json::Value>> {
    driver
        .execute(
            crate::js_scripts::page_objects::BROWSER_PREFERENCE_01,
            vec![field.into()],
        )
        .await?
        .convert::<Option<serde_json::Value>>()
}

pub async fn dictionary_row_count(driver: &WebDriver) -> crate::HarnessResult<u32> {
    let element = driver
        .query_named(
            "[data-database-state='ready'][data-database-rows]",
            "Browser - dictionary row count",
        )
        .await?;
    let value = element
        .attr("data-database-rows")
        .await?
        .ok_or_else(|| "Missing dictionary row-count attribute".to_owned())?;
    let count = value
        .parse::<u32>()
        .map_err(|error| format!("Invalid dictionary row-count attribute: {error}"))?;
    if count == 0 {
        return Err("Ready dictionary reported zero words".into());
    }
    Ok(count)
}

pub async fn wait_for_hash_route(driver: &WebDriver, expected: &str) -> crate::HarnessResult<()> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let actual = driver
            .execute(
                crate::js_scripts::page_objects::BROWSER_WAIT_FOR_HASH_ROUTE_01,
                vec![],
            )
            .await?
            .convert::<String>()?;
        if actual.trim_start_matches('#') == expected {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(
                format!("Route did not settle on {expected:?}, last hash was {actual:?}").into(),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub async fn persisted_json(
    driver: &WebDriver,
    key: &str,
) -> crate::HarnessResult<Option<serde_json::Value>> {
    let raw = local_storage(driver, key).await?;
    raw.map(|text| {
        serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|error| format!("Invalid JSON under localStorage key {key}: {error}").into())
    })
    .transpose()
}
