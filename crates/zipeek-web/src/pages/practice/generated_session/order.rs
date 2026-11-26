pub(super) fn shuffled_indices(len: usize, seed: u64) -> Vec<usize> {
    let mut indices = (0..len).collect::<Vec<_>>();
    let mut state = seed | 1;
    for index in (1..indices.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        indices.swap(index, (state as usize) % (index + 1));
    }
    if indices.len() > 1
        && indices
            .iter()
            .enumerate()
            .all(|(index, tile)| *tile == index)
    {
        indices.rotate_left(1);
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::shuffled_indices;

    #[test]
    fn shuffled_indices_are_a_deterministic_permutation() {
        for len in 0..32 {
            let left = shuffled_indices(len, 42);
            let right = shuffled_indices(len, 42);
            assert_eq!(left, right);
            let mut sorted = left;
            sorted.sort_unstable();
            assert_eq!(sorted, (0..len).collect::<Vec<_>>());
        }
    }
}
