# Comprehensive Plan: Transform Portable EXE into Full Desktop Application with Installer

## 🎯 Current State vs Target State

### **Current State** (Portable Application)
```
❌ Single .exe file that runs once
❌ No installation process
❌ No automatic startup configuration
❌ User must manually configure everything
❌ No uninstaller
❌ No Start Menu integration
❌ Looks unprofessional
❌ Not discoverable in Windows
```

### **Target State** (Professional Desktop Application)
```
✅ MSI installer package
✅ Proper installation to Program Files
✅ Startup registration during install
✅ Start Menu shortcuts
✅ Add/Remove Programs entry
✅ Professional uninstaller
✅ First-run welcome wizard
✅ Persistent system tray service
✅ Auto-updates capability
```

---

## 📋 Complete Transformation Plan

### **Phase 1: Make EXE Truly Persistent** (CRITICAL - Do This First)

**Problem:** Even with system tray code, the app still exits after one use.

**Root Cause Analysis:**
```
Current flow:
1. User runs nexus.exe
2. Alt+Space works ONCE
3. When window closes, entire app exits
4. System tray icon disappears
5. Must manually run .exe again

Why it's failing:
- Slint's run_event_loop() might be exiting when window closes
- TrayManager being dropped prematurely
- Event loop not properly handling window hide vs app exit
- Missing window close prevention mechanism
```

**Solution Prompt:**
```
Fix the application persistence issue where the app exits after first use:

CURRENT BEHAVIOR:
- Run nexus.exe
- Press Alt+Space → window shows
- Close window → ENTIRE APP EXITS (wrong!)
- System tray icon disappears
- Must manually run exe again

REQUIRED BEHAVIOR:
- Run nexus.exe ONCE
- Press Alt+Space → window shows
- Close window → window hides, app STAYS RUNNING
- System tray icon STAYS visible
- Alt+Space works again immediately
- App runs until user clicks "Exit" in tray menu

DEBUGGING STEPS:
1. Add log::info!("Event loop started") right before slint::run_event_loop()
2. Add log::info!("Event loop ended - THIS SHOULD NEVER HAPPEN") right after it
3. Check if "Event loop ended" appears in logs when window closes
4. If yes, the event loop is terminating incorrectly

FIX REQUIRED:
1. Prevent Slint window from closing the event loop:
   - Use window.on_close_requested() callback
   - Return slint::CloseRequestResponse::KeepWindowShown
   - This tells Slint: "I handled the close, don't exit event loop"

2. Ensure TrayManager lifetime:
   - Move _tray declaration BEFORE launcher creation
   - Store _tray in a way that keeps it alive during entire event loop
   - Maybe wrap in Arc<Mutex<>> if needed

3. Add keep-alive mechanism if Slint still exits:
   - Create a background thread that just sleeps forever
   - This thread holds reference to app_running flag
   - Ensures at least one thread keeps app alive

4. Test manually:
   - Run exe from command line (not VS Code)
   - Check Task Manager → Details → nexus.exe should STAY RUNNING
   - Close window, check Task Manager again → still running
   - Press Alt+Space again → should work immediately

Show me the exact code changes needed in src/main.rs to fix this persistence issue.
```

---

### **Phase 2: Create Professional WiX Installer**

**Why WiX over Inno Setup:**
- Microsoft's official installer toolkit
- Creates proper .msi files
- Better Windows integration
- Shows in "Apps & Features" correctly
- Supports Windows Installer features (repair, modify)
- Better for enterprise deployment

**What the Installer Must Do:**

1. **Installation Location:**
   ```
   Default: C:\Program Files\Nexus\
   User-selectable during install
   Files to install:
   - nexus.exe
   - config_default.json (template)
   - README.txt
   - LICENSE.txt
   ```

2. **Registry Operations:**
   ```
   Add to Windows startup:
   HKCU\Software\Microsoft\Windows\CurrentVersion\Run
   "Nexus" = "C:\Program Files\Nexus\nexus.exe"

   Add uninstaller entry:
   HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\Nexus
   - DisplayName: "Nexus"
   - DisplayIcon: nexus.exe
   - UninstallString: msiexec /x {PRODUCT_GUID}
   ```

