# Nexus Testing Checklist

## Phase 5: Testing & Quality Assurance

This document provides a comprehensive testing checklist for Nexus based on the plan.md requirements.

---

## 🎯 Test 1: Persistence Testing

**Objective:** Verify the app stays running in background after window close (Phase 1 fix).

### Prerequisites
- Build release version: `cargo build --release`
- Have at least 3 installed applications for testing

### Test Steps
```
[ ] Run nexus.exe from command line (not VS Code)
[ ] Verify Task Manager shows nexus.exe process
[ ] Press Alt+Space → window appears
[ ] Type "notepad" → select Notepad
[ ] Press Enter → Notepad launches, Nexus window hides
[ ] Check Task Manager → nexus.exe STILL RUNNING ✓
[ ] Press Alt+Space again → window appears immediately ✓
[ ] Repeat 10 times → no crashes or memory leaks
[ ] Right-click tray icon → click "Exit" → app closes completely
```

### Expected Results
- App never exits when window closes
- Tray icon always visible
- Hotkey works immediately after window hide
- Memory usage stable after multiple cycles

---

## 🎯 Test 2: Search Functionality Testing

**Objective:** Verify search works correctly for all content types.

### Test Cases
```
[ ] App Search:
   - Type "notepad" → Notepad appears ✓
   - Type "calc" → Calculator appears ✓
   - Type "chrome" → Chrome appears ✓
   - Type "vsc" → Visual Studio Code appears (initials match) ✓

[ ] Calculator:
   - Type "2+2" → shows "= 4" ✓
   - Type "10*5" → shows "= 50" ✓
   - Type "sqrt(16)" → shows "= 4" ✓

[ ] Web Search:
   - Type "g rust" → shows "Search Google for rust" ✓
   - Type "g weather" → shows "Search Google for weather" ✓

[ ] Special Actions:
   - Type "settings" → shows system settings option ✓
   - Type "control" → shows Control Panel option ✓

[ ] Fuzzy Matching:
   - Type "v" → shows VLC, Visual Studio, etc. ✓
   - Type "note" → shows Notepad, OneNote ✓
   - Type "mic" → shows Microsoft apps ✓
```

---

## 🎯 Test 3: Startup Integration Testing

**Objective:** Verify Windows startup registration works correctly.

### Test Steps
```
[ ] Fresh Installation:
   - Run installer MSI
   - Check "Run on startup" in wizard
   - Complete installation
   - Verify registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Run
   - Restart computer
   - Verify nexus.exe running in Task Manager ✓
   - Verify tray icon visible ✓
   - Press Alt+Space → works immediately ✓

[ ] Manual Startup Toggle:
   - Right-click tray → Settings
   - Uncheck "Run on startup"
   - Restart computer
   - Verify nexus.exe NOT running ✓
   - Run nexus.exe manually
   - Check "Run on startup"
   - Restart computer
   - Verify nexus.exe running ✓

[ ] Registry Integrity:
   - Check startup registry key exists when enabled
   - Check startup registry key removed when disabled
   - Verify startup delay (2 seconds) works correctly
```

---

## 🎯 Test 4: Installer Testing

**Objective:** Verify MSI installer works correctly (Phase 2).

### Fresh Install Test
```
[ ] Run Nexus-Setup.msi on clean Windows machine
[ ] Verify installer shows license agreement
[ ] Verify default install path: C:\Program Files\Nexus\
[ ] Verify files installed:
   - nexus.exe ✓
   - README.txt ✓
   - LICENSE.rtf ✓
   - config_default.json ✓
[ ] Verify Start Menu shortcuts created
[ ] Verify Desktop shortcut option works
[ ] Verify Add/Remove Programs entry exists
[ ] Launch from Start Menu → works ✓
[ ] First-run wizard appears ✓
```

### Upgrade Install Test
```
[ ] Install version 1.0.0
[ ] Modify some config files
[ ] Run installer for version 1.1.0
[ ] Verify upgrade detected
[ ] Verify config preserved in %APPDATA%
[ ] Verify new files replace old ones
[ ] Verify app runs with existing settings
[ ] Check upgrade appears in Add/Remove Programs
```

### Uninstall Test
```
[ ] Open Add/Remove Programs
[ ] Select Nexus
[ ] Click Uninstall
[ ] Verify all files removed from Program Files
[ ] Verify shortcuts removed
[ ] Verify registry entries cleaned up
[ ] Verify startup disabled
[ ] Verify %APPDATA% config optionally kept
```

---

## 🎯 Test 5: First-Run Wizard Testing

**Objective:** Verify setup wizard works correctly (Phase 3).

