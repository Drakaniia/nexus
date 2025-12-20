//! Application discovery module
//! Scans Start Menu, Desktop, and UWP apps to build a searchable index

use std::path::PathBuf;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use crate::{AppEntry, AppType};

/// Discover all installed applications
pub fn discover_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    // Scan common Start Menu locations
    if let Some(start_menu) = dirs::data_dir() {
        let paths = [
            start_menu.join("Microsoft\\Windows\\Start Menu\\Programs"),
        ];
        
        for path in &paths {
            scan_directory(path, &mut apps, 3);
        }
    }

    // Scan ProgramData Start Menu
    let program_data_start = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    if program_data_start.exists() {
        scan_directory(&program_data_start, &mut apps, 3);
    }

    // Scan user's Desktop
    if let Some(desktop) = dirs::desktop_dir() {
        scan_directory(&desktop, &mut apps, 1);
    }

    // Add common system utilities
    apps.extend(get_system_apps());

    // Deduplicate by name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    apps.retain(|app| seen.insert(app.name.to_lowercase()));

    apps
}

/// Recursively scan a directory for .lnk and .exe files
fn scan_directory(dir: &PathBuf, apps: &mut Vec<AppEntry>, max_depth: u32) {
    if max_depth == 0 || !dir.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Skip certain directories
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if !dir_name.starts_with('.') && dir_name != "Startup" {
                    scan_directory(&path, apps, max_depth - 1);
                }
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                
                if ext_str == "lnk" || ext_str == "exe" {
                    if let Some(app) = parse_shortcut(&path) {
                        apps.push(app);
                    }
                }
            }
        }
    }
}

/// Parse a shortcut (.lnk) or executable file into an AppEntry
fn parse_shortcut(path: &PathBuf) -> Option<AppEntry> {
    let file_name = path.file_stem()
        .and_then(|n| n.to_str())?;

    // Skip uninstall entries and other system shortcuts
    let lower_name = file_name.to_lowercase();
    if lower_name.contains("uninstall") 
        || lower_name.contains("readme")
        || lower_name.contains("help")
        || lower_name.contains("manual")
        || lower_name.starts_with(".")
    {
        return None;
    }

    // Clean up the name
    let name = file_name
        .replace(" - Shortcut", "")
        .replace(".lnk", "")
        .trim()
        .to_string();

    if name.is_empty() {
        return None;
    }

    // Get the parent folder as description
    let description = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Application".to_string());

    Some(AppEntry {
        name,
        path: path.clone(),
        description,
        app_type: AppType::DesktopApp,
    })
}

/// Get common system applications
fn get_system_apps() -> Vec<AppEntry> {
    vec![
        AppEntry {
            name: "Settings".to_string(),
            path: PathBuf::from("ms-settings:"),
            description: "Windows Settings".to_string(),
            app_type: AppType::UwpApp,
        },
        AppEntry {
            name: "Control Panel".to_string(),
            path: PathBuf::from("control.exe"),
            description: "System Control Panel".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Task Manager".to_string(),
            path: PathBuf::from("taskmgr.exe"),
            description: "System Task Manager".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "File Explorer".to_string(),
            path: PathBuf::from("explorer.exe"),
            description: "Windows File Explorer".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Command Prompt".to_string(),
            path: PathBuf::from("cmd.exe"),
            description: "Windows Command Line".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "PowerShell".to_string(),
            path: PathBuf::from("powershell.exe"),
            description: "Windows PowerShell".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Calculator".to_string(),
            path: PathBuf::from("calc.exe"),
            description: "Windows Calculator".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Notepad".to_string(),
            path: PathBuf::from("notepad.exe"),
            description: "Text Editor".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Snipping Tool".to_string(),
            path: PathBuf::from("snippingtool.exe"),
            description: "Screenshot Tool".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Device Manager".to_string(),
            path: PathBuf::from("devmgmt.msc"),
            description: "Hardware Management".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Disk Management".to_string(),
            path: PathBuf::from("diskmgmt.msc"),
            description: "Disk Partitioning".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Event Viewer".to_string(),
            path: PathBuf::from("eventvwr.msc"),
            description: "System Event Logs".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "Registry Editor".to_string(),
            path: PathBuf::from("regedit.exe"),
            description: "Windows Registry".to_string(),
            app_type: AppType::DesktopApp,
        },
        AppEntry {
            name: "System Information".to_string(),
            path: PathBuf::from("msinfo32.exe"),
            description: "System Details".to_string(),
            app_type: AppType::DesktopApp,
        },
    ]
}

/// Discover UWP/Store apps (placeholder for future implementation)
#[allow(dead_code)]
pub fn discover_uwp_apps() -> Vec<AppEntry> {
    // This would use Windows.Management.Deployment.PackageManager
    // to enumerate installed UWP apps
    Vec::new()
}
