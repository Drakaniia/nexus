@echo off
REM ============================================================================
REM Nexus Asset Preparation Script
REM ============================================================================
REM This script prepares installer assets. It requires ImageMagick for PNG to ICO
REM conversion. If ImageMagick is not installed, it will try Python with PIL.
REM ============================================================================

echo.
echo ============================================================================
echo Preparing Installer Assets
echo ============================================================================
echo.

REM Copy logo from docs if it exists
if exist "..\docs\logoNexus.png" (
    echo [0/3] Copying logo from docs...
    copy "..\docs\logoNexus.png" "assets\icon.png" >nul
    echo    Logo copied successfully
) else (
    echo [0/3] Logo not found in docs, using existing icon.png
)
echo.

REM Check if icon.png exists
if not exist "assets\icon.png" (
    echo ERROR: assets\icon.png not found!
    echo Please ensure the icon file is in the assets directory or docs\logoNexus.png exists.
    pause
    exit /b 1
)

echo [1/3] Checking for ImageMagick...
where magick >nul 2>&1
if %errorlevel% equ 0 (
    echo    Found ImageMagick
    echo.
    goto :convert_imagemagick
)

echo    ImageMagick not found, trying Python with PIL...
python prepare-assets.py
if %errorlevel% equ 0 (
    goto :success
)

echo.
echo ERROR: No conversion tools available!
echo.
echo Please install one of the following:
echo.
echo Option 1 - ImageMagick:
echo   Download from: https://imagemagick.org/script/download.php#windows
echo.
echo Option 2 - Python with PIL:
echo   pip install pillow
echo.
echo Option 3 - Manual conversion:
echo   Use online converter: https://convertio.co/png-ico/
echo   Save multiple sizes: 256x256, 128x128, 64x64, 48x48, 32x32, 16x16
echo   Save as: assets\icon.ico
echo.
pause
exit /b 1

:convert_imagemagick
REM Convert PNG to ICO with multiple sizes
echo [2/3] Converting icon.png to icon.ico...
magick convert assets\icon.png -define icon:auto-resize=256,128,64,48,32,16 assets\icon.ico

if %errorlevel% neq 0 (
    echo ERROR: ImageMagick conversion failed!
    pause
    exit /b 1
)

echo    Conversion successful with ImageMagick
echo.
goto :success

:success
echo ============================================================================
echo Asset preparation complete!
echo ============================================================================
echo.
echo Files ready:
echo   assets\icon.png - Application logo
echo   assets\icon.ico - Installer icon
echo.
echo You can now build the installer with build.bat
echo.
pause
