use std::rc::Rc;
use yew::Reducible;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RecordingPhase {
    #[default]
    Idle,
    Requesting,
    Recording,
    Processing,
    Ready,
    Error,
}

impl RecordingPhase {
    pub(super) fn from_bridge(value: &str) -> Self {
        match value {
            "idle" => Self::Idle,
            "requesting" => Self::Requesting,
            "recording" => Self::Recording,
            "processing" => Self::Processing,
            "ready" => Self::Ready,
            _ => Self::Error,
        }
    }

    pub(super) fn as_data_value(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Requesting => "requesting",
            Self::Recording => "recording",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }

    pub(super) fn is_busy(self) -> bool {
        matches!(self, Self::Requesting | Self::Processing)
    }

    pub(super) fn status(self) -> &'static str {
        match self {
            Self::Idle => "Microphone is off",
            Self::Requesting => "Waiting for microphone permission…",
            Self::Recording => "Recording speech",
            Self::Processing => "Preparing audio playback…",
            Self::Ready => "Recording ready for playback",
            Self::Error => "Microphone unavailable",
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct RecorderState {
    pub(super) levels: Vec<f32>,
    pub(super) phase: RecordingPhase,
    pub(super) audio_url: String,
    pub(super) error: String,
    pub(super) initialized: bool,
    pub(super) action_count: u32,
}

impl Default for RecorderState {
    fn default() -> Self {
        Self {
            levels: vec![0.0; 40],
            phase: RecordingPhase::default(),
            audio_url: String::new(),
            error: String::new(),
            initialized: false,
            action_count: 0,
        }
    }
}

pub(super) enum RecorderAction {
    Initialized,
    Disposed,
    LevelsChanged(Vec<f32>),
    PhaseChanged(RecordingPhase),
    RecordingFinished(String),
    Failed(String),
    StartRequested,
    Discarded,
}

impl Reducible for RecorderState {
    type Action = RecorderAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next = (*self).clone();
        match action {
            RecorderAction::Initialized => next.initialized = true,
            RecorderAction::Disposed => next.initialized = false,
            RecorderAction::LevelsChanged(levels) => {
                next.levels = levels
                    .into_iter()
                    .take(128)
                    .map(|level| {
                        if level.is_finite() {
                            level.clamp(0.0, 1.0)
                        } else {
                            0.0
                        }
                    })
                    .collect();
            }
            RecorderAction::PhaseChanged(phase) => next.phase = phase,
            RecorderAction::RecordingFinished(url) => next.audio_url = url,
            RecorderAction::Failed(message) => {
                next.phase = RecordingPhase::Error;
                next.error = message;
            }
            RecorderAction::StartRequested => {
                next.action_count = next.action_count.wrapping_add(1);
                next.audio_url.clear();
                next.error.clear();
            }
            RecorderAction::Discarded => {
                next.audio_url.clear();
                next.error.clear();
            }
        }
        Rc::new(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_levels_are_finite_bounded_and_limited() {
        let state = Rc::new(RecorderState::default());
        let adjusted = state.reduce(RecorderAction::LevelsChanged(vec![f32::NAN, -1.0, 3.0]));
        assert_eq!(adjusted.levels, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn starting_and_discarding_clear_previous_recording() {
        let original = Rc::new(RecorderState {
            audio_url: "blob:previous".into(),
            error: "previous error".into(),
            ..RecorderState::default()
        });
        let started = original.reduce(RecorderAction::StartRequested);
        assert!(started.audio_url.is_empty());
        assert!(started.error.is_empty());
        assert_eq!(started.action_count, 1);
        let finished = started.reduce(RecorderAction::RecordingFinished("blob:next".into()));
        assert_eq!(finished.audio_url, "blob:next");
        let discarded = finished.reduce(RecorderAction::Discarded);
        assert!(discarded.audio_url.is_empty());
    }

    #[test]
    fn initialization_and_disposal_toggle_readiness() {
        let original = Rc::new(RecorderState::default());
        let initialized = original.reduce(RecorderAction::Initialized);
        assert!(initialized.initialized);
        let disposed = initialized.reduce(RecorderAction::Disposed);
        assert!(!disposed.initialized);
    }

    #[test]
    fn bridge_phase_unknown_values_fail_closed() {
        assert_eq!(RecordingPhase::from_bridge("ready"), RecordingPhase::Ready);
        assert_eq!(
            RecordingPhase::from_bridge("unknown"),
            RecordingPhase::Error
        );
        assert_ne!(RecordingPhase::from_bridge("Ready"), RecordingPhase::Ready);
        for value in [
            "idle",
            "requesting",
            "recording",
            "processing",
            "ready",
            "error",
        ] {
            assert_eq!(RecordingPhase::from_bridge(value).as_data_value(), value);
        }
    }

    #[test]
    fn only_transitional_phases_are_busy() {
        for phase in [RecordingPhase::Requesting, RecordingPhase::Processing] {
            assert!(phase.is_busy());
        }
        for phase in [
            RecordingPhase::Idle,
            RecordingPhase::Recording,
            RecordingPhase::Ready,
            RecordingPhase::Error,
        ] {
            assert!(!phase.is_busy());
        }
    }
}
