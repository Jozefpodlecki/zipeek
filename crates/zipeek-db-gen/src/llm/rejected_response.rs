use serde::Serialize;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
struct RejectedResponse<'a> {
    timestamp_unix_seconds: u64,
    model: &'a str,
    word_ids: Vec<(i32, i32)>,
    response: &'a str,
    error: String,
    diagnostics: String,
}

fn destination() -> Option<PathBuf> {
    match env::var("ZIPEEK_CLASSIFICATION_REJECTION_LOG") {
        Ok(value) if value.eq_ignore_ascii_case("off") || value.trim().is_empty() => None,
        Ok(value) => Some(PathBuf::from(value)),
        Err(_) => Some(PathBuf::from(
            "target/db-workflow/classification-rejections.jsonl",
        )),
    }
}

pub(super) fn record_details(
    model: &str,
    word_ids: &[(i32, i32)],
    response: &str,
    error: &str,
    diagnostics: &str,
) {
    let Some(path) = destination() else {
        return;
    };
    if let Err(failure) = write_record(&path, model, word_ids, response, error, diagnostics) {
        tracing::warn!(path = %path.display(), %failure, "could not write rejected classification response");
    }
}

fn write_record(
    path: &std::path::Path,
    model: &str,
    word_ids: &[(i32, i32)],
    response: &str,
    error: &str,
    diagnostics: &str,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let record = RejectedResponse {
        timestamp_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        model,
        word_ids: word_ids.to_vec(),
        response,
        error: error.to_owned(),
        diagnostics: diagnostics.to_owned(),
    };
    let mut bytes = serde_json::to_vec(&record).map_err(std::io::Error::other)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&bytes)?;
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saves_full_raw_response_as_jsonl() {
        let location = std::env::temp_dir().join(format!(
            "zipeek-rejected-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        write_record(
            &location,
            "test",
            &[(1, 2)],
            "Assistant\nreasoning\n{\"pos\":[\"noun\"]}",
            "invalid JSON",
            "finish=Length; reasoning=Some(..)",
        )
        .unwrap();
        let content = fs::read_to_string(&location).unwrap();
        let json: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert!(
            json["response"]
                .as_str()
                .unwrap()
                .starts_with("Assistant\nreasoning")
        );
        assert_eq!(json["word_ids"][0][1], 2);
        let _ = fs::remove_file(location);
    }
}
