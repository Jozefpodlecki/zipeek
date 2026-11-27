mod archive;
mod asset_preflight;
mod build;
mod command;
mod database;
mod db_gen;
mod db_ai_workflow;
mod desktop_frontend;
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
    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => match tracing_subscriber::EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(error) => {
                eprintln!("Invalid RUST_LOG filter: {error}");
                return ExitCode::FAILURE;
            }
        },
        None => tracing_subscriber::EnvFilter::new("info"),
    };
    if let Err(error) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false)
        .try_init()
    {
        eprintln!("Unable to initialize logging: {error}");
        return ExitCode::FAILURE;
    }
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
