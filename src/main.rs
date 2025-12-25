//! Nexus - Windows Launcher Application
//! A lightweight, fast launcher with modern Windows 11 aesthetics

#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::path::PathBuf;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use slint::{Model, SharedString, VecModel, LogicalPosition, CloseRequestResponse, ComponentHandle};

// Windows API imports for monitor positioning
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Graphics::Gdi::{
    MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};


mod ui;

// Re-export generated UI types
pub use ui::{Launcher, SearchResult};
mod actions;
mod app_discovery;
mod config;
mod platform_window;
mod search;
mod single_instance;
mod startup;
mod tray;
mod wizard;
mod settings_ui;
mod updater;

use updater::UpdateInfo;

use config::AppConfig;
use single_instance::{SingleInstance, PortableMode, detect_portable_mode};
use tray::{TrayEvent, TrayManager, check_tray_event};

/// Parse modifier string to Modifiers enum
fn parse_modifier(modifier: &str) -> Option<Modifiers> {
    match modifier.to_lowercase().as_str() {
        "alt" => Some(Modifiers::ALT),
        "ctrl" | "control" => Some(Modifiers::CONTROL),
        "shift" => Some(Modifiers::SHIFT),
        "win" | "super" | "meta" => Some(Modifiers::META),
        _ => None,
    }
}

/// Parse key string to Code enum
fn parse_key(key: &str) -> Option<Code> {
    match key.to_lowercase().as_str() {
        "space" => Some(Code::Space),
        "enter" | "return" => Some(Code::Enter),
        "escape" | "esc" => Some(Code::Escape),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        "home" => Some(Code::Home),
        "end" => Some(Code::End),
        "pageup" => Some(Code::PageUp),
        "pagedown" => Some(Code::PageDown),
        "arrowup" | "uparrow" => Some(Code::ArrowUp),
        "arrowdown" | "downarrow" => Some(Code::ArrowDown),
        "arrowleft" | "leftarrow" => Some(Code::ArrowLeft),
        "arrowright" | "rightarrow" => Some(Code::ArrowRight),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        // Add single letter keys for common shortcuts
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        _ => None,
    }
}

/// Create a HotKey from config
fn create_hotkey_from_config(config: &AppConfig) -> Result<HotKey, String> {
    // For now, support only the first modifier (simplified implementation)
    let modifier_opt = if config.hotkey.modifiers.is_empty() {
        None
    } else {
        Some(parse_modifier(&config.hotkey.modifiers[0])
            .ok_or_else(|| format!("Unknown modifier: {}", config.hotkey.modifiers[0]))?)
    };

    // Parse key
    let key = parse_key(&config.hotkey.key)
        .ok_or_else(|| format!("Unknown key: {}", config.hotkey.key))?;

    // Create hotkey
    Ok(HotKey::new(modifier_opt, key))
}

/// Application state
struct LauncherState {
    apps: Vec<AppEntry>,
    config: AppConfig,
}

/// Represents a discovered application
#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub app_type: AppType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppType {
    DesktopApp,
    UwpApp,
    File,
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::DesktopApp => write!(f, "Desktop"),
            AppType::UwpApp => write!(f, "UWP"),
            AppType::File => write!(f, "File"),
        }
    }
}

impl LauncherState {
    fn new(config: AppConfig) -> Self {
        Self {
            apps: Vec::new(),
            config,
        }
    }

    /// Two-tier search: prefix matching (high priority) + fuzzy matching (fallback)
    fn search(&self, query: &str) -> Vec<SearchResultData> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Check for special prefixes first
        if let Some(action_result) = actions::check_special_query(query) {
            return vec![action_result];
        }

        // Check for calculator expression
        if let Some(calc_result) = actions::try_calculate(query) {
            results.push(calc_result);
        }

        // Check for web search
        if let Some(web_result) = actions::check_web_search(query) {
            results.push(web_result);
        }

        // === TIER 1: Prefix Matching (Highest Priority) ===
        let mut prefix_matches: Vec<(&AppEntry, i64)> = Vec::new();
        let mut fuzzy_only_matches: Vec<(&AppEntry, i64)> = Vec::new();

