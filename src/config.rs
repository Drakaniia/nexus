//! Configuration Management Module
//! Handles loading/saving application settings to %APPDATA%\Nexus\config.json (installed)
//! or to executable directory (portable mode)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

// Import portable mode detection
use crate::single_instance::PortableMode;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Hotkey configuration
    pub hotkey: HotkeyConfig,

    /// Startup settings
    pub startup: StartupConfig,

    /// Appearance settings
    pub appearance: AppearanceConfig,

    /// Search settings
    #[serde(default)]
    pub search: SearchConfig,

    /// Update settings
    #[serde(default)]
    pub update: UpdateConfig,

    /// Most Recently Used tracking
    #[serde(default)]
    pub mru: HashMap<String, u32>,

    /// First run flag
    #[serde(default = "default_first_run")]
    pub first_run: bool,

    /// Portable mode flag (auto-detected, stored for reference)
    #[serde(default)]
    pub portable_mode: bool,
}

fn default_first_run() -> bool {
    true
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Modifier keys (Alt, Ctrl, Shift, Win)
    pub modifiers: Vec<String>,
    
    /// Main key
    pub key: String,
}

/// Startup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Whether to run on Windows startup
    pub enabled: bool,
    
    /// Whether to show launcher on startup
    #[serde(default)]
    pub show_on_startup: bool,
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Theme (dark, light, system)
    pub theme: String,
    
    /// Window opacity (0.0 - 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    
    /// Maximum number of search results to show
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    
    /// Font size in pixels
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    
    /// Window size preset (compact, normal, large)
    #[serde(default = "default_window_size")]
    pub window_size: String,
}

fn default_opacity() -> f32 {
    0.96
}

fn default_max_results() -> usize {
    8
}

fn default_font_size() -> u32 {
    14
}

fn default_window_size() -> String {
    "normal".to_string()
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Folders to exclude from search
    #[serde(default)]
    pub excluded_folders: Vec<String>,
    
    /// File type filters (e.g., "Documents", "Images")
    #[serde(default)]
    pub file_type_filters: Vec<String>,
    
    /// Search delay in milliseconds (debounce)
    #[serde(default = "default_search_delay")]
    pub search_delay_ms: u32,
    
    /// Enable fuzzy matching
    #[serde(default = "default_fuzzy_search")]
    pub fuzzy_search: bool,
}

fn default_search_delay() -> u32 {
    150
}

fn default_fuzzy_search() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            startup: StartupConfig::default(),
            appearance: AppearanceConfig::default(),
            search: SearchConfig::default(),
            update: UpdateConfig::default(),
            mru: HashMap::new(),
            first_run: true,
            portable_mode: false, // Will be set during load
        }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Alt".to_string()],
            key: "Space".to_string(),
        }
    }
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_on_startup: false,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            opacity: 0.96,
            max_results: 8,
            font_size: 14,
            window_size: "normal".to_string(),
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            excluded_folders: vec![],
            file_type_filters: vec![],
            search_delay_ms: 150,
            fuzzy_search: true,
        }
    }
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update checking
    #[serde(default = "default_auto_check")]
    pub auto_check: bool,
    
    /// Check frequency in hours
    #[serde(default = "default_check_frequency")]
    pub check_frequency_hours: u32,
    
    /// Enable beta/pre-release updates
    #[serde(default)]
    pub beta_channel: bool,
    
    /// Last update check timestamp (ISO 8601)
    #[serde(default)]
    pub last_check: Option<String>,
}

fn default_auto_check() -> bool {
    true
}

fn default_check_frequency() -> u32 {
    24
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            check_frequency_hours: 24,
            beta_channel: false,
            last_check: None,
        }
    }
}

impl AppConfig {
    /// Get the configuration directory path based on portable mode
    pub fn config_dir(portable_mode: PortableMode) -> Option<PathBuf> {
        match portable_mode {
            PortableMode::Portable => {
                // Use executable directory for portable mode
                env::current_exe().ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            }
            PortableMode::Installed => {
                // Use %APPDATA% for installed mode
                dirs::config_dir().map(|p| p.join("Nexus"))
            }
        }
    }

    /// Get the configuration file path based on portable mode
    pub fn config_path(portable_mode: PortableMode) -> Option<PathBuf> {
        Self::config_dir(portable_mode).map(|p| p.join("config.json"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Self {
        Self::load_with_mode(crate::single_instance::detect_portable_mode())
    }

    /// Load configuration with specific portable mode
    pub fn load_with_mode(portable_mode: PortableMode) -> Self {
        if let Some(path) = Self::config_path(portable_mode) {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<AppConfig>(&content) {
                            Ok(mut config) => {
                                // Update portable mode flag in loaded config
                                config.portable_mode = matches!(portable_mode, PortableMode::Portable);
                                log::info!("Loaded configuration from {:?} (mode: {:?})", path, portable_mode);
                                return config;
                            }
                            Err(e) => {
                                log::warn!("Failed to parse config file: {}. Using defaults.", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read config file: {}. Using defaults.", e);
                    }
                }
            } else {
                log::info!("No config file found, creating default (mode: {:?})", portable_mode);
            }
        }

        let mut config = Self::default();
        config.portable_mode = matches!(portable_mode, PortableMode::Portable);
        config.save_with_mode(portable_mode); // Save default config
        config
    }

    /// Save configuration to file
    pub fn save(&self) {
        let portable_mode = if self.portable_mode { PortableMode::Portable } else { PortableMode::Installed };
        self.save_with_mode(portable_mode);
    }

    /// Save configuration to file with specific portable mode
    pub fn save_with_mode(&self, portable_mode: PortableMode) {
        if let Some(dir) = Self::config_dir(portable_mode) {
            // Create directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&dir) {
                log::error!("Failed to create config directory: {}", e);
                return;
            }

            if let Some(path) = Self::config_path(portable_mode) {
                match serde_json::to_string_pretty(self) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&path, content) {
                            log::error!("Failed to write config file: {}", e);
                        } else {
                            log::debug!("Configuration saved to {:?} (mode: {:?})", path, portable_mode);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to serialize config: {}", e);
                    }
                }
            }
        }
    }

    /// Record a usage for MRU tracking
    pub fn record_usage(&mut self, name: &str) {
        *self.mru.entry(name.to_string()).or_insert(0) += 1;
        
        // Save periodically (every 5 uses of any app)
        let total_uses: u32 = self.mru.values().sum();
        if total_uses % 5 == 0 {
            self.save();
        }
    }

    /// Get MRU score for an app
    pub fn get_mru_score(&self, name: &str) -> u32 {
        *self.mru.get(name).unwrap_or(&0)
    }

    /// Mark first run as complete
    pub fn complete_first_run(&mut self) {
        self.first_run = false;
        self.save();
    }

    /// Check if this is the first run
    pub fn is_first_run(&self) -> bool {
        self.first_run
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.hotkey.key, "Space");
        assert!(config.hotkey.modifiers.contains(&"Alt".to_string()));
        assert!(config.first_run);
    }

    #[test]
    fn test_mru_tracking() {
        let mut config = AppConfig::default();
        config.record_usage("Notepad");
        config.record_usage("Notepad");
        assert_eq!(config.get_mru_score("Notepad"), 2);
        assert_eq!(config.get_mru_score("Unknown"), 0);
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.hotkey.key, config.hotkey.key);
    }
}
