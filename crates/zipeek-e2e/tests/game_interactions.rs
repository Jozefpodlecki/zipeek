use zipeek_e2e::page_objects::WebDriverActions;
#[path = "cloud_match_scripts.rs"]
mod cloud_match_scripts;
use thirtyfour::prelude::*;
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects::games};

fn cloud_travel_is_continuous(distance: f64, elapsed_ms: f64, velocity: f64) -> bool {
    let maximum_travel = (velocity + 175.0) * (elapsed_ms / 1000.0) + 12.0;
    elapsed_ms.is_finite()
        && elapsed_ms >= 0.0
        && velocity.is_finite()
        && velocity > 0.0
        && distance.is_finite()
        && distance >= 0.0
        && distance <= maximum_travel
}

#[test]
fn cloud_motion_threshold_accounts_for_real_elapsed_time_and_entry_acceleration() {
    assert!(cloud_travel_is_continuous(27.8, 150.0, 45.0));
    assert!(cloud_travel_is_continuous(12.0, 200.0, 38.0));
    assert!(!cloud_travel_is_continuous(100.0, 150.0, 45.0));
    assert!(!cloud_travel_is_continuous(-1.0, 150.0, 45.0));
    assert!(!cloud_travel_is_continuous(f64::NAN, 150.0, 45.0));
    assert!(!cloud_travel_is_continuous(10.0, f64::NAN, 45.0));
    assert!(cloud_travel_is_continuous(12.0, 0.0, 45.0));
    assert!(!cloud_travel_is_continuous(12.01, 0.0, 45.0));
    assert!(cloud_travel_is_continuous(45.0, 150.0, 45.0));
    assert!(!cloud_travel_is_continuous(45.01, 150.0, 45.0));
    assert!(!cloud_travel_is_continuous(0.0, -1.0, 45.0));
    assert!(!cloud_travel_is_continuous(0.0, f64::INFINITY, 45.0));
    assert!(!cloud_travel_is_continuous(0.0, 150.0, 0.0));
    assert!(!cloud_travel_is_continuous(0.0, 150.0, -45.0));
    assert!(!cloud_travel_is_continuous(0.0, 150.0, f64::INFINITY));
    assert!(!cloud_travel_is_continuous(0.0, 150.0, f64::NAN));
    assert!(!cloud_travel_is_continuous(f64::INFINITY, 150.0, 45.0));
}

#[test]
fn cloud_motion_bound_is_monotonic_in_elapsed_time_and_velocity() {
    for elapsed_ms in [0.0, 25.0, 150.0, 500.0] {
        for velocity in [38.0, 45.0, 56.0] {
            let limit = (velocity + 175.0) * (elapsed_ms / 1000.0) + 12.0;
            assert!(cloud_travel_is_continuous(limit, elapsed_ms, velocity));
            assert!(!cloud_travel_is_continuous(
                limit + 0.01,
                elapsed_ms,
                velocity
            ));
            assert!(cloud_travel_is_continuous(
                limit,
                elapsed_ms + 1.0,
                velocity
            ));
            assert!(cloud_travel_is_continuous(
                limit,
                elapsed_ms,
                velocity + 1.0
            ));
        }
    }
}

async fn crossword_setup_requires_selection(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/crosswords").await?;
    let setup = driver
        .query_single(zipeek_e2e::page_objects::crossword::SETUP)
        .await?;
    zipeek_e2e::e2e_ensure!(setup.is_displayed().await?);
    let bands = driver.find_all_css(games::CROSSWORD_BANDS).await?;
    zipeek_e2e::e2e_ensure_eq!(
        bands.len(),
        13,
        "Crossword setup must expose six HSK 2.0 and seven HSK 3.0 bands"
    );
    for band in bands {
        zipeek_e2e::e2e_ensure!(matches!(
            band.attr("aria-checked").await?.as_deref(),
            Some("true" | "false")
        ));
    }
    let start = driver.query_single(games::CROSSWORD_START).await?;
    zipeek_e2e::e2e_ensure!(
        start.is_enabled().await?,
        "The default HSK 3.0 level 1 band should be playable"
    );
    games::choose_crossword_band(driver, 3, 1).await?;
    zipeek_e2e::e2e_ensure!(
        !driver
            .query_single(games::CROSSWORD_START)
            .await?
            .is_enabled()
            .await?,
        "Removing the only selected band must disable Start"
    );
    games::choose_crossword_band(driver, 2, 1).await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single(games::CROSSWORD_START)
            .await?
            .is_enabled()
            .await?,
        "Choosing a new band must re-enable Start"
    );
    ctx.checkpoint("crossword-setup-validation").await?;
    Ok(())
}

