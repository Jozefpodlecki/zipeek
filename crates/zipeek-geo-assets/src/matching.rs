use crate::{
    Region, Result,
    catalog::CatalogRecord,
    geojson::{Feature, source_name},
    geometry::geometry_path,
};
use std::collections::{HashMap, HashSet};

fn canonical_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

pub(crate) fn matching_features(
    features: Vec<Feature>,
    catalog: &[CatalogRecord],
) -> Result<Vec<Region>> {
    let mut by_name = HashMap::new();
    for feature in features {
        let key = canonical_name(source_name(&feature)?);
        if by_name.insert(key.clone(), feature).is_some() {
            return Err(format!("Duplicate geography name: {key}"));
        }
    }
    let mut used: HashSet<String> = HashSet::new();
    let mut resolved = Vec::with_capacity(catalog.len());
    for record in catalog {
        let keys: HashSet<_> = record
            .source_names
            .iter()
            .map(|name| canonical_name(name))
            .collect();
        let matched: Vec<_> = keys
            .iter()
            .filter(|key| by_name.contains_key(*key))
            .collect();
        if matched.len() != 1 {
            return Err(format!(
                "Missing or ambiguous polygon for {}: {:?}",
                record.hanzi, keys
            ));
        }
        let key = matched[0];
        if !used.insert(key.to_string()) {
            return Err(format!(
                "Polygon reused by multiple administrative divisions: {key}"
            ));
        }
        let geometry = &by_name
            .get(key)
            .expect("matched feature must exist")
            .geometry;
        let (path, bbox) = geometry_path(geometry)?;
        resolved.push(Region {
            id: record.id.clone(),
            parent_id: record.parent_id.clone(),
            hanzi: record.hanzi.clone(),
            pinyin: record.pinyin.clone(),
            english: record.english.clone(),
            capital_hanzi: record.capital_hanzi.clone(),
            capital_pinyin: record.capital_pinyin.clone(),
            kind: record.kind.clone(),
            path,
            bbox,
        });
    }
    let mut unknown: Vec<_> = by_name
        .keys()
        .filter(|name| !used.contains(name.as_str()))
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        return Err(format!(
            "Unmapped geographic features ({}): {}",
            unknown.len(),
            unknown
                .iter()
                .take(15)
                .map(|name| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(resolved)
}
