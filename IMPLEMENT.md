1. System Tray Problem

Current behavior: When you press Escape or Alt+Space again, the window hides BUT the application completely exits.

Why: Slint's event loop terminates when the window closes.

What needs to happen:

* App must create a system tray icon using the tray-icon crate (already in Cargo.toml)
* When window closes → hide window but keep running in background
* Tray icon shows menu: "Show/Hide", "Settings", "Exit"
* Only "Exit" menu item should terminate the app

Key code location: src/main.rs needs a new callback before slint::run_event_loop()

2. Single Instance Problem

Current behavior: You can run winlauncher.exe multiple times, creating conflicts.

What needs to happen:

* Use Windows mutex named "Global\\WinLauncher_Instance"
* First instance creates the mutex and runs normally
* Second instance tries to create same mutex → detects it exists → exits immediately
* Bonus: Second instance should signal first instance to show its window

Key code location: Very first thing in main(), before anything else

3. First-Letter Search Problem

Current behavior: Type "v" → no results or wrong results appear first

Why: Fuzzy matcher gives low scores to single-letter queries. "Visual Studio" might score lower than "Developer" if you recently used "Developer".

What needs to happen:

* Two-tier search system:
  1. Prefix matching (highest priority): "v" matches anything starting with "v" or containing a word starting with "v"
  2. Fuzzy matching (fallback): Only after prefix matches, do fuzzy matching for "vsal stdio" → "Visual Studio"

Example:

User types: "v"
Step 1 - Prefix matches (score 1000+):
  - Visual Studio Code
  - VLC Media Player
  - VS Code

Step 2 - Fuzzy matches (score 50-500):
  - Developer Tools (contains 'v' somewhere)
  
Result order: Visual Studio Code, VLC, VS Code, then Developer Tools

Key code location: src/main.rs in LauncherState::search() method

4. Configuration Persistence Problem

Current behavior: No config file exists. All settings are hardcoded.

What needs to happen:

* Create %APPDATA%\WinLauncher\config.json file
* Store settings like:
  * Hotkey combination (currently hardcoded Alt+Space)
  * Whether to run on startup
  * Theme preferences
  * MRU (Most Recently Used) data

File structure:

{
  "hotkey": {
    "modifiers": ["Alt"],
    "key": "Space"
  },
  "startup": {
    "enabled": true
  },
  "appearance": {
    "theme": "dark"
  }
}

Key code location: New file src/config.rs, loaded in main() before UI creation

5. Windows Startup Registration Problem

Current behavior: Must manually run winlauncher.exe after every reboot.

What needs to happen:

* Write to Windows registry: HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
* Add entry: "WinLauncher" = "C:\path\to\winlauncher.exe"
* User should see "WinLauncher" in Task Manager → Startup tab

Key code location: New file src/startup.rs with functions:

* enable_startup() → writes registry
* disable_startup() → deletes registry key
* is_startup_enabled() → checks if key exists

6. First-Run Experience Problem

Current behavior: App just starts with no guidance.

What needs to happen:

* On very first launch, show a welcome window
* Ask user:
  1. "Enable Alt+Space hotkey?" (or let them choose different key)
  2. "Run on Windows startup?" (checkbox)
  3. "Show brief tutorial?"
* Save choices to config file
* Then start normal app

Key code location: src/main.rs check if config file exists, if not → show wizard

7. Installer Creation

Current behavior: Only have winlauncher.exe - user must manually configure everything.

What needs to happen:

Option A - WiX Toolset (Microsoft's official):

* Creates .msi installer file
* Professional, shows in Windows "Add/Remove Programs"
* Can bundle multiple files, create shortcuts
* Harder to set up initially

Both should:

* Copy winlauncher.exe to %LOCALAPPDATA%\WinLauncher\
* Create Start Menu shortcut
* Optionally add to Windows startup
* Create uninstaller

8. Auto-Update System

Two strategies needed:

Strategy A - GitHub Releases Check:

* Every 24 hours, check https://api.github.com/repos/Drakaniia/nexus/releases/latest
* Compare version in response vs current version
* If newer: Show notification "Update available" in tray tooltip

Strategy B - Built-in Updater:

* Download new .exe from GitHub release
* Save as winlauncher_new.exe
* On next restart, replace old file with new one
* Can use Windows Task Scheduler for this

Key code location: New file src/updater.rs, called from tray menu "Check for Updates"

Implementation Roadmap

Create a system tray implementation for my Rust Slint app:
1. Use tray-icon crate to create tray icon with menu items: Show, Settings, Exit
2. Modify main.rs to handle window close → hide instead of exit
3. Only Exit menu item should call std::process::exit(0)
4. Slint window should use on_close_requested callback returning KeepWindowShown
5. Create src/tray.rs module with TrayManager struct

Implement Windows single instance enforcement:
1. Create src/single_instance.rs with InstanceGuard struct
2. Use Windows CreateMutexW API with "Global\\WinLauncher_Instance" name
3. Check GetLastError for ERROR_ALREADY_EXISTS (183)
4. In main.rs, call this FIRST before anything else
5. If already running, log warning and exit gracefully

Fix first-letter search in src/main.rs LauncherState::search():
1. Before fuzzy matching, do prefix matching
2. Prefix match: name.starts_with(query) OR any word starts with query
3. Give prefix matches score of 1000
4. Give fuzzy matches their normal score (10-500)
5. Sort all matches by score descending
6. Take top 6 results total

Create persistent configuration:
1. New file src/config.rs with AppConfig struct using serde
2. Fields: hotkey (modifiers + key), startup (bool), appearance
3. load_or_default() reads from %APPDATA%\WinLauncher\config.json
4. save() writes to that file
5. In main.rs, load config before creating UI

Implement Windows startup registration:
1. Create src/startup.rs using windows crate registry APIs
2. enable_startup() writes to HKCU\Software\Microsoft\Windows\CurrentVersion\Run
3. Key name: "WinLauncher", value: full path to exe
4. disable_startup() deletes that registry key
5. is_startup_enabled() checks if key exists

Add first-run experience:
1. In main.rs, check if config file exists
2. If not exists, show Slint dialog window before main launcher
3. Ask: hotkey preference, startup enabled checkbox
4. Save choices to config
5. Then proceed with normal app startup

Create WiX installer configuration:
1. Install to %LOCALAPPDATA%\WinLauncher
2. Create Start Menu shortcut via ComponentGroup
3. Registry entry for startup
4. Add to Programs and Features
5. Build with candle + light tools

Implement auto-update system:
1. Create src/updater.rs with check_for_updates() function
2. Call GitHub API: https://api.github.com/repos/Drakaniia/nexus/releases/latest
3. Parse JSON response, compare version tag with current version
4. If newer, show tray notification
5. Optional: download_update() function to fetch new .exe
6. Add "Check for Updates" to tray menu

🎯 Recommended Order for Implementation

1. System Tray (Prompt 1) - Makes app persistent ⭐ CRITICAL
2. Single Instance (Prompt 2) - Prevents conflicts ⭐ CRITICAL
3. Search Fix (Prompt 3) - Solves "v" problem ⭐ CRITICAL
4. Configuration (Prompt 4) - Saves settings
5. Startup (Prompt 5) - Auto-launch
6. First-Run (Prompt 6) - User onboarding
7. Installer (Prompt 7) - Distribution
8. Updater (Prompt 8) - Maintenance