3. **Shortcuts Creation:**
   ```
   Start Menu:
   C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Nexus\
   - Nexus.lnk → nexus.exe
   - Uninstall Nexus.lnk → msiexec uninstall

   Desktop (optional):
   C:\Users\{User}\Desktop\Nexus.lnk
   ```

4. **First-Run Configuration:**
   ```
   After install completes:
   - Launch nexus.exe automatically
   - Show first-run welcome dialog
   - Configure hotkey (default Alt+Space)
   - Enable/disable startup (checkbox)
   - Start app in system tray
   ```

**Installer Prompt:**
```
Create a complete WiX Toolset installer for Nexus:

INSTALLER REQUIREMENTS:
1. Product Information:
   - Name: Nexus
   - Version: 1.0.0
   - Manufacturer: Qwenzy
   - Upgrade GUID: Generate stable GUID for upgrades

2. Installation Directory:
   - Default: C:\Program Files\Nexus
   - User can change during install
   - Create directory if doesn't exist

3. Files to Install:
   - nexus.exe (from target/release/)
   - README.md → README.txt
   - Include embedded icon/resources

4. Registry Entries:
   - Add to HKCU\Software\Microsoft\Windows\CurrentVersion\Run
   - Key: "Nexus"
   - Value: "[INSTALLDIR]nexus.exe"
   - Only if user checks "Run at startup" during install

5. Shortcuts:
   - Start Menu: Programs\Nexus\Nexus.lnk
   - Desktop: Optional, ask during install
   - Context menu should have: Open, Run at Startup, Uninstall

6. Post-Install Actions:
   - Launch application: checkbox in final screen
   - Show "Run on startup": checkbox (default checked)
   - Open README: checkbox (optional)

7. Uninstaller:
   - Remove all files from install directory
   - Remove registry entries
   - Remove shortcuts
   - Optionally keep config in %APPDATA% (ask user)

8. Upgrade Handling:
   - Detect existing installation
   - Preserve user config from %APPDATA%
   - Offer to upgrade in-place
   - Show "What's New" if upgrading

DELIVERABLES:
1. installer/nexus.wxs - Main WiX source file
2. installer/build.bat - Build script for creating MSI
3. installer/banner.bmp - Installer banner (493x58)
4. installer/dialog.bmp - Welcome dialog (493x312)
5. Documentation on how to build the installer

STRUCTURE:
nexus/
├── installer/
│   ├── nexus.wxs       # Main WiX config
│   ├── build.bat             # Build script
│   ├── banner.bmp            # Top banner
│   ├── dialog.bmp            # Welcome screen
│   └── README_INSTALLER.md   # Build instructions
├── src/
├── target/release/nexus.exe
└── Cargo.toml

Show me the complete WiX configuration and build process.
```

---

### **Phase 3: First-Run Wizard Implementation**

**When to Show Wizard:**
- First time app launches after installation
- Detected by: config file doesn't exist at `%APPDATA%\Nexus\config.json`
- Or: `config.first_run = true`

**Wizard Screens:**

**Screen 1: Welcome**
```
┌─────────────────────────────────────┐
│  Welcome to Nexus! 🚀         │
│                                     │
│  Press Alt+Space anywhere in        │
│  Windows to instantly search and    │
│  launch applications.               │
│                                     │
│  Let's get you set up...            │
│                                     │
│              [Next →]               │
└─────────────────────────────────────┘
```

**Screen 2: Hotkey Configuration**
```
┌─────────────────────────────────────┐
│  Choose Your Hotkey                 │
│                                     │
│  ⌨️ Current: [Alt + Space]          │
│                                     │
│  ☐ Alt + Space (Default)            │
│  ☐ Ctrl + Space                     │
│  ☐ Win + Space                      │
│  ☐ Custom: [___] + [___]            │
│                                     │
│  [← Back]              [Next →]     │
└─────────────────────────────────────┘
```

