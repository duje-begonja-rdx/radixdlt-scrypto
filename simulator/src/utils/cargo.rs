use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use cargo_toml::Manifest;
use radix_engine::types::*;
use radix_engine::utils::*;
use wasm_opt::OptimizationError;

#[derive(Debug)]
pub enum BuildError {
    NotCargoPackage(PathBuf),

    MissingPackageName,

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    CargoTargetDirectoryResolutionError,

    CargoFailure(ExitStatus),

    SchemaExtractionError(ExtractSchemaError),

    SchemaEncodeError(sbor::EncodeError),

    InvalidManifestFile(PathBuf),

    OptimizationError(OptimizationError),
}

#[derive(Debug)]
pub enum TestError {
    NotCargoPackage,

    BuildError(BuildError),

    IOError(io::Error),

    CargoFailure(ExitStatus),
}

#[derive(Debug)]
pub enum FormatError {
    BuildError(BuildError),

    IOError(io::Error),

    CargoFailure(ExitStatus),
}

fn run_cargo_build(
    manifest_path: impl AsRef<OsStr>,
    target_path: impl AsRef<OsStr>,
    trace: bool,
    no_schema: bool,
    log_level: Level,
    coverage: bool,
) -> Result<(), BuildError> {
    let mut features = String::new();
    if trace {
        features.push_str(",scrypto/trace");
    }
    if no_schema {
        features.push_str(",scrypto/no-schema");
    }
    if Level::Error <= log_level {
        features.push_str(",scrypto/log-error");
    }
    if Level::Warn <= log_level {
        features.push_str(",scrypto/log-warn");
    }
    if Level::Info <= log_level {
        features.push_str(",scrypto/log-info");
    }
    if Level::Debug <= log_level {
        features.push_str(",scrypto/log-debug");
    }
    if Level::Trace <= log_level {
        features.push_str(",scrypto/log-trace");
    }
    if coverage {
        features.push_str(",scrypto/coverage");
    }

    let rustflags = if coverage {
        "-Clto=off\x1f-Cinstrument-coverage\x1f-Zno-profiler-runtime\x1f--emit=llvm-ir".to_owned()
    } else {
        env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
    };

    let status = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .arg("--target-dir")
        .arg(target_path.as_ref())
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .args(if features.is_empty() {
            vec![]
        } else {
            vec!["--features", &features[1..]]
        })
        .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
        .status()
        .map_err(BuildError::IOError)?;
    if status.success() {
        Ok(())
    } else {
        Err(BuildError::CargoFailure(status))
    }
}

/// Gets the default cargo directory for the given crate.
/// This respects whether the crate is in a workspace.
pub fn get_default_target_directory(
    manifest_path: impl AsRef<OsStr>,
) -> Result<String, BuildError> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .map_err(BuildError::IOError)?;
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .map_err(|_| BuildError::CargoTargetDirectoryResolutionError)?;
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .ok_or(BuildError::CargoTargetDirectoryResolutionError)?;
        Ok(target_directory.to_owned())
    } else {
        Err(BuildError::CargoFailure(output.status))
    }
}

