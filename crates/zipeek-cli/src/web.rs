#[cfg(test)]
use crate::archive::Result;
#[cfg(test)]
use std::fs;

const DUCKDB_FILES: &[&str] = &[
    "duckdb-browser.mjs",
    "duckdb-base.wasm",
    "duckdb-browser-base.worker.js",
    "arrow.mjs",
    "arrow.js",
    "DUCKDB-LICENSE.txt",
    "ARROW-LICENSE.txt",
    "ARROW-NOTICE.txt",
];
const SUMMARY_FILES: &[&str] = &[
    "summary.mjs",
    "bundle-meta.json",
    "CHARTJS-LICENSE.txt",
    "COLOR-LICENSE.txt",
    "ESBUILD-LICENSE.txt",
];
const DUCKDB_VERSION: &str = "1.33.1-dev114.0";
const CHART_VERSION: &str = "4.6.1";
const ESBUILD_VERSION: &str = "0.28.2";
const PREPARE_VERSION: &str = "1";

#[derive(Clone, Copy, Debug)]
pub enum Asset {
    Duckdb,
    Summary,
}

impl Asset {
    fn name(self) -> &'static str {
        match self {
            Self::Duckdb => "duckdb",
            Self::Summary => "summary",
        }
    }

    fn files(self) -> &'static [&'static str] {
        match self {
            Self::Duckdb => DUCKDB_FILES,
            Self::Summary => SUMMARY_FILES,
        }
    }
}

mod backgrounds;
mod fingerprint;
mod inputs;
mod install;
mod prepare;

pub fn verify_all(root: &std::path::Path) -> crate::archive::Result<()> {
    fingerprint::verify_all(root)?;
    backgrounds::verify(root)
}

pub fn verify_desktop(root: &std::path::Path) -> crate::archive::Result<()> {
    fingerprint::verify(root, Asset::Summary)?;
    backgrounds::verify(root)
}

pub use backgrounds::prepare as backgrounds_prepare;
pub use install::install;
pub use prepare::{prepare, prepare_all};

#[cfg(test)]
use prepare::{publish, rewrite_arrow};

#[cfg(test)]
mod tests;
