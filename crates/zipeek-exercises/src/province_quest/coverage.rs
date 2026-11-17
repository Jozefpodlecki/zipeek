use super::{Difficulty, GeoDataset, RegionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapCoverage {
    pub provincial_regions: usize,
    pub prefecture_regions: usize,
    pub orphan_prefectures: usize,
    pub missing_capitals: usize,
}

impl MapCoverage {
    pub fn from_dataset(dataset: &GeoDataset) -> Self {
        let province_ids: std::collections::HashSet<_> = dataset
            .provinces()
            .iter()
            .map(|region| region.id.as_str())
            .collect();
        Self {
            provincial_regions: dataset
                .regions
                .iter()
                .filter(|region| region.parent_id.is_none())
                .count(),
            prefecture_regions: dataset
                .regions
                .iter()
                .filter(|region| region.parent_id.is_some())
                .count(),
            orphan_prefectures: dataset
                .regions
                .iter()
                .filter(|region| {
                    region
                        .parent_id
                        .as_ref()
                        .is_some_and(|id| !province_ids.contains(id.as_str()))
                })
                .count(),
            missing_capitals: dataset
                .regions
                .iter()
                .filter(|region| {
                    region.parent_id.is_none()
                        && !matches!(region.kind, RegionKind::SpecialAdministrativeRegion)
                        && region.capital_hanzi.is_none()
                })
                .count(),
        }
    }

    pub fn playable(&self, difficulty: Difficulty) -> bool {
        self.orphan_prefectures == 0
            && self.missing_capitals == 0
            && self.provincial_regions == 34
            && (difficulty != Difficulty::Hard || self.prefecture_regions >= 300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_is_never_mistaken_for_complete_geography() {
        let dataset = GeoDataset {
            source: "x".into(),
            version: "1".into(),
            attribution: "x".into(),
            view_box: "0 0 10 10".into(),
            regions: vec![],
        };
        let coverage = MapCoverage::from_dataset(&dataset);
        assert!(!coverage.playable(Difficulty::Easy));
        assert!(!coverage.playable(Difficulty::Hard));
    }
}