**Screen 3: Startup Options**
```
┌─────────────────────────────────────┐
│  Startup Settings                   │
│                                     │
│  ☑ Run Nexus on Windows       │
│     startup (Recommended)           │
│                                     │
│  ☐ Show launcher window on          │
│     startup (optional)              │
│                                     │
│  💡 Tip: Nexus runs quietly   │
│     in the system tray. Press your  │
│     hotkey anytime to launch!       │
│                                     │
│  [← Back]              [Next →]     │
└─────────────────────────────────────┘
```

**Screen 4: Complete**
```
┌─────────────────────────────────────┐
│  Setup Complete! ✅                  │
│                                     │
│  Nexus is now running in      │
│  your system tray.                  │
│                                     │
│  Quick tips:                        │
│  • Press Alt+Space to search        │
│  • Type app names to find them      │
│  • Use "g query" for Google         │
│  • Type "2+2" for calculator        │
│                                     │
│  [Finish]                           │
└─────────────────────────────────────┘
```

**Implementation Prompt:**
```
Create a first-run wizard for Nexus using Slint UI:

REQUIREMENTS:
1. Wizard appears ONLY on first run (check config.first_run flag)
2. Must be a separate window from the main launcher
3. Four screens: Welcome, Hotkey, Startup, Complete
4. User can go Back/Next between screens
5. Final screen saves config and starts main app

IMPLEMENTATION APPROACH:

Option A - Slint Wizard Window:
Create ui/wizard.slint with:
- Separate Window component
- Four screens as conditional elements
- Back/Next buttons
- Save config on "Finish"

Option B - Native Windows Dialog:
Use Windows API MessageBox or custom dialog
- Simpler but less pretty
- Faster to implement
- Native look and feel

RECOMMENDED: Option A (Slint) for consistency

FILE STRUCTURE:
src/
├── wizard.rs          # NEW FILE - Wizard logic
├── main.rs            # Check first_run, launch wizard
ui/
├── wizard.slint       # NEW FILE - Wizard UI
├── main.slint         # Existing launcher UI

WIZARD LOGIC FLOW:
1. In main(), after config load:
   if config.is_first_run() {
       wizard::show_wizard(&mut config)?;
       config.save();
   }

2. wizard::show_wizard() shows wizard window
3. User completes wizard
4. Config is updated with choices
5. config.first_run = false
6. Wizard closes
7. Main app starts normally

CONFIGURATION COLLECTED:
- Hotkey choice (Alt+Space, Ctrl+Space, etc.)
- Run on startup (bool)
- Show on startup (bool)

Show me the complete implementation for:
1. ui/wizard.slint (all 4 screens)
2. src/wizard.rs (wizard logic)
3. Changes needed in src/main.rs to integrate wizard
```

---

### **Phase 4: Auto-Update System**

**Update Architecture:**

```
┌─────────────────────────────────────────────────┐
│  Nexus App                                │
│  ┌─────────────────────────────────────────┐   │
│  │ Background Update Checker Thread        │   │
│  │ - Runs every 24 hours                    │   │
│  │ - Checks GitHub Releases API             │   │
│  │ - Compares versions                      │   │
│  └─────────────────────────────────────────┘   │
│                      ↓                           │
│  ┌─────────────────────────────────────────┐   │
│  │ If update available:                     │   │
│  │ - Show tray notification                 │   │
│  │ - Add "Update Available" to tray menu    │   │
│  └─────────────────────────────────────────┘   │
│                      ↓                           │
│  ┌─────────────────────────────────────────┐   │
│  │ User clicks "Update Now":                │   │
│  │ - Download new installer (.msi)          │   │
│  │ - Save to %TEMP%                         │   │
│  │ - Launch: msiexec /i installer.msi      │   │
│  │ - Close current app                      │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

**Update Check Implementation Prompt:**
```
Implement auto-update system for Nexus:

REQUIREMENTS:
1. Check for updates every 24 hours in background
2. Use GitHub Releases API: https://api.github.com/repos/Drakaniia/nexus/releases/latest
3. Compare current version (from Cargo.toml) with latest release tag
4. Show tray notification if update available
5. Download and launch installer on user request

IMPLEMENTATION:

File: src/updater.rs (NEW)

Components needed:
1. UpdateChecker struct:
   - current_version: String (from Cargo.toml)
   - last_check: SystemTime
   - latest_version: Option<String>
   - download_url: Option<String>

