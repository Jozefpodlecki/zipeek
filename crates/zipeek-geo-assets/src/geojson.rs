use crate::{Result, load_json};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

#[derive(Deserialize)]
pub(crate) struct FeatureCollection {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) features: Vec<Feature>,
}

#[derive(Deserialize)]
pub(crate) struct Feature {
    pub(crate) properties: Option<serde_json::Map<String, Value>>,
    pub(crate) geometry: Geometry,
}

#[derive(Deserialize)]
pub(crate) struct Geometry {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) coordinates: Value,
}

pub(crate) fn read_geojson(path: &Path) -> Result<Vec<Feature>> {
    let collection: FeatureCollection = load_json(path)?;
    if collection.kind != "FeatureCollection" {
        return Err(format!(
            "Expected GeoJSON FeatureCollection: {}",
            path.display()
        ));
    }
    Ok(collection.features)
}

pub(crate) fn source_name(feature: &Feature) -> Result<&str> {
    let properties = feature
        .properties
        .as_ref()
        .ok_or("Feature has no administrative name")?;
    ["shapeName", "name", "NAME", "name_en", "NAME_EN"]
        .into_iter()
        .filter_map(|key| properties.get(key).and_then(Value::as_str))
        .find(|name| !name.trim().is_empty())
        .ok_or_else(|| "Feature has no administrative name".to_owned())
}
