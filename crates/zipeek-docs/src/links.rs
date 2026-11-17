use crate::is_safe_path;

fn resolve_path(current: &str, target: &str) -> Option<String> {
    if target.starts_with('/')
        || target.contains('\\')
        || target.contains('?')
        || target.contains('%')
    {
        return None;
    }
    let mut components = current.split('/').collect::<Vec<_>>();
    components.pop();
    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            part => components.push(part),
        }
    }
    let result = components.join("/");
    is_safe_path(&result).then_some(result)
}

pub(crate) fn resolve_destination(current: &str, target: &str, asset_base: &str) -> Option<String> {
    if target.is_empty() || target.trim() != target || target.chars().any(char::is_control) {
        return None;
    }
    let lower = target.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
    {
        return (lower.starts_with("mailto:")
            || target
                .split_once("://")
                .is_some_and(|(_, host)| !host.is_empty() && !host.starts_with('/')))
        .then(|| target.to_string());
    }
    if target.starts_with('#') {
        return (target.len() > 1).then(|| target.to_string());
    }
    let (path, fragment) = target.split_once('#').unwrap_or((target, ""));
    let path = resolve_path(current, path)?;
    if path.ends_with(".md") {
        let mut route = format!("#/docs/{}", path.replace('/', "~"));
        if !fragment.is_empty() {
            if fragment.len() > 512 || !fragment.chars().all(|ch| ch.is_alphanumeric() || ch == '-')
            {
                return None;
            }
            route.push_str("?heading=");
            route.push_str(fragment);
        }
        Some(route)
    } else {
        Some(format!("{asset_base}{path}"))
    }
}

#[cfg(test)]
pub(crate) fn destination(current: &str, target: &str, asset_base: &str) -> String {
    resolve_destination(current, target, asset_base).unwrap_or_else(|| "#".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: &str = "https://example.test/zipeek/docs/";

    #[test]
    fn routes_relative_markdown_and_assets_under_base_path() {
        assert_eq!(
            destination("architecture/architecture.md", "../project-management/tasks.md", BASE),
            "#/docs/project-management~tasks.md"
        );
        assert_eq!(
            destination(
                "architecture/architecture.md",
                "../project-management/tasks.md#e2e-evidence-verification",
                BASE
            ),
            "#/docs/project-management~tasks.md?heading=e2e-evidence-verification"
        );
        assert_eq!(
            destination("architecture/sub/overview.md", "figure.png", BASE),
            "https://example.test/zipeek/docs/architecture/sub/figure.png"
        );
    }

    #[test]
    fn rejects_escaping_root_and_unsafe_urls() {
        assert_eq!(destination("README.md", "../../.env", BASE), "#");
        assert_eq!(destination("README.md", "javascript:alert", BASE), "#");
        assert_eq!(destination("README.md", "data:text/html", BASE), "#");
    }

    #[test]
    fn rejects_obfuscated_schemes_and_ambiguous_paths() {
        for target in [
            "JAVASCRIPT:alert(1)",
            " data:text/html",
            "https://",
            "https:///missing",
            "//example.org/escape",
            "../outside.md",
            "hello%2fworld.md",
            "file:///etc/passwd",
            "https://example.org\nX-Test: yes",
            "#",
            "other.md#heading&injected=true",
            "other.md#heading?injected=true",
            "other.md#heading%20bad",
        ] {
            assert_eq!(
                resolve_destination("README.md", target, BASE),
                None,
                "{target}"
            );
        }
        assert_eq!(
            resolve_destination("README.md", "HTTPS://EXAMPLE.ORG/safe", BASE),
            Some("HTTPS://EXAMPLE.ORG/safe".to_string()),
        );
    }

    #[test]
    fn preserves_cjk_and_disambiguated_heading_targets() {
        assert_eq!(
            destination("README.md", "guide.md#介绍-1", BASE),
            "#/docs/guide.md?heading=介绍-1"
        );
        assert_eq!(
            destination("README.md", "guide.md#", BASE),
            "#/docs/guide.md"
        );
    }

    #[test]
    fn preserves_external_and_heading_links() {
        for url in ["https://example.org", "#history", "mailto:hi@example.org"] {
            assert_eq!(destination("README.md", url, BASE), url);
        }
    }
}
