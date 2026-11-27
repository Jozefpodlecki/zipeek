use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::error::CliError;
use crate::*;

#[derive(Debug, Parser)]
#[command(
    name = "zipeek-cli",
    version,
    about = "Pack or restore Zipeek source archives"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Build {
        #[command(subcommand)]
        command: BuildCommand,
    },

    Pack {
        archive: Option<PathBuf>,
    },

    Unpack {
        archive: Option<PathBuf>,
    },

    Hanzi {
        #[command(subcommand)]
        command: HanziCommand,
    },

    DesktopFrontend {
        #[arg(long, conflicts_with = "release")]
        serve: bool,
        #[arg(long)]
        release: bool,
        #[arg(long, conflicts_with_all = ["serve", "release"])]
        prepare_only: bool,
    },

    DesktopSidecars {
        #[arg(long)]
        release: bool,
    },

    DbBrowseCheck,

    DbCompress,

    DbGen {
        #[command(subcommand)]
        command: crate::db_gen::Command,
    },

    DbGenerate {
        #[arg(long)]
        selected_classification_cache: bool,
    },

    E2eClean,

    PackE2e {
        #[arg(long, conflicts_with = "run")]
        all: bool,
        #[arg(long)]
        run: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        diagnostics: bool,
        #[arg(long, conflicts_with = "no_videos", hide = true)]
        include_videos: bool,
        #[arg(long)]
        no_videos: bool,
        #[arg(long)]
        recorder: Option<PathBuf>,
    },

    DbInstall {
        #[arg(long)]
        require_enrichment: bool,
    },

    Geo {
        #[command(subcommand)]
        command: GeoCommand,
    },

    Web {
        #[command(subcommand)]
        command: WebCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum BuildCommand {
    PedagogyWorker,
    Docs,

    VerifyAssets {
        #[arg(long)]
        require_worker: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum GeoCommand {
    Fetch {
        #[arg(long, default_value = "geoboundaries")]
        provider: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Audit {
        #[arg(long, default_value = "areacity")]
        provider: String,
        #[arg(long)]
        input: PathBuf,
    },
    Build {
        #[arg(long, default_value = "geoboundaries")]
        provider: String,
        #[arg(long)]
        input: Option<PathBuf>,
        #[arg(long)]
        provinces: Option<PathBuf>,
        #[arg(long)]
        prefectures: Option<PathBuf>,
        #[arg(long)]
        catalog: Option<PathBuf>,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        attribution: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum HanziCommand {
    Prepare {
        #[arg(long)]
        force: bool,
    },
    PrepareReferences {
        input: PathBuf,
        output: PathBuf,
        license: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum WebCommand {
    Install,
    Prepare,
    Verify,
    PrepareDuckdb,
    PrepareSummary,
    PrepareBackgrounds,
}

pub async fn run(args: Cli) -> Result<(), CliError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(CliError::Workspace)?;
    match args.command {
        Command::Build { command } => match command {
            BuildCommand::PedagogyWorker => build::pedagogy_worker(&root)
                .map_err(|error| CliError::command("build pedagogy-worker", error)),
            BuildCommand::VerifyAssets { require_worker } => {
                asset_preflight::verify(&root, require_worker)
                    .map(|summary| {
                        println!(
                            "Verified {} docs and worker assets ({})",
                            summary.docs_count,
                            if require_worker { "deploy" } else { "source" }
                        )
                    })
                    .map_err(|error| CliError::command("build verify-assets", error.into()))
            }
            BuildCommand::Docs => docs::build(&root)
                .map(|count| println!("Indexed {count} documentation files for Trunk copy-dir"))
                .map_err(|error| CliError::command("build docs", error.into())),
        },
        Command::Pack { archive } => {
            let archive = archive.unwrap_or_else(|| {
                let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                root.join(format!(
                    "target/zipeek-source-{}-{timestamp}.zip",
                    env!("CARGO_PKG_VERSION")
                ))
            });
            archive::pack(&root, &archive)
                .await
                .map_err(|error| CliError::command("pack", error))
        }
        Command::Unpack { archive } => {
            let archive = match archive {
                Some(archive) => archive,
                None => latest_download_release()?,
            };
            println!("Installing source archive: {}", archive.display());
            archive::unpack(&root, &archive)
                .await
                .map_err(|error| CliError::command("unpack", error))
        }
        Command::Hanzi { command } => match command {
            HanziCommand::Prepare { force } => {
                let count = hanzi::prepare_from_upstream(&root, force)
                    .map_err(|error| CliError::command("hanzi prepare", error))?;
                println!("Ready: {count} Hanzi reference assets");
                Ok(())
            }
            HanziCommand::PrepareReferences {
                input,
                output,
                license,
            } => {
                let count = hanzi::prepare_references(&input, &output, &license)
                    .map_err(|error| CliError::command("hanzi prepare-references", error))?;
                println!("Packaged {count} Hanzi reference assets");
                Ok(())
            }
        },
        Command::DesktopFrontend {
            serve,
            release,
            prepare_only,
        } => desktop_frontend::run(&root, serve, release, prepare_only)
            .map_err(|error| CliError::command("desktop-frontend", error)),
        Command::DesktopSidecars { release } => desktop_sidecars::stage(&root, release)
            .map_err(|error| CliError::command("desktop-sidecars", error.into())),
        Command::DbGen { command } => {
            if crate::db_gen::run_cli(command) == std::process::ExitCode::SUCCESS {
                Ok(())
            } else {
                Err(CliError::command("db-gen", std::io::Error::other("generator command failed").into()))
            }
        }
        Command::DbBrowseCheck => database::browse_check(&root)
            .map_err(|error| CliError::command("db-browse-check", error)),
        Command::DbCompress => database::compress_latest(&root)
            .map_err(|error| CliError::command("db-compress", error)),
        Command::DbGenerate {
            selected_classification_cache,
        } => generation::generate(&root, selected_classification_cache)
            .map_err(|error| CliError::command("db-generate", error)),
        Command::E2eClean => evidence::clean(&root)
            .map(|_| ())
            .map_err(|error| CliError::command("e2e-clean", error)),
        Command::PackE2e {
            all,
            run,
            output,
            diagnostics,
            include_videos: _,
            no_videos,
            recorder,
        } => evidence::pack(
            &root,
            run.as_deref(),
            output.as_deref(),
            diagnostics,
            recorder.as_deref(),
            all,
            !no_videos,
        )
        .map_err(|error| CliError::command("pack-e2e", error)),
        Command::DbInstall { require_enrichment } => database::install(&root, require_enrichment)
            .map_err(|error| CliError::command("db-install", error)),
        Command::Geo { command } => match command {
            GeoCommand::Fetch {
                provider,
                output,
                force,
            } => {
                match provider.as_str() {
                    "areacity" => zipeek_geo_assets::areacity::fetch(
                        zipeek_geo_assets::areacity::FetchOptions { output, force },
                    )
                    .await
                    .map_err(|error| {
                        CliError::command("geo fetch", std::io::Error::other(error).into())
                    }),
                    "geoboundaries" => {
                        zipeek_geo_assets::fetch(zipeek_geo_assets::FetchOptions { output, force })
                            .await
                            .map_err(|error| {
                                CliError::command("geo fetch", std::io::Error::other(error).into())
                            })
                    }
                    _ => Err(CliError::command(
                        "geo fetch",
                        std::io::Error::other(format!("Unknown geography provider {provider}"))
                            .into(),
                    )),
                }
            }
            GeoCommand::Audit { provider, input } => match provider.as_str() {
                "areacity" => {
                    let report = zipeek_geo_assets::areacity::audit(&input).map_err(|error| {
                        CliError::command("geo audit", std::io::Error::other(error).into())
                    })?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .map_err(|error| CliError::command("geo audit", error.into()))?
                    );
                    Ok(())
                }
                _ => Err(CliError::command(
                    "geo audit",
                    std::io::Error::other(format!("Unknown geography provider {provider}")).into(),
                )),
            },
            GeoCommand::Build {
                provider,
                input,
                provinces,
                prefectures,
                catalog,
                output,
                source,
                version,
                attribution,
            } => match provider.as_str() {
                "areacity" => {
                    let input = input.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--input is required for AreaCity").into(),
                        )
                    })?;
                    let report = zipeek_geo_assets::areacity::build(
                        &zipeek_geo_assets::areacity::BuildOptions { input, output },
                    )
                    .map_err(|error| {
                        CliError::command("geo build", std::io::Error::other(error).into())
                    })?;
                    println!(
                        "AreaCity assets: {} provinces, {} playable prefectures",
                        report.provinces, report.prefectures
                    );
                    if !report.missing_prefectures.is_empty() {
                        eprintln!(
                            "AreaCity warning: {} Taiwan prefecture records have no upstream polygons and are excluded from Hard mode (see geo audit)",
                            report.missing_prefectures.len()
                        );
                    }
                    Ok(())
                }
                "geoboundaries" => zipeek_geo_assets::build(&zipeek_geo_assets::BuildOptions {
                    provinces: provinces.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--provinces is required").into(),
                        )
                    })?,
                    prefectures,
                    catalog: catalog.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--catalog is required").into(),
                        )
                    })?,
                    output,
                    source: source.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--source is required").into(),
                        )
                    })?,
                    version: version.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--version is required").into(),
                        )
                    })?,
                    attribution: attribution.ok_or_else(|| {
                        CliError::command(
                            "geo build",
                            std::io::Error::other("--attribution is required").into(),
                        )
                    })?,
                })
                .map_err(|error| {
                    CliError::command("geo build", std::io::Error::other(error).into())
                }),
                _ => Err(CliError::command(
                    "geo build",
                    std::io::Error::other(format!("Unknown geography provider {provider}")).into(),
                )),
            },
        },
        Command::Web { command } => match command {
            WebCommand::Install => {
                web::install(&root).map_err(|error| CliError::command("web install", error))
            }
            WebCommand::Prepare => {
                web::prepare_all(&root).map_err(|error| CliError::command("web prepare", error))
            }
            WebCommand::Verify => {
                web::verify_all(&root).map_err(|error| CliError::command("web verify", error))
            }
            WebCommand::PrepareDuckdb => web::prepare(&root, web::Asset::Duckdb)
                .map_err(|error| CliError::command("web prepare-duckdb", error)),
            WebCommand::PrepareSummary => web::prepare(&root, web::Asset::Summary)
                .map_err(|error| CliError::command("web prepare-summary", error)),
            WebCommand::PrepareBackgrounds => web::backgrounds_prepare(&root)
                .map_err(|error| CliError::command("web prepare-backgrounds", error)),
        },
    }
}

