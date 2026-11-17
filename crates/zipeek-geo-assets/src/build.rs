use crate::{
    Region, Result, catalog::CatalogRecord, geojson::read_geojson, load_json,
    matching::matching_features,
};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct BuildOptions {
    pub provinces: PathBuf,
    pub prefectures: Option<PathBuf>,
    pub catalog: PathBuf,
    pub output: PathBuf,
    pub source: String,
    pub version: String,
    pub attribution: String,
}

#[derive(Serialize)]
struct MapAsset<'a> {
    source: &'a str,
    version: &'a str,
    attribution: &'a str,
    view_box: &'static str,
    regions: Vec<Region>,
}

fn write_asset(target: &Path, asset: &MapAsset<'_>) -> Result<()> {
    let bytes = serde_json::to_vec(asset).map_err(|error| error.to_string())?;
    fs::write(target, bytes).map_err(|error| format!("{}: {error}", target.display()))?;
    println!("{}: {} regions", target.display(), asset.regions.len());
    Ok(())
}

pub fn build(arguments: &BuildOptions) -> Result<()> {
    if [
        arguments.source.as_str(),
        arguments.version.as_str(),
        arguments.attribution.as_str(),
    ]
    .iter()
    .any(|field| field.trim().is_empty())
    {
        return Err("Data source, version, and attribution are mandatory".to_owned());
    }
    let catalog: Vec<CatalogRecord> = load_json(&arguments.catalog)?;
    let provinces: Vec<_> = catalog
        .iter()
        .filter(|record| record.parent_id.is_none())
        .cloned()
        .collect();
    let prefectures: Vec<_> = catalog
        .iter()
        .filter(|record| record.parent_id.is_some())
        .cloned()
        .collect();
    if provinces.len() != 34 {
        return Err(format!(
            "Catalog must contain 34 audited provincial-level divisions; found {}",
            provinces.len()
        ));
    }
    let mut ids = HashSet::new();
    if catalog
        .iter()
        .any(|record| record.id.trim().is_empty() || !ids.insert(record.id.as_str()))
    {
        return Err("Duplicate or empty administrative identifiers in catalog".to_owned());
    }
    let polygons = matching_features(read_geojson(&arguments.provinces)?, &provinces)?;
    let mut assets = vec![("provinces.json", polygons.clone())];
    if let Some(prefecture_path) = &arguments.prefectures {
        if prefectures.len() < 300 {
            return Err("Hard map requires a reviewed catalog of prefecture-level divisions, not a substitute ADM2 layer".to_owned());
        }
        let children = matching_features(read_geojson(prefecture_path)?, &prefectures)?;
        let province_ids: HashSet<_> = provinces.iter().map(|region| region.id.as_str()).collect();
        if children.iter().any(|region| {
            !region
                .parent_id
                .as_ref()
                .is_some_and(|parent| province_ids.contains(parent.as_str()))
        }) {
            return Err("A prefecture is attached to a missing province".to_owned());
        }
        assets.push(("prefectures.json", [polygons, children].concat()));
    }
    fs::create_dir_all(&arguments.output)
        .map_err(|error| format!("{}: {error}", arguments.output.display()))?;
    for (name, regions) in assets {
        write_asset(
            &arguments.output.join(name),
            &MapAsset {
                source: &arguments.source,
                version: &arguments.version,
                attribution: &arguments.attribution,
                view_box: "0 0 1000 680",
                regions,
            },
        )?;
    }
    Ok(())
}
