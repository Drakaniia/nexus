//! System Tray Integration Module
//! Creates and manages the system tray icon with context menu

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

/// Tray event types
#[derive(Debug, Clone, PartialEq)]
pub enum TrayEvent {
    Show,
    Settings,
    Exit,
    None,
}

/// Menu item IDs for event matching
pub const MENU_ID_SHOW: &str = "show";
pub const MENU_ID_SETTINGS: &str = "settings";
pub const MENU_ID_EXIT: &str = "exit";

/// Manages the system tray icon and events
pub struct TrayManager {
    _tray_icon: tray_icon::TrayIcon,
}

impl TrayManager {
    /// Create a new tray manager with the system tray icon
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu items with IDs
        let show_item = MenuItem::with_id(MENU_ID_SHOW, "Show", true, None);
        let settings_item = MenuItem::with_id(MENU_ID_SETTINGS, "Settings", true, None);
        let quit_item = MenuItem::with_id(MENU_ID_EXIT, "Exit", true, None);

        // Create the context menu
        let tray_menu = Menu::with_items(&[
            &show_item,
            &PredefinedMenuItem::separator(),
            &settings_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])
        .map_err(|e| format!("Failed to create tray menu: {}", e))?;

        // Create tray icon
        let builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("WinLauncher - Press Alt+Space to search");

        // Try to load an icon from resources or use a default
        let tray_icon = if let Some(icon) = Self::load_icon() {
            builder.with_icon(icon).build()
        } else {
            builder.build()
        };

        let tray_icon = tray_icon.map_err(|e| format!("Failed to create tray icon: {}", e))?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    /// Load icon for the tray (try to load from resources)
    fn load_icon() -> Option<tray_icon::Icon> {
        // Create a simple 32x32 pixel icon with a gradient
        let mut pixels = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32 {
            for x in 0..32 {
                // Create a simple gradient icon (purple to blue)
                let r = 99u8;  // Purple-ish
                let g = 102u8;
                let b = (200 + (x + y) as u16 / 2).min(255) as u8;
                let a = 255u8;
                pixels.extend_from_slice(&[r, g, b, a]);
            }
        }
        tray_icon::Icon::from_rgba(pixels, 32, 32).ok()
    }
}

/// Check for pending menu events (non-blocking)
/// This function is separate from TrayManager because the MenuEvent receiver
/// is global and can be called from any thread
pub fn check_tray_event() -> TrayEvent {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        let id = event.id.0.as_str();
        if id == MENU_ID_SHOW {
            return TrayEvent::Show;
        } else if id == MENU_ID_SETTINGS {
            return TrayEvent::Settings;
        } else if id == MENU_ID_EXIT {
            return TrayEvent::Exit;
        }
    }
    TrayEvent::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_event_enum() {
        assert_eq!(TrayEvent::Show, TrayEvent::Show);
        assert_ne!(TrayEvent::Show, TrayEvent::Exit);
    }
}