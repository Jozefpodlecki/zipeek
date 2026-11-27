use zipeek_exercises::practice_draft::{HskBand, PracticeDraft, VocabularySource};

pub(super) fn toggle_hsk_band(draft: &mut PracticeDraft, band: HskBand) {
    if let VocabularySource::Dictionary(filters) = &mut draft.vocabulary {
        if !filters.hsk.remove(&band) {
            filters.hsk.insert(band);
        }
    }
}

pub(super) fn select_hsk_standard(draft: &mut PracticeDraft, system: u8) -> bool {
    if !matches!(system, 2 | 3) {
        return false;
    }
    if let VocabularySource::Dictionary(filters) = &mut draft.vocabulary {
        filters.hsk.retain(|band| band.system == system);
        if filters.hsk.is_empty() {
            filters
                .hsk
                .insert(HskBand::new(system, 1).expect("valid HSK system"));
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_hsk_system_does_not_modify_draft() {
        let mut draft = PracticeDraft::default();
        let before = draft.clone();
        assert!(!select_hsk_standard(&mut draft, 4));
        assert_eq!(draft, before);
    }

    #[test]
    fn mixed_mode_keeps_both_hsk_systems() {
        let mut draft = PracticeDraft::default();
        toggle_hsk_band(&mut draft, HskBand::new(2, 5).unwrap());
        if let VocabularySource::Dictionary(filters) = &draft.vocabulary {
            assert!(filters.hsk.contains(&HskBand::new(2, 5).unwrap()));
            assert!(filters.hsk.contains(&HskBand::new(3, 1).unwrap()));
        }
    }

    #[test]
    fn valid_system_replaces_other_system_bands() {
        let mut draft = PracticeDraft::default();
        assert!(select_hsk_standard(&mut draft, 2));
        if let VocabularySource::Dictionary(filters) = &draft.vocabulary {
            assert!(filters.hsk.iter().all(|band| band.system == 2));
            assert!(!filters.hsk.is_empty());
        }
    }
}
