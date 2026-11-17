use super::HarnessResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppRoute {
    path: String,
    query: Vec<(String, String)>,
}

impl AppRoute {
    pub fn new(path: impl Into<String>) -> HarnessResult<Self> {
        let path = path.into();
        let contains_fragment_or_query = path
            .chars()
            .any(|character| matches!(character, '#' | '?' | '\r' | '\n'));
        if !path.starts_with('/')
            || path.starts_with("//")
            || contains_fragment_or_query
            || path.chars().any(char::is_control)
        {
            return Err(format!("Invalid application route path: {path:?}").into());
        }
        Ok(Self {
            path,
            query: Vec::new(),
        })
    }

    pub fn with_query(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> HarnessResult<Self> {
        let key = key.into();
        let value = value.into();
        if key.is_empty() {
            return Err("Route query key must not be empty".into());
        }
        self.query
            .push((encode_component(&key), encode_component(&value)));
        Ok(self)
    }

    pub fn from_deep_link(link: &str) -> HarnessResult<Self> {
        let (path, query) = link.split_once('?').unwrap_or((link, ""));
        let mut route = Self::new(path)?;
        if !query.is_empty() {
            for parameter in query.split('&') {
                let (key, value) = parameter.split_once('=').unwrap_or((parameter, ""));
                if key.is_empty() {
                    return Err("Route query key must not be empty".into());
                }
                route.query.push((key.to_owned(), value.to_owned()));
            }
        }
        Ok(route)
    }

    pub fn hash_path(&self) -> String {
        let mut path = self.path.clone();
        if !self.query.is_empty() {
            path.push('?');
            for (index, (key, value)) in self.query.iter().enumerate() {
                if index > 0 {
                    path.push('&');
                }
                path.push_str(key);
                path.push('=');
                path.push_str(value);
            }
        }
        path
    }

    pub fn url(&self, base_url: &str) -> String {
        format!("{}/#{}", base_url.trim_end_matches('/'), self.hash_path())
    }
}

fn encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_link_round_trips_encoded_queries() {
        let route = AppRoute::from_deep_link("/browse?query=%E4%B8%AD%E6%96%87&level=1").unwrap();
        assert_eq!(
            route.url("http://localhost:8080/zipeek/"),
            "http://localhost:8080/zipeek/#/browse?query=%E4%B8%AD%E6%96%87&level=1"
        );
    }

    #[test]
    fn reject_invalid_route_and_query_keys() {
        for path in [
            "browse",
            "//example.com",
            "/bad#fragment",
            "/bad\npath",
            "/path?query=raw",
        ] {
            assert!(AppRoute::new(path).is_err());
        }
        assert!(
            AppRoute::new("/browse")
                .unwrap()
                .with_query("", "value")
                .is_err()
        );
    }

    #[test]
    fn typed_options_produce_repeatable_urls() {
        let route = AppRoute::new("/games/classifier-market")
            .unwrap()
            .with_query("source", "hsk")
            .unwrap()
            .with_query("level", "3")
            .unwrap();
        assert_eq!(
            route.hash_path(),
            "/games/classifier-market?source=hsk&level=3"
        );
        let unicode = AppRoute::new("/browse")
            .unwrap()
            .with_query("query", "中文 & ok")
            .unwrap();
        assert_eq!(
            unicode.hash_path(),
            "/browse?query=%E4%B8%AD%E6%96%87%20%26%20ok"
        );
    }

    #[test]
    fn root_and_deployment_base_are_stable() {
        assert_eq!(
            AppRoute::new("/").unwrap().url("http://127.0.0.1:3000/"),
            "http://127.0.0.1:3000/#/"
        );
        assert_eq!(
            AppRoute::new("/practice")
                .unwrap()
                .url("http://127.0.0.1:3000/zipeek/"),
            "http://127.0.0.1:3000/zipeek/#/practice"
        );
    }
}
