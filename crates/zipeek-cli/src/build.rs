use std::{
    fs, io,
    path::Path,
    process::{Command, Stdio},
};

const BINDGEN_VERSION: &str = "0.2.130";
const TARGET: &str = "wasm32-unknown-unknown";
const GENERATED_FILES: [&str; 4] = [
    "pedagogy_wasm.js",
    "pedagogy_wasm_bg.wasm",
    "pedagogy_wasm.d.ts",
    "pedagogy_wasm_bg.wasm.d.ts",
];

const WORKER_BOOTSTRAP: &str = r#"

let pending = [];
let startupFailure;

function reportInitializationFailure(data) {
  const correlationId = typeof data?.correlation_id === 'string'
    ? data.correlation_id : null;
  self.postMessage({
    version: 1,
    correlation_id: correlationId,
    error: `worker initialization failed: ${startupFailure}`,
  });
}

function duringStartup(event) {
  if (startupFailure !== undefined) {
    reportInitializationFailure(event.data);
  } else if (pending !== null) {
    pending.push(event.data);
  }
}

self.addEventListener('message', duringStartup);


import('./pedagogy_wasm.js').then(async ({ default: init, start_worker }) => {
  await init();


  start_worker(pending);
  self.removeEventListener('message', duringStartup);
  pending = null;
}).catch(error => {
  startupFailure = String(error);
  for (const message of pending ?? []) reportInitializationFailure(message);
  pending = [];
});
"#;

pub fn pedagogy_worker(root: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output_dir = root.join("crates/zipeek-web/static/pedagogy-worker");
    ensure_worker_bootstrap(&output_dir)?;
    let wasm = root.join("target/wasm32-unknown-unknown/release/pedagogy_worker.wasm");
    let staging = root.join("target/pedagogy-worker-bindgen-staging");

    fs::create_dir_all(&output_dir)?;
    remove_outputs(&output_dir)?;
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }

    let bindgen_version = tool_output(root, "wasm-bindgen", &["--version"])?;
    if !matches_bindgen_version(&bindgen_version) {
        return Err(format!(
            "expected wasm-bindgen-cli {BINDGEN_VERSION}, got {bindgen_version:?}; install with `cargo install wasm-bindgen-cli --version {BINDGEN_VERSION} --locked`"
        )
        .into());
    }

    let targets = tool_output(root, "rustup", &["target", "list", "--installed"])?;
    if !targets.lines().any(|line| line.trim() == TARGET) {
        return Err(format!(
            "Rust target {TARGET} is not installed; run `rustup target add {TARGET}`"
        )
        .into());
    }

    run_tool(
        root,
        "cargo",
        &[
            "build",
            "--locked",
            "--release",
            "--target",
            TARGET,
            "-p",
            "zipeek-web",
            "--bin",
            "pedagogy_worker",
        ],
    )?;
    require_nonempty(&wasm)?;

    fs::create_dir_all(&staging)?;
    let staged_result = (|| {
        let args = [
            "--target",
            "web",
            "--out-dir",
            staging.to_str().ok_or("non-UTF8 staging path")?,
            "--out-name",
            "pedagogy_wasm",
            wasm.to_str().ok_or("non-UTF8 WASM path")?,
        ];
        run_tool(root, "wasm-bindgen", &args)?;
        for name in GENERATED_FILES {
            require_nonempty(&staging.join(name))?;
        }

        let publish_result = (|| -> io::Result<()> {
            for name in GENERATED_FILES {
                fs::copy(staging.join(name), output_dir.join(name))?;
            }
            Ok(())
        })();
        if let Err(error) = publish_result {
            remove_outputs(&output_dir)?;
            return Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>);
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })();

    let cleanup_result = fs::remove_dir_all(&staging);
    staged_result?;
    if let Err(error) = cleanup_result {
        eprintln!("warning: could not remove worker build staging: {error}");
    }
    println!("Built pedagogy worker bindings in {}", output_dir.display());
    Ok(())
}

fn ensure_worker_bootstrap(output_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    let bootstrap = output_dir.join("worker.mjs");
    if fs::metadata(&bootstrap).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
        let existing = fs::read_to_string(&bootstrap)?;
        if existing.replace("\r\n", "\n") != WORKER_BOOTSTRAP {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "worker bootstrap differs from R348 Rust runtime: {}. Replace it with the canonical worker.mjs from this release before building",
                    bootstrap.display()
                ),
            ));
        }
        return Ok(());
    }

    fs::write(&bootstrap, WORKER_BOOTSTRAP)?;
    require_nonempty(&bootstrap)
}

fn remove_outputs(dir: &Path) -> io::Result<()> {
    for file in GENERATED_FILES {
        let file = dir.join(file);
        if file.exists() {
            fs::remove_file(file)?;
        }
    }
    Ok(())
}

fn require_nonempty(file: &Path) -> io::Result<()> {
    if fs::metadata(file).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing or empty worker build artifact: {}", file.display()),
        ))
    }
}

fn matches_bindgen_version(output: &str) -> bool {
    output.trim() == format!("wasm-bindgen {BINDGEN_VERSION}")
}

fn tool_output(root: &Path, program: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!(
            "{program} {} failed ({}): {stderr}",
            args.join(" "),
            output.status
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn run_tool(root: &Path, program: &str, args: &[&str]) -> io::Result<()> {
    println!("Running: {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{program} {} exited with {status}",
            args.join(" ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindgen_pin_is_exact_not_a_substring() {
        assert!(matches_bindgen_version("wasm-bindgen 0.2.130\n"));
        for invalid in [
            "wasm-bindgen 0.2.130-rc1",
            "wasm-bindgen 0.2.1300",
            "wasm-bindgen 0.2.131",
            "wasm-bindgen 0.2.129",
            "wasm-bindgen 1.0.2.130",
            "other-tool 0.2.130",
        ] {
            assert!(!matches_bindgen_version(invalid), "{invalid}");
        }
    }

    #[test]
    fn bootstrap_writer_produces_canonical_module() {
        let directory = std::env::temp_dir().join(format!(
            "zipeek-bootstrap-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        ensure_worker_bootstrap(&directory).unwrap();
        assert_eq!(
            std::fs::read_to_string(directory.join("worker.mjs")).unwrap(),
            WORKER_BOOTSTRAP
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn restores_missing_or_empty_bootstrap_and_rejects_incompatible_existing_content() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "zipeek-worker-bootstrap-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let bootstrap = dir.join("worker.mjs");

        ensure_worker_bootstrap(&dir).unwrap();
        assert_eq!(fs::read_to_string(&bootstrap).unwrap(), WORKER_BOOTSTRAP);

        fs::write(&bootstrap, "").unwrap();
        ensure_worker_bootstrap(&dir).unwrap();
        assert_eq!(fs::read_to_string(&bootstrap).unwrap(), WORKER_BOOTSTRAP);

        fs::write(&bootstrap, "// existing custom bootstrap\n").unwrap();
        let error = ensure_worker_bootstrap(&dir).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            fs::read_to_string(&bootstrap).unwrap(),
            "// existing custom bootstrap\n"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn artifact_pair_matches_worker_bootstrap() {
        assert_eq!(GENERATED_FILES.len(), 4);
        assert_eq!(GENERATED_FILES[0], "pedagogy_wasm.js");
        assert_eq!(GENERATED_FILES[1], "pedagogy_wasm_bg.wasm");
        assert_eq!(TARGET, "wasm32-unknown-unknown");
    }
}
