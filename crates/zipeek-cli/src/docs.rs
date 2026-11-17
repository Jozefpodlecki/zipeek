use serde_json::{Value, json};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn build(root: &Path) -> io::Result<usize> {
    let source = root.join("docs");
    let target = root.join("crates/zipeek-web/static/docs");
    fs::create_dir_all(&target)?;
    let mut entries = Vec::new();
    collect(&source, &source, &mut entries)?;
    entries.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let index = serde_json::to_vec_pretty(&entries).map_err(io::Error::other)?;
    fs::write(target.join("index.json"), index)?;
    Ok(entries.len())
}

fn collect(root: &Path, dir: &Path, entries: &mut Vec<Value>) -> io::Result<()> {
    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();
        let kind = fs::symlink_metadata(&path)?.file_type();
        if kind.is_symlink() {
            return Err(io::Error::other(format!(
                "Docs symlink unsupported: {}",
                path.display()
            )));
        }
        if kind.is_dir() {
            collect(root, &path, entries)?;
            continue;
        }
        if !kind.is_file() {
            return Err(io::Error::other("Unsupported docs entry"));
        }
        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let parts: Vec<_> = relative
            .components()
            .map(|v| v.as_os_str().to_string_lossy().into_owned())
            .collect();
        let relative_url = parts.join("/");
        let markdown = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        let title = if markdown {
            title_from_markdown(&path)?
        } else {
            None
        }
        .unwrap_or_else(|| parts.last().cloned().unwrap_or_default());
        let search_text = if markdown {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };
        entries.push(json!({"path": relative_url, "title": title, "markdown": markdown, "search_text": search_text}));
    }
    Ok(())
}

fn title_from_markdown(path: &PathBuf) -> io::Result<Option<String>> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(|v| v.trim().to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexes_nested_documents_and_binary_assets_without_copying() {
        let temporary =
            std::env::temp_dir().join(format!("zipeek-docs-publisher-{}", std::process::id()));
        let nested = temporary.join("docs/architecture/data");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("design.md"), "# Design\nImportant content").unwrap();
        fs::write(temporary.join("docs/logo.bin"), [0_u8, 1, 255]).unwrap();
        assert_eq!(build(&temporary).unwrap(), 2);
        let published = temporary.join("crates/zipeek-web/static/docs");
        assert!(!published.join("logo.bin").exists());
        let manifest: Vec<serde_json::Value> =
            serde_json::from_slice(&fs::read(published.join("index.json")).unwrap()).unwrap();
        assert_eq!(manifest.len(), 2);
        assert_eq!(manifest[0]["title"], "Design");
        assert!(
            manifest[0]["search_text"]
                .as_str()
                .unwrap()
                .contains("Important content")
        );
        fs::remove_dir_all(temporary).unwrap();
    }

    #[test]
    fn detects_markdown_title() {
        let root = std::env::temp_dir().join(format!("zipeek-docs-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let doc = root.join("a.md");
        fs::write(&doc, "text\n# Meaningful heading\n").unwrap();
        assert_eq!(
            title_from_markdown(&doc).unwrap().as_deref(),
            Some("Meaningful heading")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
