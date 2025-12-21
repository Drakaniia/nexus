# Comprehensive Bug Fix Plan for Windows Launcher

## 🚨 Critical Issues Identified

### **Issue 1: Search Results Not Displaying (Type "v" → No Results)**
**Root Cause Analysis:**
- UI update thread might not be running correctly
- Results are being calculated but not rendered to Slint UI
- The 50ms polling delay in the UI update thread could be causing lag
- Race condition between search calculation and UI rendering

**Fix Strategy:**
```
Problem: Results exist in memory but Slint UI doesn't show them
Solution Approach:
1. Add debug logging in search() method to confirm results are generated
2. Add debug logging in UI update thread to confirm it's receiving results
3. Verify VecModel is being created and set correctly
4. Check if launcher_weak.upgrade_in_event_loop() is succeeding
5. Consider moving UI update to immediate callback instead of polling thread
6. Verify results aren't being cleared immediately after being set
```

**Detailed Implementation Prompt:**
```
Debug and fix the search results display issue:
1. In src/main.rs LauncherState::search(), add log::info! statements showing:
   - Query received: log::info!("Searching for: {}", query)
   - Prefix matches found: log::info!("Prefix matches: {:?}", prefix_matches)
   - Final results count: log::info!("Returning {} results", results.len())

2. In on_search_changed callback, add logging:
   - log::info!("Search changed: {}", query_str)
   - log::info!("Results generated: {}", search_results.len())

3. In the UI update thread, add logging:
   - log::info!("UI thread: {} results in current_results", results.len())
   - Log if upgrade_in_event_loop succeeds or fails

4. Change UI update strategy from polling to immediate:
   - Remove the 50ms sleep loop thread
   - Instead, call launcher_weak.upgrade_in_event_loop() directly in on_search_changed callback
   - Set results immediately after search completes

5. Verify Slint bindings:
   - Check if results property is properly bound in ui/main.slint
   - Ensure ListView or result rendering is triggered on results change
   - Add trigger mechanism if needed
```

---

### **Issue 2: Auto-Focus Not Working (Must Click to Type)**
**Root Cause Analysis:**
- `invoke_focus_input()` is being called but focus isn't actually set
- Window might not have focus when shown
- Slint focus mechanism might need explicit window activation
- Windows API SetForegroundWindow might be needed

**Fix Strategy:**
```
Problem: Alt+Space shows window but TextInput doesn't have focus
Solution Approach:
1. Show window FIRST, then focus input AFTER window is visible
2. Add Windows API call to bring window to foreground
3. Use SetForegroundWindow + SetFocus combination
4. Add delay between show() and focus() calls if needed
5. Verify TextInput has proper focus-policy in Slint
```

**Detailed Implementation Prompt:**
```
Fix the auto-focus issue when Alt+Space is pressed:
1. In hotkey event handler, change the sequence:
   BEFORE:
   - launcher.invoke_clear_search()
   - launcher.show()
   - launcher.set_is_visible(true)
   - launcher.invoke_focus_input()

   AFTER:
   - launcher.show()
   - launcher.set_is_visible(true)
   - Use Windows API to bring window to foreground
   - Add 10ms delay using std::thread::sleep
   - launcher.invoke_clear_search()
   - launcher.invoke_focus_input()

2. Add Windows API call in src/main.rs:
   - Import: use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, GetForegroundWindow}
   - Get Slint window's HWND handle
   - Call SetForegroundWindow(hwnd) after show()
   - This ensures Windows gives our window keyboard focus

3. In ui/main.slint, verify TextInput has focus properties:
   - Check if search-input has forward-focus: auto or similar
   - Ensure no other element is stealing focus
   - Consider using has-focus property binding

4. Alternative approach - use Slint's window.request_focus():
   - After show(), call window.request_focus() on Slint window object
   - This is Slint's internal way to request system focus
```

---

### **Issue 3: Window Not Centered on Screen**
**Root Cause Analysis:**
- Slint window has no position specified
- Window spawns at default OS location (usually top-left)
- Need to calculate screen center and set window position

**Fix Strategy:**
```
Problem: Window appears at random/default position instead of screen center
Solution Approach:
1. Get screen dimensions using Windows API
2. Calculate center position: (screen_width/2 - window_width/2, screen_height/2 - window_height/2)
3. Set window position before showing
4. Consider multi-monitor setup - center on primary monitor or monitor with cursor
```

