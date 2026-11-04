use crate::theme::{ThemeId, ThemeRequest};

pub fn resolve_theme(request: &ThemeRequest) -> ThemeId {
    let mut counts = [0_usize; 4];
    for category in &request.categories {
        let category = category.trim().to_ascii_lowercase();
        let index = match category.as_str() {
            "travel" | "transportation" | "transport" => Some(0),
            "food" | "cooking" | "dining" => Some(1),
            "shopping" | "commerce" | "money" => Some(2),
            "nature" | "animals" | "plants" | "environment" => Some(3),
            _ => None,
        };
        if let Some(index) = index {
            counts[index] += 1;
        }
    }
    let max = counts.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return ThemeId::Neutral;
    }
    let candidates: Vec<usize> = counts.iter().enumerate()
        .filter_map(|(index, count)| (*count == max).then_some(index))
        .collect();
    let selected = candidates[(request.seed % candidates.len() as u64) as usize];
    match selected {
        0 => ThemeId::Travel,
        1 => ThemeId::Food,
        2 => ThemeId::Shopping,
        _ => ThemeId::Nature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_topics_use_neutral() {
        assert_eq!(resolve_theme(&ThemeRequest { categories: vec!["unknown".into()], seed: 0 }), ThemeId::Neutral);
    }

    #[test]
    fn majority_wins_independent_of_order() {
        let request = ThemeRequest { categories: vec!["shopping".into(), "travel".into(), "travel".into()], seed: 9 };
        assert_eq!(resolve_theme(&request), ThemeId::Travel);
    }

    #[test]
    fn ties_are_seeded_and_stable() {
        let request = ThemeRequest { categories: vec!["shopping".into(), "travel".into()], seed: 0 };
        assert_eq!(resolve_theme(&request), resolve_theme(&request));
        assert_ne!(resolve_theme(&request), resolve_theme(&ThemeRequest { seed: 1, ..request }));
    }
}