2. Functions:
   - check_for_updates() -> Result<UpdateInfo>
   - download_update(url) -> Result<PathBuf>
   - install_update(installer_path)

API CALL:
GET https://api.github.com/repos/Drakaniia/nexus/releases/latest
Response JSON:
{
  "tag_name": "v1.0.1",
  "assets": [
    {
      "name": "Nexus-Setup.msi",
      "browser_download_url": "https://github.com/.../Nexus-Setup.msi"
    }
  ]
}

VERSION COMPARISON:
Current: "0.1.0" (from Cargo.toml)
Latest: "v1.0.1" (from GitHub tag_name)
Parse both, compare: 1.0.1 > 0.1.0 → update available

DOWNLOAD PROCESS:
1. Use reqwest crate (add to Cargo.toml)
2. Download .msi to %TEMP%\Nexus-Update.msi
3. Show progress in tray tooltip if possible
4. Verify download completed (check file size)

INSTALLATION PROCESS:
1. Launch: msiexec /i "%TEMP%\Nexus-Update.msi" /qb
   - /qb = basic UI, progress bar only
2. Current app must exit before installer runs
3. Installer will:
   - Close running instance if needed
   - Upgrade files
   - Restart app automatically

INTEGRATION:
1. Add to tray menu: "Check for Updates"
2. Show notification: "Update available: v1.0.1"
3. Notification clicks → start download
4. Add to config: last_update_check timestamp

DEPENDENCIES TO ADD:
[dependencies]
reqwest = { version = "0.12", features = ["blocking", "json"] }
semver = "1.0"  # For version comparison

Show me complete implementation of src/updater.rs with all functions.
```

---

### **Phase 5: Testing & Quality Assurance**

**Testing Checklist:**

**Installer Testing:**
```
Test 1: Fresh Install
[ ] Run installer on clean Windows machine
[ ] Verify files copied to C:\Program Files\Nexus\
[ ] Check Start Menu shortcut exists
[ ] Check Desktop shortcut (if selected)
[ ] Verify "Add/Remove Programs" entry
[ ] Check startup registry entry created
[ ] Launch app after install completes
[ ] Verify first-run wizard appears

Test 2: Upgrade Install
[ ] Install version 1.0.0
[ ] Run installer for version 1.1.0
[ ] Verify upgrade detected
[ ] Check old files replaced
[ ] Verify config preserved in %APPDATA%
[ ] Check app runs with existing settings

Test 3: Uninstall
[ ] Run uninstaller from "Add/Remove Programs"
[ ] Verify all files deleted from Program Files
[ ] Check registry entries removed
[ ] Verify shortcuts deleted
[ ] Check if config kept in %APPDATA% (optional)
[ ] Verify system tray icon removed
```

**Application Testing:**
```
Test 1: Persistence
[ ] Run nexus.exe
[ ] Press Alt+Space → window shows
[ ] Press Escape → window hides
[ ] Check Task Manager: nexus.exe still running ✓
[ ] Check system tray: icon visible ✓
[ ] Press Alt+Space again → window shows immediately ✓
[ ] Repeat 10 times → no crashes

Test 2: Search Functionality
[ ] Type "v" → see Visual Studio, VLC, etc.
[ ] Type "note" → see Notepad
[ ] Type "calc" → see Calculator
[ ] Type "2+2" → see "= 4"
[ ] Type "g rust" → see Google search option
[ ] Press Enter on each → verify launch

Test 3: Startup Integration
[ ] Restart computer
[ ] Check Task Manager → nexus.exe running
[ ] Check system tray → icon visible
[ ] Press Alt+Space → works immediately
[ ] No error messages in Event Viewer

Test 4: Updates
[ ] Manually set old version in code
[ ] Click "Check for Updates" in tray
[ ] Verify update notification appears
[ ] Click "Update Now"
[ ] Verify download starts
[ ] Verify installer launches
[ ] Verify app updates successfully
```

---

### **Phase 6: Build & Distribution Process**

**Complete Build Pipeline:**

**Step 1: Build Release Binary**
```batch
@echo off
echo ================================================
echo Building Nexus Release Binary
echo ================================================

