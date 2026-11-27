use std::path::Path;

use super::ArtifactError;

pub(super) fn sql_path(path: &Path) -> Result<String, ArtifactError> {
    let path = path.to_str().ok_or_else(|| ArtifactError::InvalidPath {
        path: path.to_owned(),
    })?;
    let normalized = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!("//{}", unc.replace('\\', "/"))
    } else {
        path.strip_prefix(r"\\?\")
            .unwrap_or(path)
            .replace('\\', "/")
    };
    Ok(format!("'{}'", normalized.replace('\'', "''")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_paths_escape_quotes_without_losing_unicode() {
        let path = Path::new("dictionary/学'习.duckdb");
        assert_eq!(sql_path(path).expect("valid path"), "'dictionary/学''习.duckdb'");
    }

    #[test]
    fn windows_extended_paths_are_normalized() {
        let path = Path::new(r"\\?\C:\data\hanzi.duckdb");
        assert_eq!(sql_path(path).expect("valid path"), "'C:/data/hanzi.duckdb'");
    }
}
