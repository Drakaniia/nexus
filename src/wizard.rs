//! First-Run Setup Wizard Module
//! Displays a multi-screen configuration wizard on first application launch

use crate::config::AppConfig;
use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

slint::include_modules!();
use crate::ui::{SetupWizard, WizardScreen};

/// Show the setup wizard and collect user configuration
pub fn show_wizard(config: &mut AppConfig) -> Result<(), Box<dyn Error>> {
    log::info!("Launching first-run setup wizard");

    // Create the wizard window
    let wizard = crate::ui::SetupWizard::new()?;

    // Initialize wizard state from config
    wizard.set_run_on_startup(config.startup.enabled);
    wizard.set_show_on_startup(config.startup.show_on_startup);

    // Set initial hotkey selection based on config
    let hotkey_index = get_hotkey_index_from_config(config);
    wizard.set_selected_hotkey_index(hotkey_index as i32);

    // Create a shared config reference for the callbacks
    let config_ref = Rc::new(RefCell::new(config));

    // Handle Next button clicks
    let config_next = Rc::clone(&config_ref);
    wizard.on_next_clicked(move || {
        let mut config = config_next.borrow_mut();
        let current_screen = wizard.get_current_screen();

        match current_screen {
            WizardScreen::Welcome => {
                wizard.set_current_screen(WizardScreen::Hotkey);
            }
            WizardScreen::Hotkey => {
                // Apply hotkey selection to config
                let selected_index = wizard.get_selected_hotkey_index() as usize;
                apply_hotkey_selection(&mut config, selected_index);
                wizard.set_current_screen(WizardScreen::Startup);
            }
            WizardScreen::Startup => {
                // Apply startup settings to config
                config.startup.enabled = wizard.get_run_on_startup();
                config.startup.show_on_startup = wizard.get_show_on_startup();
                wizard.set_current_screen(WizardScreen::Complete);
            }
            _ => {}
        }
    });

    // Handle Back button clicks
    wizard.on_back_clicked(move || {
        let current_screen = wizard.get_current_screen();

        match current_screen {
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
    });

    // Handle Test Hotkey button
    wizard.on_test_hotkey_clicked(move || {
        let selected_index = wizard.get_selected_hotkey_index() as usize;
        let (modifiers, key) = get_hotkey_from_index(selected_index);

        log::info!("Testing hotkey: {} + {}", modifiers, key);
        // TODO: Could show a notification or temporarily test the hotkey
        // For now, just log it
    });

    // Handle Finish button
    let config_finish = Rc::clone(&config_ref);
    wizard.on_finish_clicked(move || {
        let mut config = config_finish.borrow_mut();

        // Apply final settings
        let selected_index = wizard.get_selected_hotkey_index() as usize;
        apply_hotkey_selection(&mut config, selected_index);
        config.startup.enabled = wizard.get_run_on_startup();
        config.startup.show_on_startup = wizard.get_show_on_startup();

        log::info!("Wizard completed - settings applied:");
        log::info!("  Hotkey: {} + {}", config.hotkey.modifiers.join("+"), config.hotkey.key);
        log::info!("  Run on startup: {}", config.startup.enabled);
        log::info!("  Show on startup: {}", config.startup.show_on_startup);

        // Close the wizard
        wizard.hide().ok();
    });

    // Show the wizard modally
    wizard.show()?;
    slint::run_event_loop_until_quit()?;

    Ok(())
}

/// Get the hotkey index from current config
fn get_hotkey_index_from_config(config: &AppConfig) -> usize {
    let modifiers = config.hotkey.modifiers.join("+");
    let key = &config.hotkey.key;

    match (modifiers.as_str(), key.as_str()) {
        ("Alt", "Space") => 0,
        ("Ctrl", "Space") => 1,
        ("Win", "Space") => 2,
        ("Ctrl+Shift", "Space") => 3,
        _ => 0, // Default to Alt+Space
    }
}

/// Apply hotkey selection to config
fn apply_hotkey_selection(config: &mut AppConfig, index: usize) {
    let (modifiers, key) = get_hotkey_from_index(index);

    config.hotkey.modifiers = if modifiers.contains("+") {
        modifiers.split("+").map(|s| s.to_string()).collect()
    } else {
        vec![modifiers.to_string()]
    };
    config.hotkey.key = key.to_string();
}

/// Get hotkey configuration from preset index
fn get_hotkey_from_index(index: usize) -> (&'static str, &'static str) {
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