**Detailed Implementation Prompt:**
```
Center the launcher window on screen when Alt+Space is pressed:
1. Create helper function get_screen_center() in src/main.rs:
   - Use Windows API: GetSystemMetrics(SM_CXSCREEN) for width
   - Use GetSystemMetrics(SM_CYSCREEN) for height
   - OR use GetMonitorInfo to get primary monitor dimensions
   - Calculate center point: (screen_width/2 - 340, screen_height/2 - 100)
   - Window is 680px wide, so subtract half = 340px
   - Return (x, y) coordinates as (i32, i32)

2. In hotkey handler, BEFORE launcher.show():
   - Call get_screen_center() to get position
   - Use Slint's window.set_position() method if available
   - OR use Windows API SetWindowPos(hwnd, x, y) with the HWND from Slint

3. Handle multi-monitor scenarios:
   - Use GetCursorPos() to get mouse position
   - Use MonitorFromPoint() to find which monitor cursor is on
   - Center on that monitor instead of primary monitor
   - This gives better UX if user has multiple screens

4. Update on every show:
   - Don't set position only once at startup
   - Recalculate and set position every time Alt+Space is pressed
   - This handles if user moves between monitors
```

---

### **Issue 4: App Runs Only Once (Not Persistent/Portable)**
**Root Cause Analysis:**
- System tray is created but app still exits after window closes
- Event loop might be terminating prematurely
- App needs to stay running in background even when window is hidden
- Startup registration not working correctly

**Fix Strategy:**
```
Problem: App exits instead of staying in system tray
Solution Approach:
1. Verify slint::run_event_loop() doesn't exit when window hides
2. Check if TrayManager is being dropped prematurely
3. Ensure app_running flag is properly controlling all threads
4. Verify startup registry entry is actually created
5. Test manually running exe to confirm it stays in tray
```

**Detailed Implementation Prompt:**
```
Make the application truly persistent in the background:
1. Verify TrayManager lifetime:
   - In main(), ensure _tray variable is NOT dropped
   - Move _tray declaration BEFORE launcher creation
   - Keep _tray in scope until very end of main()
   - Add log::info!("Tray still alive") periodically to confirm

2. Check Slint event loop behavior:
   - Slint's run_event_loop() should NOT exit when window hides
   - Verify no code is calling std::process::exit() unexpectedly
   - Check if slint::quit_event_loop() is being called anywhere
   - Ensure window close doesn't terminate event loop

3. Fix startup registration verification:
   - After calling enable_startup(), immediately call is_startup_enabled()
   - Log the result: log::info!("Startup enabled: {}", is_startup_enabled())
   - If false, there's a registry writing issue
   - Add error handling to see exact Windows error code

4. Test persistence manually:
   - Run the exe directly (not from VS Code terminal)
   - Check Task Manager → Details tab for winlauncher.exe
   - It should stay running even when window is hidden
   - System tray icon should remain visible
   - If exe disappears from Task Manager, app is exiting prematurely

5. Add keep-alive mechanism:
   - Create a dedicated thread that does nothing but sleep
   - This thread holds a reference to app_running flag
   - While app_running is true, sleep for 1 second in loop
   - This ensures at least one thread keeps app alive
```

---

### **Issue 5: Cannot Verify Which App Is Opening**
**Root Cause Analysis:**
- Search results not displaying means no visual feedback
- Related to Issue 1, but also need better logging
- User types "vscode", presses Enter, but can't confirm it's the right match

**Fix Strategy:**
```
Problem: Blind execution - can't see what will be launched
Solution Approach:
1. Fix Issue 1 first (results not displaying)
2. Add visual confirmation before launching
3. Add logging to show which app was selected
4. Consider adding a brief toast/notification after launch
```

**Detailed Implementation Prompt:**
```
Add better visibility and confirmation for app launches:
1. Fix the results display (see Issue 1) so user can see options

2. In on_result_activated callback, add detailed logging:
   - log::info!("User selected index: {}", index)
   - log::info!("Launching: {} ({})", result.name, result.path.display())
   - log::info!("Result type: {}", result.result_type)

3. Add pre-launch validation:
   - Before open::that(&result.path), check if path exists
   - Log error if path doesn't exist: log::error!("Path not found: {}", path)
   - Show error message in UI if launch fails

4. Add visual feedback after launch:
   - After successful launch, could show brief notification
   - Or briefly change window background color to green
   - Or add a status label at bottom: "Launched: Visual Studio Code"
   - This confirms the action completed

5. During development, keep console visible:
   - Remove #![windows_subsystem = "windows"] temporarily
   - This shows console with all log messages
   - Helps debug what's happening during search and launch
   - Add back once everything works
```

---

## 📋 Implementation Order (Priority-Based)

### **Phase 1: Critical Fixes (Do First)**
1. **Auto-Focus Fix** (Issue 2) - Most urgent, makes app unusable
2. **Window Centering** (Issue 3) - Quick fix, improves UX significantly
3. **Results Display** (Issue 1) - Core functionality, needed to see what's happening

### **Phase 2: Persistence Fixes**
4. **App Staying Alive** (Issue 4) - Makes it truly background app
5. **Launch Confirmation** (Issue 5) - Better user feedback

---

## 🎯 Prompts to Use with AI (In Order)

