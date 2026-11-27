use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use reqwest::{Client, Url, redirect::Policy};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{Result, catalog::CatalogRecord, geometry::projected, model::Region};

pub const VERSION: &str = "2025.251231.260403";
const REVISION: &str = "c6c6e35";
const ROOT: &str = "https://github.com/xiangyuecn/AreaCity-JsSpider-StatsGov";
const HIERARCHY_URL: &str = "https://raw.githubusercontent.com/xiangyuecn/AreaCity-JsSpider-StatsGov/c6c6e35/src/%E9%87%87%E9%9B%86%E5%88%B0%E7%9A%84%E6%95%B0%E6%8D%AE/ok_data_level3.csv";
const GEOMETRY_URL: &str = "https://github.com/xiangyuecn/AreaCity-JsSpider-StatsGov/releases/download/2025.251231.260403/ok_geo.csv.7z";
const MAX_HIERARCHY: usize = 2 * 1024 * 1024;
const MAX_ARCHIVE: usize = 40 * 1024 * 1024;

const PROVINCE_IDS: [&str; 34] = [
    "11", "12", "13", "14", "15", "21", "22", "23", "31", "32", "33", "34", "35", "36", "37", "41",
    "42", "43", "44", "45", "46", "50", "51", "52", "53", "54", "61", "62", "63", "64", "65", "71",
    "81", "82",
];
const NON_GEOGRAPHIC_ROOT: &str = "91";

const KNOWN_UNMAPPED_TAIWAN_PREFECTURES: [&str; 20] = [
    "7101", "7102", "7103", "7104", "7106", "7107", "7108", "7109", "7111", "7112", "7113", "7114",
    "7115", "7117", "7119", "7121", "7124", "7125", "7126", "7127",
];

fn unexpected_missing_prefectures(report: &Audit) -> Vec<&str> {
    report
        .missing_prefectures
        .iter()
        .filter_map(|entry| {
            let id = entry.split_whitespace().next()?;
            (!KNOWN_UNMAPPED_TAIWAN_PREFECTURES.contains(&id)).then_some(id)
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct FetchOptions {
    pub output: PathBuf,
    pub force: bool,
}

#[derive(Clone, Debug)]
pub struct BuildOptions {
    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct Audit {
    pub provider: &'static str,
    pub version: &'static str,
    pub coordinate_system: &'static str,
    pub provinces: usize,
    pub prefectures: usize,
    pub excluded_synthetic: Vec<String>,
    pub excluded_non_geographic: Vec<String>,
    pub missing_provinces: Vec<String>,
    pub missing_prefectures: Vec<String>,
    pub unknown_geometry_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct Division {
    id: String,
    pid: String,
    deep: u8,
    name: String,
    pinyin: String,
    ext_name: String,
}

fn csv_fields(line: &str) -> Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.trim_end_matches(['\r', '\n']).chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                fields.push(std::mem::take(&mut field));
            }
            _ => field.push(ch),
        }
    }
    if quoted {
        return Err("Unterminated CSV quotation".to_owned());
    }
    fields.push(field);
    Ok(fields)
}

fn header_indices(header: &str, required: &[&str]) -> Result<Vec<usize>> {
    let fields = csv_fields(header.trim_start_matches('\u{feff}'))?;
    required
        .iter()
        .map(|name| {
            fields
                .iter()
                .position(|field| field.trim().eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("Missing CSV column {name}"))
        })
        .collect()
}

fn column<'a>(fields: &'a [String], index: usize) -> Result<&'a str> {
    fields
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| "Truncated CSV row".to_owned())
}

fn validate_roots(roots: &HashSet<&str>) -> Result<()> {
    let expected: HashSet<&str> = PROVINCE_IDS.into_iter().collect();
    let missing: Vec<_> = expected.difference(roots).copied().collect();
    let unexpected: Vec<_> = roots
        .iter()
        .copied()
        .filter(|id| !expected.contains(id) && *id != NON_GEOGRAPHIC_ROOT)
        .collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        let mut missing = missing;
        let mut unexpected = unexpected;
        missing.sort_unstable();
        unexpected.sort_unstable();
        return Err(format!(
            "AreaCity province IDs differ from the 34-region catalog: missing {missing:?}, unexpected {unexpected:?}"
        ));
    }
    Ok(())
}

