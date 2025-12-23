//! First-Run Setup Wizard Module
//! Displays a multi-screen configuration wizard on first application launch
//! 
//! NOTE: Wizard UI temporarily disabled due to Slint module compilation issues
//! The wizard.slint file exists but needs proper integration strategy

use crate::config::AppConfig;
use std::error::Error;

/// Show the setup wizard and collect user configuration
/// 
/// TEMPORARY: Wizard UI is disabled, returns Ok immediately and applies defaults
pub fn show_wizard(config: &mut AppConfig) -> Result<(), Box<dyn Error>> {
    log::info!("Wizard functionality temporarily disabled - applying defaults");
    log::info!("First run setup will use default configuration:");
    log::info!("  Hotkey: Alt + Space");
    log::info!("  Run on startup: {}", config.startup.enabled);
    
    // For now, just return Ok to allow the application to continue
    // TODO: Implement proper wizard once Slint multi-file compilation is resolved
    Ok(())
}

// Wizard will be re-enabled once we resolve the Slint compilation strategy
// Options:
// 1. Use build.rs with proper module naming
// 2. Use slint! macro with embedded .slint code
// 3. Create wizard as separate binary/library

/// Get hotkey configuration from preset index
#[allow(dead_code)]
fn get_hotkey_from_index(index:usize) -> (&'static str, &'static str) {
    match index {
        0 => ("Alt", "Space"),            // Alt + Space (default)
        1 => ("Ctrl", "Space"),           // Ctrl + Space
        2 => ("Win", "Space"),            // Win + Space
        3 => ("Ctrl+Shift", "Space"),     // Ctrl + Shift + Space
        _ => ("Alt", "Space"),            // Fallback to default
    }
}

/// Validate hotkey configuration for conflicts
/// Returns Ok if hotkey is available, Err with conflict description if not
#[allow(dead_code)]
fn validate_hotkey(modifiers: &[String], key: &str) -> Result<(), String> {
    // Check for system-reserved hotkeys
    let system_reserved = [
        ("Win", "L"),  // Lock screen
        ("Win", "D"),  // Show desktop
        ("Ctrl+Alt", "Delete"),  // Task manager
        ("Alt", "F4"),  // Close window
        ("Win", "R"),  // Run dialog
        ("Win", "E"),  // File Explorer
    ];
    
    let mods = modifiers.join("+");
    for (reserved_mods, reserved_key) in &system_reserved {
        if mods == *reserved_mods && key == *reserved_key {
            return Err(format!("Hotkey {}+{} is reserved by Windows", mods, key));
        }
    }
    
    // In a real implementation, we would check if the hotkey is already registered
    // by another application using Windows API
    // For now, just return Ok
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_hotkey_from_index() {
        assert_eq!(get_hotkey_from_index(0), ("Alt", "Space"));
        assert_eq!(get_hotkey_from_index(1), ("Ctrl", "Space"));
        assert_eq!(get_hotkey_from_index(2), ("Win", "Space"));
        assert_eq!(get_hotkey_from_index(3), ("Ctrl+Shift", "Space"));
        assert_eq!(get_hotkey_from_index(999), ("Alt", "Space")); // Fallback
    }
    
    #[test]
    fn test_validate_hotkey() {
        // Valid hotkeys
        assert!(validate_hotkey(&vec!["Alt".to_string()], "Space").is_ok());
        assert!(validate_hotkey(&vec!["Ctrl".to_string()], "Space").is_ok());
        
        // System reserved hotkeys should fail
        assert!(validate_hotkey(&vec!["Win".to_string()], "L").is_err());
        assert!(validate_hotkey(&vec!["Alt".to_string()], "F4").is_err());
    }
}
