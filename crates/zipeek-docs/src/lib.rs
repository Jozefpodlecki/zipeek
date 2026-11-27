#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

mod links;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DocumentEntry {
    pub path: String,
    pub title: String,
    pub markdown: bool,
    #[serde(default)]
    pub search_text: String,
}

pub fn is_safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|part| part == ".." || part == "." || part.is_empty())
        && !path
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '?' | '#' | '%'))
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn heading_id(title: &str) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    slug.trim_matches('-').to_owned()
}

mod markdown;
pub use markdown::{DocumentHeading, document_headings, render_document};

pub fn markdown_to_html(source: &str) -> String {
    render_document(source, "README.md", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_embedded_html() {
        assert!(!markdown_to_html("<script>alert(1)</script>").contains("<script>"));
    }
    #[test]
    fn headings_and_fenced_code() {
        let html = markdown_to_html("# Hello docs\n\n```rust\nlet x = 3;\n```\n");
        assert!(html.contains("id=\"hello-docs\""));
        assert!(html.contains("let x = 3;"));
    }
    #[test]
    fn tables_render() {
        assert!(markdown_to_html("| A | B |\n| --- | --- |\n| 1 | 2 |\n").contains("<table"));
    }
    #[test]
    fn ordered_lists_and_checkboxes() {
        let html = markdown_to_html("1. First\n2. Second\n\n- [x] Complete\n");
        assert!(html.contains("<ol"));
        assert!(html.contains("checked"));
    }
    #[test]
    fn images_escape_attributes() {
        let html = markdown_to_html("![Cat](https://example.org/cat.png)");
        assert!(html.contains("<img"));
        assert!(!markdown_to_html("![x](javascript:alert(1))").contains("<img"));
    }
    #[test]
    fn paths_reject_traversal() {
        assert!(is_safe_path("architecture/overview.md"));
        assert!(!is_safe_path("../.env"));
        assert!(!is_safe_path("docs/a\n.md"));
        assert!(!is_safe_path("docs/a\t.md"));
    }
}
