use anyhow::{Context, Result};
use reqwest::blocking::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/hotaq/Sprit-mutil/releases/latest";
const GITHUB_API_ALL: &str = "https://api.github.com/repos/hotaq/Sprit-mutil/releases";
const REPO_URL: &str = "https://github.com/hotaq/Sprit-mutil.git";

#[derive(Debug, Deserialize, Serialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: Option<String>,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug)]
enum InstallMethod {
    Cargo,
    Binary,
}

/// Get the current version of sprite
fn get_current_version() -> Result<Version> {
    let version = env!("CARGO_PKG_VERSION");
    Version::parse(version).context("Failed to parse current version")
}

/// Check if a new version is available
fn check_for_updates() -> Result<Option<GitHubRelease>> {
    println!("🔍 Checking for updates...");

    let client = Client::builder().user_agent("sprite-updater").build()?;

    // Try to get latest release first
    let response = client.get(GITHUB_API_LATEST).send()?;

    let release: GitHubRelease = if response.status().is_success() {
        response
            .json()
            .context("Failed to parse release information")?
    } else {
        // If no "latest" release exists, get all releases and find the newest
        let releases: Vec<GitHubRelease> = client
            .get(GITHUB_API_ALL)
            .send()
            .context("Failed to fetch releases")?
            .json()
            .context("Failed to parse releases")?;

        if releases.is_empty() {
            println!("ℹ️  No releases found yet");
            return Ok(None);
        }

        // Find the most recent release by parsing versions
        releases
            .into_iter()
            .max_by_key(|r| {
                Version::parse(r.tag_name.trim_start_matches('v'))
                    .unwrap_or_else(|_| Version::new(0, 0, 0))
            })
            .context("No valid releases found")?
    };

    let current_version = get_current_version()?;
    let latest_version = Version::parse(release.tag_name.trim_start_matches('v'))
        .context("Failed to parse latest version")?;

    if latest_version > current_version {
        Ok(Some(release))
    } else {
        Ok(None)
    }
}

/// Detect how sprite was installed
fn detect_install_method() -> Result<InstallMethod> {
    let exe_path = env::current_exe()?;
    let cargo_home =
        env::var("CARGO_HOME").or_else(|_| env::var("HOME").map(|h| format!("{}/.cargo", h)))?;

    if exe_path.starts_with(&cargo_home) {
        Ok(InstallMethod::Cargo)
    } else {
        Ok(InstallMethod::Binary)
    }
}

/// Update via cargo
fn update_via_cargo() -> Result<()> {
    println!("📦 Updating via Cargo...");

    let status = Command::new("cargo")
        .args(["install", "--git", REPO_URL, "sprite", "--force"])
        .status()
        .context("Failed to execute cargo install")?;

    if status.success() {
        println!("✅ Successfully updated sprite via Cargo!");
        Ok(())
    } else {
        anyhow::bail!("Cargo update failed")
    }
}

/// Update by downloading binary
fn update_via_binary(release: &GitHubRelease) -> Result<()> {
    println!("📥 Downloading binary...");

    // Detect platform
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let asset_name = match (os, arch) {
        ("linux", "x86_64") => "sprite-x86_64-unknown-linux-gnu.tar.gz",
        ("linux", "aarch64") => "sprite-aarch64-unknown-linux-gnu.tar.gz",
        ("macos", "x86_64") => "sprite-x86_64-apple-darwin.tar.gz",
        ("macos", "aarch64") => "sprite-aarch64-apple-darwin.tar.gz",
        _ => anyhow::bail!("Unsupported platform: {}-{}", os, arch),
    };

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .context(format!("Asset {} not found in release", asset_name))?;

    // Download the binary
    let client = Client::builder().user_agent("sprite-updater").build()?;

    let response = client
        .get(&asset.browser_download_url)
        .send()
        .context("Failed to download binary")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download binary: HTTP {}", response.status());
    }

    let bytes = response.bytes().context("Failed to read binary")?;

    // Create temp directory
    let temp_dir = tempfile::tempdir()?;
    let archive_path = temp_dir.path().join(asset_name);

    fs::write(&archive_path, bytes)?;

    // Extract archive
    let output = Command::new("tar")
        .args(["-xzf", archive_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output()
        .context("Failed to extract archive")?;

    if !output.status.success() {
        anyhow::bail!("Failed to extract archive");
    }

    let extracted_binary = temp_dir.path().join("sprite");
    if !extracted_binary.exists() {
        anyhow::bail!("Binary not found in archive");
    }

    // Get current binary path
    let current_exe = env::current_exe()?;

    // Backup current binary
    let backup_path = current_exe.with_extension("backup");
    println!("💾 Creating backup at {}...", backup_path.display());
    fs::copy(&current_exe, &backup_path).context("Failed to create backup")?;

    // Replace binary
    println!("🔄 Replacing binary...");
    fs::copy(&extracted_binary, &current_exe).context("Failed to replace binary")?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&current_exe, perms)?;
    }

    println!("✅ Successfully updated sprite!");
    println!("💡 Backup saved to: {}", backup_path.display());

    Ok(())
}

/// Execute the update command
pub fn execute(check_only: bool, yes: bool, version: Option<String>) -> Result<()> {
    let current_version = get_current_version()?;
    println!("📌 Current version: {}", current_version);

    if let Some(_target_version) = version {
        anyhow::bail!("Updating to specific version is not yet implemented. Use 'sprite update' to get the latest version.");
    }

    // Check for updates
    let release = match check_for_updates()? {
        Some(r) => r,
        None => {
            println!("✅ You are already on the latest version!");
            return Ok(());
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v');
    println!("🆕 New version available: {}", latest_version);
    println!("📝 Release: {}", release.name);

    if let Some(body) = &release.body {
        println!("\n📋 Release Notes:");
        println!("{}", "-".repeat(60));
        // Show first 10 lines of release notes
        for (i, line) in body.lines().enumerate() {
            if i >= 10 {
                println!("... (see full notes at {})", release.html_url);
                break;
            }
            println!("{}", line);
        }
        println!("{}", "-".repeat(60));
    }

    if check_only {
        println!("\n💡 Run 'sprite update' to install the new version");
        return Ok(());
    }

    // Confirm update
    if !yes {
        print!(
            "\n❓ Do you want to update to version {}? [y/N]: ",
            latest_version
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !matches!(response.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("❌ Update cancelled");
            return Ok(());
        }
    }

    // Detect install method and update
    let method = detect_install_method()?;

    match method {
        InstallMethod::Cargo => {
            update_via_cargo()?;
        }
        InstallMethod::Binary => {
            update_via_binary(&release)?;
        }
    }

    println!(
        "\n🎉 Update complete! Version {} is now installed.",
        latest_version
    );
    println!("💡 Run 'sprite --version' to verify");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        assert!(version.is_ok());
        println!("Current version: {}", version.unwrap());
    }

    #[test]
    fn test_detect_install_method() {
        // This might fail in some test environments, so we just check it doesn't panic
        let _ = detect_install_method();
    }
}