### **Prompt 1: Fix Auto-Focus**
```
I have a Rust Slint application where Alt+Space shows a search window, but the TextInput doesn't automatically receive focus. Users must click the input field to start typing. 

The current code calls launcher.invoke_focus_input() after show(), but it's not working.

I need to:
1. Use Windows API SetForegroundWindow to bring the window to front
2. Add a small delay (10-20ms) between showing window and setting focus
3. Get the HWND handle from the Slint window
4. Ensure the window actually receives keyboard focus from the OS

Show me how to modify the hotkey event handler in src/main.rs to properly focus the TextInput when Alt+Space is pressed. Include the necessary Windows API imports and HWND retrieval from Slint.
```

### **Prompt 2: Center Window on Screen**
```
My Slint window appears at a default position (usually top-left) instead of centered on the screen. The window is 680px wide and approximately 70-400px tall depending on content.

I need to:
1. Get the primary monitor's screen dimensions using Windows API
2. Calculate center position: (screen_width/2 - 340, screen_height/3)
3. Set the window position BEFORE calling show()
4. Handle multi-monitor setups by centering on the monitor where the cursor is located

Show me how to:
- Create a get_screen_center() function using Windows API (GetSystemMetrics or GetMonitorInfo)
- Get the HWND from Slint window
- Use SetWindowPos to position the window before showing
- Call this every time Alt+Space is pressed (in the hotkey handler)

Include all necessary Windows API imports and error handling.
```

### **Prompt 3: Fix Results Display**
```
My search function generates results correctly (I can verify this with logging), but they don't appear in the Slint UI. The results are stored in Arc<Mutex<Vec<SearchResultData>>> and should be displayed in the ListView.

Current implementation:
- on_search_changed callback generates results and stores in current_results
- A separate thread polls current_results every 50ms
- Thread calls launcher_weak.upgrade_in_event_loop() to update UI
- Creates VecModel and calls launcher.set_results()

The problem is the UI never shows the results, even though:
- Logging confirms search() returns results
- current_results mutex contains the data
- No errors are thrown

I need to:
1. Remove the polling thread approach
2. Update UI immediately in the on_search_changed callback
3. Call set_results directly from the callback using upgrade_in_event_loop
4. Add proper error handling and logging
5. Ensure the Slint ListView actually triggers re-render when results change

Show me how to refactor the on_search_changed callback and remove the polling thread to make results display immediately. Include logging at each step to debug if still not working.
```

### **Prompt 4: Fix App Persistence**
```
My Windows launcher creates a system tray icon but the app still exits when the window is closed or hidden. The app should stay running in the background permanently until user clicks "Exit" in tray menu.

Current code:
- TrayManager is created in main()
- slint::run_event_loop() is called
- Event loop might be terminating when window closes

I need to:
1. Ensure TrayManager lifetime extends throughout entire program
2. Prevent slint event loop from exiting when window hides
3. Verify no unexpected std::process::exit() calls
4. Keep app running in background with system tray visible
5. Verify startup registry entry is created correctly

Show me:
- How to ensure TrayManager stays alive
- How to prevent Slint event loop from exiting on window close
- How to add a keep-alive thread if needed
- How to debug why app might be exiting (logging, checks)
- How to verify startup registry entry was successfully created
```

### **Prompt 5: Add Launch Confirmation**
```
When user presses Enter to launch an app, there's no visual feedback about which app is being launched or if launch succeeded.

I need to:
1. Add detailed logging in on_result_activated callback showing:
   - Selected index
   - App name and path
   - Result type
2. Validate that the file path exists before launching
3. Show error if launch fails
4. Optionally add a brief visual confirmation (green flash or status text)

Show me how to enhance the on_result_activated callback with proper logging, validation, and user feedback. Include error handling for cases where path doesn't exist or launch fails.
```

---

## 🔍 Debugging Checklist

Before implementing fixes, verify these:

```
[ ] Run exe with console visible (remove #![windows_subsystem = "windows"])
[ ] Check if search() is being called (add log at start of function)
[ ] Check if results are generated (log results.len() at end)
[ ] Check if on_search_changed callback fires (add log)
[ ] Check if UI update thread is running (add log in loop)
[ ] Check if upgrade_in_event_loop succeeds (add Result handling)
[ ] Check if set_results is called (add log)
[ ] Check Task Manager → Details for winlauncher.exe process
[ ] Check Task Manager → Startup for WinLauncher entry
[ ] Check system tray for icon (bottom-right near clock)
[ ] Test with simple query like "notepad" (known to exist)
```

---

## 💡 Quick Wins (Test These First)

1. **Remove windows_subsystem attribute** → See console logs immediately
2. **Add log::info! everywhere** → Understand execution flow
3. **Test with hardcoded results** → Verify UI rendering works
4. **Run exe directly** → Not from IDE to test real behavior