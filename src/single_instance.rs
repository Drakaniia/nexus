//! Single instance management with crash detection
//! Uses a lock file with timestamp to detect crashed instances

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the lock file path based on portable mode
fn get_lock_path(portable_mode: PortableMode) -> PathBuf {
    match portable_mode {
        PortableMode::Portable => {
            // Use executable directory for portable mode
            let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            exe_dir.join("nexus.lock")
        }
        PortableMode::Installed => {
            // Use %APPDATA% for installed mode
            let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("Nexus");
            path.push("nexus.lock");
            path
        }
    }
}

/// Get the keepalive file path based on portable mode
fn get_keepalive_path(portable_mode: PortableMode) -> PathBuf {
    match portable_mode {
        PortableMode::Portable => {
            // Use executable directory for portable mode
            let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            exe_dir.join("nexus.keepalive")
        }
        PortableMode::Installed => {
            // Use %APPDATA% for installed mode
            let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("Nexus");
            path.push("nexus.keepalive");
            path
        }
    }
}

/// Touch the keepalive file with current timestamp
pub fn touch_keepalive() -> std::io::Result<()> {
    touch_keepalive_with_mode(detect_portable_mode())
}

/// Touch the keepalive file with current timestamp (internal with mode)
fn touch_keepalive_with_mode(portable_mode: PortableMode) -> std::io::Result<()> {
    let path = get_keepalive_path(portable_mode);
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
    should_restart_after_crash_with_mode(detect_portable_mode())
}

/// Check if a previous instance appears to have crashed (internal with mode)
fn should_restart_after_crash_with_mode(portable_mode: PortableMode) -> bool {
    let path = get_keepalive_path(portable_mode);
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
    cleanup_keepalive_with_mode(detect_portable_mode());
}

/// Clean up keepalive file on exit (internal with mode)
fn cleanup_keepalive_with_mode(portable_mode: PortableMode) {
    let _ = fs::remove_file(get_keepalive_path(portable_mode));
}

/// Portable mode detection and handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortableMode {
    /// Installed mode: uses %APPDATA% for data storage
    Installed,
    /// Portable mode: uses executable directory for data storage
    Portable,
}

/// Detect if running in portable mode
/// Portable mode is detected by:
/// 1. --portable command line argument
/// 2. Presence of "portable" file in executable directory
/// 3. Executable being in a non-standard location (not Program Files, not Windows directory)
pub fn detect_portable_mode() -> PortableMode {
    // Check command line arguments first
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--portable") {
        log::info!("Portable mode enabled via command line argument");
        return PortableMode::Portable;
    }

    // Check for portable marker file
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_marker = exe_dir.join("portable");
            if portable_marker.exists() {
                log::info!("Portable mode detected via marker file: {:?}", portable_marker);
                return PortableMode::Portable;
            }
        }
    }

    // Check if executable is in a typical portable location
    // (not in Program Files, Windows directory, or standard install locations)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_dir_str = exe_dir.to_string_lossy().to_lowercase();

            // If not in standard Windows install locations, assume portable
            if !exe_dir_str.contains("program files") &&
               !exe_dir_str.contains("\\windows\\") &&
               !exe_dir_str.contains("appdatalocal") &&
               !exe_dir_str.contains("programdata") {
                log::info!("Portable mode detected via executable location: {:?}", exe_dir);
                return PortableMode::Portable;
            }
        }
    }

    log::info!("Running in installed mode");
    PortableMode::Installed
}

/// Single instance manager with crash detection
pub struct SingleInstance {
    _lock_file: fs::File,
    _portable_mode: PortableMode,
}

impl SingleInstance {
    /// Try to acquire single instance lock
    pub fn acquire() -> Result<Self, String> {
        let portable_mode = detect_portable_mode();
        Self::acquire_with_mode(portable_mode)
    }

    /// Try to acquire single instance lock with specific portable mode
    pub fn acquire_with_mode(portable_mode: PortableMode) -> Result<Self, String> {
        let lock_path = get_lock_path(portable_mode);

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
                log::info!("Single instance lock acquired (mode: {:?})", portable_mode);

                // Touch keepalive file
                if let Err(e) = touch_keepalive_with_mode(portable_mode) {
                    log::warn!("Failed to create keepalive file: {}", e);
                }

                Ok(Self {
                    _lock_file: file,
                    _portable_mode: portable_mode,
                })
            }
            Err(_) => {
                // Check if previous instance crashed
                if should_restart_after_crash_with_mode(portable_mode) {
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
                            let _ = touch_keepalive_with_mode(portable_mode);
                            Ok(Self {
                                _lock_file: file,
                                _portable_mode: portable_mode,
                            })
                        }
                        Err(e) => Err(format!("Failed to acquire lock after crash recovery: {}", e)),
                    }
                } else {
                    Err("Another instance is already running".to_string())
                }
            }
        }
    }

    /// Get the portable mode this instance is using
    pub fn portable_mode(&self) -> PortableMode {
        self._portable_mode
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        // Clean up files on exit
        let _ = fs::remove_file(get_lock_path(self._portable_mode));
        cleanup_keepalive_with_mode(self._portable_mode);
    }
}