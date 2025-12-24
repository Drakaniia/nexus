use crate::config::{AppConfig};
use crate::startup;
use std::error::Error;
use slint::{ComponentHandle};

/// Settings window manager
#[allow(dead_code)]
pub struct SettingsManager {
    pub window: crate::SettingsWindow,
}

impl SettingsManager {
    /// Create and show the settings window
    pub fn show(config: &AppConfig, launcher_weak: slint::Weak<crate::Launcher>) -> Result<Self, Box<dyn Error>> {
        let settings = crate::SettingsWindow::new()?;
        
        // Load values from config
        settings.set_theme(config.appearance.theme.clone().into());
        settings.set_window_opacity(config.appearance.opacity as f32);
        settings.set_max_results(config.appearance.max_results as f32);
        settings.set_font_size(config.appearance.font_size as f32);
        settings.set_window_size(config.appearance.window_size.clone().into());
        
        settings.set_fuzzy_search(config.search.fuzzy_search);
        settings.set_search_delay(config.search.search_delay_ms as f32);
        
        settings.set_run_on_startup(config.startup.enabled);
        settings.set_show_on_startup(config.startup.show_on_startup);
        
        settings.set_auto_check_updates(config.update.auto_check);
        settings.set_version_text(crate::updater::VERSION.into());
        
        // Handle Apply callback
        let settings_weak = settings.as_weak();
        let launcher_weak_apply = launcher_weak.clone();
        settings.on_apply_clicked(move || {
            if let Some(settings) = settings_weak.upgrade() {
                log::info!("Settings apply clicked - Saving configuration");
                
                // 1. Create a new config based on UI values
                let mut new_config = AppConfig::load(); // Refresh from disk first to be safe
                
                new_config.appearance.theme = settings.get_theme().to_string();
                new_config.appearance.opacity = settings.get_window_opacity() as f32;
                new_config.appearance.max_results = settings.get_max_results() as usize;
                new_config.appearance.font_size = settings.get_font_size() as u32;
                new_config.appearance.window_size = settings.get_window_size().to_string();
                
                new_config.search.fuzzy_search = settings.get_fuzzy_search();
                new_config.search.search_delay_ms = settings.get_search_delay() as u32;
                
                new_config.startup.enabled = settings.get_run_on_startup();
                new_config.startup.show_on_startup = settings.get_show_on_startup();
                
                new_config.update.auto_check = settings.get_auto_check_updates();
                
                // 2. Save to disk
                new_config.save();
                log::info!("Configuration saved to disk");
                
                // 3. Update startup registration
                if new_config.startup.enabled {
                    let _ = startup::enable_startup();
                } else {
                    let _ = startup::disable_startup();
                }
                
                // 4. Update Launcher UI live if possible
                let opacity = new_config.appearance.opacity;
                let _ = launcher_weak_apply.upgrade_in_event_loop(move |_launcher| {
                    log::info!("Live update: Applying opacity {} to launcher", opacity);
                });
            }
        });

        // Handle Reset callback
        let settings_weak = settings.as_weak();
        settings.on_reset_clicked(move || {
            if let Some(settings) = settings_weak.upgrade() {
                log::info!("Settings reset to defaults");
                let default_config = AppConfig::default();
                
                settings.set_theme(default_config.appearance.theme.clone().into());
                settings.set_window_opacity(default_config.appearance.opacity as f32);
                settings.set_max_results(default_config.appearance.max_results as f32);
                settings.set_font_size(default_config.appearance.font_size as f32);
                settings.set_window_size(default_config.appearance.window_size.clone().into());
                
                settings.set_fuzzy_search(default_config.search.fuzzy_search);
                settings.set_search_delay(default_config.search.search_delay_ms as f32);
                
                settings.set_run_on_startup(default_config.startup.enabled);
                settings.set_show_on_startup(default_config.startup.show_on_startup);
                
                settings.set_auto_check_updates(default_config.update.auto_check);
            }
        });

        // Handle Config Folder callback
        settings.on_open_config_folder(move || {
            if let Some(config_dir) = AppConfig::config_dir() {
                let _ = std::process::Command::new("explorer").arg(config_dir).spawn();
            }
        });

        // Handle Check Updates callback
        let settings_weak = settings.as_weak();
        settings.on_check_updates(move || {
            if let Some(settings) = settings_weak.upgrade() {
                settings.set_update_status("Checking for updates...".into());
                
                let settings_weak_cb = settings_weak.clone();
                let _ = std::thread::spawn(move || {
                    // Call the actual updater
                    let result = crate::updater::check_for_updates(false);
                    
                    // Convert result to something Send + 'static
                    let response = match result {
                        Ok(Some(info)) => Ok(Some(info)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(e.to_string()),
                    };
                    
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(settings) = settings_weak_cb.upgrade() {
                            match response {
                                Ok(Some(info)) => {
                                    settings.set_update_status(format!("New version {} available!", info.version).into());
                                }
                                Ok(None) => {
                                    settings.set_update_status("Your software is up to date".into());
                                }
                                Err(e) => {
                                    settings.set_update_status(format!("Update failed: {}", e).into());
                                }
                            }
                        }
                    });
                });
            }
        });

        settings.show()?;
        
        Ok(Self { window: settings })
    }

    /// Bring the settings window to the front
    #[allow(dead_code)]
    pub fn request_focus(&self) {
        let _ = self.window.show();
    }
}
