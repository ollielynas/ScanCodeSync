use anyhow::{Context, Result, bail};
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

const MANIFEST_URL: &str = "https://sync-home.ollielynas.com/latest.json";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    notes: String,
    platforms: HashMap<String, PlatformAsset>,
}

#[derive(Debug, Deserialize)]
struct PlatformAsset {
    url: String,
    signature: String,
}

#[derive(Debug)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable { version: String, notes: String },
    PlatformNotSupported,
}

fn platform_key() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x86_64",
        ("macos", "x86_64")   => "darwin-x86_64",
        ("macos", "aarch64")  => "darwin-aarch64",
        ("linux", "x86_64")   => "linux-x86_64",
        _                     => "unknown",
    }
}

pub fn check_for_update() -> Result<UpdateStatus> {
    let manifest: Manifest = reqwest::blocking::get(MANIFEST_URL)
        .context("Failed to fetch update manifest")?
        .json()
        .context("Failed to parse update manifest")?;

    let latest = Version::parse(&manifest.version)
        .context("Server returned an invalid version string")?;
    let current = Version::parse(CURRENT_VERSION)
        .context("CARGO_PKG_VERSION is not valid semver")?;

    if latest <= current {
        return Ok(UpdateStatus::UpToDate);
    }

    if !manifest.platforms.contains_key(platform_key()) {
        return Ok(UpdateStatus::PlatformNotSupported);
    }

    Ok(UpdateStatus::UpdateAvailable {
        version: manifest.version,
        notes: manifest.notes,
    })
}

pub fn download_and_install() -> Result<()> {
    let manifest: Manifest = reqwest::blocking::get(MANIFEST_URL)
        .context("Failed to fetch update manifest")?
        .json()
        .context("Failed to parse update manifest")?;

    let asset = match manifest.platforms.get(platform_key()) {
        Some(a) => a,
        None => return Ok(()), // No update for this platform — silently skip
    };

    let bytes = reqwest::blocking::get(&asset.url)
        .context("Failed to download update")?
        .bytes()
        .context("Failed to read update download")?;

    verify_sha256(&bytes, &asset.signature)?;

    let ext = if cfg!(target_os = "windows") { ".msi" }
              else if cfg!(target_os = "macos") { ".dmg" }
              else { ".tar.gz" };

    let mut tmp = tempfile::Builder::new()
        .suffix(ext)
        .tempfile()
        .context("Failed to create temp file for update")?;

    std::io::Write::write_all(&mut tmp, &bytes)
        .context("Failed to write update to temp file")?;

    let tmp_path = tmp.into_temp_path();
    install_update(tmp_path.to_str().context("Temp path is not valid UTF-8")?)
}

fn verify_sha256(data: &[u8], expected: &str) -> Result<()> {
    use sha2::{Digest, Sha256};
    let hash = format!("sha256:{:x}", Sha256::digest(data));
    if hash != expected {
        bail!("Checksum mismatch: expected {}, got {}", expected, hash);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_update(path: &str) -> Result<()> {
    let status = Command::new("msiexec")
        .args(["/i", path, "/quiet", "/norestart"])
        .status()
        .context("Failed to launch msiexec")?;

    if !status.success() {
        bail!("msiexec failed with exit code {:?}", status.code());
    }
    std::process::exit(0);
}

#[cfg(target_os = "macos")]
fn install_update(path: &str) -> Result<()> {
    Command::new("open")
        .arg(path)
        .status()
        .context("Failed to open DMG")?;
    std::process::exit(0);
}

#[cfg(target_os = "linux")]
fn install_update(path: &str) -> Result<()> {
    let path = Path::new(path);
    self_update::Extract::from_source(path)
        .extract_into(&std::env::current_exe()
            .context("Could not determine current exe path")?
            .parent()
            .context("Exe has no parent directory")?)
        .context("Extraction failed")?;
    Ok(())
}