        for app in &self.apps {
            let name_lower = app.name.to_lowercase();
            let mru_bonus = (self.config.get_mru_score(&app.name) as i64) * 10;

            // Check if name starts with query
            if name_lower.starts_with(&query_lower) {
                // Exact prefix match - highest score
                let score = 1000 + mru_bonus + (100 - name_lower.len() as i64);
                prefix_matches.push((app, score));
                continue;
            }

            // Check if any word starts with query
            let words: Vec<&str> = name_lower.split_whitespace().collect();
            let mut word_match = false;
            for word in &words {
                if word.starts_with(&query_lower) {
                    let score = 800 + mru_bonus;
                    prefix_matches.push((app, score));
                    word_match = true;
                    break;
                }
            }

            if word_match {
                continue;
            }

            // Check initials match (e.g., "vsc" matches "Visual Studio Code")
            if query.len() >= 2 {
                let initials: String = words
                    .iter()
                    .filter_map(|w| w.chars().next())
                    .collect();
                if initials.starts_with(&query_lower) {
                    let score = 700 + mru_bonus;
                    prefix_matches.push((app, score));
                    continue;
                }
            }

            // === TIER 2: Fuzzy Matching (Fallback) ===
            // Check if query is a subsequence of name
            if is_subsequence(&query_lower, &name_lower) {
                let score = 300 + mru_bonus;
                fuzzy_only_matches.push((app, score));
            } else if name_lower.contains(&query_lower) {
                // Substring match
                let score = 200 + mru_bonus;
                fuzzy_only_matches.push((app, score));
            }
        }

        // Sort prefix matches by score
        prefix_matches.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Sort fuzzy matches by score
        fuzzy_only_matches.sort_by(|a, b| b.1.cmp(&a.1));

        // Combine: prefix matches first, then fuzzy matches
        let max_results = self.config.appearance.max_results;
        let mut app_count = 0;

        for (app, _score) in prefix_matches.into_iter() {
            if app_count >= max_results {
                break;
            }
            results.push(SearchResultData {
                name: app.name.clone(),
                description: app.description.clone(),
                path: app.path.clone(),
                result_type: match app.app_type {
                    AppType::DesktopApp | AppType::UwpApp => "app".to_string(),
                    AppType::File => "file".to_string(),
                },
            });
            app_count += 1;
        }

        // Add fuzzy matches if we have room
        for (app, _score) in fuzzy_only_matches.into_iter() {
            if app_count >= max_results {
                break;
            }
            results.push(SearchResultData {
                name: app.name.clone(),
                description: app.description.clone(),
                path: app.path.clone(),
                result_type: match app.app_type {
                    AppType::DesktopApp | AppType::UwpApp => "app".to_string(),
                    AppType::File => "file".to_string(),
                },
            });
            app_count += 1;
        }

        results
    }

    fn record_usage(&mut self, name: &str) {
        self.config.record_usage(name);
    }
}

/// Check if pattern is a subsequence of text
fn is_subsequence(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    
    for ch in text.chars() {
        if let Some(&p) = pattern_chars.peek() {
            if ch == p {
                pattern_chars.next();
            }
        } else {
            return true;
        }
    }
    
    pattern_chars.peek().is_none()
}

/// Search result data for passing between Rust and Slint
#[derive(Clone)]
pub struct SearchResultData {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub result_type: String,
}

impl From<&SearchResultData> for SearchResult {
    fn from(data: &SearchResultData) -> Self {
        SearchResult {
            name: SharedString::from(&data.name),
            description: SharedString::from(&data.description),
            icon_path: SharedString::new(),
            result_type: SharedString::from(&data.result_type),
        }
    }
}

/// Show a system notification about available updates
fn show_update_notification(update_info: &UpdateInfo) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Showing update notification for version {}", update_info.version);

    // For now, use Windows toast notifications if available
    // In a real implementation, you might use a proper notification library

    #[cfg(windows)]
    {
        // Try to use Windows toast notifications
        // This is a simplified implementation - in practice you'd use a notification crate
        log::info!("Update available: {}", update_info.version);
        log::info!("Download URL: {}", update_info.download_url);
        log::info!("Release notes: {}", update_info.release_notes);
        // TODO: Implement actual Windows toast notification
    }

    #[cfg(not(windows))]
    {
        log::info!("Update notification: {} is available", update_info.version);
    }

    Ok(())
}

