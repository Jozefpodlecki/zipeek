use thiserror::Error;
use web_sys::Document;

#[derive(Debug, Error)]
pub enum BaseBundleError {
    #[error("Document base URL is unavailable")]
    MissingBaseUri,
    #[error("Invalid asset base URL: {0}")]
    InvalidAssetBase(String),
    #[error("Invalid bundle asset URL for {asset}: {reason}")]
    InvalidAssetUrl { asset: &'static str, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseBundleInner {
    pub main_module: String,
    pub main_worker: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseBundle {
    pub base: BaseBundleInner,
}

impl BaseBundle {
    pub fn new(asset_base: &str, document: &Document) -> Result<Self, BaseBundleError> {
        let base_uri = js_sys::Reflect::get(document.as_ref(), &"baseURI".into())
            .ok()
            .and_then(|value| value.as_string())
            .ok_or(BaseBundleError::MissingBaseUri)?;
        let root = web_sys::Url::new_with_base(asset_base, &base_uri)
            .map_err(|error| BaseBundleError::InvalidAssetBase(format!("{error:?}")))?;
        let url = |asset: &'static str| -> Result<String, BaseBundleError> {
            web_sys::Url::new_with_base(asset, &root.href())
                .map(|url| url.href())
                .map_err(|error| BaseBundleError::InvalidAssetUrl {
                    asset,
                    reason: format!("{error:?}"),
                })
        };
        Ok(Self {
            base: BaseBundleInner {
                main_module: url("duckdb-base.wasm")?,
                main_worker: url("duckdb-browser-base.worker.js")?,
            },
        })
    }

    pub fn module_url(&self, filename: &'static str) -> Result<String, BaseBundleError> {
        web_sys::Url::new_with_base(filename, &self.base.main_module)
            .map(|url| url.href())
            .map_err(|error| BaseBundleError::InvalidAssetUrl {
                asset: filename,
                reason: format!("{error:?}"),
            })
    }

    pub fn browser_default(document: &Document) -> Result<Self, BaseBundleError> {
        Self::new("vendor/duckdb/", document)
    }
}
