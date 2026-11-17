use std::{error::Error, fmt};

use wasm_bindgen::JsValue;
use web_sys::{Document, Element, HtmlElement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserDocumentError {
    MissingRoot,
    MissingBody,
    DomOperation {
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for BrowserDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot => write!(formatter, "Document root is unavailable"),
            Self::MissingBody => write!(formatter, "Document body is unavailable"),
            Self::DomOperation { operation, message } => {
                write!(formatter, "Could not {operation}: {message}")
            }
        }
    }
}

impl Error for BrowserDocumentError {}

impl BrowserDocumentError {
    fn dom(operation: &'static str, error: JsValue) -> Self {
        Self::DomOperation {
            operation,
            message: error.as_string().unwrap_or_else(|| format!("{error:?}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserDocument {
    document: Document,
    root: Element,
    body: HtmlElement,
}

impl BrowserDocument {
    pub fn new(document: Document) -> Result<Self, BrowserDocumentError> {
        let root = document
            .document_element()
            .ok_or(BrowserDocumentError::MissingRoot)?;
        let body = document.body().ok_or(BrowserDocumentError::MissingBody)?;
        Ok(Self {
            document,
            root,
            body,
        })
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn root(&self) -> &Element {
        &self.root
    }

    pub fn body(&self) -> &HtmlElement {
        &self.body
    }

    pub fn set_root_attribute(&self, name: &str, value: &str) -> Result<(), BrowserDocumentError> {
        self.root
            .set_attribute(name, value)
            .map_err(|error| BrowserDocumentError::dom("set root attribute", error))
    }

    pub fn remove_root_attribute(&self, name: &str) -> Result<(), BrowserDocumentError> {
        self.root
            .remove_attribute(name)
            .map_err(|error| BrowserDocumentError::dom("remove root attribute", error))
    }

    pub fn toggle_root_class(
        &self,
        name: &str,
        enabled: bool,
    ) -> Result<bool, BrowserDocumentError> {
        self.root
            .class_list()
            .toggle_with_force(name, enabled)
            .map_err(|error| BrowserDocumentError::dom("toggle root class", error))
    }

    pub fn toggle_body_class(
        &self,
        name: &str,
        enabled: bool,
    ) -> Result<bool, BrowserDocumentError> {
        self.body
            .class_list()
            .toggle_with_force(name, enabled)
            .map_err(|error| BrowserDocumentError::dom("toggle body class", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_document_parts_have_distinct_errors() {
        assert_ne!(
            BrowserDocumentError::MissingRoot,
            BrowserDocumentError::MissingBody
        );
        assert!(
            BrowserDocumentError::MissingRoot
                .to_string()
                .contains("root")
        );
        assert!(
            BrowserDocumentError::MissingBody
                .to_string()
                .contains("body")
        );
    }

    #[test]
    fn dom_operations_include_the_failed_action() {
        let error = BrowserDocumentError::DomOperation {
            operation: "set root attribute",
            message: "invalid attribute".to_owned(),
        };
        assert!(error.to_string().contains("set root attribute"));
        assert!(error.to_string().contains("invalid attribute"));
    }
}
