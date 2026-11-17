use super::standard;
use super::{FullRunConfig, FullRunKind, FullRunPlan, capture_journey_step};
use crate::page_objects::WebDriverActions;
use crate::{HarnessResult, ScenarioContext, page_objects};
use thirtyfour::prelude::*;

pub async fn run_journey(ctx: ScenarioContext, config: FullRunConfig) -> HarnessResult<()> {
    let variant = config.kind.as_str();
    let driver = &ctx.driver;
    page_objects::browser::remove_local_storage(driver, "zipeek.preferences.v1").await?;
    driver.goto(&ctx.base_url).await?;

    let landing = driver
        .query_single("[data-component='welcome-landing']")
        .await?;
    if !landing.is_displayed().await? {
        return Err("Full run could not start from a fresh landing page".into());
    }
    capture_journey_step(&ctx, "01-landing", config.seed, variant).await?;

    driver
        .query_single("[data-welcome-enter]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-onboarding-form]").await?;
    let themes = driver.find_all_css("[data-onboarding-theme]").await?;
    if themes.is_empty() {
        return Err("No onboarding themes available for seeded selection".into());
    }
    let theme = &themes[config.theme_index(themes.len())];
    let theme_key = theme
        .attr("data-onboarding-theme")
        .await?
        .ok_or("Theme has no key")?;
    theme.click().await?;
    capture_journey_step(&ctx, "02-onboarding", config.seed, variant).await?;
    driver
        .query_single("[data-onboarding-finish]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-component='home-page']").await?;
    let saved = page_objects::browser::local_storage(driver, "zipeek.preferences.v1")
        .await?
        .ok_or("Onboarding did not persist preferences")?;
    let persisted: serde_json::Value = serde_json::from_str(&saved)
        .map_err(|error| format!("Invalid saved onboarding preferences: {error}"))?;
    if persisted["preset"] != theme_key {
        return Err(format!("Onboarding persistence mismatch: {persisted}").into());
    }
    capture_journey_step(&ctx, "03-dashboard", config.seed, variant).await?;

    for (index, step) in FullRunPlan::steps(config.kind).iter().enumerate().skip(3) {
        let route = step
            .route
            .ok_or("Journey step unexpectedly lacks a route")?;
        ctx.open_route(route).await?;
        driver.query_single(step.selector).await?;
        let label = format!("{:02}-{}", index + 1, step.name);
        capture_journey_step(&ctx, &label, config.seed, variant).await?;
        if config.kind == FullRunKind::Standard {
            match step.name {
                "practice-first" => {
                    standard::exercise_five_scored_rounds(&ctx, config.seed).await?
                }
                "wordsets" => standard::interact_word_sets(&ctx, config.seed).await?,
                "dictionary" => standard::interact_dictionary(&ctx, config.seed).await?,
                "hanzi-writer" => standard::interact_hanzi_writer(&ctx, config.seed).await?,
                "games" => standard::interact_games(&ctx, config.seed).await?,
                "settings" => standard::interact_settings(&ctx, config.seed).await?,
                "courses" => standard::interact_courses(&ctx, config.seed).await?,
                "practice-final" => standard::final_provider_exercises(&ctx, config.seed).await?,
                _ => {}
            }
        }
        if step.name == "settings" {
            driver
                .query_single("[data-settings-tab-button='learning']")
                .await?
                .click()
                .await?;
            driver.query_single("[data-settings-learning]").await?;
            capture_journey_step(&ctx, "settings-learning-data", config.seed, variant).await?;
        }
    }

    match config.kind {
        FullRunKind::Standard => {
            standard::assert_no_recognition_opfs_evidence(&ctx, config.seed).await?;
            std::fs::write(
                ctx.artifacts.join("full-run-result.txt"),
                "PASS: scored mixed practice, Province Quest, settings persistence, fixture-backed AI practice, and empty OPFS in fresh profile verified. Live AI provider connectivity not assessed.",
            )?;
            Ok(())
        }
        FullRunKind::Pedagogical => {
            let prerequisite = "BLOCKED: onboarding opt-in does not enable evidence writes. No verified DuckDB readback, authenticated scored-event producer, test-clock injection into browser learning evidence, or Golden Dragon award implementation exists. Cannot simulate a 7-day award or pass this journey without those production gates.";
            std::fs::write(
                ctx.artifacts.join("full-run-prerequisites.txt"),
                prerequisite,
            )?;
            Err(prerequisite.into())
        }
    }
}
