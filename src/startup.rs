//! Windows Startup Registration Module
//! Manages adding/removing the app from Windows startup via registry

#![allow(dead_code)]

use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyW, RegQueryValueExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, REG_SZ,
};

/// Registry key path for startup programs
const STARTUP_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// App name in registry
const APP_NAME: &str = "Nexus";

/// Enable startup - add to Windows startup programs
pub fn enable_startup() -> Result<(), StartupError> {
    let exe_path = env::current_exe().map_err(|_| StartupError::ExePathNotFound)?;
    let exe_path_str = exe_path.to_string_lossy();

    // Add quotes around path in case of spaces
    let value = format!("\"{}\"", exe_path_str);

    unsafe {
        // Open the Run key
        let mut hkey: HKEY = HKEY::default();
        let key_path = to_wide_string(STARTUP_KEY);

        let result = RegOpenKeyW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path.as_ptr()),
            &mut hkey,
        );

        if result.is_err() {
            return Err(StartupError::RegistryAccessDenied);
        }

        // Set the value
        let app_name = to_wide_string(APP_NAME);
        let value_wide = to_wide_string(&value);
        let value_bytes: Vec<u8> = value_wide
            .iter()
            .flat_map(|&w| w.to_le_bytes())
            .collect();

        let result = RegSetValueExW(
            hkey,
            PCWSTR::from_raw(app_name.as_ptr()),
            0,
            REG_SZ,
            Some(&value_bytes),
        );

        let _ = RegCloseKey(hkey).ok();

        if result.is_err() {
            return Err(StartupError::RegistryWriteFailed);
        }
    }

    log::info!("Startup registration enabled");
    Ok(())
}

/// Disable startup - remove from Windows startup programs
pub fn disable_startup() -> Result<(), StartupError> {
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        let key_path = to_wide_string(STARTUP_KEY);

        let result = RegOpenKeyW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path.as_ptr()),
            &mut hkey,
        );

        if result.is_err() {
            return Err(StartupError::RegistryAccessDenied);
        }

        let app_name = to_wide_string(APP_NAME);
        let result = RegDeleteValueW(hkey, PCWSTR::from_raw(app_name.as_ptr()));

        let _ = RegCloseKey(hkey).ok();

        if result.is_err() {
            // Value might not exist, which is fine
            log::debug!("Startup entry not found or already removed");
        }
    }

    log::info!("Startup registration disabled");
    Ok(())
}

/// Check if startup is currently enabled
pub fn is_startup_enabled() -> bool {
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        let key_path = to_wide_string(STARTUP_KEY);

        let result = RegOpenKeyW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path.as_ptr()),
            &mut hkey,
        );

        if result.is_err() {
            return false;
        }

        let app_name = to_wide_string(APP_NAME);
        let mut data_type: u32 = 0;
        let mut data_size: u32 = 0;

        let result = RegQueryValueExW(
            hkey,
            PCWSTR::from_raw(app_name.as_ptr()),
            None,
            Some(&mut data_type as *mut u32 as *mut _),
            None,
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey).ok();

        result.is_ok() && data_size > 0
    }
}

/// Toggle startup registration
pub fn toggle_startup() -> Result<bool, StartupError> {
    if is_startup_enabled() {
        disable_startup()?;
        Ok(false)
    } else {
        enable_startup()?;
        Ok(true)
    }
}

/// Convert a string to a null-terminated wide string (UTF-16)
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Errors that can occur during startup registration
#[derive(Debug)]
pub enum StartupError {
    ExePathNotFound,
    RegistryAccessDenied,
    RegistryWriteFailed,
}

impl std::fmt::Display for StartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExePathNotFound => write!(f, "Could not determine executable path"),
            Self::RegistryAccessDenied => write!(f, "Registry access denied"),
            Self::RegistryWriteFailed => write!(f, "Failed to write to registry"),
        }
    }
}

impl std::error::Error for StartupError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wide_string() {
        let wide = to_wide_string("Test");
        assert_eq!(wide.len(), 5); // 4 chars + null terminator
        assert_eq!(wide[4], 0); // Null terminator
    }

    #[test]
    fn test_startup_check() {
        // Just test that the function runs without crashing
        let _ = is_startup_enabled();
    }
}