async fn crossword_generation_exposes_playable_board(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/crosswords").await?;
    zipeek_e2e::page_objects::crossword::open_setup(driver).await?;
    zipeek_e2e::page_objects::crossword::start(driver).await?;
    zipeek_e2e::e2e_ensure!(
        !zipeek_e2e::page_objects::crossword::score(driver)
            .await?
            .is_empty()
    );
    let board_layout = driver
        .execute(
            zipeek_e2e::js_scripts::games::CROSSWORD_TAILWIND_BOARD_LAYOUT,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        board_layout,
        "Crossword cell alignment must be provided by Tailwind utilities"
    );
    ctx.checkpoint("crossword-generated-board").await?;
    Ok(())
}

async fn cloud_match_configuration_and_navigation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/cloud-match").await?;
    driver.query_single(games::CLOUD).await?;
    driver.query_single(games::CLOUD_BACK).await?;
    let labels = ["Calm", "Normal", "Storm"];
    for label in labels {
        games::choose_cloud_difficulty(driver, label).await?;
        let selected = driver
            .query(By::XPath(&format!(
                "//main[@data-component='cloud-match']//button[normalize-space()='{label}']"
            )))
            .single()
            .await?;
        let styles = selected.attr("class").await?.unwrap_or_default();
        zipeek_e2e::e2e_ensure!(
            styles.contains("bg-brand-action"),
            "Choosing {label} did not update the Cloud Match difficulty state: {styles}"
        );
    }
    for (group, choices) in [
        (
            "data-cloud-match-game-length",
            ["10", "20", "30", "Endless"],
        ),
        ("data-cloud-match-life-limit", ["1", "2", "3", "Endless"]),
    ] {
        for choice in choices {
            let selected = driver
                .query(By::XPath(&format!(
                    "//*[@{}]//button[normalize-space()='{}']",
                    group, choice
                )))
                .single()
                .await?;
            selected.click().await?;
            zipeek_e2e::e2e_ensure_eq!(
                selected.attr("aria-pressed").await?.as_deref(),
                Some("true"),
                "Cloud Match choice should remain selected"
            );
        }
    }
    ctx.checkpoint("cloud-match-configurable-game-length-and-lives")
        .await?;
    let back_link = driver.query_single(games::CLOUD_BACK).await?;
    driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_CONFIGURATION_AND_NAVIGATION_01,
        vec![],
    ).await?;
    let link_unobstructed = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_CONFIGURATION_AND_NAVIGATION_02,
        vec![],
    ).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        link_unobstructed,
        "All Games navigation must be unobstructed by the sticky app header"
    );
    back_link.click().await?;
    driver
        .query_single("[data-component='games-gallery']")
        .await?;
    let links = driver
        .find_all_css("[data-component='games-gallery'] section[aria-label='Available games'] a")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(links.len(), 6);
    driver.query_single(games::PROVINCE_FEATURED_CARD).await?;
    driver
        .query_single("button[aria-label='Next games page']")
        .await?
        .click()
        .await?;
    let second_page = driver
        .find_all_css("[data-component='games-gallery'] section[aria-label='Available games'] a")
        .await?;
    zipeek_e2e::e2e_ensure_eq!(second_page.len(), 5);
    driver.query_single(games::SENTENCE_FEATURED_CARD).await?;
    ctx.checkpoint("cloud-match-return-to-games").await?;
    Ok(())
}

