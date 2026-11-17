use zipeek_e2e::{
    Harness, HarnessResult, ScenarioContext, SuiteKind,
    page_objects::{WebDriverActions, audio_probes, selectors::speaking as recording, speaking},
};

async fn speaking_targets_change_the_exercise(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/speaking").await?;
    let first_id = speaking::instance_id(driver).await?;
    let choices = driver.find_all_css(speaking::CHOICE).await?;
    zipeek_e2e::e2e_ensure!(
        choices.len() >= 2,
        "Speaking practice must offer at least two targets"
    );
    speaking::choose(driver, 1).await?;
    let second_id = speaking::instance_id(driver).await?;
    zipeek_e2e::e2e_ensure_ne!(
        first_id,
        second_id,
        "Choosing another target must change the exercise instance"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css(&format!("{}[aria-pressed='true']", speaking::CHOICE))
            .await?
            .len(),
        1
    );
    speaking::choose(driver, 0).await?;
    zipeek_e2e::e2e_ensure_eq!(
        speaking::instance_id(driver).await?,
        first_id,
        "Returning to the first pronunciation target must restore that target's identity"
    );
    zipeek_e2e::e2e_ensure_eq!(
        driver
            .find_all_css(&format!("{}[aria-pressed='true']", speaking::CHOICE))
            .await?
            .len(),
        1,
        "Switching pronunciation targets must leave one active choice"
    );
    ctx.checkpoint("speaking-target-change").await?;
    Ok(())
}

async fn speaking_selection_is_accessible(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/speaking").await?;
    let selector = driver.element(speaking::SELECTOR).await?;
    zipeek_e2e::e2e_ensure_eq!(
        selector.attr("aria-label").await?.as_deref(),
        Some("Choose a pronunciation practice sentence")
    );
    let options = driver.find_all_css(speaking::CHOICE).await?;
    zipeek_e2e::e2e_ensure!(!options.is_empty());
    for option in options {
        zipeek_e2e::e2e_ensure!(
            !option.text().await?.trim().is_empty(),
            "Speaking target must have a label"
        );
        zipeek_e2e::e2e_ensure!(matches!(
            option.attr("aria-pressed").await?.as_deref(),
            Some("true" | "false")
        ));
    }
    zipeek_e2e::e2e_ensure_eq!(
        driver.find_all_css(speaking::CONTINUE).await?.len(),
        0,
        "Standalone speaking practice must not display managed-session continuation"
    );
    Ok(())
}

async fn denied_microphone_can_retry_and_change_target(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/speaking").await?;
    driver
        .query_wait_first(
            recording::INITIALIZED,
            std::time::Duration::from_secs(10),
            std::time::Duration::from_millis(100),
        )
        .await?;
    let installed = driver
        .execute(
            zipeek_e2e::js_scripts::practice::SPEAKING_DENY_MICROPHONE,
            vec![],
        )
        .await?
        .convert::<bool>()?;
    zipeek_e2e::e2e_ensure!(installed, "Microphone permission fixture is unavailable");
    for attempt in 0..2 {
        driver.click(recording::START).await?;
        audio_probes::wait_for_recorder_phase(driver, "error").await?;
        zipeek_e2e::e2e_ensure!(
            driver.find_all_css(recording::PLAYBACK).await?.is_empty(),
            "Denied microphone attempt {attempt} must not create a playback element"
        );
    }
    zipeek_e2e::e2e_ensure_eq!(
        audio_probes::microphone_requests(driver).await?,
        2,
        "Both microphone attempts must reach the recording service"
    );
    speaking::choose(driver, 1).await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css(recording::ALERT).await?.is_empty(),
        "Changing the speaking target must clear the prior recorder error"
    );
    ctx.checkpoint("speaking-microphone-denied-retry").await?;
    Ok(())
}

async fn recording_capture_stop_playback_discard(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/practice/speaking").await?;
    driver
        .query_wait_first(
            recording::INITIALIZED,
            std::time::Duration::from_secs(10),
            std::time::Duration::from_millis(100),
        )
        .await?;
    zipeek_e2e::e2e_ensure!(
        audio_probes::install_capture_probe(driver).await?,
        "Chromium microphone fixture could not instrument the capture boundary"
    );
    driver.click(recording::START).await?;
    audio_probes::wait_for_recorder_phase(driver, "recording").await?;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    driver.click(recording::STOP).await?;
    audio_probes::wait_for_recorder_phase(driver, "ready").await?;
    let playback = driver.query_single(recording::PLAYBACK).await?;
    let url = playback.attr("src").await?.unwrap_or_default();
    zipeek_e2e::e2e_ensure!(
        url.starts_with("blob:"),
        "Successful recording must produce browser-owned playback media, got {url:?}"
    );
    let counts = audio_probes::capture_counts(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(
        counts.requests,
        1,
        "Recording must request the microphone once"
    );
    zipeek_e2e::e2e_ensure!(counts.acquired > 0, "Recording must acquire a media track");
    zipeek_e2e::e2e_ensure!(
        counts.all_tracks_released(),
        "Stopping a recording must release every acquired track: acquired={}, stopped={}",
        counts.acquired,
        counts.stopped
    );
    ctx.checkpoint("speaking-recording-ready").await?;
    driver.click(recording::DISCARD).await?;
    let discarded = audio_probes::capture_counts(driver).await?;
    zipeek_e2e::e2e_ensure!(
        discarded.revoked_url(&url),
        "Discard must revoke the original recording blob URL"
    );
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css(recording::PLAYBACK).await?.is_empty(),
        "Discarding must remove the playback media"
    );
    driver.click(recording::START).await?;
    audio_probes::wait_for_recorder_phase(driver, "recording").await?;
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    driver.click(recording::STOP).await?;
    audio_probes::wait_for_recorder_phase(driver, "ready").await?;
    let counts = audio_probes::capture_counts(driver).await?;
    zipeek_e2e::e2e_ensure_eq!(counts.requests, 2, "Retry must reacquire the microphone");
    zipeek_e2e::e2e_ensure!(
        counts.all_tracks_released(),
        "Retry must release every media track: acquired={}, stopped={}",
        counts.acquired,
        counts.stopped
    );
    let retry_url = driver
        .query_single(recording::PLAYBACK)
        .await?
        .attr("src")
        .await?
        .unwrap_or_default();
    speaking::choose(driver, 1).await?;
    driver.query_single(recording::RECORDER).await?;
    let after_target_change = audio_probes::capture_counts(driver).await?;
    zipeek_e2e::e2e_ensure!(
        after_target_change.revoked_url(&retry_url),
        "Changing the speaking target must revoke the previous playback URL"
    );
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css(recording::PLAYBACK).await?.is_empty(),
        "Changing the speaking target must remove the old playback element"
    );
    ctx.checkpoint("speaking-recording-retry").await?;
    Ok(())
}

#[tokio::test]
async fn speaking_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "speaking-target-selection",
            speaking_targets_change_the_exercise,
        )
        .await?;
    harness
        .run("speaking-accessibility", speaking_selection_is_accessible)
        .await?;
    harness
        .run(
            "speaking-microphone-denied-retry",
            denied_microphone_can_retry_and_change_target,
        )
        .await?;
    harness
        .run(
            "speaking-recording-stop-discard-retry",
            recording_capture_stop_playback_discard,
        )
        .await?;
    harness.finish().await
}