fn divisions(path: &Path) -> Result<Vec<Division>> {
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .ok_or("Empty hierarchy CSV")?
        .map_err(|error| error.to_string())?;
    let columns = header_indices(
        &header,
        &["id", "pid", "deep", "name", "pinyin", "ext_name"],
    )?;
    let mut records = Vec::new();
    let mut seen = HashSet::new();
    for line in lines {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = csv_fields(&line)?;
        let id = column(&fields, columns[0])?.to_owned();
        if !seen.insert(id.clone()) {
            return Err(format!("Duplicate AreaCity ID {id}"));
        }
        let deep = column(&fields, columns[2])?
            .parse::<u8>()
            .map_err(|error| error.to_string())?;
        records.push(Division {
            id,
            pid: column(&fields, columns[1])?.to_owned(),
            deep,
            name: column(&fields, columns[3])?.to_owned(),
            pinyin: column(&fields, columns[4])?.to_owned(),
            ext_name: column(&fields, columns[5])?.to_owned(),
        });
    }
    let roots: HashSet<_> = records
        .iter()
        .filter(|record| record.deep == 0)
        .map(|record| record.id.as_str())
        .collect();
    validate_roots(&roots)?;
    for record in &records {
        if record.deep == 0 && record.pid != "0" {
            return Err(format!("Province {} has nonzero parent", record.id));
        }
        if record.deep == 1 && !roots.contains(record.pid.as_str()) {
            return Err(format!(
                "Prefecture {} has unknown province {}",
                record.id, record.pid
            ));
        }
    }
    Ok(records)
}

fn is_synthetic(record: &Division, parent: &Division) -> bool {
    if record.deep != 1 {
        return false;
    }
    matches!(parent.id.as_str(), "11" | "12" | "31" | "50")
        || matches!(
            record.name.as_str(),
            "省直辖县级行政区划" | "自治区直辖县级行政区划" | "省直辖" | "自治区直辖"
        )
        || record.ext_name.contains("直辖县级行政区划")
        || record.id.ends_with("90")
}

fn polygon_path(text: &str) -> Result<(String, [f64; 4])> {
    if text.trim().eq_ignore_ascii_case("EMPTY") || text.trim().is_empty() {
        return Err("Polygon missing".to_owned());
    }
    let mut path = String::new();
    let mut bounds = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for piece in text.split(';') {
        for ring in piece.split('~') {
            let points: Vec<&str> = ring
                .split(',')
                .filter(|point| !point.trim().is_empty())
                .collect();
            if points.len() < 3 {
                return Err("Polygon ring has too few points".to_owned());
            }
            for (index, point) in points.iter().enumerate() {
                let mut values = point.split_whitespace();
                let longitude = values
                    .next()
                    .ok_or("Missing longitude")?
                    .parse::<f64>()
                    .map_err(|error| error.to_string())?;
                let latitude = values
                    .next()
                    .ok_or("Missing latitude")?
                    .parse::<f64>()
                    .map_err(|error| error.to_string())?;
                if values.next().is_some() {
                    return Err("Unexpected polygon coordinate".to_owned());
                }
                let (x, y) = projected(&serde_json::json!([longitude, latitude]))?;
                path.push(if index == 0 { 'M' } else { 'L' });
                path.push_str(&format!("{x:.2},{y:.2}"));
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x);
                bounds[3] = bounds[3].max(y);
            }
            path.push('Z');
        }
    }
    Ok((
        path,
        [
            round(bounds[0]),
            round(bounds[1]),
            round(bounds[2] - bounds[0]),
            round(bounds[3] - bounds[1]),
        ],
    ))
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn geometries(
    path: &Path,
    wanted: &HashSet<&str>,
) -> Result<(HashMap<String, (String, [f64; 4])>, Vec<String>)> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .ok_or("Empty geometry CSV")?
        .map_err(|error| error.to_string())?;
    let indices = header_indices(&header, &["id", "deep", "polygon"])?;
    let mut found = HashMap::new();
    let mut unexpected = Vec::new();
    for line in lines {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = csv_fields(&line)?;
        let id = column(&fields, indices[0])?;
        let depth = column(&fields, indices[1])?;
        if depth != "0" && depth != "1" {
            continue;
        }
        if !wanted.contains(id) {
            unexpected.push(id.to_owned());
            continue;
        }
        let polygon = column(&fields, indices[2])?;
        if polygon.eq_ignore_ascii_case("EMPTY") || polygon.trim().is_empty() {
            continue;
        }
        if found
            .insert(
                id.to_owned(),
                polygon_path(polygon).map_err(|error| format!("AreaCity polygon {id}: {error}"))?,
            )
            .is_some()
        {
            return Err(format!("Duplicate AreaCity polygon for {id}"));
        }
    }
    Ok((found, unexpected))
}

