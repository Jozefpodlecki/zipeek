use zipeek_exercises::province_quest::{GeoDataset, validate_dataset};

pub async fn load_map(difficulty_is_hard: bool) -> Result<GeoDataset, String> {
    let file_name = if difficulty_is_hard {
        "prefectures.json"
    } else {
        "provinces.json"
    };
    let url = format!("province-quest/{file_name}");
    let response = gloo::net::http::Request::get(&url)
        .send()
        .await
        .map_err(|error| format!("Could not load map asset: {error}"))?;
    if !response.ok() {
        return Err(format!(
            "Map asset {file_name} is not installed (HTTP {}). Generate the pinned geography assets before starting this game.",
            response.status()
        ));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap_or_default();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read map asset {file_name}: {error}"))?;
    let map = decode_map_asset(file_name, &content_type, &body)?;
    validate_dataset(&map, difficulty_is_hard)
        .map_err(|issues| format!("Map asset failed validation: {issues:?}"))?;
    Ok(map)
}

fn decode_map_asset(file_name: &str, content_type: &str, body: &str) -> Result<GeoDataset, String> {
    if body.trim_start().starts_with('<') || content_type.contains("text/html") {
        return Err(format!(
            "Map asset {file_name} returned HTML instead of geography JSON. Generate and install the Province Quest geography assets, then verify the province-quest asset path for this deployment."
        ));
    }
    if body.trim().is_empty() {
        return Err(format!("Map asset {file_name} is empty. Regenerate the Province Quest geography assets."));
    }
    serde_json::from_str(body).map_err(|error| {
        format!("Map asset {file_name} has an invalid JSON schema: {error}. Regenerate the pinned geography assets.")
    })
}

#[cfg(test)]
mod tests {
    use super::decode_map_asset;

    #[test]
    fn html_fallback_is_not_reported_as_a_json_schema_failure() {
        let error = decode_map_asset("provinces.json", "text/html", "<!doctype html><html></html>").err().unwrap();
        assert!(error.contains("returned HTML"));
        assert!(error.contains("provinces.json"));
    }

    #[test]
    fn empty_map_asset_has_actionable_error() {
        let error = decode_map_asset("provinces.json", "application/json", "  ").err().unwrap();
        assert!(error.contains("is empty"));
    }

    #[test]
    fn malformed_json_is_reported_distinctly() {
        let error = decode_map_asset("provinces.json", "application/json", "not json").err().unwrap();
        assert!(error.contains("invalid JSON schema"));
    }
}