### Wizard Flow Test
```
[ ] Run nexus.exe for first time (or delete config)
[ ] Verify wizard appears automatically
[ ] Test Welcome screen → Next works ✓
[ ] Test Hotkey screen:
   - Select different options ✓
   - Test "Test Hotkey" button ✓
   - Verify Next advances ✓
[ ] Test Startup screen:
   - Toggle checkboxes ✓
   - Verify Next advances ✓
[ ] Test Complete screen:
   - Verify selected options displayed ✓
   - Click Finish → wizard closes ✓
[ ] Verify config updated with wizard choices
[ ] Verify hotkey works immediately
```

### Wizard Skip Test
```
[ ] Run app second time
[ ] Verify wizard does NOT appear
[ ] Verify previous settings preserved
```

---

## 🎯 Test 6: Update System Testing

**Objective:** Verify auto-update system works (Phase 4).

### Manual Update Check
```
[ ] Right-click tray icon → "Check for Updates"
[ ] Verify update check starts (log messages)
[ ] If no update: verify appropriate message
[ ] If update available: verify notification shown
[ ] Verify version comparison works correctly
```

### Background Update Check
```
[ ] Let app run for 30+ seconds
[ ] Verify background update check happens
[ ] Check logs for update check activity
[ ] Verify 24-hour intervals work
```

### Update Download & Install (when available)
```
[ ] Trigger update check when new version available
[ ] Verify download starts
[ ] Verify progress indication (if implemented)
[ ] Verify file downloads to %TEMP%
[ ] Verify MSI launches automatically
[ ] Verify old version closes
[ ] Verify new version installs and starts
```

---

## 🎯 Test 7: Performance Testing

**Objective:** Verify app performance and resource usage.

### Memory & CPU Testing
```
[ ] Monitor Task Manager during normal use
[ ] Memory usage: < 50MB ✓
[ ] CPU usage: < 5% during search ✓
[ ] Memory stable after 100 searches ✓
[ ] No memory leaks over 1 hour use ✓
```

### Search Performance
```
[ ] Cold start: < 2 seconds to show window
[ ] Search response: < 100ms for local apps
[ ] Fuzzy search: < 200ms for large app lists
[ ] Calculator: < 50ms evaluation
```

### Startup Performance
```
[ ] Tray icon appears within 5 seconds of login
[ ] First hotkey press works within 2 seconds
[ ] App discovery completes within 10 seconds
```

---

## 🎯 Test 8: Edge Cases & Error Handling

**Objective:** Verify app handles edge cases gracefully.

### Error Scenarios
```
[ ] Network unavailable during update check ✓
[ ] GitHub API returns error ✓
[ ] Config file corrupted → fallback to defaults ✓
[ ] Hotkey conflicts → warning shown ✓
[ ] Multiple instances → second instance shows first ✓
[ ] Window closed while searching → no crash ✓
```

### Configuration Edge Cases
```
[ ] Empty config file → defaults loaded ✓
[ ] Invalid hotkey → fallback to Alt+Space ✓
[ ] Missing %APPDATA% folder → graceful handling ✓
[ ] Read-only config → warning logged ✓
```

---

## 🎯 Test 9: Cross-Version Compatibility

**Objective:** Verify upgrades work between versions.

### Version Upgrade Path
```
[ ] Install v0.1.0 → upgrade to v0.2.0
[ ] Install v0.2.0 → upgrade to v1.0.0
[ ] Verify config migration works
[ ] Verify registry entries updated
[ ] Verify shortcuts preserved
[ ] Verify startup settings maintained
```

---

## 📋 Automated Testing Setup

### Unit Tests
```bash
cargo test                    # Run all unit tests
cargo test -- --nocapture     # Show test output
```

### Integration Tests
```bash
# Create test script for installer
./installer/build.bat
# Test MSI installation in VM/sandbox
```

### Performance Benchmarks
```bash
cargo bench                  # Run performance benchmarks
```

---

## 🐛 Bug Tracking Template

When reporting bugs, include:

```
**Test Case:** [Which test case failed]
**Environment:** [Windows version, hardware specs]
**Steps to Reproduce:**
1. [Step 1]
2. [Step 2]
3. [Expected vs Actual]
**Logs:** [Relevant log output]
**Screenshots:** [If applicable]
```

---

## ✅ Test Completion Checklist

- [ ] All Phase 1 persistence tests pass
- [ ] All search functionality tests pass
- [ ] Startup integration tests pass
- [ ] Installer tests pass (fresh, upgrade, uninstall)
- [ ] Wizard tests pass
- [ ] Update system tests pass
- [ ] Performance requirements met
- [ ] Error handling verified
- [ ] Cross-version compatibility verified

**Final Sign-off:** All tests completed successfully ✅