/// Get the center position for the launcher window on the monitor where the cursor is located.
/// Returns a LogicalPosition for use with Slint's set_position method.
fn get_window_center_position() -> LogicalPosition {
    const WINDOW_WIDTH: i32 = 680;
    const WINDOW_HEIGHT: i32 = 200; // Approximate height

    unsafe {
        // Get cursor position
        let mut cursor_pos = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos).is_ok() {
            // Get the monitor where the cursor is located
            let hmonitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTONEAREST);
            
            let mut monitor_info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            
            if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
                let work_area = monitor_info.rcWork;
                let monitor_width = work_area.right - work_area.left;
                let monitor_height = work_area.bottom - work_area.top;
                
                let x = work_area.left + (monitor_width - WINDOW_WIDTH) / 2;
                let y = work_area.top + (monitor_height - WINDOW_HEIGHT) / 3; // Upper third for better UX
                
                log::debug!("Window position: ({}, {}) on monitor at ({}, {})", x, y, work_area.left, work_area.top);
                return LogicalPosition::new(x as f32, y as f32);
            }
        }
        
        // Fallback to screen center (primary monitor)
        log::debug!("Using fallback screen center");
        LogicalPosition::new(400.0, 200.0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up panic handler to log panics instead of crashing silently
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Application panic: {:?}", panic_info);
        // Don't exit here - let the application try to continue
    }));

    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    log::info!("Starting Nexus...");

    // === DETECT PORTABLE MODE ===
    let portable_mode = detect_portable_mode();
    log::info!("Application mode: {:?}", portable_mode);

    // === SINGLE INSTANCE CHECK (must be first!) ===
    let _instance_lock = match SingleInstance::acquire_with_mode(portable_mode) {
        Ok(lock) => {
            log::info!("Single instance lock acquired");
            lock
        }
        Err(e) => {
            log::info!("Another instance is running: {}", e);
            return Ok(()); // Exit gracefully - other instance will show
        }
    };

    // === LOAD CONFIGURATION ===
    let mut config = AppConfig::load_with_mode(portable_mode);
    
    // First run setup - show wizard if first run
    if config.is_first_run() {
        log::info!("First run detected, showing setup wizard");

        // For portable mode, disable startup by default since portable apps shouldn't modify system settings
        if matches!(portable_mode, PortableMode::Portable) {
            config.startup.enabled = false;
            log::info!("Portable mode detected - startup registration disabled");
        }

        // Show the wizard and let user configure the application
        match wizard::show_wizard(&mut config) {
            Ok(_) => {
                log::info!("Wizard completed successfully");

                // Apply startup registration based on wizard settings (skip for portable mode)
                if matches!(portable_mode, PortableMode::Portable) {
                    log::info!("Skipping startup registration for portable mode");
                } else if config.startup.enabled {
                    if let Err(e) = startup::enable_startup() {
                        log::warn!("Failed to enable startup: {}", e);
                    }
                } else {
                    // Ensure startup is disabled if user unchecked it
                    if let Err(e) = startup::disable_startup() {
                        log::warn!("Failed to disable startup: {}", e);
                    }
                }

                // Verify startup registration integrity (skip for portable mode)
                if !matches!(portable_mode, PortableMode::Portable) {
                    if let Err(e) = startup::verify_startup_registration() {
                        log::warn!("Failed to verify startup registration: {}", e);
                    }
                }

                // Mark first run as complete
                config.complete_first_run();
                config.save_with_mode(portable_mode);
            }
            Err(e) => {
                log::warn!("Wizard error or cancelled: {}", e);
                log::info!("Continuing with default configuration");

                // Still register startup if enabled in default config (skip for portable mode)
                if matches!(portable_mode, PortableMode::Portable) {
                    log::info!("Skipping startup registration for portable mode");
                } else if config.startup.enabled {
                    if let Err(e) = startup::enable_startup() {
                        log::warn!("Failed to enable startup: {}", e);
                    }
                }

                // Mark as completed even if wizard failed/cancelled
                config.complete_first_run();
                config.save_with_mode(portable_mode);
            }
        }
    }

    // === CREATE SYSTEM TRAY ===
    let _tray = TrayManager::new()?;
    log::info!("System tray created");

    // === CREATE UI ===
    let launcher = Launcher::new()?;
    log::info!("Launcher UI created");

    // Note: Dummy window removed - modern Slint should keep event loop alive with main window

    // CRITICAL: Prevent window close from exiting the event loop!
    // When user clicks X or closes the window, we hide it but keep it "shown" to Slint
    let launcher_weak_for_close = launcher.as_weak();
    launcher.window().on_close_requested(move || {
        log::info!("=== WINDOW CLOSE REQUESTED ===");
        log::info!("Hiding window but keeping it 'shown' to prevent event loop exit");

        // Move window off-screen and hide it, but return KeepWindowShown so Slint doesn't exit
        if let Some(launcher) = launcher_weak_for_close.upgrade() {
            let window = slint::ComponentHandle::window(&launcher);
            window.set_position(slint::LogicalPosition::new(-10000.0, -10000.0));
            launcher.hide().ok();
            launcher.set_is_visible(false);
            log::info!("Window moved off-screen and hidden successfully");
        } else {
            log::error!("Failed to upgrade launcher weak reference in close handler!");
        }

        log::info!("Returning KeepWindowShown to prevent event loop exit");
        CloseRequestResponse::KeepWindowShown
    });

    let launcher_weak = launcher.as_weak();

    // Initialize application state with config
    let state = Arc::new(Mutex::new(LauncherState::new(config.clone())));
    let current_results: Arc<Mutex<Vec<SearchResultData>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Flag to control app running state
    let app_running = Arc::new(AtomicBool::new(true));

    // Discover installed applications in background
    {
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            log::info!("Starting app discovery...");
            let apps = app_discovery::discover_apps();
            log::info!("App discovery completed: found {} applications", apps.len());

            // Log some examples
            for (i, app) in apps.iter().take(5).enumerate() {
                log::debug!("  Discovered app {}: {} ({})", i+1, app.name, app.app_type);
            }
            if apps.len() > 5 {
                log::debug!("  ... and {} more apps", apps.len() - 5);
            }

            if let Ok(mut state) = state.lock() {
                state.apps = apps;
                log::info!("App discovery results stored in state");
            } else {
                log::error!("Failed to store discovered apps in state!");
            }
        });
    }

    // Set up global hotkey from config
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let hotkey = create_hotkey_from_config(&config)
        .map_err(|e| format!("Failed to create hotkey from config: {}", e))?;
    let hotkey_id = hotkey.id();
    hotkey_manager.register(hotkey.clone())?;
    log::info!("Registered hotkey: {} + {}", config.hotkey.modifiers.join("+"), config.hotkey.key);

    // Track when the window was last shown to avoid immediate hiding due to focus race condition
    let last_shown_time = Arc::new(Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(10))); // Start with old timestamp

    // Handle hotkey events
    let launcher_weak_hotkey = launcher_weak.clone();
    let receiver = GlobalHotKeyEvent::receiver();
    let app_running_hotkey = Arc::clone(&app_running);
    let last_shown_time_hotkey = Arc::clone(&last_shown_time);

    std::thread::spawn(move || {
        loop {
            if !app_running_hotkey.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                if event.id == hotkey_id && event.state == HotKeyState::Pressed {
                    // Get window position BEFORE upgrading to event loop (avoid blocking main thread)
                    let position = get_window_center_position();
                    log::info!("Hotkey pressed, centering window at ({}, {})", position.x, position.y);

                    let last_shown_time_clone = Arc::clone(&last_shown_time_hotkey);
                    let _ = launcher_weak_hotkey.upgrade_in_event_loop(move |launcher: Launcher| {
                        let is_visible = launcher.get_is_visible();
                        if is_visible {
                            // Move off-screen and hide, but keep "shown" to prevent event loop exit
                            launcher.window().set_position(slint::LogicalPosition::new(-10000.0, -10000.0));
                            launcher.hide().ok();
                            launcher.set_is_visible(false);
                            log::debug!("Window hidden (moved off-screen)");
                        } else {
                            // Update last shown time to prevent immediate hiding due to focus race
                            *last_shown_time_clone.lock().unwrap() = std::time::Instant::now();

                            // Position window correctly (not off-screen)
                            launcher.window().set_position(position);

                            // Show the window FIRST (required for window handle to be valid)
                            launcher.show().ok();
                            launcher.set_is_visible(true);

                            // Configure platform-specific window styles (no taskbar, topmost)
                            // This MUST happen after show() to ensure HWND is valid
                            if let Err(e) = platform_window::configure_launcher_window(launcher.window()) {
                                log::warn!("Failed to configure window styles: {}", e);
                            }

                            // Clear search and prepare UI first
                            launcher.set_search_text("".into());
                            launcher.invoke_clear_search();
                            launcher.set_selected_index(0);

                            // Enable focus for the launcher window so it can receive keyboard input
                            log::debug!("Enabling focus for launcher window...");
                            if let Err(e) = platform_window::enable_launcher_focus(launcher.window()) {
                                log::warn!("Failed to enable focus for launcher: {}", e);
                            } else {
                                log::debug!("Focus enabled successfully");
                            }

                            // Small delay to ensure Windows focus APIs have taken effect
                            std::thread::sleep(std::time::Duration::from_millis(10));

                            // Now focus the input field
                            launcher.invoke_focus_input();
                            log::debug!("Window shown and focused (hotkey)");
                        }
                    });
                }
            }
        }
    });

    // === HANDLE TRAY MENU EVENTS ===
    // Note: TrayManager must stay on main thread, but we can check events from any thread
    // because MenuEvent::receiver() is a global static
    let launcher_weak_tray = launcher_weak.clone();
    let app_running_tray = Arc::clone(&app_running);
    let config_for_tray = config.clone();
    let last_shown_time_tray = Arc::clone(&last_shown_time);

    std::thread::spawn(move || {
        loop {
            if !app_running_tray.load(Ordering::Relaxed) {
                break;
            }

            match check_tray_event() {
                TrayEvent::Show => {
                    log::info!("Tray: Show clicked");
                    // Get window position before upgrading
                    let position = get_window_center_position();

                    let last_shown_time_clone = Arc::clone(&last_shown_time_tray);
                    let _ = launcher_weak_tray.upgrade_in_event_loop(move |launcher: Launcher| {
                        // Update last shown time to prevent immediate hiding due to focus race
                        *last_shown_time_clone.lock().unwrap() = std::time::Instant::now();

                        // Position window first using Slint's built-in method
                        launcher.window().set_position(position);

                        // Show the window FIRST (required for window handle to be valid)
                        launcher.show().ok();
                        launcher.set_is_visible(true);

                        // Configure platform-specific window styles (no taskbar, topmost)
                        if let Err(e) = platform_window::configure_launcher_window(launcher.window()) {
                            log::warn!("Failed to configure window styles: {}", e);
                        }

                        // Enable focus for the launcher window so it can receive keyboard input
                        log::debug!("Enabling focus for launcher window (tray)...");
                        if let Err(e) = platform_window::enable_launcher_focus(launcher.window()) {
                            log::warn!("Failed to enable focus for launcher: {}", e);
                        } else {
                            log::debug!("Focus enabled successfully (tray)");
                        }

                        // Clear search state
                        launcher.set_search_text("".into());
                        launcher.invoke_clear_search();
                        launcher.set_selected_index(0);

                        // Small delay to ensure Windows focus APIs have taken effect
                        std::thread::sleep(std::time::Duration::from_millis(10));

                        // Focus the input field
                        launcher.invoke_focus_input();
                        log::debug!("Search cleared and input focused for tray show");
                    });
                }
                TrayEvent::Settings => {
                    log::info!("Tray: Settings clicked");
                    let config_clone = config_for_tray.clone();

                    let launcher_weak_settings = launcher_weak_tray.clone();

                    // Use a thread to show the settings window
                    // In a more complex app we would track the window instance
                    // but for now we'll just spawn it
                    let _ = std::thread::spawn(move || {
                        if let Err(e) = settings_ui::SettingsManager::show(&config_clone, launcher_weak_settings) {
                            log::error!("Failed to show settings: {}", e);
                        }
                    });
                }
                TrayEvent::CheckUpdates => {
                    log::info!("Tray: Check for Updates clicked");

                    // Run update check in a background thread
                    std::thread::spawn(move || {
                        match updater::check_for_updates(false) {
                            Ok(Some(update_info)) => {
                                log::info!("Update available: {} ({})", update_info.version, update_info.published_at);

                                // Show notification about available update
                                if let Err(e) = show_update_notification(&update_info) {
                                    log::warn!("Failed to show update notification: {}", e);
                                }

                                // TODO: Add "Update Now" option to tray menu or show dialog
                                // For now, just log the update info
                            }
                            Ok(None) => {
                                log::info!("No updates available");
                                // TODO: Show "You're up to date" notification
                            }
                            Err(e) => {
                                log::error!("Failed to check for updates: {}", e);
                                // TODO: Show error notification
                            }
                        }
                    });
                }
                TrayEvent::Exit => {
                    log::info!("Tray: Exit clicked - shutting down");
                    app_running_tray.store(false, Ordering::Relaxed);
                    // The application will exit naturally when all threads stop
                }
                TrayEvent::None => {
                    // No event, sleep briefly to avoid busy loop
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
    });

    // === DISABLED: WINDOW FOCUS LOSS MONITORING ===
    // The focus monitoring was causing issues with WS_EX_NOACTIVATE windows
    // which never receive focus. Instead, we rely on explicit user actions
    // (Escape key, item selection) to hide the window.

    // Handle search input changes - UPDATE UI IMMEDIATELY (Fix for Issue #1)
    {
        let state = Arc::clone(&state);
        let current_results = Arc::clone(&current_results);
        let launcher_weak_search = launcher_weak.clone();

        launcher.on_search_changed(move |query: slint::SharedString| {
            let query_str = query.to_string();
            log::debug!("Search changed: '{}' (len: {})", query_str, query_str.len());

            if query_str.is_empty() {
                log::debug!("Query is empty, clearing results");
                // Clear results immediately
                if let Ok(mut results) = current_results.lock() {
                    results.clear();
                }
                // Update UI immediately
                let _ = launcher_weak_search.upgrade_in_event_loop(|launcher: Launcher| {
                    // Always create a fresh model to ensure UI updates properly
                    let model: slint::ModelRc<SearchResult> = std::rc::Rc::new(VecModel::<SearchResult>::default()).into();
                    launcher.set_results(model);
                    launcher.set_selected_index(0);
                    log::debug!("UI cleared - no results shown");
                });
                return;
            }

            // Perform search
            let search_results = if let Ok(state) = state.lock() {
                log::debug!("Searching among {} discovered apps", state.apps.len());

                // If no apps are discovered yet, add a placeholder result
                if state.apps.is_empty() && !query_str.is_empty() {
                    log::debug!("No apps discovered yet, showing calculator/web search only");

                    let mut results = Vec::new();

                    // Check for calculator expression
                    if let Some(calc_result) = actions::try_calculate(&query_str) {
                        results.push(calc_result);
                    }

                    // Check for web search
                    if let Some(web_result) = actions::check_web_search(&query_str) {
                        results.push(web_result);
                    }

                    // Add fallback results for testing if no apps are found
                    if results.is_empty() {
                        // Always add calculator for testing
                        if let Some(calc_result) = actions::try_calculate("2+2") {
                            results.push(calc_result);
                        }

                        // Add web search for testing
                        if let Some(web_result) = actions::check_web_search("test") {
                            results.push(web_result);
                        }

                        // Add a status message
                        results.push(SearchResultData {
                            name: "Type to search applications...".to_string(),
                            description: "Calculator and web search are always available".to_string(),
                            path: std::path::PathBuf::new(),
                            result_type: "info".to_string(),
                        });
                    }
                    results
                } else {
                    let results = state.search(&query_str);
                    log::debug!("Search for '{}' returned {} results", query_str, results.len());

                    // Debug: Log first few results
                    for (i, result) in results.iter().take(3).enumerate() {
                        log::debug!("  Result {}: {} ({})", i+1, result.name, result.result_type);
                    }
                    if results.len() > 3 {
                        log::debug!("  ... and {} more results", results.len() - 3);
                    }
                    results
                }
            } else {
                log::error!("Failed to lock state for search!");
                Vec::new()
            };

            // Store results
            if let Ok(mut results) = current_results.lock() {
                *results = search_results.clone();
            } else {
                log::error!("Failed to lock current_results for storage!");
            }

            // Update UI IMMEDIATELY (not in polling thread)
            let slint_results: Vec<SearchResult> = search_results.iter().map(|r: &SearchResultData| r.into()).collect();
            log::debug!("Converted {} results to Slint format", slint_results.len());

            let _ = launcher_weak_search.upgrade_in_event_loop(move |launcher: Launcher| {
                // Always create a fresh model to ensure UI updates properly
                let model: slint::ModelRc<SearchResult> = std::rc::Rc::new(VecModel::from(slint_results)).into();
                launcher.set_results(model);
                launcher.set_selected_index(0);
                log::debug!("UI updated with {} search results (fresh model)", slint_results.len());
            });
        });
    }

    // Handle result activation - with enhanced logging (Fix for Issue #5)
    {
        let state = Arc::clone(&state);
        let current_results = Arc::clone(&current_results);
        let launcher_weak = launcher_weak.clone();
        
        launcher.on_result_activated(move |index| {
            let index = index as usize;
            log::info!("Result activated at index: {}", index);
            
            if let Ok(results) = current_results.lock() {
                if let Some(result) = results.get(index) {
                    log::info!("Launching: {} (type: {})", result.name, result.result_type);
                    log::info!("Path: {:?}", result.path);
                    
                    // Record usage for MRU
                    if let Ok(mut state) = state.lock() {
                        state.record_usage(&result.name);
                    }

                    // Execute the action with validation
                    match result.result_type.as_str() {
                        "app" | "file" => {
                            // Validate path exists before launching
                            if result.path.exists() {
                                match open::that(&result.path) {
                                    Ok(_) => log::info!("Successfully launched: {}", result.name),
                                    Err(e) => log::error!("Failed to launch {}: {}", result.name, e),
                                }
                            } else {
                                log::error!("Path does not exist: {:?}", result.path);
                            }
                        }
                        "calc" => {
                            // TODO: Copy to clipboard
                            log::info!("Calculator result: {}", result.description);
                        }
                        "web" => {
                            match open::that(&result.path) {
                                Ok(_) => log::info!("Opened URL: {:?}", result.path),
                                Err(e) => log::error!("Failed to open URL: {}", e),
                            }
                        }
                        "action" => {
                            log::info!("Executing system action: {}", result.name);
                            actions::execute_system_action(&result.name);
                        }
                        _ => {
                            log::warn!("Unknown result type: {}", result.result_type);
                        }
                    }

                    // Hide launcher after successful launch (expected behavior for a launcher)
                    let _ = launcher_weak.upgrade_in_event_loop(|launcher: Launcher| {
                        // Move off-screen and hide, but keep "shown" to prevent event loop exit
                        launcher.window().set_position(slint::LogicalPosition::new(-10000.0, -10000.0));
                        launcher.hide().ok();
                        launcher.set_is_visible(false);
                        log::debug!("Window hidden after launch (moved off-screen)");
                    });
                } else {
                    log::warn!("No result found at index {}", index);
                }
            }
        });
    }

    // Handle escape key - hide window but DON'T exit
    {
        let launcher_weak = launcher_weak.clone();
        launcher.on_escape_pressed(move || {
            let _ = launcher_weak.upgrade_in_event_loop(|launcher: Launcher| {
                // Move off-screen and hide, but keep "shown" to prevent event loop exit
                launcher.window().set_position(slint::LogicalPosition::new(-10000.0, -10000.0));
                launcher.hide().ok();
                launcher.set_is_visible(false);
                log::debug!("Window hidden via escape (moved off-screen)");
            });
        });
    }

    // Handle arrow navigation
    {
        let launcher_weak_up = launcher_weak.clone();
        launcher.on_arrow_up(move || {
            let _ = launcher_weak_up.upgrade_in_event_loop(|launcher: Launcher| {
                let current = launcher.get_selected_index();
                if current > 0 {
                    launcher.set_selected_index(current - 1);
                }
            });
        });
    }

    {
        let launcher_weak_down = launcher_weak.clone();
        launcher.on_arrow_down(move || {
            let _ = launcher_weak_down.upgrade_in_event_loop(|launcher: Launcher| {
                let current = launcher.get_selected_index();
                let result_count = launcher.get_results().row_count() as i32;
                if current < result_count - 1 {
                    launcher.set_selected_index(current + 1);
                }
            });
        });
    }

    // Service watchdog thread - ensures tray stays alive and monitors health
    {
        let app_running_watchdog = Arc::clone(&app_running);
        std::thread::spawn(move || {
            log::info!("Service watchdog started");
            while app_running_watchdog.load(Ordering::Relaxed) {
                // Update keepalive file every 30 seconds
                if let Err(e) = single_instance::touch_keepalive() {
                    log::warn!("Failed to update keepalive file: {}", e);
                }
                std::thread::sleep(std::time::Duration::from_secs(30));
                log::debug!("Service watchdog: application is healthy");
            }
            log::info!("Service watchdog shutting down");
        });
    }

    // Background update checker thread - checks for updates every 24 hours
    {
        let app_running_updater = Arc::clone(&app_running);
        std::thread::spawn(move || {
            log::info!("Background update checker started");

            // Check for updates immediately on startup (after a short delay)
            std::thread::sleep(std::time::Duration::from_secs(30));

            while app_running_updater.load(Ordering::Relaxed) {
                log::debug!("Checking for updates in background...");

                match updater::check_for_updates(false) {
                    Ok(Some(update_info)) => {
                        log::info!("Background update check found: {}", update_info.version);
                        if let Err(e) = show_update_notification(&update_info) {
                            log::warn!("Failed to show background update notification: {}", e);
                        }
                    }
                    Ok(None) => {
                        log::debug!("No updates available");
                    }
                    Err(e) => {
                        log::debug!("Background update check failed: {}", e);
                    }
                }

                // Wait 24 hours before next check
                for _ in 0..(24 * 60 * 2) { // Check every 30 seconds for 24 hours
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    if !app_running_updater.load(Ordering::Relaxed) {
                        break;
                    }
                }
            }
            log::info!("Background update checker shutting down");
        });
    }

    // NOTE: UI update polling thread removed - results are now updated immediately in on_search_changed

    // Start "hidden" but keep window "shown" to prevent event loop exit
    // Position off-screen so it's not visible to user
    launcher.window().set_position(slint::LogicalPosition::new(-10000.0, -10000.0));
    launcher.hide()?;
    launcher.set_is_visible(false);
    log::info!("Launcher window positioned off-screen and hidden, but kept 'shown' for event loop");

    // Run the event loop with error recovery
    log::info!("=== EVENT LOOP STARTING ===");
    log::info!("Nexus ready. Press Alt+Space to activate. Running in system tray.");
    log::info!("The event loop should run FOREVER until Exit is clicked in tray menu.");
    log::info!("Window is kept 'shown' off-screen to prevent event loop exit.");

    // Run event loop with error handling - if it fails, log and retry
    loop {
        log::info!("Starting event loop iteration...");
        match slint::run_event_loop() {
            Ok(()) => {
                log::warn!("Event loop returned Ok(()) - this should only happen on explicit exit!");
                log::info!("Checking if user clicked Exit in tray menu...");
                // Check if app_running was set to false (user clicked Exit)
                if !app_running.load(Ordering::Relaxed) {
                    log::info!("User clicked Exit in tray menu - shutting down normally");
                    break;
                } else {
                    log::error!("Event loop exited unexpectedly! Window may have been destroyed.");
                    log::info!("Attempting to restart event loop...");
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
            }
            Err(e) => {
                log::error!("Event loop error: {}. Attempting to restart...", e);
                // Small delay before retry to prevent tight loop on persistent errors
                std::thread::sleep(std::time::Duration::from_millis(1000));
                continue;
            }
        }
    }

    // If we get here, the event loop has exited - this should only happen when user clicks "Exit" in tray
    log::info!("=== EVENT LOOP ENDED ===");
    log::info!("This should ONLY appear when user explicitly exits via tray menu.");
    
    // Cleanup
    log::info!("Nexus shutting down...");
    app_running.store(false, Ordering::Relaxed);

    Ok(())
}
