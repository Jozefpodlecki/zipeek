mod archive;
mod asset_preflight;
mod build;
mod command;
mod database;
mod desktop_sidecars;
mod docs;
mod error;
mod evidence;
mod generation;
mod hanzi;
mod web;

use std::{error::Error, process::ExitCode};

use clap::Parser;

use crate::command::{Cli, run};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::try_init().ok();
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("zipeek-cli: {error}");
            let mut source = error.source();
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
            ExitCode::FAILURE
        }
    }
}