fn inspect(input: &Path) -> Result<(Vec<Region>, Vec<Region>, Audit)> {
    let hierarchy = divisions(&input.join("ok_data_level3.csv"))?;
    let parents: HashMap<_, _> = hierarchy
        .iter()
        .filter(|row| row.deep == 0)
        .map(|row| (row.id.as_str(), row))
        .collect();

    let candidates: Vec<_> = hierarchy
        .iter()
        .filter(|row| {
            row.deep <= 1 && row.id != NON_GEOGRAPHIC_ROOT && row.pid != NON_GEOGRAPHIC_ROOT
        })
        .collect();
    let mut excluded = Vec::new();
    let active: Vec<_> = candidates
        .into_iter()
        .filter(|row| {
            if row.deep == 1
                && parents
                    .get(row.pid.as_str())
                    .is_some_and(|parent| is_synthetic(row, parent))
            {
                excluded.push(format!("{} {} (parent {})", row.id, row.ext_name, row.pid));
                false
            } else {
                true
            }
        })
        .collect();
    let ids: HashSet<_> = active.iter().map(|record| record.id.as_str()).collect();
    let (geometry, unknown) = geometries(&input.join("ok_geo.csv"), &ids)?;
    let capital_catalog: Vec<CatalogRecord> = serde_json::from_str(include_str!(
        "provinces.catalog.json"
    ))
    .map_err(|error| error.to_string())?;
    let capitals: HashMap<_, _> = capital_catalog
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let mut provinces = Vec::new();
    let mut prefectures = Vec::new();
    let mut missing_provinces = Vec::new();
    let mut missing_prefectures = Vec::new();
    for item in active {
        let Some((path, bbox)) = geometry.get(&item.id) else {
            let failure = format!("{} {}", item.id, item.ext_name);
            if item.deep == 0 {
                missing_provinces.push(failure);
            } else {
                missing_prefectures.push(failure);
            }
            continue;
        };
        let prior = capitals.get(item.id.as_str());
        let region = Region {
            id: item.id.clone(),
            parent_id: (item.deep != 0).then(|| item.pid.clone()),
            hanzi: item.ext_name.clone(),
            pinyin: prior
                .map(|record| record.pinyin.clone())
                .unwrap_or_else(|| item.pinyin.clone()),
            english: prior
                .map(|record| record.english.clone())
                .unwrap_or_else(|| item.pinyin.clone()),
            capital_hanzi: prior.and_then(|record| record.capital_hanzi.clone()),
            capital_pinyin: prior.and_then(|record| record.capital_pinyin.clone()),
            kind: prior.map(|record| record.kind.clone()).unwrap_or_else(|| {
                let kind = if item.ext_name.ends_with("自治州") {
                    "autonomous_prefecture"
                } else if item.ext_name.ends_with('盟') {
                    "league"
                } else if item.ext_name.ends_with('市') {
                    "prefecture_city"
                } else {
                    "prefecture"
                };
                kind.to_owned()
            }),
            path: path.clone(),
            bbox: *bbox,
        };
        if item.deep == 0 {
            provinces.push(region);
        } else {
            prefectures.push(region);
        }
    }
    provinces.sort_by(|a, b| a.id.cmp(&b.id));
    prefectures.sort_by(|a, b| a.id.cmp(&b.id));
    let audit = Audit {
        provider: "areacity",
        version: VERSION,
        coordinate_system: "GCJ-02",
        provinces: provinces.len(),
        prefectures: prefectures.len(),
        excluded_synthetic: excluded,
        excluded_non_geographic: hierarchy
            .iter()
            .filter(|row| {
                row.id == NON_GEOGRAPHIC_ROOT || (row.deep == 1 && row.pid == NON_GEOGRAPHIC_ROOT)
            })
            .map(|row| format!("{} {}", row.id, row.ext_name))
            .collect(),
        missing_provinces,
        missing_prefectures,
        unknown_geometry_ids: unknown,
    };
    Ok((provinces, prefectures, audit))
}

