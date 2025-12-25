//! Update System Module
//! Handles version checking, update downloads, and installation

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository information
const GITHUB_OWNER: &str = "Qwenzy";  // Update with actual GitHub username
const GITHUB_REPO: &str = "nexus";    // Update with actual repository name

/// GitHub release information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub release asset structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Update information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub published_at: String,
    pub file_size: u64,
}

/// Check for available updates on GitHub
pub fn check_for_updates(include_beta: bool) -> Result<Option<UpdateInfo>, Box<dyn Error>> {
    log::info!("Checking for updates (current version: {})", VERSION);

    let endpoint = if include_beta {
        format!("https://api.github.com/repos/{}/{}/releases", GITHUB_OWNER, GITHUB_REPO)
    } else {
        format!("https://api.github.com/repos/{}/{}/releases/latest", GITHUB_OWNER, GITHUB_REPO)
    };

    log::info!("Checking for updates at: {}", endpoint);

    // Create HTTP client with user agent
    let client = reqwest::blocking::Client::builder()
        .user_agent("Nexus-Updater/1.0")
        .build()?;

    // Make the request
    let response = client.get(&endpoint).send()?;
    let status = response.status();

    if !status.is_success() {
        log::warn!("GitHub API returned status: {}", status);
        return Ok(None);
    }

    if include_beta {
        // Handle multiple releases (beta channel)
        let releases: Vec<GitHubRelease> = response.json()?;
        process_releases_for_update(releases)
    } else {
        // Handle single latest release
        let release: GitHubRelease = response.json()?;
        process_release_for_update(release)
    }
}

/// Process multiple releases to find updates
fn process_releases_for_update(releases: Vec<GitHubRelease>) -> Result<Option<UpdateInfo>, Box<dyn Error>> {
    let current_version = semver::Version::parse(VERSION)?;

    for release in releases {
        // Skip pre-releases unless beta channel is enabled
        if release.tag_name.contains("rc") || release.tag_name.contains("beta") || release.tag_name.contains("alpha") {
            continue;
        }

        let release_version_str = release.tag_name.trim_start_matches('v');
        if let Ok(release_version) = semver::Version::parse(release_version_str) {
            if release_version > current_version {
                return find_msi_asset(&release).map(Some);
            }
        }
    }

    Ok(None)
}

/// Process single release for update
fn process_release_for_update(release: GitHubRelease) -> Result<Option<UpdateInfo>, Box<dyn Error>> {
    let current_version = semver::Version::parse(VERSION)?;
    let release_version_str = release.tag_name.trim_start_matches('v');

    if let Ok(release_version) = semver::Version::parse(release_version_str) {
        if release_version > current_version {
            return find_msi_asset(&release).map(Some);
        }
    }

    Ok(None)
}

/// Find MSI asset in release assets
fn find_msi_asset(release: &GitHubRelease) -> Result<UpdateInfo, Box<dyn Error>> {
    // Look for MSI files in assets
    let msi_extensions = [".msi", ".MSI"];

    for asset in &release.assets {
        if msi_extensions.iter().any(|ext| asset.name.ends_with(ext)) {
            log::info!("Found update: {} ({} bytes)", release.tag_name, asset.size);

            return Ok(UpdateInfo {
                version: release.tag_name.clone(),
                download_url: asset.browser_download_url.clone(),
                release_notes: release.body.clone(),
                published_at: release.published_at.clone(),
                file_size: asset.size,
            });
        }
    }

    Err("No MSI installer found in release assets".into())
}

/// Download update MSI to temp directory
pub fn download_update(download_url: &str, expected_size: u64) -> Result<PathBuf, Box<dyn Error>> {
    log::info!("Downloading update from: {} ({} bytes)", download_url, expected_size);

    // Create HTTP client
    let client = reqwest::blocking::Client::builder()
        .user_agent("Nexus-Updater/1.0")
        .build()?;

    // Make the request
    let mut response = client.get(download_url).send()?;
    let status = response.status();

    if !status.is_success() {
        return Err(format!("Download failed with status: {}", status).into());
    }

    // Create temp file path
    let temp_dir = std::env::temp_dir();
    let filename = "Nexus-Update.msi";
    let msi_path = temp_dir.join(filename);

    // Download to file
    let mut file = std::fs::File::create(&msi_path)?;
    std::io::copy(&mut response, &mut file)?;

    // Verify file size
    let metadata = file.metadata()?;
    if metadata.len() != expected_size {
        log::warn!("Downloaded file size ({}) doesn't match expected size ({})",
                  metadata.len(), expected_size);
    }

    log::info!("Downloaded update to: {:?}", msi_path);
    Ok(msi_path)
}

/// Install downloaded update
pub fn install_update(msi_path: PathBuf) -> Result<(), Box<dyn Error>> {
    log::info!("Installing update from: {:?}", msi_path);

    #[cfg(windows)]
    {
        log::info!("Launching MSI installer...");

        // Launch msiexec with quiet installation
        let status = std::process::Command::new("msiexec")
            .arg("/i")
            .arg(&msi_path)
            .arg("/qb")  // Basic UI with progress bar
            .arg("/norestart")  // Don't restart automatically
            .status()?;

        if status.success() {
            log::info!("Update installation completed successfully");
            // Exit current application
            std::process::exit(0);
        } else {
            return Err(format!("MSI installer exited with code: {}", status.code().unwrap_or(-1)).into());
        }
    }

    #[cfg(not(windows))]
    {
        Err("Update installation only supported on Windows".into())
    }
}

/// Clean up downloaded update file
pub fn cleanup_update_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        std::fs::remove_file(path)?;
        log::info!("Cleaned up update file: {:?}", path);
    }
    Ok(())
}

/// Compare two version strings using semantic versioning
#[allow(dead_code)]
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (semver::Version::parse(current), semver::Version::parse(latest.trim_start_matches('v'))) {
        (Ok(current_ver), Ok(latest_ver)) => latest_ver > current_ver,
        _ => latest > current, // Fallback to string comparison
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(!is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
    }
}
