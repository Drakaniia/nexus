//! First-Run Setup Wizard Module
//! Displays a multi-screen configuration wizard on first application launch

use crate::config::{AppConfig, HotkeyConfig};
use std::error::Error;

slint::include_modules!();

/// Show the setup wizard and collect user configuration
pub fn show_wizard(config: &mut AppConfig) -> Result<(), Box<dyn Error>> {
    log::info!("Launching first-run setup wizard");
    
    // Create the wizard window
    let wizard = SetupWizard::new()?;
    
    // Set initial values from config
    wizard.set_run_on_startup(config.startup.enabled);
    wizard.set_show_on_startup(config.startup.show_on_startup);
    
    // Track whether wizard was completed or cancelled
    let completed = std::rc::Rc::new(std::cell::RefCell::new(false));
    let completed_clone = completed.clone();
    
    // Handle Next button clicks
    let wizard_weak = wizard.as_weak();
    wizard.on_next_clicked(move || {
        if let Some(wizard) = wizard_weak.upgrade() {
            let current = wizard.get_current_screen();
            
            match current {
                WizardScreen::Welcome => {
                    wizard.set_current_screen(WizardScreen::Hotkey);
                }
                WizardScreen::Hotkey => {
                    wizard.set_current_screen(WizardScreen::Startup);
                }
                WizardScreen::Startup => {
                    wizard.set_current_screen(WizardScreen::Complete);
                }
                _ => {}
            }
        }
    });
    
    // Handle Back button clicks
    let wizard_weak = wizard.as_weak();
    wizard.on_back_clicked(move || {
        if let Some(wizard) = wizard_weak.upgrade() {
            let current = wizard.get_current_screen();
            
            match current {
                WizardScreen::Hotkey => {
                    wizard.set_current_screen(WizardScreen::Welcome);
                }
                WizardScreen::Startup => {
                    wizard.set_current_screen(WizardScreen::Hotkey);
                }
                WizardScreen::Complete => {
                    wizard.set_current_screen(WizardScreen::Startup);
                }
                _ => {}
            }
        }
    });
    
    // Handle Test Hotkey button
    let wizard_weak = wizard.as_weak();
    wizard.on_test_hotkey_clicked(move || {
        if let Some(wizard) = wizard_weak.upgrade() {
            let hotkey_index = wizard.get_selected_hotkey_index();
            log::info!("Testing hotkey with index: {}", hotkey_index);
            
            // In a real implementation, we would register a temporary hotkey here
            // For now, just log it
            // TODO: Implement temporary hotkey registration for testing
        }
    });
    
    // Handle Finish button clicks
    let wizard_weak = wizard.as_weak();
    let completed_clone2 = completed.clone();
    wizard.on_finish_clicked(move || {
        *completed_clone2.borrow_mut() = true;
        if let Some(wizard) = wizard_weak.upgrade() {
            wizard.hide().ok();
        }
    });
    
    // Run the wizard
    wizard.run()?;
    
    // If wizard was completed, update config
    if *completed.borrow() {
        log::info!("Wizard completed, updating configuration");
        
        // Update hotkey configuration
        let hotkey_index = wizard.get_selected_hotkey_index() as usize;
        let (modifiers, key) = get_hotkey_from_index(hotkey_index);
        config.hotkey = HotkeyConfig {
            modifiers: modifiers.split('+').map(|s| s.trim().to_string()).collect(),
            key: key.to_string(),
        };
        
        // Update startup configuration
        config.startup.enabled = wizard.get_run_on_startup();
        config.startup.show_on_startup = wizard.get_show_on_startup();
        
        log::info!("Configuration updated from wizard:");
        log::info!("  Hotkey: {:?} + {}", config.hotkey.modifiers, config.hotkey.key);
        log::info!("  Run on startup: {}", config.startup.enabled);
        log::info!("  Show on startup: {}", config.startup.show_on_startup);
        
        Ok(())
    } else {
        log::info!("Wizard cancelled or closed, using default configuration");
        Err("Wizard was cancelled".into())
    }
}

/// Get hotkey configuration from preset index
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