pub fn audit(input: &Path) -> Result<Audit> {
    inspect(input).map(|(_, _, report)| report)
}

#[derive(Serialize)]
struct Asset<'a> {
    source: &'static str,
    version: &'static str,
    attribution: &'static str,
    view_box: &'static str,
    regions: &'a [Region],
}

pub fn build(options: &BuildOptions) -> Result<Audit> {
    let (provinces, prefectures, report) = inspect(&options.input)?;
    let unexpected = unexpected_missing_prefectures(&report);
    if !report.missing_provinces.is_empty()
        || !unexpected.is_empty()
        || provinces.len() != 34
        || prefectures.len() < 300
    {
        return Err(format!(
            "AreaCity audit failed: {} provinces ({} missing), {} playable prefectures ({} known Taiwan polygons unavailable; unexpected missing IDs: {unexpected:?}). Run geo audit --provider areacity for details",
            provinces.len(),
            report.missing_provinces.len(),
            prefectures.len(),
            report.missing_prefectures.len()
        ));
    }

    fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let hard_regions: Vec<Region> = provinces
        .iter()
        .chain(prefectures.iter())
        .cloned()
        .collect();
    for (filename, regions) in [
        ("provinces.json", provinces.as_slice()),
        ("prefectures.json", hard_regions.as_slice()),
    ] {
        let asset = Asset {
            source: "AreaCity-JsSpider-StatsGov",
            version: VERSION,
            attribution: "AreaCity / National Geographical Names Database / Tencent Maps / Amap (upstream rights to be reviewed)",
            view_box: "0 0 1000 680",
            regions,
        };
        let bytes = serde_json::to_vec(&asset).map_err(|error| error.to_string())?;
        let destination = options.output.join(filename);
        let temporary = destination.with_extension("tmp");
        fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &destination).map_err(|error| error.to_string())?;
    }
    Ok(report)
}

#[derive(Serialize, serde::Deserialize)]
struct SourceManifest {
    version: String,
    revision: String,
    hierarchy_sha256: String,
    archive_sha256: String,
}

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

fn approved(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some_and(|host| {
            host == "github.com"
                || host == "raw.githubusercontent.com"
                || host == "release-assets.githubusercontent.com"
                || host == "objects.githubusercontent.com"
                || host.ends_with(".githubusercontent.com")
        })
}

