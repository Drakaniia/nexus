@echo off
REM ============================================================================
REM Nexus MSI Installer Build Script
REM ============================================================================
REM This script builds the Nexus MSI installer using WiX Toolset
REM Prerequisites: WiX Toolset v3.x installed and in PATH
REM ============================================================================

echo.
echo ============================================================================
echo Building Nexus MSI Installer
echo ============================================================================
echo.

REM Check if WiX Toolset is installed
echo [1/6] Checking for WiX Toolset...
where candle >nul 2>nul
if %errorlevel% neq 0 (
    echo.
    echo ERROR: WiX Toolset not found!
    echo.
    echo Please install WiX Toolset from:
    echo https://wixtoolset.org/releases/
    echo.
    echo Make sure to add WiX bin directory to PATH
    echo Default location: C:\Program Files ^(x86^)\WiX Toolset v3.11\bin
    echo.
    pause
    exit /b 1
)
echo    Found WiX Toolset
echo.

REM Check if Cargo is installed
echo [2/6] Checking for Rust/Cargo...
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Cargo not found!
    echo Please install Rust from: https://rustup.rs/
    echo.
    pause
    exit /b 1
)
echo    Found Cargo
echo.

REM Build the Rust application in release mode
echo [3/6] Building Nexus release binary...
cd ..

REM Kill running instances to avoid "Access is denied" errors
taskkill /f /im nexus.exe >nul 2>&1
timeout /t 2 /nobreak >nul
echo    Ensured no running instances...

cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Cargo build failed!
    echo.
    pause
    exit /b 1
)
echo    Build successful
echo.

REM Verify executable exists
if not exist "target\release\nexus.exe" (
    echo.
    echo ERROR: nexus.exe not found in target\release\
    echo.
    pause
    exit /b 1
)

REM Return to installer directory
cd installer

REM Compile WiX source file
echo [4/6] Compiling WiX source ^(nexus.wxs^)...
candle nexus.wxs -ext WixUIExtension
if %errorlevel% neq 0 (
    echo.
    echo ERROR: WiX compilation failed!
    echo Check nexus.wxs for syntax errors
    echo.
    pause
    exit /b 1
)
echo    Compilation successful
echo.

REM Link the MSI package
echo [5/6] Linking MSI package...
light nexus.wixobj -ext WixUIExtension -out ..\Nexus-Setup.msi
if %errorlevel% neq 0 (
    echo.
    echo ERROR: MSI linking failed!
    echo.
    pause
    exit /b 1
)
echo    Linking successful
echo.

REM Clean up intermediate files
echo [6/6] Cleaning up...
del nexus.wixobj 2>nul
del nexus.wixpdb 2>nul
echo    Cleanup complete
echo.

REM Display success message
echo ============================================================================
echo SUCCESS!
echo ============================================================================
echo.
echo MSI installer created: Nexus-Setup.msi
echo Location: %CD%\..\
echo.

cd ..
dir Nexus-Setup.msi

echo.
echo To test the installer:
echo 1. Double-click Nexus-Setup.msi
echo 2. Follow the installation wizard
echo 3. Launch Nexus from Start Menu or Desktop
echo.
echo To create a release:
echo 1. Tag version: git tag v1.0.0
echo 2. Push tag: git push origin v1.0.0
echo 3. Upload Nexus-Setup.msi to GitHub Releases
echo.
pause