async fn cloud_match_motion_remains_continuous(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/cloud-match").await?;
    let start = driver
        .query_single("[data-cloud-match-setup] [data-cloud-match-start]")
        .await?;
    for _ in 0..60 {
        if start.is_enabled().await? {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    zipeek_e2e::e2e_ensure!(
        start.is_enabled().await?,
        "Cloud Match vocabulary did not become playable"
    );
    let hint = driver
        .query_single("[data-cloud-match-hint-option] input[type='checkbox']")
        .await?;
    hint.click().await?;
    start.click().await?;
    let mixed_sides = driver
        .execute(cloud_match_scripts::MIXED_CLOUD_SIDES, vec![])
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        mixed_sides,
        "Both columns must contain Hanzi and meaning clouds"
    );
    let clouds = driver.find_all_css("[data-cloud-side='left']").await?;
    zipeek_e2e::e2e_ensure!(
        clouds.len() >= 2,
        "Cloud Match must render at least two left clouds"
    );
    for _ in 0..40 {
        let initialized = driver.execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_01,
            vec![],
        ).await?.convert::<bool>()?;
        if initialized {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let starts_offscreen = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_02,
        vec![],
    ).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        starts_offscreen,
        "New clouds must enter from off-screen, not spawn in the middle"
    );
    let mut visible_after_entry = 0;
    for _ in 0..80 {
        visible_after_entry = driver.execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_03,
            vec![],
        ).await?.convert::<usize>()?;
        if visible_after_entry >= 4 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(125)).await;
    }
    zipeek_e2e::e2e_ensure!(
        visible_after_entry >= 4,
        "Clouds must become accessible quickly after entering from the edges: {visible_after_entry} visible",
    );
    let sky_ready = driver
        .execute(cloud_match_scripts::ACTIVE_SKY_SCENE, vec![])
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        sky_ready,
        "Exactly one of the five Cloud Match scenic layers must be active"
    );
    let velocities = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_04,
        vec![],
    ).await?.convert::<Vec<f64>>()?;
    zipeek_e2e::e2e_ensure!(
        velocities.len() == 6 && velocities.iter().all(|v| *v >= 38.0 && *v <= 56.0),
        "Normal difficulty must assign each of the six clouds a speed in its approved band: {velocities:?}"
    );
    zipeek_e2e::e2e_ensure!(
        velocities.iter().any(|v| (v - velocities[0]).abs() > 1.0),
        "Clouds must not all share the same velocity"
    );
    let first_id = clouds[0]
        .attr("data-cloud-pair-id")
        .await?
        .unwrap_or_default();
    let observed_id = clouds[1]
        .attr("data-cloud-pair-id")
        .await?
        .unwrap_or_default();
    let observed = format!("[data-cloud-side='left'][data-cloud-pair-id='{observed_id}']");
    for _ in 0..40 {
        let initialized = driver
            .execute(
                zipeek_e2e::js_scripts::games::HAS_CLOUD_MOTION_PROGRESS,
                vec![observed.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if initialized {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .execute(
                zipeek_e2e::js_scripts::games::HAS_CLOUD_MOTION_PROGRESS,
                vec![observed.clone().into()],
            )
            .await?
            .convert::<bool>()?,
        "Cloud Match's frame controller failed to initialize the observed cloud",
    );
    driver
        .execute(
            zipeek_e2e::js_scripts::games::REMEMBER_STABLE_CLOUD,
            vec![observed.clone().into()],
        )
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::games::CAPTURE_CLOUD_MOTION_BASELINE,
            vec![observed.clone().into()],
        )
        .await?;
    driver
        .execute(
            zipeek_e2e::js_scripts::games::CLICK_CLOUD_BY_SIDE_AND_PAIR,
            vec!["left".into(), first_id.clone().into()],
        )
        .await?;
    let partner_glows = driver
        .execute(
            zipeek_e2e::js_scripts::games::PARTNER_CLOUD_HINT_VISIBLE,
            vec![first_id.clone().into()],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        partner_glows,
        "Selected cloud must illuminate its correct partner without completing a match"
    );
    driver
        .execute(
            zipeek_e2e::js_scripts::games::CLICK_CLOUD_BY_SIDE_AND_PAIR,
            vec!["right".into(), first_id.clone().into()],
        )
        .await?;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let matched_fade = driver
        .execute(cloud_match_scripts::MATCH_FADE_ACCESSIBLE, vec![])
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        matched_fade,
        "Correctly matched clouds must briefly glow and fade before disappearing"
    );
    let matched_is_inactive = driver
        .execute(
            zipeek_e2e::js_scripts::games::MATCHED_CLOUDS_HIDDEN,
            vec![first_id.clone().into()],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        matched_is_inactive,
        "Matched clouds must remain keyed in the wave but become hidden and disabled",
    );
    let motion_sample = driver
        .execute(
            zipeek_e2e::js_scripts::games::SAMPLE_CLOUD_MOTION,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<Vec<f64>>()?;
    let visible_after_match = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_05,
        vec![],
    ).await?.convert::<usize>()?;
    zipeek_e2e::e2e_ensure!(
        visible_after_match >= 2,
        "Matching another pair must not leave an almost empty cloud field: {visible_after_match} visible",
    );
    let same_node = driver
        .execute(
            zipeek_e2e::js_scripts::games::IS_STABLE_CLOUD_NODE,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        same_node,
        "A remaining cloud was replaced by a new DOM node after matching a different pair"
    );
    zipeek_e2e::e2e_ensure_eq!(
        motion_sample.len(),
        4,
        "Cloud motion sample must include both positions, elapsed time and velocity"
    );
    let baseline = motion_sample[0];
    let after = motion_sample[1];
    let elapsed_ms = motion_sample[2];
    let velocity = motion_sample[3];
    let distance_travelled = after - baseline;
    zipeek_e2e::e2e_ensure!(
        cloud_travel_is_continuous(distance_travelled, elapsed_ms, velocity),
        "An unmatched cloud must move within its velocity and entry-acceleration limit: before={baseline}, after={after}, distance={distance_travelled}, elapsed_ms={elapsed_ms}, velocity={velocity}",
    );
    driver
        .query_single("[data-component='cloud-match-hud'] button[aria-label='Pause game']")
        .await?
        .click()
        .await?;
    let pause_dialog = driver
        .query_single("[data-cloud-match-pause][role='dialog']")
        .await?;
    zipeek_e2e::e2e_ensure!(pause_dialog.is_displayed().await?);
    let paused_before = driver
        .execute(
            zipeek_e2e::js_scripts::games::CLOUD_MOTION_PROGRESS,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<f64>()?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let paused_after = driver
        .execute(
            zipeek_e2e::js_scripts::games::CLOUD_MOTION_PROGRESS,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<f64>()?;
    zipeek_e2e::e2e_ensure!(
        (paused_after - paused_before).abs() < 0.1,
        "Cloud movement should stop while the game is paused: {paused_before}, {paused_after}"
    );
    let focused_resume = driver
        .execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_06,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(focused_resume, "Pause dialog should focus Resume game");
    ctx.checkpoint("cloud-match-paused-progress").await?;
    driver
        .query_single("[data-cloud-match-pause] button")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .find_all_css("[data-cloud-match-pause]")
            .await?
            .is_empty(),
        "Resuming should close the pause dialog"
    );
    let label_fits = driver
        .execute(cloud_match_scripts::CLOUD_LABELS_FIT, vec![])
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(label_fits, "Cloud labels must remain inside their clouds");
    driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_07,
        vec![],
    ).await?;
    let wave_before = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_08,
        vec![],
    ).await?.convert::<String>()?;
    let remaining = driver
        .find_all_css("[data-cloud-side='left'][data-cloud-matched='false']")
        .await?;
    for cloud in remaining {
        let id = cloud.attr("data-cloud-pair-id").await?.unwrap_or_default();
        driver
            .execute(
                zipeek_e2e::js_scripts::games::CLICK_CLOUD_BY_SIDE_AND_PAIR,
                vec!["left".into(), id.clone().into()],
            )
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        driver
            .execute(
                zipeek_e2e::js_scripts::games::CLICK_CLOUD_BY_SIDE_AND_PAIR,
                vec!["right".into(), id.clone().into()],
            )
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }
    let mut new_wave_ready = false;
    for _ in 0..40 {
        new_wave_ready = driver
            .execute(
                zipeek_e2e::js_scripts::games::NEW_CLOUD_WAVE_READY,
                vec![wave_before.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if new_wave_ready {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    zipeek_e2e::e2e_ensure!(
        new_wave_ready,
        "Next wave must be initialized and positioned after completing the current wave"
    );
    let wave_enters_at_edges = driver.execute(
        zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_09,
        vec![],
    ).await?.convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        wave_enters_at_edges,
        "Next wave clouds must appear at the edges, never in the center"
    );
    let no_center_flash = driver
        .execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_MOTION_REMAINS_CONTINUOUS_10,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        no_center_flash,
        "New wave tiles must remain hidden until their initial edge positions are applied"
    );
    ctx.checkpoint("cloud-match-next-wave-edges").await?;
    ctx.checkpoint("cloud-match-motion-and-labels").await?;
    Ok(())
}

async fn cloud_match_replenishes_without_resetting_survivors(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/cloud-match").await?;
    driver
        .query(By::XPath(
            "//*[@data-cloud-match-spawn-mode]//button[normalize-space()='Continuous']",
        ))
        .single()
        .await?
        .click()
        .await?;
    let start = driver
        .query_single("[data-cloud-match-setup] [data-cloud-match-start]")
        .await?;
    for _ in 0..60 {
        if start.is_enabled().await? {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    zipeek_e2e::e2e_ensure!(
        start.is_enabled().await?,
        "Continuous game did not become playable"
    );
    start.click().await?;
    let initial = driver.find_all_css("[data-cloud-side='left']").await?;
    zipeek_e2e::e2e_ensure_eq!(initial.len(), 3, "Continuous game starts with three pairs");
    let mut initial_ids = Vec::new();
    for cloud in &initial {
        initial_ids.push(cloud.attr("data-cloud-pair-id").await?.unwrap_or_default());
    }
    let matched_id = initial[0]
        .attr("data-cloud-pair-id")
        .await?
        .unwrap_or_default();
    let survivor_id = initial[1]
        .attr("data-cloud-pair-id")
        .await?
        .unwrap_or_default();
    let observed = format!("[data-cloud-side='left'][data-cloud-pair-id='{survivor_id}']");
    for _ in 0..40 {
        let ready = driver
            .execute(
                zipeek_e2e::js_scripts::games::HAS_CLOUD_MOTION_PROGRESS,
                vec![observed.clone().into()],
            )
            .await?
            .convert::<bool>()?;
        if ready {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    driver
        .execute(
            zipeek_e2e::js_scripts::games::REMEMBER_ROLLING_SURVIVOR,
            vec![observed.clone().into()],
        )
        .await?;
    let before = driver
        .execute(
            zipeek_e2e::js_scripts::games::CLOUD_MOTION_PROGRESS,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<f64>()?;
    driver
        .execute(
            zipeek_e2e::js_scripts::games::CLICK_MATCHING_CLOUD_PAIR,
            vec![matched_id.clone().into()],
        )
        .await?;
    let mut replaced = false;
    for _ in 0..40 {
        replaced = driver
            .execute(
                zipeek_e2e::js_scripts::games::HAS_NEW_REPLACEMENT_CLOUD,
                vec![serde_json::json!(initial_ids)],
            )
            .await?
            .convert::<bool>()?;
        if replaced {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    }
    zipeek_e2e::e2e_ensure!(
        replaced,
        "A new pair must enter immediately after a correct match"
    );
    let stable = driver
        .execute(
            zipeek_e2e::js_scripts::games::IS_ROLLING_SURVIVOR_STABLE,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        stable,
        "Continuous replenishment replaced an unmatched cloud"
    );
    let after = driver
        .execute(
            zipeek_e2e::js_scripts::games::CLOUD_MOTION_PROGRESS,
            vec![observed.clone().into()],
        )
        .await?
        .convert::<f64>()?;
    zipeek_e2e::e2e_ensure!(
        after >= before && after - before < 30.0,
        "Continuous replenishment reset the surviving cloud: before={before}, after={after}"
    );
    let no_wave_reset = driver
        .execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_REPLENISHES_WITHOUT_RESETTING_SURVIVORS_01,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(
        no_wave_reset,
        "Continuous mode must not start a replacement wave"
    );
    ctx.checkpoint("cloud-match-continuous-replenishment")
        .await?;
    Ok(())
}

async fn tone_detective_scores_and_restarts(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/tone-detective").await?;
    driver
        .query_single("[data-component='tone-detective'] [data-tone-start]")
        .await?
        .click()
        .await?;
    for round in 0..12 {
        let listen = driver
            .query_single("[data-tone-question] [data-tone-listen]")
            .await?;
        if round == 0 {
            driver
                .execute(
                    zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_TONE_DETECTIVE_SCORES_AND_RESTARTS_01,
                    vec![],
                )
                .await?;
            listen.click().await?;
            let spoke = driver
                .execute(
                    zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_TONE_DETECTIVE_SCORES_AND_RESTARTS_02,
                    vec![],
                )
                .await?
                .convert::<bool>()?;
            zipeek_e2e::e2e_ensure!(spoke, "Silent speech must record Mandarin pronunciation");
        }
        driver
            .query_single("[data-tone-answer='1']")
            .await?
            .click()
            .await?;
        driver.query_single("[data-tone-feedback]").await?;
        zipeek_e2e::e2e_ensure!(
            !driver
                .query_single("[data-tone-answer='2']")
                .await?
                .is_enabled()
                .await?,
            "Answers must be locked after the first choice"
        );
        driver
            .query_single("[data-tone-next]")
            .await?
            .click()
            .await?;
    }
    driver
        .query_single("[data-tone-finished] [data-tone-score]")
        .await?;
    ctx.checkpoint("tone-detective-finished").await?;
    driver
        .query_single("[data-tone-replay]")
        .await?
        .click()
        .await?;
    driver.query_single("[data-tone-question]").await?;
    ctx.checkpoint("tone-detective-restarted").await?;
    Ok(())
}

async fn cloud_match_wrong_pair_feedback_is_temporary(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/cloud-match").await?;
    let start = driver
        .query_single("[data-cloud-match-setup] [data-cloud-match-start]")
        .await?;
    for _ in 0..60 {
        if start.is_enabled().await? {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    zipeek_e2e::e2e_ensure!(start.is_enabled().await?);
    start.click().await?;
    let chosen = driver
        .execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_01,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(chosen, "Two different clouds must be selectable");
    let mut marked = false;
    for _ in 0..15 {
        marked = driver
            .execute(
                zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_02,
                vec![],
            )
            .await?
            .convert::<bool>()?;
        if marked {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    zipeek_e2e::e2e_ensure!(
        marked,
        "Only the two tapped clouds should show incorrect feedback"
    );
    ctx.checkpoint("cloud-match-incorrect-feedback").await?;
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let cleared = driver
        .execute(
            zipeek_e2e::js_scripts::games::GAME_INTERACTIONS_CLOUD_MATCH_WRONG_PAIR_FEEDBACK_IS_TEMPORARY_03,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(cleared, "Incorrect glow must clear automatically");
    Ok(())
}

#[path = "mini_game_scenarios/classifier.rs"]
mod classifier_scenario;
#[path = "mini_game_scenarios/dialogue.rs"]
mod dialogue_scenario;
#[path = "mini_game_scenarios/market.rs"]
mod market_scenario;
#[path = "mini_game_scenarios/navigator.rs"]
mod navigator_scenario;
#[path = "mini_game_scenarios/radical.rs"]
mod radical_scenario;
#[path = "mini_game_scenarios/sentence.rs"]
mod sentence_scenario;
#[path = "mini_game_scenarios/vocabulary.rs"]
mod vocabulary_scenario;

async fn mini_game_state_isolation(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/games/classifier-market").await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-classifier-choice='2']")
        .await?
        .click()
        .await?;
    games::assert_mini_game_mistakes(driver, 1).await?;
    ctx.open_route("/games/listening-navigator").await?;
    driver
        .query_single("[data-game-setup='Listening Navigator']")
        .await?;
    driver
        .query_single("[data-game-start]")
        .await?
        .click()
        .await?;
    games::assert_mini_game_mistakes(driver, 0).await?;
    ctx.open_route("/games/classifier-market").await?;
    driver
        .query_single("[data-game-setup='Classifier Market']")
        .await?;
    ctx.checkpoint("mini-game-independent-state").await?;
    Ok(())
}

#[tokio::test]
async fn game_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "crossword-band-validation",
            crossword_setup_requires_selection,
        )
        .await?;
    harness
        .run(
            "crossword-generate-board",
            crossword_generation_exposes_playable_board,
        )
        .await?;
    harness
        .run(
            "cloud-match-setup-navigation",
            cloud_match_configuration_and_navigation,
        )
        .await?;
    harness
        .run(
            "cloud-match-motion-continuity",
            cloud_match_motion_remains_continuous,
        )
        .await?;
    harness
        .run(
            "cloud-match-continuous-replenishment",
            cloud_match_replenishes_without_resetting_survivors,
        )
        .await?;
    harness
        .run(
            "cloud-match-temporary-wrong-pair-feedback",
            cloud_match_wrong_pair_feedback_is_temporary,
        )
        .await?;
    harness
        .run(
            "tone-detective-play-and-restart",
            tone_detective_scores_and_restarts,
        )
        .await?;
    harness
        .run(
            "classifier-market-play-and-restart",
            classifier_scenario::play,
        )
        .await?;
    harness
        .run(
            "listening-navigator-play-and-finish",
            navigator_scenario::play,
        )
        .await?;
    harness
        .run("sentence-relay-ai-setup", sentence_scenario::play)
        .await?;
    harness
        .run("mini-game-independent-state", mini_game_state_isolation)
        .await?;
    harness
        .run(
            "mini-game-vocabulary-validation",
            vocabulary_scenario::validation,
        )
        .await?;
    harness
        .run("market-bargain-play-and-finish", market_scenario::play)
        .await?;
    harness
        .run("radical-forge-play-and-finish", radical_scenario::play)
        .await?;
    harness
        .run("dialogue-quest-ai-setup", dialogue_scenario::setup)
        .await?;
    harness.finish().await
}