/// Builds a package.
pub fn build_package<P: AsRef<Path>>(
    base_path: P,
    trace: bool,
    force_local_target: bool,
    disable_wasm_opt: bool,
    log_level: Level,
    coverage: bool,
) -> Result<(PathBuf, PathBuf), BuildError> {
    let base_path = base_path.as_ref().to_owned();

    let mut manifest_path = base_path.clone();
    manifest_path.push("Cargo.toml");

    if !manifest_path.exists() {
        return Err(BuildError::NotCargoPackage(manifest_path));
    }

    // Use the scrypto directory as a target, even if the scrypto crate is part of a workspace
    // This allows us to find where the WASM and SCHEMA ends up deterministically.
    let mut target_path = if force_local_target {
        let mut target_path = base_path.clone();
        target_path.push("target");
        target_path
    } else {
        PathBuf::from_str(&get_default_target_directory(&manifest_path)?).unwrap()
    };

    // If coverage is enabled we change target directory to coverage directory
    if coverage {
        target_path.pop();
        target_path.push("coverage");
    }

    let mut out_path = target_path.clone();
    out_path.push("wasm32-unknown-unknown");
    out_path.push("release");

    // Build with SCHEMA
    run_cargo_build(&manifest_path, &target_path, trace, false, log_level, false)?;

    // Find the binary paths
    let manifest = Manifest::from_path(&manifest_path)
        .map_err(|_| BuildError::InvalidManifestFile(manifest_path.clone()))?;
    let mut wasm_name = None;
    if let Some(lib) = manifest.lib {
        wasm_name = lib.name.clone();
    }
    if wasm_name == None {
        if let Some(pkg) = manifest.package {
            wasm_name = Some(pkg.name.replace("-", "_"));
        }
    }
    let mut bin_path = out_path.clone();
    bin_path.push(wasm_name.ok_or(BuildError::InvalidManifestFile(manifest_path.clone()))?);

    let wasm_path = bin_path.with_extension("wasm");
    let definition_path = bin_path.with_extension("rpd");

    // Extract SCHEMA
    let wasm =
        fs::read(&wasm_path).map_err(|err| BuildError::IOErrorAtPath(err, wasm_path.clone()))?;
    let definition = extract_definition(&wasm).map_err(BuildError::SchemaExtractionError)?;
    fs::write(
        &definition_path,
        manifest_encode(&definition).map_err(BuildError::SchemaEncodeError)?,
    )
    .map_err(|err| BuildError::IOErrorAtPath(err, definition_path.clone()))?;

    // Build without SCHEMA
    run_cargo_build(
        &manifest_path,
        &target_path,
        trace,
        true,
        log_level,
        coverage,
    )?;

    // Optimizes the built wasm using Binaryen's wasm-opt tool. The code that follows is equivalent
    // to running the following commands in the CLI:
    // wasm-opt -0z --strip-debug --strip-dwarf --strip-procedures $some_path $some_path
    if !disable_wasm_opt {
        wasm_opt::OptimizationOptions::new_optimize_for_size_aggressively()
            .add_pass(wasm_opt::Pass::StripDebug)
            .add_pass(wasm_opt::Pass::StripDwarf)
            .add_pass(wasm_opt::Pass::StripProducers)
            .run(&wasm_path, &wasm_path)
            .map_err(BuildError::OptimizationError)?;
    }

    Ok((wasm_path, definition_path))
}

/// Runs tests within a package.
pub fn test_package<P: AsRef<Path>, I, S>(path: P, args: I, coverage: bool) -> Result<(), TestError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if !coverage {
        build_package(&path, false, false, false, Level::Trace, false)
            .map_err(TestError::BuildError)?;
    }

    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let features = if coverage {
            vec!["--features", "scrypto-unit/coverage"]
        } else {
            vec![]
        };
        let status = Command::new("cargo")
            .arg("test")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args(features)
            .arg("--")
            .args(args)
            .status()
            .map_err(TestError::IOError)?;
        if !status.success() {
            return Err(TestError::CargoFailure(status));
        }
        Ok(())
    } else {
        Err(TestError::NotCargoPackage)
    }
}

/// Format a package.
pub fn fmt_package<P: AsRef<Path>>(path: P, check: bool, quiet: bool) -> Result<(), FormatError> {
    let mut cargo = path.as_ref().to_owned();
    cargo.push("Cargo.toml");
    if cargo.exists() {
        let status = Command::new("cargo")
            .arg("fmt")
            .arg("--manifest-path")
            .arg(cargo.to_str().unwrap())
            .args({
                let mut args = Vec::new();
                if check {
                    args.push("--check")
                }
                if quiet {
                    args.push("--quiet")
                }
                args
            })
            .status()
            .map_err(FormatError::IOError)?;

        if status.success() {
            Ok(())
        } else {
            Err(FormatError::CargoFailure(status))
        }
    } else {
        Ok(())
    }
}
