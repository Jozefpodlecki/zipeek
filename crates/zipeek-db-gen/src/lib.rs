#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]
#![allow(unused_imports)]

mod ai_workflow;
pub mod artifact;
mod cedict_classifications;
mod cli;
pub mod database;
mod generator;
pub mod import;
pub mod inputs;
pub mod llm;
mod policy;
pub mod schema;

pub use cli::run_cli;
pub use generator::{GeneratorError, PreparedGeneration, import_primary, prepare};

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod digest_tests {
    #[test]
    fn sha256_hex_uses_lowercase_64_character_digest() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
    }
}
