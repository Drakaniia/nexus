//! System Tray Integration Module
//! Creates and manages the system tray icon with context menu

use std::sync::mpsc::{channel, Receiver, TryRecvError};
use tray_icon::{menu::{CheckMenuItem, IconId, Menu, MenuItem, PredefinedMenuItem}, TrayIconBuilder, TrayIconEvent, TrayIconId};
use std::path::PathBuf;

/// Tray event types
#[derive(Debug, Clone)]
pub enum TrayEvent {
    Show,
    LeftClick,
    Settings,
    Exit,
}

/// Manages the system tray icon and events
pub struct TrayManager {
    _tray_icon: tray_icon::TrayIcon,
    event_receiver: Receiver<TrayIconEvent>,
}

// Define a unique ID for our tray icon
const TRAY_ID: TrayIconId = TrayIconId::new("winlauncher-tray");

impl TrayManager {
    /// Create a new tray manager with the system tray icon
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu items
        let show_item = MenuItem::new("Show", true, None);
        let settings_item = MenuItem::new("Settings", true, None);
        let quit_item = MenuItem::new("Exit", true, None);
        
        // Create the context menu
        let tray_menu = Menu::with_items(&[
            &show_item,
            &PredefinedMenuItem::separator(),
            &settings_item,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ]).map_err(|e| format!("Failed to create tray menu: {}", e))?;

        // Create tray icon
        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("WinLauncher - Quick App Launcher")
            .with_id(TRAY_ID);

        // Try to load an icon from resources or use a default
        if let Some(icon) = Self::load_icon() {
            builder = builder.with_icon(icon);
        }

        let (event_sender, event_receiver) = channel();
        
        let tray_icon = builder
            .build_with_channel(Some(event_sender))
            .map_err(|e| format!("Failed to create tray icon: {}", e))?;

        Ok(Self {
            _tray_icon: tray_icon,
            event_receiver,
        })
    }

    /// Load icon for the tray (try to load from resources)
    fn load_icon() -> Option<tray_icon::Icon> {
        // Try to load from embedded resource or default icon
        // This is a placeholder - in a real app, you'd embed an icon
        // For now, we'll create a simple 32x32 pixel icon
        let pixels = vec![255u8; 32 * 32 * 4]; // All white pixels (RGBA)
        tray_icon::Icon::from_rgba(pixels, 32, 32).ok()
    }

    /// Check for any pending tray events
    pub fn check_events(&self) -> Option<TrayEvent> {
        match self.event_receiver.try_recv() {
            Ok(event) => {
                match event.id {
                    id if id == TrayIconEvent::id(&self._tray_icon) => {
                        // Handle left-click on tray icon
                        Some(TrayEvent::LeftClick)
                    }
                    id if id == TrayIconEvent::id(&self._tray_icon) => {
                        // Handle right-click context menu handled by OS
                        None
                    }
                    _ => {
                        // Check for specific menu item clicks
                        if let Some(menu_item_id) = event.menu_item_id() {
                            // Compare with our menu items
                            if menu_item_id == &IconId::from(0) { // Show item
                                Some(TrayEvent::Show)
                            } else if menu_item_id == &IconId::from(2) { // Settings item
                                Some(TrayEvent::Settings)
                            } else if menu_item_id == &IconId::from(4) { // Quit item
                                Some(TrayEvent::Exit)
                            } else {
                                None
                            }
                        } else {
                            // Handle left-click as showing the launcher
                            Some(TrayEvent::LeftClick)
                        }
                    }
                }
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_event_enum() {
        let show_event = TrayEvent::Show;
        match show_event {
            TrayEvent::Show => assert!(true), // This should pass
            _ => assert!(false),
        }
    }
}