async fn download(client: &Client, url: &str, limit: usize) -> Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err(format!("AreaCity download exceeds {limit} byte limit"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn extract(archive: &Path, target: &Path) -> Result<()> {
    for executable in ["7z", "7zz", "7za"] {
        let result = Command::new(executable)
            .arg("e")
            .arg("-y")
            .arg(format!("-o{}", target.display()))
            .arg(archive)
            .arg("ok_geo.csv")
            .output();
        match result {
            Ok(status) if status.status.success() => return Ok(()),
            Ok(status) => {
                return Err(format!(
                    "{executable} failed extracting ok_geo.csv: {}",
                    String::from_utf8_lossy(&status.stderr)
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Install 7z/7zz or manually extract ok_geo.csv into the input directory".to_owned())
}

pub async fn fetch(options: FetchOptions) -> Result<()> {
    fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let manifest_path = options.output.join("areacity.source.json");
    let hierarchy_path = options.output.join("ok_data_level3.csv");
    let archive_path = options.output.join("ok_geo.csv.7z");
    let geometry_path = options.output.join("ok_geo.csv");
    if !options.force && manifest_path.exists() {
        let manifest: SourceManifest = crate::load_json(&manifest_path)?;
        if manifest.version != VERSION || manifest.revision != REVISION {
            return Err("Cached AreaCity version differs from approved pin".to_owned());
        }
        if digest(&fs::read(&hierarchy_path).map_err(|error| error.to_string())?)
            != manifest.hierarchy_sha256
            || digest(&fs::read(&archive_path).map_err(|error| error.to_string())?)
                != manifest.archive_sha256
        {
            return Err("Cached AreaCity checksum mismatch".to_owned());
        }
        if !geometry_path.exists() {
            extract(&archive_path, &options.output)?;
        }
        return Ok(());
    }
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .redirect(Policy::custom(|attempt| {
            if attempt.previous().len() < 8 && approved(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|error| error.to_string())?;
    let hierarchy = download(&client, HIERARCHY_URL, MAX_HIERARCHY).await?;
    let archive = download(&client, GEOMETRY_URL, MAX_ARCHIVE).await?;
    if !hierarchy.starts_with(b"\xef\xbb\xbf") && !hierarchy.starts_with(b"id,") {
        return Err("Downloaded hierarchy does not look like AreaCity CSV".to_owned());
    }
    if !archive.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return Err("Downloaded boundary archive is not 7z".to_owned());
    }
    fs::write(&hierarchy_path, &hierarchy).map_err(|error| error.to_string())?;
    fs::write(&archive_path, &archive).map_err(|error| error.to_string())?;
    extract(&archive_path, &options.output)?;
    let manifest = SourceManifest {
        version: VERSION.to_owned(),
        revision: REVISION.to_owned(),
        hierarchy_sha256: digest(&hierarchy),
        archive_sha256: digest(&archive),
    };
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    println!(
        "AreaCity source: {ROOT} ({VERSION}); hierarchy SHA-256: {} / archive SHA-256: {}",
        manifest.hierarchy_sha256, manifest.archive_sha256
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_explicit_overseas_placeholder_but_rejects_unknown_provinces() {
        let mut ids: HashSet<&str> = PROVINCE_IDS.into_iter().collect();
        assert!(validate_roots(&ids).is_ok());
        ids.insert("91");
        assert!(validate_roots(&ids).is_ok());
        ids.insert("99");
        assert!(validate_roots(&ids).unwrap_err().contains("99"));
        ids.remove("99");
        ids.remove("71");
        assert!(validate_roots(&ids).unwrap_err().contains("71"));
    }

    #[test]
    fn only_pinned_unmapped_taiwan_prefectures_are_allowed() {
        let mut report = Audit {
            provider: "areacity",
            version: VERSION,
            coordinate_system: "GCJ-02",
            provinces: 34,
            prefectures: 367,
            excluded_synthetic: Vec::new(),
            excluded_non_geographic: Vec::new(),
            missing_provinces: Vec::new(),
            missing_prefectures: KNOWN_UNMAPPED_TAIWAN_PREFECTURES
                .iter()
                .map(|id| format!("{id} 台湾测试地名"))
                .collect(),
            unknown_geometry_ids: Vec::new(),
        };
        assert!(unexpected_missing_prefectures(&report).is_empty());
        report.missing_prefectures.push("7201 未知地区".into());
        assert_eq!(unexpected_missing_prefectures(&report), vec!["7201"]);
        report.missing_prefectures.pop();
        report.missing_prefectures.push("1301 石家庄市".into());
        assert_eq!(unexpected_missing_prefectures(&report), vec!["1301"]);
    }

    #[test]
    fn parses_quoted_csv_and_bom() {
        assert_eq!(
            csv_fields("1,\"a,b\",\"c\"\"d\"").unwrap(),
            vec!["1", "a,b", "c\"d"]
        );
        assert_eq!(header_indices("\u{feff}id,pid", &["pid"]).unwrap(), vec![1]);
    }

    #[test]
    fn polygons_preserve_islands_and_holes() {
        let (path, bounds) =
            polygon_path("100 30,101 30,101 31,100 30;102 32,103 32,103 33,102 32").unwrap();
        assert_eq!(path.matches('M').count(), 2);
        assert!(bounds[2] > 0.0);
        assert!(polygon_path("EMPTY").is_err());
    }

    #[test]
    fn filters_synthetic_city_levels() {
        let parent = Division {
            id: "11".into(),
            pid: "0".into(),
            deep: 0,
            name: "北京".into(),
            pinyin: "".into(),
            ext_name: "北京市".into(),
        };
        let city = Division {
            id: "1101".into(),
            pid: "11".into(),
            deep: 1,
            name: "北京".into(),
            pinyin: "".into(),
            ext_name: "北京市".into(),
        };
        assert!(is_synthetic(&city, &parent));
    }
}
