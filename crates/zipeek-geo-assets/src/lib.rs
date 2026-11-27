#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

pub mod areacity;
mod build;
mod catalog;
mod fetch;
mod geojson;
mod geometry;
mod matching;
mod model;

use serde::de::DeserializeOwned;
use std::{fs, path::Path};

pub use build::{BuildOptions, build};
pub use fetch::{FetchOptions, fetch};
use model::Region;

type Result<T> = std::result::Result<T, String>;

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let data = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&data).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use crate::{
        catalog::CatalogRecord,
        geojson::{Feature, Geometry},
        geometry::{geometry_path, projected},
        matching::matching_features,
        model::Region,
    };
    use serde_json::Value;

    fn feature(name: &str, geometry: Value) -> Feature {
        Feature {
            properties: Some(
                serde_json::from_value(serde_json::json!({"shapeName": name})).unwrap(),
            ),
            geometry: serde_json::from_value(geometry).unwrap(),
        }
    }

    fn record(name: &str) -> CatalogRecord {
        CatalogRecord {
            id: "53".to_owned(),
            parent_id: None,
            hanzi: "云南省".to_owned(),
            pinyin: "Yúnnán Shěng".to_owned(),
            english: "Yunnan".to_owned(),
            capital_hanzi: Some("昆明市".to_owned()),
            capital_pinyin: Some("Kūnmíng Shì".to_owned()),
            kind: "province".to_owned(),
            source_names: vec![name.to_owned()],
        }
    }

    fn ring() -> Value {
        serde_json::json!([[100, 30], [101, 30], [101, 31], [100, 30]])
    }

    #[test]
    fn catalog_allows_regions_without_capitals() {
        let record: CatalogRecord = serde_json::from_value(serde_json::json!({
            "id": "81",
            "parent_id": null,
            "hanzi": "香港特别行政区",
            "pinyin": "Xiānggǎng Tèbié Xíngzhèngqū",
            "english": "Hong Kong",
            "capital_hanzi": null,
            "capital_pinyin": null,
            "kind": "special_administrative_region",
            "source_names": ["Hong Kong"]
        }))
        .unwrap();
        assert!(record.capital_hanzi.is_none());
        assert!(record.capital_pinyin.is_none());
        let region = Region {
            id: record.id,
            parent_id: record.parent_id,
            hanzi: record.hanzi,
            pinyin: record.pinyin,
            english: record.english,
            capital_hanzi: record.capital_hanzi,
            capital_pinyin: record.capital_pinyin,
            kind: record.kind,
            path: "M0,0Z".to_owned(),
            bbox: [0.0, 0.0, 1.0, 1.0],
        };
        let output = serde_json::to_value(region).unwrap();
        assert!(output["capital_hanzi"].is_null());
        assert!(output["capital_pinyin"].is_null());
    }

    #[test]
    fn projection_preserves_finite_coordinates() {
        let (x, y) = projected(&serde_json::json!([105, 35])).unwrap();
        assert!(x > 0.0 && y > 0.0);
    }

    #[test]
    fn multipolygon_preserves_all_islands() {
        let geometry: Geometry = serde_json::from_value(serde_json::json!({
            "type": "MultiPolygon", "coordinates": [[[ [100,30],[101,30],[101,31],[100,30] ]], [[ [102,32],[103,32],[103,33],[102,32] ]]]
        })).unwrap();
        let (path, bbox) = geometry_path(&geometry).unwrap();
        assert_eq!(path.matches('M').count(), 2);
        assert_eq!(path.matches('Z').count(), 2);
        assert!(bbox[2] > 0.0 && bbox[3] > 0.0);
    }

    #[test]
    fn missing_region_is_rejected() {
        let error = matching_features(vec![], &[record("Yunnan")])
            .err()
            .unwrap();
        assert!(error.contains("Missing or ambiguous polygon"));
    }

    #[test]
    fn unknown_region_is_rejected() {
        let source = feature(
            "Unexpected",
            serde_json::json!({"type":"Polygon","coordinates":[ring()]}),
        );
        assert!(matching_features(vec![source], &[record("Yunnan")]).is_err());
    }

    #[test]
    fn invalid_latitude_is_rejected() {
        assert!(
            projected(&serde_json::json!([100, 91]))
                .unwrap_err()
                .contains("Invalid geographic coordinate")
        );
    }

    #[test]
    fn reused_polygon_is_rejected() {
        let source = feature(
            "Yunnan",
            serde_json::json!({"type":"Polygon","coordinates":[ring()]}),
        );
        let mut second = record("Yunnan");
        second.id = "54".to_owned();
        assert!(
            matching_features(vec![source], &[record("Yunnan"), second])
                .unwrap_err()
                .contains("reused")
        );
    }

    #[test]
    fn duplicate_source_names_are_rejected() {
        let first = feature(
            "Yunnan",
            serde_json::json!({"type":"Polygon","coordinates":[ring()]}),
        );
        let second = feature(
            "Yunnan",
            serde_json::json!({"type":"Polygon","coordinates":[ring()]}),
        );
        assert!(
            matching_features(vec![first, second], &[record("Yunnan")])
                .unwrap_err()
                .contains("Duplicate")
        );
    }
}
