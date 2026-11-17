use std::{error::Error, fmt};

#[derive(Debug)]
pub enum HarnessError {
    Io(std::io::Error),
    Browser(Box<thirtyfour::error::WebDriverError>),
    DevTools(Box<thirtyfour::cdp::CdpError>),
    Task(tokio::task::JoinError),
    Clock(std::time::SystemTimeError),
    Lifecycle(String),
    Assertion(super::assertions::AssertionFailure),
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Harness I/O: {error}"),
            Self::Browser(error) => write!(formatter, "WebDriver: {error}"),
            Self::DevTools(error) => write!(formatter, "Chrome DevTools: {error}"),
            Self::Task(error) => write!(formatter, "Harness task: {error}"),
            Self::Clock(error) => write!(formatter, "Artifact clock: {error}"),
            Self::Lifecycle(message) => formatter.write_str(message),
            Self::Assertion(failure) => write!(formatter, "{failure}"),
        }
    }
}

impl Error for HarnessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Browser(error) => Some(error),
            Self::DevTools(error) => Some(error),
            Self::Task(error) => Some(error),
            Self::Clock(error) => Some(error),
            Self::Lifecycle(_) => None,
            Self::Assertion(_) => None,
        }
    }
}

impl From<std::io::Error> for HarnessError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<thirtyfour::error::WebDriverError> for HarnessError {
    fn from(error: thirtyfour::error::WebDriverError) -> Self {
        Self::Browser(Box::new(error))
    }
}

impl From<thirtyfour::cdp::CdpError> for HarnessError {
    fn from(error: thirtyfour::cdp::CdpError) -> Self {
        Self::DevTools(Box::new(error))
    }
}

impl From<tokio::task::JoinError> for HarnessError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::Task(error)
    }
}

impl From<std::time::SystemTimeError> for HarnessError {
    fn from(error: std::time::SystemTimeError) -> Self {
        Self::Clock(error)
    }
}

impl From<String> for HarnessError {
    fn from(message: String) -> Self {
        Self::Lifecycle(message)
    }
}

impl From<&str> for HarnessError {
    fn from(message: &str) -> Self {
        Self::Lifecycle(message.to_owned())
    }
}

impl From<super::assertions::AssertionFailure> for HarnessError {
    fn from(failure: super::assertions::AssertionFailure) -> Self {
        Self::Assertion(failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_errors_keep_context_without_fake_cause() {
        let error = HarnessError::from("Route /browse failed: query never settled");
        assert!(error.to_string().contains("/browse"));
        assert!(error.source().is_none());
    }

    #[test]
    fn assertion_errors_preserve_expected_and_actual_values() {
        let failure = super::super::assertions::AssertionFailure {
            kind: super::super::assertions::AssertionKind::Equal,
            expression: "page == 2",
            actual: "1".to_owned(),
            expected: "2".to_owned(),
            context: Some("Browse Next after database reload".to_owned()),
        };
        let error = HarnessError::from(failure).to_string();
        assert!(error.contains("page == 2"));
        assert!(error.contains("Browse Next after database reload"));
    }
}
