mod error;
mod manifest;
mod verify;

pub use error::InputError;
pub use verify::{VerifiedFile, VerifiedInputs, VerifiedSource, verify};
