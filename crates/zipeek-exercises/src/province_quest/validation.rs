use std::collections::HashSet;

use super::{GeoDataset, MapCoverage, RegionKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatasetIssue {
    MissingProvenance,
    DuplicateId(String),
    MissingName(String),
    MissingPath(String),
    InvalidParent(String),
    MissingCapital(String),
    NoProvinces,
    NoPrefectures,
    IncompleteCoverage,
}

pub fn validate_dataset(dataset: &GeoDataset, hard: bool) -> Result<(), Vec<DatasetIssue>> {
    let mut issues = Vec::new();
    if dataset.source.trim().is_empty()
        || dataset.version.trim().is_empty()
        || dataset.attribution.trim().is_empty()
    {
        issues.push(DatasetIssue::MissingProvenance);
    }
    let ids: HashSet<_> = dataset
        .regions
        .iter()
        .map(|region| region.id.as_str())
        .collect();
    if ids.len() != dataset.regions.len() {
        let mut seen = HashSet::new();
        for region in &dataset.regions {
            if !seen.insert(region.id.as_str()) {
                issues.push(DatasetIssue::DuplicateId(region.id.clone()));
            }
        }
    }
    if dataset.provinces().is_empty() {
        issues.push(DatasetIssue::NoProvinces);
    }
    if hard
        && !dataset
            .regions
            .iter()
            .any(|region| region.parent_id.is_some())
    {
        issues.push(DatasetIssue::NoPrefectures);
    }
    for region in &dataset.regions {
        if region.hanzi.trim().is_empty() || region.pinyin.trim().is_empty() {
            issues.push(DatasetIssue::MissingName(region.id.clone()));
        }
        if region.path.trim().is_empty() || !region.path.starts_with('M') {
            issues.push(DatasetIssue::MissingPath(region.id.clone()));
        }
        if let Some(parent_id) = &region.parent_id {
            if !ids.contains(parent_id.as_str()) {
                issues.push(DatasetIssue::InvalidParent(region.id.clone()));
            }
        } else if !matches!(region.kind, RegionKind::SpecialAdministrativeRegion)
            && region.capital_hanzi.is_none()
        {
            issues.push(DatasetIssue::MissingCapital(region.id.clone()));
        }
    }
    let coverage = MapCoverage::from_dataset(dataset);
    if !coverage.playable(if hard {
        super::Difficulty::Hard
    } else {
        super::Difficulty::Easy
    }) {
        issues.push(DatasetIssue::IncompleteCoverage);
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::province_quest::{MapRegion, RegionKind};

    #[test]
    fn orphan_parent_is_rejected() {
        let map = GeoDataset {
            source: "source".into(),
            version: "2026".into(),
            attribution: "credit".into(),
            view_box: "0 0 100 100".into(),
            regions: vec![MapRegion {
                id: "1".into(),
                parent_id: Some("missing".into()),
                hanzi: "测试".into(),
                pinyin: "cè shì".into(),
                english: "Test".into(),
                capital_hanzi: None,
                capital_pinyin: None,
                kind: RegionKind::Prefecture,
                path: "M0,0L1,0Z".into(),
                bbox: [0.0, 0.0, 10.0, 10.0],
            }],
        };
        assert!(
            validate_dataset(&map, true)
                .unwrap_err()
                .contains(&DatasetIssue::InvalidParent("1".into()))
        );
    }
}
