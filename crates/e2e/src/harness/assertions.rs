use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionKind {
    Condition,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionFailure {
    pub kind: AssertionKind,
    pub expression: &'static str,
    pub actual: String,
    pub expected: String,
    pub context: Option<String>,
}

impl fmt::Display for AssertionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E2E assertion: {} ({:?}); actual {}; expected {}", self.expression, self.kind, self.actual, self.expected)?;
        if let Some(context) = &self.context {
            write!(f, "; {context}")?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! e2e_ensure {
    ($condition:expr $(, $($context:tt)+)?) => {
        if !($condition) {
            return Err($crate::HarnessError::from($crate::AssertionFailure {
                kind: $crate::AssertionKind::Condition,
                expression: stringify!($condition),
                actual: "false".to_owned(),
                expected: "true".to_owned(),
                context: $crate::e2e_assertion_context!($($($context)+)?),
            }));
        }
    };
}

#[macro_export]
macro_rules! e2e_ensure_eq {
    ($actual:expr, $expected:expr $(, $($context:tt)+)?) => {
        {
            let actual = $actual;
            let expected = $expected;
            if actual != expected {
                return Err($crate::HarnessError::from($crate::AssertionFailure {
                    kind: $crate::AssertionKind::Equal,
                    expression: concat!(stringify!($actual), " == ", stringify!($expected)),
                    actual: format!("{actual:?}"),
                    expected: format!("{expected:?}"),
                    context: $crate::e2e_assertion_context!($($($context)+)?),
                }));
            }
        }
    };
}

#[macro_export]
macro_rules! e2e_assertion_context {
    () => { None };
    ($($context:tt)+) => { Some(format!($($context)+)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertions_return_context_without_panicking() {
        fn failing() -> crate::HarnessResult<()> {
            crate::e2e_ensure_eq!(2 + 2, 5, "exercise {}", "translation");
            Ok(())
        }
        let message = failing().unwrap_err().to_string();
        assert!(message.contains("2 + 2 == 5"));
        assert!(message.contains("exercise translation"));
    }

    #[test]
    fn condition_assertions_can_fail_and_recover() {
        fn check(value: bool) -> crate::HarnessResult<()> {
            crate::e2e_ensure!(value, "not ready");
            Ok(())
        }
        assert!(check(false).is_err());
        assert!(check(true).is_ok());
    }
}
