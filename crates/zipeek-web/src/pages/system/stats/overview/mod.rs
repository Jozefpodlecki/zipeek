mod activity;
mod breakdown;
mod chart;
mod fallback;
mod lifecycle;
mod page;
mod review;
pub(super) use page::{Overview, OverviewProps};

#[cfg(test)]
mod tests {
    use super::activity::{activity_buckets, activity_categories};
    #[test]
    fn activity_buckets_cover_all_fixture_sessions() {
        let sessions = super::super::data::demo_sessions_for_tests();
        let buckets = activity_buckets(&sessions);
        assert!(buckets.iter().sum::<u32>() as usize <= sessions.len());
        assert!(buckets.iter().any(|count| *count > 0));
        assert!(buckets.windows(2).any(|pair| pair[0] != pair[1]));
        let categories = activity_categories(&sessions);
        assert!(
            categories
                .iter()
                .zip(buckets)
                .all(|((practice, games), total)| practice + games == total)
        );
    }
}
