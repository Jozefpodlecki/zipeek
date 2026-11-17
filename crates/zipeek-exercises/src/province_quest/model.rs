use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    Province,
    AutonomousRegion,
    Municipality,
    SpecialAdministrativeRegion,
    PrefectureCity,
    Prefecture,
    AutonomousPrefecture,
    League,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapRegion {
    pub id: String,
    pub parent_id: Option<String>,
    pub hanzi: String,
    pub pinyin: String,
    pub english: String,
    pub capital_hanzi: Option<String>,
    pub capital_pinyin: Option<String>,
    pub kind: RegionKind,
    pub path: String,
    pub bbox: [f64; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeoDataset {
    pub source: String,
    pub version: String,
    pub attribution: String,
    pub view_box: String,
    pub regions: Vec<MapRegion>,
}

impl GeoDataset {
    pub fn children_of(&self, parent_id: &str) -> Vec<&MapRegion> {
        self.regions
            .iter()
            .filter(|region| region.parent_id.as_deref() == Some(parent_id))
            .collect()
    }

    pub fn provinces(&self) -> Vec<&MapRegion> {
        self.regions
            .iter()
            .filter(|region| region.parent_id.is_none())
            .collect()
    }
}
