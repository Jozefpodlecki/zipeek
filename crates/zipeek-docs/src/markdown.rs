use std::collections::BTreeMap;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

use crate::{heading_id, links::resolve_destination};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHeading {
    pub level: usize,
    pub title: String,
    pub id: String,
}

fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
}

fn heading_level(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub fn document_headings(markdown: &str) -> Vec<DocumentHeading> {
    let mut headings = Vec::new();
    let mut current: Option<(usize, String)> = None;
    let mut seen = BTreeMap::<String, usize>::new();
    for event in Parser::new_ext(markdown, options()) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some((heading_level(level), String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, title)) = current.take() {
                    let root = heading_id(title.trim());
                    let root = if root.is_empty() {
                        "section".to_string()
                    } else {
                        root
                    };
                    let count = seen.entry(root.clone()).or_default();
                    let id = if *count == 0 {
                        root
                    } else {
                        format!("{root}-{count}")
                    };
                    *count += 1;
                    headings.push(DocumentHeading { level, title, id });
                }
            }
            Event::Text(text) | Event::Code(text) if current.is_some() => {
                if let Some((_, title)) = current.as_mut() {
                    title.push_str(&text);
                }
            }
            _ => {}
        }
    }
    headings
}

pub fn render_document(markdown: &str, current_path: &str, asset_base: &str) -> String {
    let headings = document_headings(markdown);
    let mut next_heading = 0;
    let mut image_depth = 0;
    let mut suppressed_links = 0;
    let mut events = Vec::new();

    for event in Parser::new_ext(markdown, options()) {
        if image_depth > 0 {
            match &event {
                Event::Start(Tag::Image { .. }) => image_depth += 1,
                Event::End(TagEnd::Image) => image_depth -= 1,
                _ => {}
            }
            continue;
        }
        if matches!(&event, Event::End(TagEnd::Link)) && suppressed_links > 0 {
            suppressed_links -= 1;
            continue;
        }
        let event = match event {
            Event::Start(mut tag) => {
                match &mut tag {
                    Tag::Heading { id, .. } => {
                        if let Some(heading) = headings.get(next_heading) {
                            *id = Some(heading.id.clone().into());
                            next_heading += 1;
                        }
                    }
                    Tag::Link { dest_url, .. } => {
                        let Some(target) = resolve_destination(current_path, dest_url, asset_base)
                        else {
                            suppressed_links += 1;
                            continue;
                        };
                        *dest_url = target.into();
                    }
                    Tag::Image { dest_url, .. } => {
                        let Some(target) = resolve_destination(current_path, dest_url, asset_base)
                        else {
                            image_depth = 1;
                            events.push(Event::Text("[Image unavailable]".into()));
                            continue;
                        };
                        if target.starts_with('#') {
                            image_depth = 1;
                            events.push(Event::Text("[Image unavailable]".into()));
                            continue;
                        }
                        *dest_url = target.into();
                    }
                    _ => {}
                }
                Event::Start(tag)
            }
            Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
            other => other,
        };
        events.push(event);
    }
    let mut rendered = String::new();
    html::push_html(&mut rendered, events.into_iter());
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://example.test/zipeek/docs/";

    #[test]
    fn renders_nested_commonmark_and_gfm() {
        let html = render_document(
            "# Heading\n\n- **Bold**\n  - ~~Removed~~\n\n| A | B |\n| --- | --- |\n| 一 | 二 |\n\n- [x] Complete\n",
            "README.md",
            BASE,
        );
        assert!(html.contains("<strong>Bold</strong>"));
        assert!(html.contains("<del>Removed</del>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
    }

    #[test]
    fn heading_ids_match_toc_and_are_unique() {
        let source = "## Chinese 中文\n\n## Chinese 中文\n\n```md\n# Not a heading\n```";
        let headings = document_headings(source);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[1].id, format!("{}-1", headings[0].id));
        let html = render_document(source, "README.md", BASE);
        for heading in headings {
            assert!(html.contains(&format!("id=\"{}\"", heading.id)));
        }
    }

    #[test]
    fn sanitizes_raw_html_and_unsafe_urls() {
        let source =
            "<script>alert(1)</script>\n\n[bad](javascript:alert) ![bad](javascript:alert)";
        let html = render_document(source, "README.md", BASE);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("href=\"javascript:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("bad"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn keeps_unsafe_link_text_without_clickable_anchor() {
        let html = render_document(
            "[unsafe](javascript:alert) [escape](../../secret.md) [safe](https://example.org)",
            "README.md",
            BASE,
        );
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("unsafe"));
        assert!(html.contains("escape"));
        assert!(html.contains("href=\"https://example.org\""));
    }

    #[test]
    fn cross_document_heading_links_use_route_queries() {
        let html = render_document(
            "[next](../project-management/tasks.md#e2e-evidence-verification)",
            "architecture/architecture.md",
            BASE,
        );
        assert!(html.contains("href=\"#/docs/project-management~tasks.md?heading=e2e-evidence-verification\""));
    }

    #[test]
    fn heading_only_links_remain_local_in_rendered_html() {
        let html = render_document("# Heading\n\n[go](#heading)", "README.md", BASE);
        assert!(html.contains("href=\"#heading\""));
        assert!(html.contains("id=\"heading\""));
    }

    #[test]
    fn handles_reference_links_and_code_without_rewriting_code() {
        let source = "[next][ref]\n\n[ref]: ../project-management/tasks.md\n\n`[literal](../no.md)`\n\n![x](figure.png)";
        let html = render_document(source, "architecture/architecture.md", BASE);
        assert!(html.contains("#/docs/project-management~tasks.md"));
        assert!(html.contains("[literal](../no.md)"));
        assert!(html.contains("https://example.test/zipeek/docs/architecture/figure.png"));
    }
}
