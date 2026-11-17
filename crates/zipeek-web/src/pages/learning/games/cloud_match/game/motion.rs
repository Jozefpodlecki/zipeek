use std::collections::{BTreeMap, BTreeSet};
use zipeek_exercises::cloud_match::{CloudSide, Difficulty};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CloudId {
    pub side: u8,
    pub pair: i32,
}

impl CloudId {
    pub fn new(side: CloudSide, pair: i32) -> Self {
        let side = match side {
            CloudSide::Left => 0,
            CloudSide::Right => 1,
        };
        Self { side, pair }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CloudMotion {
    pub variation: usize,
    pub top: usize,
    pub speed: f64,
}

impl CloudMotion {
    pub fn new(id: i32, side: CloudSide, lane: usize, wave: usize, difficulty: Difficulty) -> Self {
        let variant = id.unsigned_abs() as u64;
        let side_token = match side {
            CloudSide::Left => 0x9e37_79b9_u64,
            CloudSide::Right => 0x7f4a_7c15_u64,
        };
        let mut hash = variant
            .wrapping_mul(0xbf58_476d_1ce4_e5b9)
            .wrapping_add((wave as u64).wrapping_mul(0x94d0_49bb_1331_11eb))
            .wrapping_add(side_token)
            .wrapping_add(lane as u64);
        hash ^= hash >> 30;
        hash = hash.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        hash ^= hash >> 27;
        let variation = (hash % 12) as usize;
        let (min, max) = match difficulty {
            Difficulty::Calm => (24.0, 36.0),
            Difficulty::Normal => (38.0, 56.0),
            Difficulty::Storm => (58.0, 82.0),
        };
        let fraction = ((hash >> 8) % 10_000) as f64 / 10_000.0;
        let top = match side {
            CloudSide::Left => 12 + lane * 26,
            CloudSide::Right => 22 + lane * 25,
        };
        Self {
            variation,
            top,
            speed: min + (max - min) * fraction,
        }
    }

    pub fn initial_x(self, side: CloudSide, arena_width: f64, cloud_width: f64) -> f64 {
        match side {
            CloudSide::Left => -cloud_width - 12.0,
            CloudSide::Right => arena_width + 12.0,
        }
    }

    pub fn x_at(self, side: CloudSide, arena_width: f64, cloud_width: f64, travel: f64) -> f64 {
        let start = self.initial_x(side, arena_width, cloud_width);
        let cycle = arena_width + cloud_width + 24.0;
        let offset = travel.rem_euclid(cycle);
        match side {
            CloudSide::Left => start + offset,
            CloudSide::Right => start - offset,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MovingCloud {
    pub side: CloudSide,
    pub motion: CloudMotion,
    pub travel: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WavePhase {
    Preparing,
    Running,
}

#[derive(Debug)]
pub(super) struct CloudWorld {
    clouds: BTreeMap<CloudId, MovingCloud>,
    last_timestamp: Option<f64>,
    phase: WavePhase,
}

impl CloudWorld {
    pub fn new(entries: impl IntoIterator<Item = (CloudId, CloudSide, CloudMotion)>) -> Self {
        let clouds = entries
            .into_iter()
            .map(|(id, side, motion)| {
                (
                    id,
                    MovingCloud {
                        side,
                        motion,
                        travel: 0.0,
                    },
                )
            })
            .collect();
        Self {
            clouds,
            last_timestamp: None,
            phase: WavePhase::Preparing,
        }
    }

    pub fn activate(&mut self) {
        self.phase = WavePhase::Running;
    }

    pub fn phase(&self) -> WavePhase {
        self.phase
    }

    pub fn tick(&mut self, timestamp_ms: f64, paused: bool) {
        let Some(previous) = self.last_timestamp.replace(timestamp_ms) else {
            return;
        };
        if paused
            || self.phase == WavePhase::Preparing
            || !timestamp_ms.is_finite()
            || !previous.is_finite()
        {
            return;
        }
        let seconds = ((timestamp_ms - previous) / 1000.0).clamp(0.0, 0.05);
        for cloud in self.clouds.values_mut() {
            let entry = (1.0 - cloud.travel / 220.0).clamp(0.0, 1.0);
            cloud.travel += seconds * (cloud.motion.speed + 175.0 * entry);
        }
    }

    pub fn position(&self, id: CloudId, arena_width: f64, cloud_width: f64) -> Option<f64> {
        self.clouds.get(&id).map(|cloud| {
            cloud
                .motion
                .x_at(cloud.side, arena_width, cloud_width, cloud.travel)
        })
    }

    pub fn sync(&mut self, entries: impl IntoIterator<Item = (CloudId, CloudSide, CloudMotion)>) {
        let mut active = BTreeSet::new();
        for (id, side, motion) in entries {
            active.insert(id);
            self.clouds.entry(id).or_insert(MovingCloud {
                side,
                motion,
                travel: 0.0,
            });
        }
        self.clouds.retain(|id, _| active.contains(id));
    }

    pub fn retain(&mut self, visible: impl IntoIterator<Item = CloudId>) {
        let visible = visible.into_iter().collect::<BTreeSet<_>>();
        self.clouds.retain(|id, _| visible.contains(id));
    }

    pub fn ids(&self) -> impl Iterator<Item = CloudId> + '_ {
        self.clouds.keys().copied()
    }

    pub fn progress(&self, id: CloudId) -> Option<f64> {
        self.clouds.get(&id).map(|cloud| cloud.travel)
    }

    pub fn motion(&self, id: CloudId) -> Option<CloudMotion> {
        self.clouds.get(&id).map(|cloud| cloud.motion)
    }

    pub fn len(&self) -> usize {
        self.clouds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(pair: i32, side: CloudSide, lane: usize) -> (CloudId, CloudSide, CloudMotion) {
        (
            CloudId::new(side, pair),
            side,
            CloudMotion::new(pair, side, lane, 2, Difficulty::Normal),
        )
    }

    #[test]
    fn every_cloud_enters_at_its_own_screen_edge() {
        for wave in 0..8 {
            for side in [CloudSide::Left, CloudSide::Right] {
                for lane in 0..3 {
                    let motion =
                        CloudMotion::new(21 + lane as i32, side, lane, wave, Difficulty::Normal);
                    let x = motion.initial_x(side, 1600.0, 260.0);
                    match side {
                        CloudSide::Left => assert!(x + 260.0 < 0.0),
                        CloudSide::Right => assert!(x > 1600.0),
                    }
                }
            }
        }
    }

    #[test]
    fn velocities_are_stable_distinct_and_within_difficulty_bands() {
        for (difficulty, min, max) in [
            (Difficulty::Calm, 24.0, 36.0),
            (Difficulty::Normal, 38.0, 56.0),
            (Difficulty::Storm, 58.0, 82.0),
        ] {
            let speeds = (1..=30)
                .map(|pair| {
                    CloudMotion::new(pair, CloudSide::Left, pair as usize % 3, 1, difficulty).speed
                })
                .collect::<Vec<_>>();
            assert!(speeds.iter().all(|speed| *speed >= min && *speed <= max));
            assert!(speeds.iter().any(|speed| (speed - speeds[0]).abs() > 1.0));
            assert_eq!(
                speeds[0],
                CloudMotion::new(1, CloudSide::Left, 1, 1, difficulty).speed
            );
        }
    }

    #[test]
    fn synchronizing_continuous_roster_preserves_survivor_motion() {
        let first = entry(51, CloudSide::Left, 0);
        let survivor = entry(52, CloudSide::Left, 1);
        let replacement = entry(53, CloudSide::Left, 0);
        let mut world = CloudWorld::new([first, survivor]);
        world.activate();
        world.tick(1000.0, false);
        world.tick(1020.0, false);
        let before = world.progress(survivor.0).unwrap();
        world.sync([survivor, replacement]);
        assert_eq!(world.progress(survivor.0), Some(before));
        assert_eq!(world.progress(replacement.0), Some(0.0));
        assert_eq!(world.len(), 2);
        world.tick(1040.0, false);
        assert!(world.progress(survivor.0).unwrap() > before);
    }

    #[test]
    fn matched_pairs_do_not_move_unmatched_clouds() {
        let tracked = entry(47, CloudSide::Left, 1);
        let other = entry(48, CloudSide::Left, 0);
        let mut world = CloudWorld::new([tracked, other]);
        world.activate();
        world.tick(1000.0, false);
        world.tick(1016.0, false);
        let before = world.position(tracked.0, 1280.0, 250.0).unwrap();
        world.retain([tracked.0]);
        assert_eq!(world.position(tracked.0, 1280.0, 250.0), Some(before));
        world.tick(1032.0, false);
        let after = world.position(tracked.0, 1280.0, 250.0).unwrap();
        assert!(after > before && after - before < 4.0);
    }

    #[test]
    fn pause_and_resume_never_cause_a_position_jump() {
        let tracked = entry(47, CloudSide::Left, 1);
        let mut world = CloudWorld::new([tracked]);
        world.activate();
        world.tick(1000.0, false);
        world.tick(1016.0, false);
        let before = world.position(tracked.0, 1280.0, 250.0).unwrap();
        world.tick(60_000.0, true);
        world.tick(60_016.0, false);
        let after = world.position(tracked.0, 1280.0, 250.0).unwrap();
        assert!((after - before).abs() < 2.0);
    }

    #[test]
    fn preparing_wave_does_not_move_until_activation() {
        let tracked = entry(11, CloudSide::Left, 0);
        let mut world = CloudWorld::new([tracked]);
        assert_eq!(world.phase(), WavePhase::Preparing);
        world.tick(1000.0, false);
        world.tick(2000.0, false);
        assert_eq!(world.progress(tracked.0), Some(0.0));
        world.activate();
        world.tick(2016.0, false);
        assert!(world.progress(tracked.0).unwrap() > 0.0);
    }

    #[test]
    fn offscreen_wrap_is_continuous_until_after_the_exit() {
        let motion = entry(44, CloudSide::Left, 0).2;
        let arena = 1280.0;
        let width = 260.0;
        let cycle = arena + width + 24.0;
        let just_before = motion.x_at(CloudSide::Left, arena, width, cycle - 0.01);
        assert!(just_before >= arena);
        let wrapped = motion.x_at(CloudSide::Left, arena, width, cycle);
        assert!(wrapped < 0.0);
    }
}
