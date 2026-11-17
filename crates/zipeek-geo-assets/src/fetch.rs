use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::{Client, Url, redirect::Policy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Result, geojson::FeatureCollection, load_json};

const API_URL: &str = "https://www.geoboundaries.org/api/current/gbOpen/CHN/ADM1/";
const REVIEWED_BOUNDARY_ID: &str = "CHN-ADM1-43563684";
const MAX_GEOMETRY_BYTES: usize = 48 * 1024 * 1024;
const GEOJSON_FILE: &str = "china-adm1.geojson";
const MANIFEST_FILE: &str = "china-adm1.source.json";

#[derive(Clone, Debug)]
pub struct FetchOptions {
    pub output: PathBuf,
    pub force: bool,
}

#[derive(Deserialize, Serialize)]
struct GeoBoundariesMetadata {
    #[serde(rename = "boundaryID")]
    boundary_id: String,
    #[serde(rename = "boundaryType")]
    boundary_type: String,
    #[serde(rename = "boundaryYearRepresented")]
    boundary_year: String,
    #[serde(rename = "boundarySource")]
    boundary_source: String,
    #[serde(rename = "boundaryLicense")]
    boundary_license: String,
    #[serde(rename = "admUnitCount")]
    unit_count: String,
    #[serde(rename = "gjDownloadURL")]
    download_url: String,
}

#[derive(Serialize, Deserialize)]
struct DownloadManifest {
    api_url: String,
    boundary_id: String,
    boundary_year: String,
    boundary_source: String,
    boundary_license: String,
    unit_count: usize,
    download_url: String,
    sha256: String,
    bytes: usize,
}

fn approved_host(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some_and(|host| {
            matches!(
                host,
                "github.com"
                    | "raw.githubusercontent.com"
                    | "media.githubusercontent.com"
                    | "objects.githubusercontent.com"
                    | "www.geoboundaries.org"
            ) || host.ends_with(".githubusercontent.com")
        })
}

fn validate_metadata(metadata: &GeoBoundariesMetadata) -> Result<()> {
    if metadata.boundary_id != REVIEWED_BOUNDARY_ID {
        return Err(format!(
            "Unreviewed boundary release {} (expected {}); inspect the provider's provenance before updating this pin",
            metadata.boundary_id, REVIEWED_BOUNDARY_ID
        ));
    }
    if metadata.boundary_type != "ADM1"
        || metadata.unit_count != "34"
        || metadata.boundary_year != "2019"
    {
        return Err("Provider metadata differs from the reviewed China ADM1 dataset".to_owned());
    }
    if metadata.boundary_license != "Public Domain" {
        return Err(format!(
            "Unexpected boundary license: {}",
            metadata.boundary_license
        ));
    }
    let url = Url::parse(&metadata.download_url).map_err(|error| error.to_string())?;
    if !approved_host(&url) {
        return Err("Boundary download URL is not a reviewed HTTPS provider".to_owned());
    }
    Ok(())
}

