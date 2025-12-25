@echo off
REM ============================================================================
REM Nexus Development Environment Setup
REM ============================================================================
REM This script sets up the complete development environment for Nexus
REM Run this once after cloning the repository
REM ============================================================================

echo.
echo ============================================================================
echo Nexus Development Environment Setup
echo ============================================================================
echo.

REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% == 0 (
    echo Running as administrator ✓
) else (
    echo WARNING: Not running as administrator
    echo Some components may require admin rights
)
echo.

REM ============================================================================
REM Install Rust
REM ============================================================================

echo [1/5] Checking Rust installation...
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo Installing Rust...
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    if %errorlevel% neq 0 (
        echo ERROR: Failed to install Rust
        pause
        exit /b 1
    )
    REM Refresh environment
    call refreshenv.cmd 2>nul || call "%ProgramFiles%\Git\cmd\refreshenv.cmd" 2>nul || echo Please restart command prompt
) else (
    echo Rust already installed ✓
    rustc --version
)
echo.

REM ============================================================================
REM Install WiX Toolset
REM ============================================================================

echo [2/5] Checking WiX Toolset...
where candle >nul 2>nul
if %errorlevel% neq 0 (
    echo Installing WiX Toolset...
    echo Downloading WiX Toolset v3.14...

    REM Download WiX installer
    powershell -Command "& {Invoke-WebRequest -Uri 'https://github.com/wixtoolset/wix3/releases/download/wix314rtm/wix314.exe' -OutFile 'wix_setup.exe'}"

    if not exist wix_setup.exe (
        echo ERROR: Failed to download WiX
        pause
        exit /b 1
    )

    echo Installing WiX Toolset (this may take a moment)...
    wix_setup.exe /quiet /norestart

    REM Clean up
    del wix_setup.exe

    REM Add WiX to PATH for current session
    set "WIX_PATH=%ProgramFiles(x86)%\WiX Toolset v3.14\bin"
    if exist "%WIX_PATH%" (
        set "PATH=%PATH%;%WIX_PATH%"
        echo WiX Toolset installed ✓
    ) else (
        echo WARNING: WiX installation may have failed
        echo Please install manually from: https://wixtoolset.org/releases/
    )
) else (
    echo WiX Toolset already installed ✓
    candle -?
)
echo.

REM ============================================================================
REM Install Python (for asset preparation)
REM ============================================================================

echo [3/5] Checking Python installation...
where python >nul 2>nul
if %errorlevel% neq 0 (
    where py >nul 2>nul
    if %errorlevel% neq 0 (
        echo WARNING: Python not found
        echo Python is optional but recommended for asset processing
        echo Install from: https://python.org
    ) else (
        echo Python found via 'py' command ✓
        py --version
    )
) else (
    echo Python found ✓
    python --version
)
echo.

REM ============================================================================
REM Setup project dependencies
REM ============================================================================

echo [4/5] Installing project dependencies...
cargo fetch
if %errorlevel% neq 0 (
    echo ERROR: Failed to fetch dependencies
    pause
    exit /b 1
)
echo Dependencies fetched ✓
echo.

REM ============================================================================
REM Build test
REM ============================================================================

echo [5/5] Running initial build test...
cargo check
if %errorlevel% neq 0 (
    echo ERROR: Initial build check failed
    echo Check the error messages above
    pause
    exit /b 1
)
echo Initial build check passed ✓
echo.

REM ============================================================================
REM Setup complete
REM ============================================================================

echo.
echo ============================================================================
echo DEVELOPMENT ENVIRONMENT SETUP COMPLETE!
echo ============================================================================
echo.
echo You're ready to develop Nexus! Here are some useful commands:
echo.
echo Build debug version:    cargo build
echo Build release version:  cargo build --release
echo Run tests:             cargo test
echo Build installer:       cd installer ^& build.bat
echo Full release build:     .\scripts\build_release.bat
echo.
echo Useful files:
echo - README.md              : Project documentation
echo - docs/testing_checklist.md : Testing guide
echo - plan.md                : Original project plan
echo.
echo Happy coding! 🚀
echo.
pause