fn latest_download_release() -> Result<PathBuf, CliError> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| {
            CliError::release_discovery(
                PathBuf::from("Downloads"),
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "USERPROFILE or HOME is not set; pass an explicit ZIP path",
                )
                .into(),
            )
        })?;
    let downloads = PathBuf::from(home).join("Downloads");
    latest_release_in(&downloads).map_err(|error| CliError::release_discovery(downloads, error))
}

fn release_key(name: &str) -> Option<(u64, bool, String)> {
    let rest = name
        .strip_prefix("zipeek-R")
        .or_else(|| name.strip_prefix("zipeek-r"))?
        .strip_suffix(".zip")?;
    let (number, remaining) = rest.split_once('-')?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let number = number.parse::<u64>().ok().filter(|number| *number > 0)?;
    let (description_and_date, timestamp) = remaining.rsplit_once('-')?;
    let (description_with_marker, date) = description_and_date.rsplit_once('-')?;
    let (checkpoint, description) = match description_with_marker.strip_prefix("WIP-") {
        Some(description) => (true, description),
        None => match description_with_marker.strip_suffix("-WIP") {
            Some(description) => (true, description),
            None => (false, description_with_marker),
        },
    };
    if description.is_empty()
        || !description
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || date.len() != 8
        || !date.bytes().all(|byte| byte.is_ascii_digit())
        || timestamp.len() != 6
        || !timestamp.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let year = date[..4].parse::<i32>().ok()?;
    let month = date[4..6].parse::<u32>().ok()?;
    let day = date[6..8].parse::<u32>().ok()?;
    let hour = timestamp[..2].parse::<u32>().ok()?;
    let minute = timestamp[2..4].parse::<u32>().ok()?;
    let second = timestamp[4..6].parse::<u32>().ok()?;
    chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    chrono::NaiveTime::from_hms_opt(hour, minute, second)?;
    Some((number, checkpoint, format!("{date}-{timestamp}")))
}

