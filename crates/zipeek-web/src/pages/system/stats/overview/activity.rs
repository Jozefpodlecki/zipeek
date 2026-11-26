use super::super::data::SessionRecord;

pub(super) const WEEKS: usize = 5;
pub(super) const ACTIVITY_WEEKS: [&str; WEEKS + 1] = [
    "2026-10-15",
    "2026-10-22",
    "2026-10-29",
    "2026-11-05",
    "2026-11-12",
    "2026-11-19",
];

pub(super) fn activity_buckets(sessions: &[SessionRecord]) -> [u32; WEEKS] {
    let mut counts = [0; WEEKS];
    for session in sessions {
        for index in 0..WEEKS {
            if session.date.as_str() >= ACTIVITY_WEEKS[index]
                && session.date.as_str() < ACTIVITY_WEEKS[index + 1]
            {
                counts[index] += 1;
                break;
            }
        }
    }
    counts
}

pub(super) fn activity_categories(sessions: &[SessionRecord]) -> [(u32, u32); WEEKS] {
    let mut counts = [(0, 0); WEEKS];
    for session in sessions {
        for index in 0..WEEKS {
            if session.date.as_str() >= ACTIVITY_WEEKS[index]
                && session.date.as_str() < ACTIVITY_WEEKS[index + 1]
            {
                match session.category.as_str() {
                    "Practice" => counts[index].0 += 1,
                    "Games" => counts[index].1 += 1,
                    _ => {}
                }
                break;
            }
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_activity_categories_are_not_reported_as_games() {
        let mut sessions = super::super::super::data::demo_sessions_for_tests()
            .as_slice()
            .iter()
            .take(1)
            .cloned()
            .collect::<Vec<_>>();
        sessions[0].date = ACTIVITY_WEEKS[0].to_owned();
        sessions[0].category = "Other".to_owned();
        assert_eq!(activity_buckets(&sessions)[0], 1);
        assert_eq!(activity_categories(&sessions)[0], (0, 0));
        sessions[0].category = "Games".to_owned();
        assert_eq!(activity_categories(&sessions)[0], (0, 1));
        sessions[0].category = "Practice".to_owned();
        assert_eq!(activity_categories(&sessions)[0], (1, 0));
    }
}
