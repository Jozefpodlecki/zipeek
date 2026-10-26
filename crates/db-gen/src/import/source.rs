use serde::Deserialize;

use super::ImportError;

#[derive(Debug, Deserialize)]
pub(super) struct SourceEntry {
    pub simplified: String,
    #[serde(default)]
    pub level: Vec<String>,
    pub frequency: Option<i64>,
    #[serde(default)]
    pub pos: Vec<String>,
    #[serde(default)]
    pub forms: Vec<SourceForm>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SourceForm {
    pub traditional: Option<String>,
    #[serde(default)]
    pub transcriptions: Transcriptions,
    #[serde(default)]
    pub meanings: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct Transcriptions {
    pub pinyin: Option<String>,
    pub numeric: Option<String>,
}

pub(super) fn parse(bytes: &[u8]) -> Result<Vec<(SourceEntry, String)>, ImportError> {
    let entries: Vec<SourceEntry> = serde_json::from_slice(bytes)?;
    let mut originals = Vec::with_capacity(entries.len());
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0;
    let mut start = 0;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'{' => {
                if depth == 0 {
                    start = index;
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let raw = std::str::from_utf8(&bytes[start..=index])
                        .map_err(|_| ImportError::Provenance)?;
                    originals.push(raw.to_owned());
                }
            }
            _ => {}
        }
    }
    if originals.len() != entries.len() {
        return Err(ImportError::Provenance);
    }
    Ok(entries.into_iter().zip(originals).collect())
}
