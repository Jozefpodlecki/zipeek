use std::time::{Duration, Instant};

use crate::{
    HarnessResult,
    page_objects::{WebDriverActions, selectors::speaking},
};

use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
pub struct CaptureCounts {
    pub requests: u32,
    pub acquired: u32,
    pub stopped: u32,
    pub revoked: Vec<String>,
}

impl CaptureCounts {
    pub fn all_tracks_released(&self) -> bool {
        self.acquired > 0 && self.stopped == self.acquired
    }

    pub fn revoked_url(&self, url: &str) -> bool {
        !url.is_empty() && self.revoked.iter().any(|entry| entry == url)
    }
}

use thirtyfour::prelude::*;

pub async fn wait_for_recorder_phase(driver: &WebDriver, expected: &str) -> HarnessResult<()> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let recorder = driver.query_single(speaking::RECORDER).await?;
        let phase = recorder
            .attr("data-recorder-phase")
            .await?
            .unwrap_or_default();
        let alerts = driver.find_all_css(speaking::ALERT).await?;
        let message = if let Some(alert) = alerts.first() {
            alert.text().await?
        } else {
            String::new()
        };
        if phase == expected && (expected != "error" || !message.is_empty()) {
            return Ok(());
        }
        if phase == "error" && expected != "error" {
            return Err(
                format!("Recorder entered error while waiting for {expected}: {message}").into(),
            );
        }
        if Instant::now() >= deadline {
            let requests = microphone_requests(driver).await.unwrap_or(0);
            let action_count = recorder
                .attr("data-recorder-action-count")
                .await?
                .unwrap_or_default();
            let initialized = recorder
                .attr("data-recorder-initialized")
                .await?
                .unwrap_or_default();
            let status = driver
                .find_all_css("[data-component='speaking-recorder'] [role='status']")
                .await?;
            let status = if let Some(node) = status.first() {
                node.text().await?
            } else {
                String::new()
            };
            return Err(format!(
                "Recorder did not reach {expected} in 20s: phase={phase:?}, status={status:?}, alert={message:?}, microphone_requests={requests}, action_count={action_count:?}, initialized={initialized:?}"
            ).into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn speech_events(driver: &WebDriver) -> WebDriverResult<Vec<Value>> {
    driver
        .execute(crate::harness::browser_scripts::READ_SPEECH_EVENTS, vec![])
        .await?
        .convert::<Vec<Value>>()
}

pub fn has_mandarin_speak(events: &[Value]) -> bool {
    events.iter().any(|event| {
        event["event"] == "speak"
            && event["language"] == "zh-CN"
            && event["text"].as_str().is_some_and(|text| {
                !text.is_empty()
                    && text
                        .chars()
                        .any(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch))
            })
    })
}

pub fn mandarin_speaks(events: &[Value]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| {
            (event["event"] == "speak" && event["language"] == "zh-CN")
                .then(|| event["text"].as_str())
                .flatten()
        })
        .collect()
}

pub fn has_speech_cancel(events: &[Value]) -> bool {
    events.iter().any(|event| event["event"] == "cancel")
}

pub async fn microphone_requests(driver: &WebDriver) -> WebDriverResult<u64> {
    driver
        .execute(
            crate::js_scripts::practice::SPEAKING_MICROPHONE_REQUESTS,
            vec![],
        )
        .await?
        .convert::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_evidence_requires_acquisition_and_complete_cleanup() {
        let mut counts = CaptureCounts {
            requests: 1,
            acquired: 0,
            stopped: 0,
            revoked: vec!["blob:recording-one".to_owned()],
        };
        assert!(!counts.all_tracks_released());
        assert!(counts.revoked_url("blob:recording-one"));
        assert!(!counts.revoked_url(""));
        counts.acquired = 2;
        counts.stopped = 1;
        assert!(!counts.all_tracks_released());
        counts.stopped = 2;
        assert!(counts.all_tracks_released());
    }

    #[test]
    fn speech_evidence_requires_mandarin_text_and_language() {
        let valid = serde_json::json!({"event":"speak","language":"zh-CN","text":"你好"});
        assert!(has_mandarin_speak(&[valid]));
        for invalid in [
            serde_json::json!({"event":"cancel"}),
            serde_json::json!({"event":"speak","language":"en-US","text":"你好"}),
            serde_json::json!({"event":"speak","language":"zh-CN","text":"hello"}),
            serde_json::json!({"event":"speak","language":"zh-CN","text":""}),
        ] {
            assert!(!has_mandarin_speak(&[invalid]));
        }
        assert!(!has_mandarin_speak(&[]));
        assert_eq!(
            mandarin_speaks(&[
                serde_json::json!({"event":"cancel"}),
                serde_json::json!({"event":"speak","language":"zh-CN","text":"你好"}),
                serde_json::json!({"event":"speak","language":"en-US","text":"hello"}),
            ]),
            vec!["你好"]
        );
        assert!(has_speech_cancel(&[serde_json::json!({"event":"cancel"})]));
        assert!(!has_speech_cancel(&[]));
    }
}

pub async fn install_capture_probe(driver: &WebDriver) -> WebDriverResult<bool> {
    driver
        .execute(
            crate::js_scripts::practice::SPEAKING_TRACK_STOP_PROBE,
            vec![],
        )
        .await?
        .convert::<bool>()
}

pub async fn capture_counts(driver: &WebDriver) -> WebDriverResult<CaptureCounts> {
    driver
        .execute(
            crate::js_scripts::practice::SPEAKING_TRACK_STOP_COUNTS,
            vec![],
        )
        .await?
        .convert::<CaptureCounts>()
}
