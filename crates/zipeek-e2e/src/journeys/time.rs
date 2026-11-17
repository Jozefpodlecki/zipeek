use std::time::{SystemTime, UNIX_EPOCH};

pub const DAY_MS: i64 = 86_400_000;

pub use zipeek_pedagogy::RecognitionClock as Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc_ms(&self) -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock predates Unix epoch")
            .as_millis();
        i64::try_from(millis).expect("system clock timestamp exceeds i64")
    }
}

pub struct SimulatedClock {
    now_utc_ms: i64,
}

impl SimulatedClock {
    pub fn new(initial_utc_ms: i64) -> Result<Self, &'static str> {
        if initial_utc_ms < 0 {
            return Err("simulated clock must start after Unix epoch");
        }
        Ok(Self {
            now_utc_ms: initial_utc_ms,
        })
    }

    pub fn advance_to(&mut self, next_utc_ms: i64) -> Result<(), &'static str> {
        if next_utc_ms <= self.now_utc_ms {
            return Err("simulated clock may only advance forward");
        }
        self.now_utc_ms = next_utc_ms;
        Ok(())
    }

    pub fn advance_days(&mut self, days: u32) -> Result<(), &'static str> {
        if days == 0 {
            return Err("simulated clock requires positive day increment");
        }
        let delta = i64::from(days)
            .checked_mul(DAY_MS)
            .ok_or("clock day overflow")?;
        let next = self.now_utc_ms.checked_add(delta).ok_or("clock overflow")?;
        self.advance_to(next)
    }
}

impl Clock for SimulatedClock {
    fn now_utc_ms(&self) -> i64 {
        self.now_utc_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtual_days_preserve_24_hour_spacing() {
        let mut clock = SimulatedClock::new(1_700_000_000_000).unwrap();
        let mut timestamps = vec![clock.now_utc_ms()];
        for _ in 1..7 {
            clock.advance_days(1).unwrap();
            timestamps.push(clock.now_utc_ms());
        }
        assert_eq!(timestamps.len(), 7);
        assert!(
            timestamps
                .windows(2)
                .all(|pair| pair[1] - pair[0] == DAY_MS)
        );
        assert!(clock.advance_to(timestamps[0]).is_err());
        assert!(clock.advance_days(0).is_err());
    }
}
