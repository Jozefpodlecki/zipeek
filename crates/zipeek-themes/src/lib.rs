#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

pub mod provider;
pub mod resolver;
pub mod theme;

pub use provider::{BackgroundAsset, StaticBackgroundProvider};
pub use resolver::resolve_theme;
pub use theme::{ThemeId, ThemeRequest};
