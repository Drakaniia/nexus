//! Single instance management with crash detection
//! Uses a lock file with timestamp to detect crashed instances

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the lock file path
fn get_lock_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Nexus");
    path.push("nexus.lock");
    path
}

/// Get the keepalive file path
fn get_keepalive_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Nexus");
    path.push("nexus.keepalive");
    path
}

/// Touch the keepalive file with current timestamp
pub fn touch_keepalive() -> std::io::Result<()> {
    let path = get_keepalive_path();
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Write current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write(&path, timestamp.to_string())
}

/// Check if a previous instance appears to have crashed
/// Returns true if keepalive file is old (>5 minutes) or missing
pub fn should_restart_after_crash() -> bool {
    let path = get_keepalive_path();
    match fs::read_to_string(&path) {
        Ok(content) => {
            if let Ok(timestamp) = content.trim().parse::<u64>() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let age_seconds = now.saturating_sub(timestamp);
                // Consider crashed if older than 5 minutes
                age_seconds > 300
            } else {
                // Invalid timestamp, assume crashed
                true
            }
        }
        Err(_) => {
            // File doesn't exist or can't be read, assume first run or crashed
            true
        }
    }
}

/// Clean up keepalive file on exit
pub fn cleanup_keepalive() {
    let _ = fs::remove_file(get_keepalive_path());
}

/// Single instance manager with crash detection
pub struct SingleInstance {
    _lock_file: fs::File,
}

impl SingleInstance {
    /// Try to acquire single instance lock
    pub fn acquire() -> Result<Self, String> {
        let lock_path = get_lock_path();

        // Ensure directory exists
        if let Some(parent) = lock_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create lock directory: {}", e));
            }
        }

        // Try to create/open lock file
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => {
                // Successfully acquired lock
                log::info!("Single instance lock acquired");

                // Touch keepalive file
                if let Err(e) = touch_keepalive() {
                    log::warn!("Failed to create keepalive file: {}", e);
                }

                Ok(Self { _lock_file: file })
            }
            Err(_) => {
                // Check if previous instance crashed
                if should_restart_after_crash() {
                    log::warn!("Previous instance appears to have crashed, acquiring lock");
                    // Force remove lock file and try again
                    let _ = fs::remove_file(&lock_path);

                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&lock_path)
                    {
                        Ok(file) => {
                            // Touch keepalive file
                            let _ = touch_keepalive();
                            Ok(Self { _lock_file: file })
                        }
                        Err(e) => Err(format!("Failed to acquire lock after crash recovery: {}", e)),
                    }
                } else {
                    Err("Another instance is already running".to_string())
                }
            }
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        // Clean up files on exit
        let _ = fs::remove_file(get_lock_path());
        cleanup_keepalive();
    }
}