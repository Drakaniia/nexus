//! Platform-specific window configuration for Windows
//! Applies Win32 window styles to Slint windows for launcher behavior

use slint::Window;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Configure window for launcher behavior:
/// - No taskbar button (WS_EX_TOOLWINDOW)
/// - Always on top (WS_EX_TOPMOST)
/// - No activation stealing (WS_EX_NOACTIVATE)
pub fn configure_launcher_window(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    // IMPORTANT: Window must be shown first for the window handle to be valid
    // Call this function AFTER showing the window

    log::debug!("Attempting to configure launcher window...");

    // Get raw window handle from Slint using HasWindowHandle trait
    let window_handle = window.window_handle();
    let raw_handle = window_handle.window_handle()?;

    // Extract HWND for Windows
    match raw_handle.as_raw() {
        RawWindowHandle::Win32(win32_handle) => {
            // Convert NonZero<isize> to *mut c_void for HWND
            let hwnd = HWND(win32_handle.hwnd.get() as *mut _);

            unsafe {
                log::debug!("Configuring window HWND: {:?}", hwnd);

                // Verify window handle is valid
                if hwnd.0.is_null() {
                    return Err("Window handle is null".into());
                }

                // Get current extended window style
                let current_ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                log::debug!("Current extended style: 0x{:08X}", current_ex_style);

                // Add required extended styles:
                // - WS_EX_TOOLWINDOW: No taskbar button (this is the critical one!)
                // - WS_EX_TOPMOST: Always on top of other windows
                // - WS_EX_NOACTIVATE: Don't steal focus when showing
                let new_ex_style = current_ex_style
                    | WS_EX_TOOLWINDOW.0
                    | WS_EX_TOPMOST.0
                    | WS_EX_NOACTIVATE.0;

                log::debug!("New extended style: 0x{:08X}", new_ex_style);

                // Apply new extended style
                let result = SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex_style as i32);
                if result == 0 {
                    log::warn!("SetWindowLongW failed, possibly invalid window handle");
                }

                // Force window to update with new styles
                // This is CRITICAL - without this, the styles don't take effect!
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,      // Place at top of Z-order
                    0, 0, 0, 0,        // Don't change position or size
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                )?;

                log::info!("✓ Window configured successfully: no taskbar button, always on top, no focus stealing");
            }

            Ok(())
        }
        _ => Err("Not a Windows window handle".into())
    }
}

