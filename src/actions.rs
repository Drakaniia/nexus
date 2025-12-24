//! Actions module
//! Handles special queries like calculator, web search, and system commands

use std::path::PathBuf;
use crate::SearchResultData;

/// Check for special query prefixes
pub fn check_special_query(query: &str) -> Option<SearchResultData> {
    let query_lower = query.to_lowercase().trim().to_string();
    
    // System commands
    match query_lower.as_str() {
        "lock" => Some(SearchResultData {
            name: "Lock Computer".to_string(),
            description: "Lock your workstation".to_string(),
            path: PathBuf::from("lock"),
            result_type: "action".to_string(),
        }),
        "sleep" => Some(SearchResultData {
            name: "Sleep".to_string(),
            description: "Put computer to sleep".to_string(),
            path: PathBuf::from("sleep"),
            result_type: "action".to_string(),
        }),
        "restart" | "reboot" => Some(SearchResultData {
            name: "Restart".to_string(),
            description: "Restart your computer".to_string(),
            path: PathBuf::from("restart"),
            result_type: "action".to_string(),
        }),
        "shutdown" | "shut down" => Some(SearchResultData {
            name: "Shutdown".to_string(),
            description: "Shut down your computer".to_string(),
            path: PathBuf::from("shutdown"),
            result_type: "action".to_string(),
        }),
        "logout" | "sign out" | "logoff" => Some(SearchResultData {
            name: "Sign Out".to_string(),
            description: "Sign out of your account".to_string(),
            path: PathBuf::from("logout"),
            result_type: "action".to_string(),
        }),
        "empty trash" | "empty recycle bin" => Some(SearchResultData {
            name: "Empty Recycle Bin".to_string(),
            description: "Permanently delete items in Recycle Bin".to_string(),
            path: PathBuf::from("emptytrash"),
            result_type: "action".to_string(),
        }),
        _ => None,
    }
}

/// Try to evaluate a mathematical expression
pub fn try_calculate(query: &str) -> Option<SearchResultData> {
    // Skip if query doesn't look like math
    if query.is_empty() {
        return None;
    }
    
    // Check if it contains math-like characters
    let has_math = query.chars().any(|c| {
        matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')' | '%')
    }) || query.contains("sqrt") || query.contains("sin") || query.contains("cos");
    
    if !has_math {
        return None;
    }
    
    // Try to evaluate
    match meval::eval_str(query) {
        Ok(result) => {
            // Format the result nicely
            let result_str = if result.fract() == 0.0 && result.abs() < 1e15 {
                format!("{}", result as i64)
            } else {
                format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            };
            
            Some(SearchResultData {
                name: format!("= {}", result_str),
                description: format!("{} = {}", query, result_str),
                path: PathBuf::from(result_str),
                result_type: "calc".to_string(),
            })
        }
        Err(_) => None,
    }
}

/// Check for web search shortcuts
pub fn check_web_search(query: &str) -> Option<SearchResultData> {
    let query_lower = query.to_lowercase();
    
    // Google search: "g query" or "google query"
    if let Some(search_term) = query_lower.strip_prefix("g ").or_else(|| query_lower.strip_prefix("google ")) {
        if !search_term.is_empty() {
            let url = format!("https://www.google.com/search?q={}", urlencoding(search_term));
            return Some(SearchResultData {
                name: format!("Search Google: {}", search_term),
                description: "Open Google search in browser".to_string(),
                path: PathBuf::from(url),
                result_type: "web".to_string(),
            });
        }
    }
    
    // YouTube search: "yt query"
    if let Some(search_term) = query_lower.strip_prefix("yt ").or_else(|| query_lower.strip_prefix("youtube ")) {
        if !search_term.is_empty() {
            let url = format!("https://www.youtube.com/results?search_query={}", urlencoding(search_term));
            return Some(SearchResultData {
                name: format!("Search YouTube: {}", search_term),
                description: "Open YouTube search in browser".to_string(),
                path: PathBuf::from(url),
                result_type: "web".to_string(),
            });
        }
    }
    
    // GitHub search: "gh query"
    if let Some(search_term) = query_lower.strip_prefix("gh ").or_else(|| query_lower.strip_prefix("github ")) {
        if !search_term.is_empty() {
            let url = format!("https://github.com/search?q={}", urlencoding(search_term));
            return Some(SearchResultData {
                name: format!("Search GitHub: {}", search_term),
                description: "Open GitHub search in browser".to_string(),
                path: PathBuf::from(url),
                result_type: "web".to_string(),
            });
        }
    }
    
    // Wikipedia search: "wiki query"
    if let Some(search_term) = query_lower.strip_prefix("wiki ").or_else(|| query_lower.strip_prefix("wikipedia ")) {
        if !search_term.is_empty() {
            let url = format!("https://en.wikipedia.org/wiki/Special:Search?search={}", urlencoding(search_term));
            return Some(SearchResultData {
                name: format!("Search Wikipedia: {}", search_term),
                description: "Open Wikipedia search in browser".to_string(),
                path: PathBuf::from(url),
                result_type: "web".to_string(),
            });
        }
    }
    
    // Direct URL detection
    if query.starts_with("http://") || query.starts_with("https://") {
        return Some(SearchResultData {
            name: format!("Open URL"),
            description: query.to_string(),
            path: PathBuf::from(query),
            result_type: "web".to_string(),
        });
    }
    
    None
}

/// Simple URL encoding for search queries
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '&' => "%26".to_string(),
            '?' => "%3F".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            '+' => "%2B".to_string(),
            _ if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

/// Execute a system action
pub fn execute_system_action(action: &str) {
    use std::process::Command;
    
    match action.to_lowercase().as_str() {
        "lock computer" => {
            let _ = Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn();
        }
        "sleep" => {
            let _ = Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0", "1", "0"])
                .spawn();
        }
        "restart" => {
            let _ = Command::new("shutdown")
                .args(["/r", "/t", "0"])
                .spawn();
        }
        "shutdown" => {
            let _ = Command::new("shutdown")
                .args(["/s", "/t", "0"])
                .spawn();
        }
        "sign out" => {
            let _ = Command::new("shutdown")
                .args(["/l"])
                .spawn();
        }
        "empty recycle bin" => {
            // Uses PowerShell to empty recycle bin
            let _ = Command::new("powershell")
                .args(["-Command", "Clear-RecycleBin", "-Force", "-ErrorAction", "SilentlyContinue"])
                .spawn();
        }
        _ => {
            log::warn!("Unknown system action: {}", action);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        let result = try_calculate("2+2");
        assert!(result.is_some());
        assert!(result.unwrap().description.contains("4"));
        
        let result = try_calculate("sqrt(16)");
        assert!(result.is_some());
        
        let result = try_calculate("hello");
        assert!(result.is_none());
    }

    #[test]
    fn test_web_search() {
        let result = check_web_search("g rust programming");
        assert!(result.is_some());
        assert!(result.unwrap().path.to_string_lossy().contains("google.com"));
        
        let result = check_web_search("yt music");
        assert!(result.is_some());
    }
}
