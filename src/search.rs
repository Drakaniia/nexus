//! Search module
//! Handles fuzzy matching and search result ranking

#![allow(dead_code)]

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::AppEntry;

/// Search configuration
pub struct SearchConfig {
    pub max_results: usize,
    pub min_score: i64,
    pub mru_bonus: i64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 8,
            min_score: 10,
            mru_bonus: 10,
        }
    }
}

/// Perform fuzzy search across applications
pub fn fuzzy_search(
    apps: &[AppEntry],
    query: &str,
    mru: &std::collections::HashMap<String, u32>,
    config: &SearchConfig,
) -> Vec<(AppEntry, i64)> {
    let matcher = SkimMatcherV2::default().smart_case();
    
    let mut matches: Vec<_> = apps
        .iter()
        .filter_map(|app| {
            // Try matching against name and description
            let name_score = matcher.fuzzy_match(&app.name, query).unwrap_or(0);
            let desc_score = matcher.fuzzy_match(&app.description, query).unwrap_or(0) / 2;
            
            let base_score = name_score.max(desc_score);
            
            if base_score >= config.min_score {
                // Apply MRU bonus
                let mru_count = *mru.get(&app.name).unwrap_or(&0) as i64;
                let final_score = base_score + (mru_count * config.mru_bonus);
                
                Some((app.clone(), final_score))
            } else {
                None
            }
        })
        .collect();

    // Sort by score (descending)
    matches.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top results
    matches.truncate(config.max_results);

    matches
}

/// Check if a query matches the start of a name (for initial character priority)
pub fn starts_with_match(name: &str, query: &str) -> bool {
    let name_lower = name.to_lowercase();
    let query_lower = query.to_lowercase();
    
    // Check if name starts with query
    if name_lower.starts_with(&query_lower) {
        return true;
    }
    
    // Check if any word in name starts with query
    for word in name_lower.split_whitespace() {
        if word.starts_with(&query_lower) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_with_match() {
        assert!(starts_with_match("Visual Studio Code", "vis"));
        assert!(starts_with_match("Visual Studio Code", "code"));
        assert!(starts_with_match("Notepad", "note"));
        assert!(!starts_with_match("Notepad", "xyz"));
    }
}