echo Cleaning previous builds...
cargo clean

echo Building optimized release...
cargo build --release

echo Checking binary...
if exist target\release\nexus.exe (
    echo ✓ Binary created successfully
    dir target\release\nexus.exe
) else (
    echo ✗ Build failed!
    exit /b 1
)

echo.
echo Build complete!
```

**Step 2: Build Installer**
```batch
@echo off
echo ================================================
echo Building Nexus Installer
echo ================================================

echo Checking WiX Toolset...
where candle >nul 2>&1
if %errorlevel% neq 0 (
    echo ✗ WiX Toolset not found!
    echo Install from: https://wixtoolset.org/
    exit /b 1
)

echo Compiling WiX source...
cd installer
candle nexus.wxs -dSourceDir=..\target\release

echo Linking installer...
light nexus.wixobj -out ..\Nexus-Setup.msi

echo Cleaning up...
del nexus.wixobj

cd ..
echo.
echo ✓ Installer created: Nexus-Setup.msi
dir Nexus-Setup.msi
```

**Step 3: Create GitHub Release**
```batch
@echo off
setlocal enabledelayedexpansion

echo ================================================
echo Creating GitHub Release
echo ================================================

:: Get version from Cargo.toml
for /f "tokens=3 delims= " %%a in ('findstr /C:"version = " Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
)

echo Version: v!VERSION!

:: Create release notes
echo Creating RELEASE_NOTES.md...
(
echo # Nexus v!VERSION!
echo.
echo ## Changes
echo - Bug fixes and improvements
echo - See CHANGELOG.md for details
echo.
echo ## Installation
echo 1. Download Nexus-Setup.msi
echo 2. Run the installer
echo 3. Press Alt+Space to launch!
) > RELEASE_NOTES.md

echo.
echo Next steps:
echo 1. Create tag: git tag v!VERSION!
echo 2. Push tag: git push origin v!VERSION!
echo 3. Go to: https://github.com/Drakaniia/nexus/releases/new
echo 4. Select tag: v!VERSION!
echo 5. Upload: Nexus-Setup.msi
echo 6. Copy contents of RELEASE_NOTES.md
echo 7. Publish release
```

---

### **Phase 7: Documentation Updates**

**README.md Updates:**
```markdown
Add new sections:

## Installation

### Option 1: Installer (Recommended)
1. Download `Nexus-Setup.msi` from [Releases](https://github.com/Drakaniia/nexus/releases/latest)
2. Run the installer
3. Follow the setup wizard
4. Press Alt+Space to launch!

### Option 2: Portable (Advanced Users)
1. Download `nexus.exe` from [Releases](https://github.com/Drakaniia/nexus/releases/latest)
2. Run the executable
3. Configure manually

## First Run

After installation, the setup wizard will guide you through:
- Hotkey configuration (default: Alt+Space)
- Startup options (run on Windows boot)
- Quick tutorial

## Updating

Nexus checks for updates automatically every 24 hours.
You can also manually check:
1. Right-click system tray icon
2. Click "Check for Updates"
3. Follow prompts to install update

## Uninstalling

### Windows 10/11:
1. Open Settings → Apps → Installed apps
2. Find "Nexus"
3. Click [...] → Uninstall

### Classic:
1. Control Panel → Programs and Features
2. Select "Nexus"
3. Click Uninstall

Your settings in %APPDATA%\Nexus will be preserved.
```

---

## 🎯 **FINAL IMPLEMENTATION SUMMARY**

### **What You Need to Build:**

**Priority 1 - Critical (Do First):**
1. ✅ Fix persistence issue (app must stay running in background)
2. ✅ Test: run exe → alt+space 10 times → still works

**Priority 2 - Installer:**
3. ✅ Create WiX installer project
4. ✅ Build MSI package
5. ✅ Test: install → use → uninstall

**Priority 3 - Polish:**
6. ✅ First-run wizard
7. ✅ Auto-update system
8. ✅ Documentation

**Priority 4 - Distribution:**
9. ✅ GitHub release with MSI
10. ✅ Update README with install instructions

---
