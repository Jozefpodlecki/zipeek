use crate::page_objects::WebDriverActions;
use thirtyfour::prelude::*;

pub const CATALOG: &str = "[data-component='courses-page']";
pub const PINYIN: &str = "[data-component='pinyin-course']";
pub const LESSON: &str = "[data-component='course-lesson']";

fn region_id_for_slug(slug: &str) -> &'static str {
    match slug {
        "tones" => "tones",
        "tone-patterns" => "tone-patterns",
        "initials" => "initials",
        "initial-contrasts" => "initial-contrasts",
        "finals" => "finals",
        "spelling-review" => "spelling-review",
        _ => panic!("Unknown pinyin slug: {slug}"),
    }
}

pub async fn open_lesson(driver: &WebDriver, slug: &str) -> crate::HarnessResult<()> {
    if !driver.is_present(PINYIN).await? {
        driver
            .query_named("a[href*='/courses/pinyin']", "Courses - open pinyin course")
            .await?
            .click()
            .await?;
        crate::page_objects::browser::wait_for_hash_route(driver, "/courses/pinyin").await?;
    }

    let region = region_id_for_slug(slug);
    driver
        .query_named(
            &format!("[data-pinyin-region='{region}']"),
            "Courses - select pinyin region",
        )
        .await?
        .click()
        .await?;
    driver
        .query_named(
            &format!("[data-pinyin-details] a[href*='/courses/pinyin/{slug}']"),
            "Courses - open lesson",
        )
        .await?
        .click()
        .await?;

    crate::page_objects::browser::wait_for_hash_route(driver, &format!("/courses/pinyin/{slug}"))
        .await?;
    driver.query_named(LESSON, "Courses - open lesson").await?;
    Ok(())
}
