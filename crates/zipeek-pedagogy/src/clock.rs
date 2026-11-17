use std::time::{SystemTime, UNIX_EPOCH};

use crate::{EvidenceError, RecognitionEvent, RecognitionProjection, project_recognition};

pub trait RecognitionClock {
    fn now_utc_ms(&self) -> i64;
}

pub struct SystemRecognitionClock;

impl RecognitionClock for SystemRecognitionClock {
    fn now_utc_ms(&self) -> i64 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock predates Unix epoch");
        i64::try_from(duration.as_millis()).expect("timestamp overflows i64")
    }
}

pub fn project_at_clock<C: RecognitionClock>(
    events: &[RecognitionEvent],
    clock: &C,
) -> Result<RecognitionProjection, EvidenceError> {
    project_recognition(events, clock.now_utc_ms())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FixedClock(i64);
    impl RecognitionClock for FixedClock {
        fn now_utc_ms(&self) -> i64 {
            self.0
        }
    }
    #[test]
    fn projection_uses_injected_time() {
        assert!(project_at_clock(&[], &FixedClock(1_700_000_000_000)).is_ok());
    }
}