fn verify_geometry(bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_GEOMETRY_BYTES {
        return Err("Downloaded GeoJSON exceeds the size limit".to_owned());
    }
    let collection: FeatureCollection =
        serde_json::from_slice(bytes).map_err(|error| format!("Invalid GeoJSON: {error}"))?;
    if collection.kind != "FeatureCollection" || collection.features.len() != 34 {
        return Err(format!(
            "Expected exactly 34 GeoJSON ADM1 features, found {}",
            collection.features.len()
        ));
    }
    for feature in &collection.features {
        if !matches!(feature.geometry.kind.as_str(), "Polygon" | "MultiPolygon") {
            return Err("ADM1 GeoJSON contains a non-polygon geometry".to_owned());
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

fn verify_existing(directory: &Path) -> Result<bool> {
    let geojson = directory.join(GEOJSON_FILE);
    let manifest_path = directory.join(MANIFEST_FILE);
    if !geojson.exists() && !manifest_path.exists() {
        return Ok(false);
    }
    let manifest: DownloadManifest = load_json(&manifest_path)?;
    if manifest.boundary_id != REVIEWED_BOUNDARY_ID || manifest.api_url != API_URL {
        return Err("Cached geographic dataset does not match the reviewed source".to_owned());
    }
    let bytes = fs::read(&geojson).map_err(|error| error.to_string())?;
    if sha256(&bytes) != manifest.sha256 || bytes.len() != manifest.bytes {
        return Err("Cached geography SHA-256 differs from its provenance manifest".to_owned());
    }
    verify_geometry(&bytes)?;
    Ok(true)
}

async fn download(client: &Client, url: &str) -> Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if chunk.len() > MAX_GEOMETRY_BYTES.saturating_sub(bytes.len()) {
            return Err("GeoJSON download exceeds the 48 MiB size limit".to_owned());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

pub async fn fetch(arguments: FetchOptions) -> Result<()> {
    fs::create_dir_all(&arguments.output).map_err(|error| error.to_string())?;
    if !arguments.force && verify_existing(&arguments.output)? {
        println!(
            "Verified cached {}",
            arguments.output.join(GEOJSON_FILE).display()
        );
        return Ok(());
    }
    let redirect = Policy::custom(|attempt| {
        if attempt.previous().len() >= 6 || !approved_host(attempt.url()) {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });
    let client = Client::builder()
        .timeout(Duration::from_secs(90))
        .redirect(redirect)
        .build()
        .map_err(|error| error.to_string())?;
    let metadata: GeoBoundariesMetadata = client
        .get(API_URL)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    validate_metadata(&metadata)?;
    let bytes = download(&client, &metadata.download_url).await?;
    verify_geometry(&bytes)?;
    let manifest = DownloadManifest {
        api_url: API_URL.to_owned(),
        boundary_id: metadata.boundary_id,
        boundary_year: metadata.boundary_year,
        boundary_source: metadata.boundary_source,
        boundary_license: metadata.boundary_license,
        unit_count: 34,
        download_url: metadata.download_url,
        sha256: sha256(&bytes),
        bytes: bytes.len(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    atomic_write(&arguments.output.join(GEOJSON_FILE), &bytes)?;
    atomic_write(&arguments.output.join(MANIFEST_FILE), &manifest_bytes)?;
    println!(
        "Downloaded 34 China ADM1 polygons to {}",
        arguments.output.join(GEOJSON_FILE).display()
    );
    println!("SHA-256: {}", manifest.sha256);
    println!(
        "Source: {} ({})",
        manifest.boundary_source, manifest.boundary_year
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> GeoBoundariesMetadata {
        serde_json::from_value(serde_json::json!({
            "boundaryID": "CHN-ADM1-43563684",
            "boundaryType": "ADM1",
            "boundaryYearRepresented": "2019",
            "boundarySource": "geoBoundaries, Wikimedia Commons",
            "boundaryLicense": "Public Domain",
            "admUnitCount": "34",
            "gjDownloadURL": "https://github.com/wmgeolab/geoBoundaries/raw/9469f09/releaseData/gbOpen/CHN/ADM1/geoBoundaries-CHN-ADM1.geojson"
        })).unwrap()
    }

    #[test]
    fn known_province_dataset_is_accepted() {
        assert!(validate_metadata(&metadata()).is_ok());
    }

    #[test]
    fn county_level_source_is_not_accepted_as_prefectures() {
        let mut source = metadata();
        source.boundary_type = "ADM2".to_owned();
        source.unit_count = "2391".to_owned();
        assert!(validate_metadata(&source).is_err());
    }

    #[test]
    fn unknown_dataset_revision_is_rejected() {
        let mut source = metadata();
        source.boundary_id = "CHN-ADM1-unreviewed".to_owned();
        assert!(validate_metadata(&source).is_err());
    }

    #[test]
    fn download_urls_require_a_reviewed_https_host() {
        assert!(!approved_host(
            &Url::parse("http://github.com/file").unwrap()
        ));
        assert!(!approved_host(
            &Url::parse("https://github.com.evil.example/file").unwrap()
        ));
        assert!(approved_host(
            &Url::parse("https://media.githubusercontent.com/file").unwrap()
        ));
    }

    #[test]
    fn missing_geometry_is_rejected() {
        let bytes = br#"{"type":"FeatureCollection","features":[]}"#;
        assert!(verify_geometry(bytes).unwrap_err().contains("34"));
    }

    #[test]
    fn hash_is_stable() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
