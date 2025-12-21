//! Windows Launcher Application
//! A lightweight, fast launcher with modern Windows 11 aesthetics

#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::path::PathBuf;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use slint::{Model, SharedString, VecModel, LogicalPosition, CloseRequestResponse};

// Windows API imports for monitor positioning
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Graphics::Gdi::{
    MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

slint::include_modules!();

mod actions;
mod app_discovery;
mod config;
mod search;
mod single_instance;
mod startup;
mod tray;

use config::AppConfig;
use single_instance::SingleInstance;
use tray::{TrayEvent, TrayManager, check_tray_event};

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
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    log::info!("Starting WinLauncher...");

    // === SINGLE INSTANCE CHECK (must be first!) ===
    let _instance_lock = match SingleInstance::acquire() {
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
    let mut config = AppConfig::load();
    
    // First run setup
    if config.is_first_run() {
        log::info!("First run detected - registering startup");
        if config.startup.enabled {
            if let Err(e) = startup::enable_startup() {
                log::warn!("Failed to enable startup: {}", e);
            }
        }
        config.complete_first_run();
    }

    // === CREATE SYSTEM TRAY ===
    let _tray = TrayManager::new()?;
    log::info!("System tray created");

    // === CREATE UI ===
    let launcher = Launcher::new()?;
    log::info!("Launcher UI created");

    // CRITICAL: Prevent window close from exiting the event loop!
    // When user clicks X or closes the window, we just hide it instead of exiting
    launcher.window().on_close_requested(|| {
        log::info!("Window close requested - hiding instead of exiting");
        // Return KeepWindowShown to tell Slint NOT to exit the event loop
        // The window will remain "shown" from Slint's perspective but we'll hide it
        CloseRequestResponse::KeepWindowShown
    });

    let launcher_weak = launcher.as_weak();

    // Initialize application state with config
    let state = Arc::new(Mutex::new(LauncherState::new(config)));
    let current_results: Arc<Mutex<Vec<SearchResultData>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Flag to control app running state
    let app_running = Arc::new(AtomicBool::new(true));

    // Discover installed applications in background
    {
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            log::info!("Starting app discovery...");
            let apps = app_discovery::discover_apps();
            log::info!("Discovered {} applications", apps.len());
            
            if let Ok(mut state) = state.lock() {
                state.apps = apps;
            }
        });
    }

    // Set up global hotkey (Alt+Space)
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::Space);
    let hotkey_id = hotkey.id();
    hotkey_manager.register(hotkey)?;
    log::info!("Registered Alt+Space hotkey");

    // Handle hotkey events
    let launcher_weak_hotkey = launcher_weak.clone();
    let receiver = GlobalHotKeyEvent::receiver();
    let app_running_hotkey = Arc::clone(&app_running);

    std::thread::spawn(move || {
        loop {
            if !app_running_hotkey.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                if event.id == hotkey_id && event.state == HotKeyState::Pressed {
                    // Get window position BEFORE upgrading to event loop (avoid blocking main thread)
                    let position = get_window_center_position();
                    log::info!("Alt+Space pressed, centering window at ({}, {})", position.x, position.y);
                    
                    let _ = launcher_weak_hotkey.upgrade_in_event_loop(move |launcher| {
                        let is_visible = launcher.get_is_visible();
                        if is_visible {
                            launcher.hide().ok();
                            launcher.set_is_visible(false);
                            log::debug!("Window hidden");
                        } else {
                            // Position window first using Slint's built-in method
                            launcher.window().set_position(position);
                            
                            // Show the window
                            launcher.show().ok();
                            launcher.set_is_visible(true);
                            
                            // Clear search and focus input
                            launcher.invoke_clear_search();
                            launcher.set_selected_index(0);
                            launcher.invoke_focus_input();
                            log::debug!("Window shown and focused");
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
                    
                    let _ = launcher_weak_tray.upgrade_in_event_loop(move |launcher| {
                        // Position window first using Slint's built-in method
                        launcher.window().set_position(position);
                        
                        // Show the window
                        launcher.show().ok();
                        launcher.set_is_visible(true);
                        
                        launcher.invoke_clear_search();
                        launcher.set_selected_index(0);
                        launcher.invoke_focus_input();
                    });
                }
                TrayEvent::Settings => {
                    log::info!("Tray: Settings clicked (not implemented yet)");
                    // TODO: Show settings dialog
                }
                TrayEvent::Exit => {
                    log::info!("Tray: Exit clicked - shutting down");
                    app_running_tray.store(false, Ordering::Relaxed);
                    // Exit the application
                    std::process::exit(0);
                }
                TrayEvent::None => {
                    // No event, sleep briefly to avoid busy loop
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
    });

    // Handle search input changes - UPDATE UI IMMEDIATELY (Fix for Issue #1)
    {
        let state = Arc::clone(&state);
        let current_results = Arc::clone(&current_results);
        let launcher_weak_search = launcher_weak.clone();
        
        launcher.on_search_changed(move |query| {
            let query_str = query.to_string();
            log::debug!("Search changed: '{}'", query_str);
            
            if query_str.is_empty() {
                // Clear results immediately
                if let Ok(mut results) = current_results.lock() {
                    results.clear();
                }
                // Update UI immediately
                let _ = launcher_weak_search.upgrade_in_event_loop(|launcher| {
                    let model = std::rc::Rc::new(VecModel::<SearchResult>::default());
                    launcher.set_results(model.into());
                    launcher.set_selected_index(0);
                });
                return;
            }

            // Perform search
            let search_results = if let Ok(state) = state.lock() {
                let results = state.search(&query_str);
                log::debug!("Search returned {} results", results.len());
                results
            } else {
                Vec::new()
            };

            // Store results
            if let Ok(mut results) = current_results.lock() {
                *results = search_results.clone();
            }

            // Update UI IMMEDIATELY (not in polling thread)
            let slint_results: Vec<SearchResult> = search_results.iter().map(|r| r.into()).collect();
            let _ = launcher_weak_search.upgrade_in_event_loop(move |launcher| {
                let model = std::rc::Rc::new(VecModel::from(slint_results));
                launcher.set_results(model.into());
                launcher.set_selected_index(0);
                log::debug!("UI updated with search results");
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

                    // Hide the launcher (but keep running in background!)
                    let _ = launcher_weak.upgrade_in_event_loop(|launcher| {
                        launcher.hide().ok();
                        launcher.set_is_visible(false);
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
            let _ = launcher_weak.upgrade_in_event_loop(|launcher| {
                launcher.hide().ok();
                launcher.set_is_visible(false);
            });
        });
    }

    // Handle arrow navigation
    {
        let launcher_weak_up = launcher_weak.clone();
        launcher.on_arrow_up(move || {
            let _ = launcher_weak_up.upgrade_in_event_loop(|launcher| {
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
            let _ = launcher_weak_down.upgrade_in_event_loop(|launcher| {
                let current = launcher.get_selected_index();
                let result_count = launcher.get_results().row_count() as i32;
                if current < result_count - 1 {
                    launcher.set_selected_index(current + 1);
                }
            });
        });
    }

    // NOTE: UI update polling thread removed - results are now updated immediately in on_search_changed

    // Start hidden, waiting for hotkey
    launcher.hide()?;
    launcher.set_is_visible(false);

    // Run the event loop
    log::info!("=== EVENT LOOP STARTING ===");
    log::info!("WinLauncher ready. Press Alt+Space to activate. Running in system tray.");
    log::info!("The event loop should run FOREVER until Exit is clicked in tray menu.");
    
    slint::run_event_loop()?;

    // If we get here, the event loop has exited - this should only happen when user clicks "Exit" in tray
    log::info!("=== EVENT LOOP ENDED ===");
    log::info!("This should ONLY appear when user explicitly exits via tray menu.");
    
    // Cleanup
    log::info!("WinLauncher shutting down...");
    app_running.store(false, Ordering::Relaxed);

    Ok(())
}
