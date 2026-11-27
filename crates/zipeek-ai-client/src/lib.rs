#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

mod client;
mod config;
mod error;
mod message;
mod prompts;
mod protocol;
mod stream;
mod structured;

pub use client::{AiClient, discover_models};
pub use error::AiClientError;
pub use message::{ChatMessage, ChatRole};
pub use prompts::{
    TranslationDirection, connectivity_messages, translation_messages, validate_source,
};
pub use stream::StreamUpdate;
pub use structured::ResponseFormat;
