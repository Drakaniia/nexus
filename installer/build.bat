@echo off
REM ============================================================================
REM WinLauncher MSI Installer Build Script
REM ============================================================================
REM This script builds the WinLauncher MSI installer using WiX Toolset
REM Prerequisites: WiX Toolset v3.x installed and in PATH
REM ============================================================================

echo.
echo ============================================================================
echo Building WinLauncher MSI Installer
echo ============================================================================
echo.

REM Check if WiX Toolset is installed
echo [1/6] Checking for WiX Toolset...
where candle >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo ERROR: WiX Toolset not found!
    echo.
    echo Please install WiX Toolset from:
    echo https://wixtoolset.org/releases/
    echo.
    echo Make sure to add WiX bin directory to PATH
    echo Default location: C:\Program Files (x86)\WiX Toolset v3.11\bin
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
echo [3/6] Building WinLauncher release binary...
cd ..
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
if not exist "target\release\winlauncher.exe" (
    echo.
    echo ERROR: winlauncher.exe not found in target\release\
    echo.
    pause
    exit /b 1
)

REM Create default config template if it doesn't exist
if not exist "installer\config_default.json" (
    echo Creating default config template...
    echo { > installer\config_default.json
    echo   "hotkey": { >> installer\config_default.json
    echo     "modifiers": ["Alt"], >> installer\config_default.json
    echo     "key": "Space" >> installer\config_default.json
    echo   }, >> installer\config_default.json
    echo   "startup": { >> installer\config_default.json
    echo     "enabled": true, >> installer\config_default.json
    echo     "show_on_startup": false >> installer\config_default.json
    echo   }, >> installer\config_default.json
    echo   "appearance": { >> installer\config_default.json
    echo     "theme": "dark", >> installer\config_default.json
    echo     "opacity": 0.96, >> installer\config_default.json
    echo     "max_results": 8 >> installer\config_default.json
    echo   }, >> installer\config_default.json
    echo   "first_run": true >> installer\config_default.json
    echo } >> installer\config_default.json
)

REM Create LICENSE.rtf for installer if it doesn't exist
if not exist "installer\LICENSE.rtf" (
    echo Creating LICENSE.rtf for installer...
    echo {\rtf1\ansi\deff0 {\fonttbl {\f0 Courier New;}} > installer\LICENSE.rtf
    echo {\colortbl;\red0\green0\blue0;\red0\green0\blue255;} >> installer\LICENSE.rtf
    echo \f0\fs20 MIT License\par >> installer\LICENSE.rtf
    echo \par >> installer\LICENSE.rtf
    echo Copyright (c) 2024 Qwenzy\par >> installer\LICENSE.rtf
    echo \par >> installer\LICENSE.rtf
    echo Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:\par >> installer\LICENSE.rtf
    echo \par >> installer\LICENSE.rtf
    echo The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\par >> installer\LICENSE.rtf
    echo \par >> installer\LICENSE.rtf
    echo THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.\par >> installer\LICENSE.rtf
    echo } >> installer\LICENSE.rtf
)

REM Return to installer directory
cd installer

REM Compile WiX source file
echo [4/6] Compiling WiX source (winlauncher.wxs)...
candle winlauncher.wxs -ext WixUIExtension
if %errorlevel% neq 0 (
    echo.
    echo ERROR: WiX compilation failed!
    echo Check winlauncher.wxs for syntax errors
    echo.
    pause
    exit /b 1
)
echo    Compilation successful
echo.

REM Link the MSI package
echo [5/6] Linking MSI package...
light winlauncher.wixobj -ext WixUIExtension -out ..\WinLauncher-Setup.msi
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
del winlauncher.wixobj 2>nul
del winlauncher.wixpdb 2>nul
echo    Cleanup complete
echo.

REM Display success message
echo ============================================================================
echo SUCCESS!
echo ============================================================================
echo.
echo MSI installer created: WinLauncher-Setup.msi
echo Location: %CD%\..\
echo.

cd ..
dir WinLauncher-Setup.msi

echo.
echo To test the installer:
echo 1. Double-click WinLauncher-Setup.msi
echo 2. Follow the installation wizard
echo 3. Launch WinLauncher from Start Menu or Desktop
echo.
echo To create a release:
echo 1. Tag version: git tag v1.0.0
echo 2. Push tag: git push origin v1.0.0
echo 3. Upload WinLauncher-Setup.msi to GitHub Releases
echo.
pause
