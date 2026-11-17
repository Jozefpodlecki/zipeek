use serde::Deserialize;
use web_sys::Url;

use crate::route::Route;

#[derive(Deserialize)]
pub(super) struct HeadingQuery {
    pub heading: String,
}

pub(super) fn heading_target(query: HeadingQuery) -> Option<String> {
    let heading = query.heading;
    (!heading.is_empty()
        && heading.len() <= 512
        && heading.chars().all(|ch| ch.is_alphanumeric() || ch == '-'))
    .then_some(heading)
}

pub(super) fn asset_url(relative: &str) -> Result<String, String> {
    let base = web_sys::window()
        .ok_or_else(|| "Browser window unavailable".to_string())?
        .location()
        .href()
        .map_err(|_| "Cannot read window location")?;
    Url::new_with_base(relative, &base)
        .map(|url| url.href())
        .map_err(|_| format!("Invalid documentation URL: {relative}"))
}

pub(super) fn doc_route(path: &str) -> Route {
    Route::DocsArticle {
        file: path.replace('/', "~"),
    }
}

pub(super) fn scroll_to_heading(target: &str) -> bool {
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(target))
    {
        element.scroll_into_view();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{HeadingQuery, heading_target};

    #[test]
    fn heading_targets_are_bounded() {
        assert_eq!(
            heading_target(HeadingQuery {
                heading: "你好-heading".into()
            }),
            Some("你好-heading".into())
        );
        assert_eq!(
            heading_target(HeadingQuery {
                heading: "a/b".into()
            }),
            None
        );
        assert_eq!(
            heading_target(HeadingQuery {
                heading: "a".repeat(513)
            }),
            None
        );
    }
}