fn latest_release_in(downloads: &std::path::Path) -> archive::Result<PathBuf> {
    let mut matches = Vec::new();
    for entry in std::fs::read_dir(downloads)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some((number, checkpoint, timestamp)) = release_key(&name) {
            matches.push((
                number,
                !checkpoint,
                timestamp,
                entry.metadata()?.modified()?,
                name.into_owned(),
                entry.path(),
            ));
        }
    }
    matches.sort();
    matches
        .pop()
        .map(|(_, _, _, _, _, path)| path)
        .ok_or_else(|| {
            format!(
                "No numbered release (zipeek-R<number>-description-YYYYMMDD-HHMMSS.zip) or WIP checkpoint (zipeek-R<number>-WIP-description-YYYYMMDD-HHMMSS.zip) found in {}. Download a source release/checkpoint or pass an explicit ZIP path",
                downloads.display()
            )
            .into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_parses_subcommands_and_optional_archive() {
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "build", "pedagogy-worker"])
                .unwrap()
                .command,
            Command::Build {
                command: BuildCommand::PedagogyWorker
            }
        ));
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "build", "verify-assets", "--require-worker"])
                .unwrap()
                .command,
            Command::Build {
                command: BuildCommand::VerifyAssets {
                    require_worker: true
                }
            }
        ));
        assert!(Cli::try_parse_from(["zipeek-cli", "build", "unknown"]).is_err());
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "pack", "example.zip"])
                .unwrap()
                .command,
            Command::Pack { archive: Some(_) }
        ));
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "unpack"])
                .unwrap()
                .command,
            Command::Unpack { archive: None }
        ));
        assert!(Cli::try_parse_from(["zipeek-cli", "bad"]).is_err());
    }

    #[test]
    fn pack_e2e_records_videos_by_default_with_opt_out() {
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "pack-e2e", "--all"])
                .unwrap()
                .command,
            Command::PackE2e {
                no_videos: false,
                ..
            }
        ));
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "pack-e2e", "--all", "--no-videos"])
                .unwrap()
                .command,
            Command::PackE2e {
                no_videos: true,
                ..
            }
        ));
        assert!(
            Cli::try_parse_from(["zipeek-cli", "pack-e2e", "--no-videos", "--include-videos"])
                .is_err()
        );
    }

    #[test]
    fn release_name_accepts_valid_utc_timestamps() {
        assert_eq!(
            release_key("zipeek-R240-practice-setup-test-reliability-20261107-144814.zip"),
            Some((240, false, "20261107-144814".to_owned()))
        );
        assert_eq!(
            release_key("zipeek-R241-release-check-20261107-145210.zip"),
            Some((241, false, "20261107-145210".to_owned()))
        );
        assert_eq!(
            release_key("zipeek-R281-WIP-source-checkpoint-20261108-195023.zip"),
            Some((281, true, "20261108-195023".to_owned()))
        );
        for name in [
            "zipeek-r308-exercise-coverage-ai-server-WIP-20261110-162808.zip",
            "zipeek-R308-exercise-coverage-ai-server-WIP-20261110-162808.zip",
            "zipeek-r308-WIP-exercise-coverage-ai-server-20261110-162808.zip",
            "zipeek-R308-WIP-exercise-coverage-ai-server-20261110-162808.zip",
        ] {
            assert_eq!(
                release_key(name),
                Some((308, true, "20261110-162808".to_owned())),
                "expected support for {name}"
            );
        }
        for name in [
            "zipeek-source-0.1.0-20261106-164530.zip",
            "zipeek-v0.1.0-20261106.zip",
            "zipeek-R0-test-20261106.zip",
            "zipeek-Rx-test-20261106.zip",
            "zipeek-R192--20261106.zip",
            "zipeek-R192-test-2026116.zip",
            "zipeek-R192-test-20261106.zip.tmp",
            "zipeek-R192-test-20261106.zip",
            "zipeek-R192-test-20261106-246001.zip",
            "zipeek-R192-test-20260230-120000.zip",
            "zipeek-R192-test-20261106-12453.zip",
            "zipeek-R192-test-20261106-120000.tar.gz",
        ] {
            assert!(release_key(name).is_none(), "unexpectedly accepted {name}");
        }
    }

    #[test]
    fn database_make_tasks_use_the_cli_binary() {
        assert!(matches!(
            Cli::try_parse_from(["zipeek-cli", "db-browse-check"])
                .unwrap()
                .command,
            Command::DbBrowseCheck
        ));
        let makefile = include_str!("../../../Makefile.toml");
        assert!(!makefile.contains("db-workflow.cjs"));
        assert!(!makefile.contains("scripts/"));
        assert!(!makefile.contains("command = \"node\""));
        assert!(makefile.contains("target/release/zipeek-cli.exe"));
        assert!(makefile.contains("\"db-browse-check\""));
        assert!(!makefile.contains("\"-p\", \"zipeek-db-gen\""));
        for command in [
            "\"db-gen\", \"check\"",
            "\"db-gen\", \"ai-workflow\", \"--smoke\"",
            "\"db-gen\", \"ai-workflow\", \"--classify-only\"",
        ] {
            assert!(makefile.contains(command), "missing CLI task: {command}");
        }
    }

    #[test]
    fn downloads_selection_prefers_release_number_then_date() -> archive::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let directory = std::env::temp_dir().join(format!(
            "zipeek-releases-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&directory)?;
        let result = (|| -> archive::Result<()> {
            for name in [
                "zipeek-R191-earlier-20261106-120000.zip",
                "zipeek-R192-old-20261105-220000.zip",
                "zipeek-R192-build-pipeline-20261106-140000.zip",
                "zipeek-R192-build-pipeline-20261106-150000.zip",
                "zipeek-source-0.1.0-20261106-164530.zip",
                "zipeek-R281-WIP-source-checkpoint-20261108-195023.zip",
                "zipeek-R281-reviewed-20261108-180000.zip",
            ] {
                std::fs::write(directory.join(name), b"fixture")?;
            }
            let selected = latest_release_in(&directory)?;
            assert_eq!(
                selected.file_name().unwrap(),
                "zipeek-R281-reviewed-20261108-180000.zip"
            );
            Ok(())
        })();
        std::fs::remove_dir_all(&directory)?;
        result
    }

    #[test]
    fn downloads_selects_checkpoint_if_no_completed_release_of_same_number() -> archive::Result<()>
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let directory = std::env::temp_dir().join(format!(
            "zipeek-checkpoints-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&directory)?;
        let result = (|| -> archive::Result<()> {
            for name in [
                "zipeek-R280-previous-20261108-180000.zip",
                "zipeek-R281-WIP-source-checkpoint-20261108-195023.zip",
            ] {
                std::fs::write(directory.join(name), b"fixture")?;
            }
            let selected = latest_release_in(&directory)?;
            assert_eq!(
                selected.file_name().unwrap(),
                "zipeek-R281-WIP-source-checkpoint-20261108-195023.zip"
            );
            Ok(())
        })();
        std::fs::remove_dir_all(&directory)?;
        result
    }

    #[test]
    fn downloads_discover_lowercase_and_suffix_wip() -> archive::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let directory = std::env::temp_dir().join(format!(
            "zipeek-wip-name-variants-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&directory)?;
        let result = (|| -> archive::Result<()> {
            let name = "zipeek-r308-exercise-coverage-ai-server-WIP-20261110-162808.zip";
            std::fs::write(directory.join(name), b"fixture")?;
            assert_eq!(latest_release_in(&directory)?.file_name().unwrap(), name);
            Ok(())
        })();
        std::fs::remove_dir_all(&directory)?;
        result
    }

    #[test]
    fn downloads_without_a_release_gives_actionable_error() -> archive::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let directory = std::env::temp_dir().join(format!(
            "zipeek-empty-releases-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        std::fs::create_dir_all(&directory)?;
        let result = latest_release_in(&directory);
        std::fs::remove_dir_all(&directory)?;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No numbered release")
        );
        Ok(())
    }
}

#[cfg(test)]
mod geo_command_tests {
    use super::*;

    #[test]
    fn parses_geo_fetch_options() {
        let cli = Cli::try_parse_from([
            "zipeek-cli",
            "geo",
            "fetch",
            "--output",
            "geo-cache",
            "--force",
        ])
        .unwrap();
        match cli.command {
            Command::Geo {
                command: GeoCommand::Fetch { output, force, .. },
            } => {
                assert_eq!(output, PathBuf::from("geo-cache"));
                assert!(force);
            }
            _ => panic!("expected geo fetch command"),
        }
    }

    #[test]
    fn parses_geo_build_options() {
        let cli = Cli::try_parse_from([
            "zipeek-cli",
            "geo",
            "build",
            "--provinces",
            "provinces.geojson",
            "--catalog",
            "catalog.json",
            "--output",
            "assets",
            "--source",
            "provider",
            "--version",
            "v1",
            "--attribution",
            "credit",
        ])
        .unwrap();
        match cli.command {
            Command::Geo {
                command:
                    GeoCommand::Build {
                        prefectures,
                        source,
                        ..
                    },
            } => {
                assert!(prefectures.is_none());
                assert_eq!(source.as_deref(), Some("provider"));
            }
            _ => panic!("expected geo build command"),
        }
    }

    #[test]
    fn parses_areacity_fetch_audit_and_build() {
        let fetch = Cli::try_parse_from([
            "zipeek-cli",
            "geo",
            "fetch",
            "--provider",
            "areacity",
            "--output",
            "cache",
        ])
        .unwrap();
        assert!(
            matches!(fetch.command, Command::Geo { command: GeoCommand::Fetch { provider, .. } } if provider == "areacity")
        );
        let audit = Cli::try_parse_from([
            "zipeek-cli",
            "geo",
            "audit",
            "--provider",
            "areacity",
            "--input",
            "cache",
        ])
        .unwrap();
        assert!(
            matches!(audit.command, Command::Geo { command: GeoCommand::Audit { input, .. } } if input == PathBuf::from("cache"))
        );
        let build = Cli::try_parse_from([
            "zipeek-cli",
            "geo",
            "build",
            "--provider",
            "areacity",
            "--input",
            "cache",
            "--output",
            "output",
        ])
        .unwrap();
        assert!(
            matches!(build.command, Command::Geo { command: GeoCommand::Build { provider, input: Some(_), .. } } if provider == "areacity")
        );
    }

    #[test]
    fn geo_errors_keep_their_source() {
        let error = CliError::command("geo fetch", std::io::Error::other("download failed").into());
        assert_eq!(error.to_string(), "geo fetch failed");
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "download failed"
        );
    }
}
