use rand::{Rng, SeedableRng, rngs::ChaCha12Rng};

pub struct SeededRandom(ChaCha12Rng);

impl SeededRandom {
    pub fn new(seed: u64) -> Self {
        Self(ChaCha12Rng::seed_from_u64(seed))
    }

    pub fn index(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        (self.0.next_u64() % upper as u64) as usize
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for index in (1..items.len()).rev() {
            let other = self.index(index + 1);
            items.swap(index, other);
        }
    }
}
