@echo off
REM ============================================================================
REM Nexus Complete Release Build Script
REM ============================================================================
REM This script builds Nexus from source to GitHub release
REM Prerequisites: Rust, WiX Toolset, Git, GitHub CLI
REM ============================================================================

setlocal enabledelayedexpansion

echo.
echo ============================================================================
echo Nexus Release Build Pipeline
echo ============================================================================
echo.

REM Get version from Cargo.toml
for /f "tokens=3 delims= " %%a in ('findstr /C:"version = " Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
)

echo Building Nexus v!VERSION!
echo.

REM ============================================================================
REM Phase 1: Clean and Setup
REM ============================================================================

echo [Phase 1/6] Cleaning workspace...
cargo clean
if exist "Nexus-Setup.msi" del "Nexus-Setup.msi"
if exist "RELEASE_NOTES.md" del "RELEASE_NOTES.md"
echo    Workspace cleaned
echo.

REM ============================================================================
REM Phase 2: Build Application
REM ============================================================================

echo [Phase 2/6] Building release binary...
cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    echo.
    pause
    exit /b 1
)

echo    Release binary built successfully
dir target\release\nexus.exe
echo.

REM ============================================================================
REM Phase 3: Build Installer
REM ============================================================================

echo [Phase 3/6] Building MSI installer...
cd installer
call build.bat
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Installer build failed!
    echo.
    pause
    exit /b 1
)

cd ..
echo    MSI installer built successfully
dir Nexus-Setup.msi
echo.

REM ============================================================================
REM Phase 4: Run Tests
REM ============================================================================

echo [Phase 4/6] Running tests...
cargo test --release
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Tests failed!
    echo.
    pause
    exit /b 1
)
echo    All tests passed
echo.

REM ============================================================================
REM Phase 5: Create Release Notes
REM ============================================================================

echo [Phase 5/6] Creating release notes...
(
echo # Nexus v!VERSION!
echo.
echo ## What's New
echo.
echo ### ✨ Features
echo - First-run setup wizard for easy configuration
echo - Professional MSI installer with automatic updates
echo - Persistent background service with system tray integration
echo - Lightning-fast fuzzy search with calculator and web integration
echo - Comprehensive settings and customization options
echo.
echo ### 🔧 Improvements
echo - Enhanced hotkey handling and conflict detection
echo - Improved app discovery and search performance
echo - Better error handling and logging
echo - Optimized memory usage and startup time
echo.
echo ### 🐛 Bug Fixes
echo - Fixed application persistence issues
echo - Resolved window focus and positioning problems
echo - Fixed startup registration and integration
echo.
echo ## Installation
echo.
echo ### Option 1: Installer (Recommended)
echo 1. Download `Nexus-Setup.msi`
echo 2. Run the installer
echo 3. Follow the setup wizard
echo 4. Press Alt+Space to launch Nexus
echo.
echo ### Option 2: Portable
echo 1. Download `nexus.exe`
echo 2. Run the executable
echo 3. Configure manually if needed
echo.
echo ## System Requirements
echo - Windows 10/11 (64-bit)
echo - 100MB free disk space
echo - Administrator rights for installation
echo.
echo ## Updating
echo Nexus includes automatic update checking. You can also manually check:
echo 1. Right-click the system tray icon
echo 2. Click "Check for Updates"
echo.
echo ## First Run
echo On first launch, the setup wizard will help you:
echo - Choose your hotkey (default: Alt+Space)
echo - Configure startup options
echo - Set up your preferences
echo.
echo ## Support
echo - Issues: https://github.com/Qwenzy/nexus/issues
echo - Documentation: https://github.com/Qwenzy/nexus#readme
echo.
echo ## Technical Details
echo - Built with Rust for performance and reliability
echo - Uses Slint for modern UI with GPU acceleration
echo - MSI installer ensures proper Windows integration
echo - Background service architecture for instant access
) > RELEASE_NOTES.md

echo    Release notes created
echo.

REM ============================================================================
REM Phase 6: Create Git Tag and Push
REM ============================================================================

echo [Phase 6/6] Creating Git release...
echo.

REM Check if tag already exists
git tag -l "v!VERSION!" | findstr "v!VERSION!" >nul
if %errorlevel% equ 0 (
    echo WARNING: Tag v!VERSION! already exists!
    echo Deleting existing tag...
    git tag -d "v!VERSION!"
    git push origin --delete "v!VERSION!"
)

echo Creating tag v!VERSION!...
git add .
git commit -m "Release v!VERSION!" 2>nul || echo "No changes to commit"
git tag -a "v!VERSION!" -m "Release v!VERSION!"

echo Pushing tag to GitHub...
git push origin main
git push origin "v!VERSION!"

if %errorlevel% neq 0 (
    echo.
    echo ERROR: Failed to push to GitHub!
    echo.
    pause
    exit /b 1
)

echo.
echo ============================================================================
echo RELEASE BUILD COMPLETE!
echo ============================================================================
echo.
echo Version: v!VERSION!
echo Files ready for GitHub release:
echo - Nexus-Setup.msi (installer)
echo - target/release/nexus.exe (portable)
echo - RELEASE_NOTES.md (release notes)
echo.
echo Next steps:
echo 1. Go to: https://github.com/Qwenzy/nexus/releases/new
echo 2. Select tag: v!VERSION!
echo 3. Title: Nexus v!VERSION!
echo 4. Copy content from RELEASE_NOTES.md
echo 5. Upload: Nexus-Setup.msi
echo 6. Upload: target/release/nexus.exe (optional)
echo 7. Click "Publish release"
echo.
echo Or use GitHub CLI:
echo gh release create v!VERSION! ^
echo    --title "Nexus v!VERSION!" ^
echo    --notes-file RELEASE_NOTES.md ^
echo    Nexus-Setup.msi target/release/nexus.exe
echo.
pause