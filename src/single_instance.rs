//! Single Instance Enforcement Module
//! Prevents multiple instances of the launcher from running simultaneously

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::{CreateMutexW, OpenMutexW, MUTEX_ALL_ACCESS};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextA, PostMessageA, SetForegroundWindow,
    ShowWindow, SW_RESTORE, WM_USER,
};

/// Custom message to signal the existing instance to show
pub const WM_SHOW_LAUNCHER: u32 = WM_USER + 100;

/// Mutex name for single instance check
const MUTEX_NAME: &str = "Global\\WinLauncher_SingleInstance_Mutex";

/// Manages single instance enforcement
pub struct SingleInstance {
    mutex_handle: Option<HANDLE>,
}

impl SingleInstance {
    /// Try to acquire the single instance lock
    /// Returns Ok(Self) if this is the first instance
    /// Returns Err if another instance is already running
    pub fn acquire() -> Result<Self, SingleInstanceError> {
        unsafe {
            // Convert the mutex name to a wide string
            let wide_name: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let mutex_name = PCWSTR::from_raw(wide_name.as_ptr());

            match CreateMutexW(None, true, mutex_name) {
                Ok(handle) => {
                    // Check if mutex already existed
                    let last_error = windows::Win32::Foundation::GetLastError();

                    if last_error.0 == 183 {
                        // ERROR_ALREADY_EXISTS - another instance is running
                        CloseHandle(handle).ok();

                        // Try to signal the existing instance
                        Self::signal_existing_instance();

                        return Err(SingleInstanceError::AlreadyRunning);
                    }

                    log::info!("Single instance lock acquired");
                    Ok(Self {
                        mutex_handle: Some(handle),
                    })
                }
                Err(e) => {
                    log::error!("Failed to create mutex: {:?}", e);
                    Err(SingleInstanceError::MutexCreationFailed)
                }
            }
        }
    }

    /// Signal the existing instance to show its window
    fn signal_existing_instance() {
        log::info!("Another instance detected, signaling to show...");
        
        unsafe {
            // Find the existing launcher window and bring it to foreground
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(0));
        }
    }

    /// Check if another instance is running without acquiring lock
    #[allow(dead_code)]
    pub fn is_another_running() -> bool {
        unsafe {
            let wide_name: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let mutex_name = PCWSTR::from_raw(wide_name.as_ptr());

            match OpenMutexW(MUTEX_ALL_ACCESS, false, mutex_name) {
                Ok(handle) => {
                    CloseHandle(handle).ok();
                    true
                }
                Err(_) => false,
            }
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        if let Some(handle) = self.mutex_handle.take() {
            unsafe {
                CloseHandle(handle).ok();
            }
            log::info!("Single instance lock released");
        }
    }
}

/// Callback for EnumWindows to find and signal the existing instance
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, _lparam: LPARAM) -> windows::Win32::Foundation::BOOL {
    let mut title = [0u8; 256];
    let len = GetWindowTextA(hwnd, &mut title);
    
    if len > 0 {
        let title_str = String::from_utf8_lossy(&title[..len as usize]);
        
        // Check if this is our launcher window
        if title_str.contains("WinLauncher") || title_str.contains("Launcher") {
            log::info!("Found existing launcher window, bringing to foreground");
            
            // Restore and bring to foreground
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
            
            // Post custom message to show the launcher
            PostMessageA(hwnd, WM_SHOW_LAUNCHER, WPARAM(0), LPARAM(0)).ok();
            
            return false.into(); // Stop enumeration
        }
    }
    
    true.into() // Continue enumeration
}

/// Errors that can occur during single instance checking
#[derive(Debug)]
pub enum SingleInstanceError {
    AlreadyRunning,
    MutexCreationFailed,
}

impl std::fmt::Display for SingleInstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "Another instance is already running"),
            Self::MutexCreationFailed => write!(f, "Failed to create mutex"),
        }
    }
}

impl std::error::Error for SingleInstanceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SingleInstanceError::AlreadyRunning;
        assert!(err.to_string().contains("already running"));
    }
}
