//! Windows Launcher Application
//! A lightweight, fast launcher with modern Windows 11 aesthetics

#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use slint::{Model, SharedString, VecModel, Weak};

slint::include_modules!();

mod app_discovery;
mod search;
mod actions;

/// Application state
struct LauncherState {
    apps: Vec<AppEntry>,
    matcher: SkimMatcherV2,
    mru: HashMap<String, u32>, // Most Recently Used tracking
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
    fn new() -> Self {
        Self {
            apps: Vec::new(),
            matcher: SkimMatcherV2::default().smart_case(),
            mru: HashMap::new(),
        }
    }

    fn search(&self, query: &str) -> Vec<SearchResultData> {
        let mut results = Vec::new();

        // Check for special prefixes
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

        // Fuzzy search through apps
        let mut app_matches: Vec<_> = self.apps
            .iter()
            .filter_map(|app| {
                self.matcher
                    .fuzzy_match(&app.name, query)
                    .map(|score| (app, score))
            })
            .collect();

        // Sort by score (higher is better) and MRU
        app_matches.sort_by(|a, b| {
            let mru_a = self.mru.get(&a.0.name).unwrap_or(&0);
            let mru_b = self.mru.get(&b.0.name).unwrap_or(&0);
            
            // Combine fuzzy score with MRU bonus
            let score_a = a.1 + (*mru_a as i64 * 10);
            let score_b = b.1 + (*mru_b as i64 * 10);
            
            score_b.cmp(&score_a)
        });

        // Take top results
        for (app, _score) in app_matches.into_iter().take(6) {
            results.push(SearchResultData {
                name: app.name.clone(),
                description: app.description.clone(),
                path: app.path.clone(),
                result_type: match app.app_type {
                    AppType::DesktopApp | AppType::UwpApp => "app".to_string(),
                    AppType::File => "file".to_string(),
                },
            });
        }

        results
    }

    fn record_usage(&mut self, name: &str) {
        *self.mru.entry(name.to_string()).or_insert(0) += 1;
    }
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    log::info!("Starting WinLauncher...");

    // Create the Slint UI
    let launcher = Launcher::new()?;
    let launcher_weak = launcher.as_weak();

    // Initialize application state
    let state = Arc::new(Mutex::new(LauncherState::new()));
    let current_results: Arc<Mutex<Vec<SearchResultData>>> = Arc::new(Mutex::new(Vec::new()));

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
    
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == hotkey_id && event.state == HotKeyState::Pressed {
                    let _ = launcher_weak_hotkey.upgrade_in_event_loop(move |launcher| {
                        let is_visible = launcher.get_is_visible();
                        if is_visible {
                            launcher.hide().ok();
                            launcher.set_is_visible(false);
                        } else {
                            launcher.invoke_clear_search();
                            launcher.show().ok();
                            launcher.set_is_visible(true);
                            launcher.invoke_focus_input();
                        }
                    });
                }
            }
        }
    });

    // Handle search input changes
    {
        let state = Arc::clone(&state);
        let current_results = Arc::clone(&current_results);
        launcher.on_search_changed(move |query| {
            let query_str = query.to_string();
            
            if query_str.is_empty() {
                if let Ok(mut results) = current_results.lock() {
                    results.clear();
                }
                return;
            }

            if let Ok(state) = state.lock() {
                let search_results = state.search(&query_str);
                
                if let Ok(mut results) = current_results.lock() {
                    *results = search_results;
                }
            }
        });
    }

    // Handle result activation
    {
        let state = Arc::clone(&state);
        let current_results = Arc::clone(&current_results);
        let launcher_weak = launcher_weak.clone();
        
        launcher.on_result_activated(move |index| {
            let index = index as usize;
            
            if let Ok(results) = current_results.lock() {
                if let Some(result) = results.get(index) {
                    log::info!("Activating: {}", result.name);
                    
                    // Record usage for MRU
                    if let Ok(mut state) = state.lock() {
                        state.record_usage(&result.name);
                    }

                    // Execute the action
                    match result.result_type.as_str() {
                        "app" | "file" => {
                            let _ = open::that(&result.path);
                        }
                        "calc" => {
                            // Copy to clipboard would go here
                            log::info!("Calculator result: {}", result.description);
                        }
                        "web" => {
                            let _ = open::that(&result.path);
                        }
                        "action" => {
                            actions::execute_system_action(&result.name);
                        }
                        _ => {}
                    }

                    // Hide the launcher
                    let _ = launcher_weak.upgrade_in_event_loop(|launcher| {
                        launcher.hide().ok();
                        launcher.set_is_visible(false);
                    });
                }
            }
        });
    }

    // Handle escape key
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

    // Update UI periodically based on search results
    {
        let current_results = Arc::clone(&current_results);
        let launcher_weak = launcher_weak.clone();
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                if let Ok(results) = current_results.lock() {
                    let slint_results: Vec<SearchResult> = results.iter().map(|r| r.into()).collect();
                    let _ = launcher_weak.upgrade_in_event_loop(move |launcher| {
                        let model = std::rc::Rc::new(VecModel::from(slint_results));
                        launcher.set_results(model.into());
                    });
                }
            }
        });
    }

    // Start hidden, waiting for hotkey
    launcher.hide()?;
    launcher.set_is_visible(false);

    // Run the event loop
    log::info!("WinLauncher ready. Press Alt+Space to activate.");
    slint::run_event_loop()?;

    Ok(())
}
