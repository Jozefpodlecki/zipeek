use std::{cell::RefCell, collections::HashMap, rc::Rc};

use futures::{
    FutureExt,
    future::{LocalBoxFuture, Shared},
};
use serde::Deserialize;
use zipeek_hanzi_writing::{character_asset_id, reference_medians};

#[derive(Clone, PartialEq, Deserialize)]
pub struct GlyphReference {
    pub strokes: Vec<String>,
    pub medians: Vec<Vec<[f64; 2]>>,
}

#[derive(Deserialize)]
struct ReferenceBundle {
    version: u32,
    characters: HashMap<String, GlyphReference>,
}

type BundleFuture = Shared<LocalBoxFuture<'static, Result<Rc<ReferenceBundle>, String>>>;
struct CachedRequest(BundleFuture);
thread_local! {
    static REFERENCE_BUNDLE: RefCell<Option<Rc<CachedRequest>>> = const { RefCell::new(None) };
}

fn parse_bundle(json: &str) -> Result<ReferenceBundle, String> {
    let bundle: ReferenceBundle = serde_json::from_str(json)
        .map_err(|error| format!("Invalid stroke-reference bundle: {error}"))?;
    if bundle.version != 1 {
        return Err(format!(
            "Unsupported stroke-reference bundle version: {}",
            bundle.version
        ));
    }
    Ok(bundle)
}

async fn fetch_bundle() -> Result<Rc<ReferenceBundle>, String> {
    let response = gloo::net::http::Request::get("hanzi-strokes/characters.json")
        .send()
        .await
        .map_err(|error| format!("Could not load stroke references: {error}"))?;
    if !response.ok() {
        return Err(
            "No packaged Hanzi stroke references (hanzi-strokes/characters.json)".to_owned(),
        );
    }
    let json = response.text().await.map_err(|error| error.to_string())?;
    Ok(Rc::new(parse_bundle(&json)?))
}

pub async fn load(character: char) -> Result<GlyphReference, String> {
    let pending = REFERENCE_BUNDLE.with(|cache| {
        cache
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(CachedRequest(fetch_bundle().boxed_local().shared())))
            .clone()
    });
    let bundle = match pending.0.clone().await {
        Ok(bundle) => bundle,
        Err(error) => {
            REFERENCE_BUNDLE.with(|cache| {
                let mut cached = cache.borrow_mut();
                if cached
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, &pending))
                {
                    *cached = None;
                }
            });
            return Err(error);
        }
    };
    let reference = bundle
        .characters
        .get(&character_asset_id(character))
        .ok_or_else(|| format!("No packaged stroke reference for {character}"))?
        .clone();
    reference_medians(&reference.medians, reference.strokes.len())
        .map_err(|error| format!("Invalid stroke-reference geometry: {error:?}"))?;
    if reference.strokes.iter().any(|path| path.trim().is_empty()) {
        return Err("Reference contains an empty stroke outline".to_owned());
    }
    Ok(reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aggregate_and_rejects_unknown_versions() {
        let json = r#"{"version":1,"characters":{"4eba":{"strokes":["M 0 0"],"medians":[[[0,0],[100,100]]]}}}"#;
        let bundle = parse_bundle(json).unwrap();
        assert_eq!(bundle.characters["4eba"].strokes.len(), 1);
        assert!(parse_bundle(&json.replace("\"version\":1", "\"version\":2")).is_err());
    }
}
