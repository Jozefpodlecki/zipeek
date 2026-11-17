const MAX_DURATION_MS: i32 = 30_000;
const SAMPLE_INTERVAL_MS: f64 = 50.0;
const BAR_COUNT: usize = 40;

mod capture;
mod error;
mod recorder;
mod state;
mod waveform;

pub use recorder::MicrophoneRecorder;

use capture::start_capture;
use error::describe_capture_error;
use state::{RecordingPhase, State};
use waveform::schedule_frame;
