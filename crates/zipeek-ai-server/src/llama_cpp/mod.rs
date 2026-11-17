mod error;
mod inference;
mod loader;
mod text_decoder;

use std::sync::Once;

pub use loader::LoadedModel;

pub use error::LlamaInferenceError;
pub use inference::{generate, generate_chat};
use llama_cpp_2::{LogOptions, send_logs_to_tracing};

static INIT_LOGGING: Once = Once::new();

pub fn initialize_logging() {
    INIT_LOGGING.call_once(|| {
        send_logs_to_tracing(LogOptions::default().with_logs_enabled(false));
    });
}
