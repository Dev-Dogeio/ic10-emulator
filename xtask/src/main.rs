use anyhow::{Context, Result, bail};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("build-wasm") => build_wasm(args.collect())?,
        Some("help") | None => print_help(),
        Some(cmd) => {
            eprintln!("Unknown xtask command: {}", cmd);
            print_help();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        "xtask commands:\n  build-wasm [--release]   Build wasm target and run wasm-bindgen into ./app/pkg"
    );
}

fn build_wasm(args: Vec<String>) -> Result<()> {
    let release = args.iter().any(|a| a == "--release");

    // Ensure the wasm-bindgen CLI exists
    if Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .is_err()
    {
        bail!("`wasm-bindgen` CLI not found. Install it with: `cargo install wasm-bindgen-cli`");
    }

    // Build the crate for wasm target using the workspace manifest so features work
    let xtask_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = xtask_manifest_dir
        .parent()
        .context("could not determine repository root")?;
    let manifest_path = repo_root.join("Cargo.toml");

    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");
    build_cmd.arg("--manifest-path");
    build_cmd.arg(manifest_path.as_os_str());
    build_cmd.arg("--target");
    build_cmd.arg("wasm32-unknown-unknown");
    build_cmd.arg("-p");
    build_cmd.arg("ic10-emulator");
    build_cmd.arg("--features");
    build_cmd.arg("wasm");
    if release {
        build_cmd.arg("--release");
    }

    println!("Running: {:?}", build_cmd);
    let status = build_cmd
        .status()
        .context("failed to run cargo build for wasm target")?;
    if !status.success() {
        bail!("cargo build failed")
    }

    // Locate wasm file
    let profile = if release { "release" } else { "debug" };
    let wasm_path = repo_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join("ic10_emulator_lib.wasm");

    if !wasm_path.exists() {
        bail!(
            "expected wasm file not found: {} (looked in repo root {})",
            wasm_path.display(),
            repo_root.display()
        );
    }

    // Ensure pkg directory exists and is empty-ish (relative to repo root)
    let pkg_dir = repo_root.join("app").join("pkg");
    if !pkg_dir.exists() {
        fs::create_dir_all(&pkg_dir).context("failed to create pkg dir")?;
    }

    // Run wasm-bindgen
    let out_name = "ic10_emulator";
    let mut bindgen_cmd = Command::new("wasm-bindgen");
    bindgen_cmd
        .arg(wasm_path.as_os_str())
        .arg("--out-dir")
        .arg(&pkg_dir)
        .arg("--target")
        .arg("web")
        .arg("--out-name")
        .arg(out_name);

    println!("Running: {:?}", bindgen_cmd);
    let status = bindgen_cmd.status().context("failed to run wasm-bindgen")?;
    if !status.success() {
        bail!("wasm-bindgen failed")
    }

    println!("WASM + bindings written to {}", pkg_dir.display());

    Ok(())
}
