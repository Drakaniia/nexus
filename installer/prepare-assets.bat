@echo off
REM ============================================================================
REM WinLauncher Asset Preparation Script
REM ============================================================================
REM This script prepares installer assets. It NOTE: requires ImageMagick for
REM PNG to ICO conversion. If ImageMagick is not installed, you'll need to
REM manually convert icon.png to icon.ico using an online tool or image editor.
REM ============================================================================

echo.
echo ============================================================================
echo Preparing Installer Assets
echo ============================================================================
echo.

REM Check if icon.png exists
if not exist "assets\icon.png" (
    echo ERROR: assets\icon.png not found!
    echo Please ensure the icon file is in the assets directory.
    pause
    exit /b 1
)

echo [1/2] Checking for ImageMagick...
where magick >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo WARNING: ImageMagick not found!
    echo.
    echo To convert PNG to ICO automatically, install ImageMagick from:
    echo https://imagemagick.org/script/download.php#windows
    echo.
    echo Alternative: Use an online converter:
    echo - https://convertio.co/png-ico/
    echo - https://www.icoconverter.com/
    echo.
    echo Save the converted file as: assets\icon.ico
    echo.
    pause
    exit /b 1
)

echo    Found ImageMagick
echo.

REM Convert PNG to ICO with multiple sizes
echo [2/2] Converting icon.png to icon.ico...
magick convert assets\icon.png -define icon:auto-resize=256,128,64,48,32,16 assets\icon.ico

if %errorlevel% neq 0 (
    echo ERROR: Conversion failed!
    pause
    exit /b 1
)

echo    Conversion successful
echo.

echo ============================================================================
echo Asset preparation complete!
echo ============================================================================
echo.
echo Created: assets\icon.ico
echo.
echo You can now build the installer with build.bat
echo.
pause
