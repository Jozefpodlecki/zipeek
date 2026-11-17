use crate::hanzi_references::{self, GlyphReference};
use zipeek_hanzi_writing::{WritingSession, reference_medians};

#[derive(Clone, PartialEq)]
pub(super) struct StrokeStudy {
    pub session: WritingSession,
    pub paths: Vec<String>,
}

impl StrokeStudy {
    pub fn parse(json: &str) -> Result<Self, String> {
        let asset: GlyphReference = serde_json::from_str(json)
            .map_err(|error| format!("Invalid stroke-reference JSON: {error}"))?;
        Self::from_asset(asset)
    }

    pub fn from_asset(asset: GlyphReference) -> Result<Self, String> {
        if asset.strokes.iter().any(|path| path.len() > 100_000) {
            return Err("Reference stroke outline exceeds the supported limit".to_owned());
        }
        let medians = reference_medians(&asset.medians, asset.strokes.len())
            .map_err(|error| format!("Invalid stroke-reference geometry: {error:?}"))?;
        let session = WritingSession::new(medians)
            .ok_or_else(|| "No usable stroke geometry is available".to_owned())?;
        if asset.strokes.iter().any(|path| path.trim().is_empty()) {
            return Err("Reference contains an empty stroke outline".to_owned());
        }
        Ok(Self {
            session,
            paths: asset.strokes,
        })
    }
}

pub(super) async fn load(character: &str) -> Result<StrokeStudy, String> {
    let mut characters = character.chars();
    let hanzi = characters
        .next()
        .ok_or_else(|| "No character selected".to_owned())?;
    if characters.next().is_some() {
        return Err("Select one character at a time".to_owned());
    }
    StrokeStudy::from_asset(hanzi_references::load(hanzi).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_reference_with_mismatched_outline_and_median_counts() {
        assert!(StrokeStudy::parse(r#"{"strokes":["M0 0"],"medians":[]}"#).is_err());
    }
    #[test]
    fn rejects_excessively_large_outline_before_geometry_processing() {
        let asset = GlyphReference {
            strokes: vec!["M".repeat(100_001)],
            medians: vec![vec![[100.0, 700.0], [800.0, 700.0]]],
        };
        assert!(StrokeStudy::from_asset(asset).is_err());
    }
    #[test]
    fn constructs_valid_study() {
        let result = StrokeStudy::parse(
            r#"{"strokes":["M 0 0 L 100 0"],"medians":[[[100,700],[800,700]]]}"#,
        )
        .unwrap();
        assert_eq!(result.session.total(), 1);
        assert_eq!(result.paths.len(), 1);
    }
}
