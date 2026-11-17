use super::HarnessResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserKind {
    Chrome,
    Firefox,
}

impl BrowserKind {
    pub fn from_environment() -> HarnessResult<Self> {
        Self::parse(std::env::var("ZIPEEK_E2E_BROWSER").ok().as_deref())
    }

    pub fn parse(value: Option<&str>) -> HarnessResult<Self> {
        match value {
            None | Some("") | Some("chrome") => Ok(Self::Chrome),
            Some("firefox") => Ok(Self::Firefox),
            Some(value) => Err(format!(
                "Unsupported ZIPEEK_E2E_BROWSER={value:?}; expected chrome or firefox"
            )
            .into()),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
        }
    }

    pub const fn supports_cdp(self) -> bool {
        matches!(self, Self::Chrome)
    }

    pub const fn supports_cdp_video(self) -> bool {
        self.supports_cdp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_chrome_without_overriding_explicit_choice() {
        assert_eq!(BrowserKind::parse(None).unwrap(), BrowserKind::Chrome);
        assert_eq!(BrowserKind::parse(Some("")).unwrap(), BrowserKind::Chrome);
        assert_eq!(
            BrowserKind::parse(Some("firefox")).unwrap(),
            BrowserKind::Firefox
        );
    }

    #[test]
    fn rejects_unknown_browser_with_actionable_message() {
        let message = BrowserKind::parse(Some("safari")).unwrap_err().to_string();
        assert!(message.contains("ZIPEEK_E2E_BROWSER"));
        assert!(message.contains("firefox"));
    }

    #[test]
    fn firefox_does_not_advertise_chrome_protocol_capabilities() {
        assert!(BrowserKind::Chrome.supports_cdp_video());
        assert!(!BrowserKind::Firefox.supports_cdp());
        assert!(!BrowserKind::Firefox.supports_cdp_video());
    }
}
