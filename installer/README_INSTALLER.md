# Nexus Installer Build Guide

This directory contains the WiX Toolset installer configuration for creating a professional Windows MSI installer for Nexus.

## Prerequisites

### 1. WiX Toolset Installation

**WiX Toolset v3.11+ Required**

Download and install from: https://wixtoolset.org/releases/

**Installation Steps:**
1. Download WiX Toolset installer (e.g., `wix311.exe`)
2. Run the installer with administrator privileges
3. Complete installation wizard
4. Verify installation by opening Command Prompt and running:
   ```cmd
   candle -?
   ```
   You should see WiX compiler help output

**Add to PATH (if not automatic):**
- Default WiX location: `C:\Program Files (x86)\WiX Toolset v3.11\bin`
- Add to System Environment Variables → Path

### 2. Rust Toolchain

Ensure Rust is installed with the stable toolchain:
```cmd
cargo --version
```

If not installed, get Rust from: https://rust up.rs/

### 3. Additional Assets

The installer expects the following assets in the `installer/assets/` directory:

- **icon.ico** - Application icon (32x32, 48x48, 256x256 sizes recommended)
- **banner.bmp** (optional) - 493x58 pixel installer banner (top of wizard)
- **dialog.bmp** (optional) - 493x312 pixel welcome dialog background

**Note:** If assets are missing, the build will use defaults or skip optional images.

## Building the Installer

### Quick Build

Simply run the build script from the `installer` directory:

```cmd
cd installer
build.bat
```

This will:
1. Check for WiX Toolset and Cargo
2. Build the Rust application in release mode
3. Compile the WiX source file
4. Link the MSI package
5. Create `Nexus-Setup.msi` in the project root

### Manual Build (Advanced)

If you need to build manually:

```cmd
# 1. Build Rust application
cargo build --release

# 2. Navigate to installer directory
cd installer

# 3. Compile WiX source
candle nexus.wxs -ext WixUIExtension

# 4. Link MSI package
light nexus.wixobj -ext WixUIExtension -out ..\Nexus-Setup.msi

# 5. Clean up intermediate files
del nexus.wixobj
del nexus.wixpdb
```

## Installer Configuration

### Product Information

Edit `nexus.wxs` to customize:

```xml
<?define ProductVersion = "1.0.0" ?>
<?define ProductName = "Nexus" ?>
<?define Manufacturer = "Qwenzy" ?>
```

**IMPORTANT:** The `UpgradeCode` GUID must remain constant across all versions to enable proper upgrade detection:

```xml
UpgradeCode="A1B2C3D4-E5F6-4789-A0B1-C2D3E4F5G6H7"
```

### Installation Components

The installer includes the following component groups:

1. **ProductComponents**
   - Main executable (`nexus.exe`)
   - Default configuration template
   - README and LICENSE files

2. **ShortcutComponents**
   - Start Menu shortcuts (Nexus, Uninstall)
   - Optional Desktop shortcut

3. **RegistryComponents**
   - Windows startup registration (optional)
   - Application configuration settings

### File Locations

**During Installation:**
- Default install path: `C:\Program Files\Nexus\`
- User can customize during installation

**After Installation:**
- Application config: `%APPDATA%\Nexus\config.json`
- Application data: `%APPDATA%\Nexus\`
- Registry settings: `HKCU\Software\Qwenzy\Nexus\`

## Testing the Installer

### Fresh Installation Test

1. Build the installer: `cd installer && build.bat`
2. Locate `Nexus-Setup.msi` in project root
3. Double-click to run installer
4. Follow wizard:
   - Accept license
   - Choose installation directory
   - Select components
   - Click Install

**Verify:**
- ✓ Files installed to chosen directory
- ✓ Start Menu shortcut exists
- ✓ Desktop shortcut (if selected)
- ✓ "Add/Remove Programs" entry
- ✓ Application launches successfully
- ✓ First-run wizard appears (if applicable)

### Upgrade Test

1. Install version 1.0.0
2. Update version in `nexus.wxs`:
   ```xml
   <?define ProductVersion = "1.0.1" ?>
   ```
3. Rebuild installer
4. Run new installer
5. Verify upgrade detected and configuration preserved

### Uninstallation Test

1. Open Settings → Apps → Installed apps
2. Find "Nexus"
3. Click Uninstall
4. Verify:
   - Program Files directory removed
   - Shortcuts removed
   - Registry entries cleaned (except user config)
   - Process terminated gracefully

## Silent Installation

For automated/enterprise deployment:

```cmd
REM Silent installation with default settings
msiexec /i Nexus-Setup.msi /quiet /qn

REM Silent installation with custom directory
msiexec /i Nexus-Setup.msi /quiet /qn INSTALLFOLDER="C:\CustomPath\Nexus"

REM Silent installation without startup registration
msiexec /i Nexus-Setup.msi /quiet /qn RUNONSTARTUP=0

REM Silent uninstallation
msiexec /x Nexus-Setup.msi /quiet /qn
```

### Silent Installation Parameters

- `/i` - Install
- `/x` - Uninstall
- `/quiet` - No user interaction
- `/qn` - No UI displayed
- `INSTALLFOLDER="path"` - Custom installation directory
- `RUNONSTARTUP=0` - Disable startup registration (default is 1)

## Code Signing (Recommended)

For distribution, sign the MSI to avoid Windows SmartScreen warnings:

### Prerequisites
- Code signing certificate (from Comodo, DigiCert, etc.)
- SignTool.exe (included with Windows SDK)

### Signing Command

```cmd
signtool sign /f YourCertificate.pfx /p YourPassword /t http://timestamp.digicert.com /d "Nexus" /du "https://github.com/Drakaniia/nexus" Nexus-Setup.msi
```

**Parameters:**
- `/f` - Certificate file
- `/p` - Certificate password
- `/t` - Timestamp server (proves when code was signed)
- `/d` - Description
- `/du` - URL for more info

## Troubleshooting

### "WiX Toolset not found"

- Verify WiX is installed: `candle -?`
- Check PATH includes WiX bin directory
- Restart Command Prompt after adding to PATH

### "Cargo build failed"

- Ensure Rust toolchain is installed
- Try `cargo clean` then rebuild
- Check for compilation errors in Rust code

### "File not found: icon.ico"

- Create or copy an icon file to `installer/assets/icon.ico`
- Or remove icon references from `nexus.wxs`

### "ICE Validation Errors"

Internal Consistency Evaluators (ICE) warnings can usually be ignored for simple installers. To suppress:

```cmd
light -sice:ICE61 nexus.wixobj -out Nexus-Setup.msi
```

### Installer Won't Uninstall Old Version

- Check `UpgradeCode` is consistent across versions
- Ensure `Product Id="*"` (generates new GUID each build)
- Verify `MajorUpgrade` element is present

## Customization

### Add Custom Dialog

Edit `nexus.wxs` UI section:

```xml
<UI>
  <UIRef Id="WixUI_InstallDir" />
  <Publish Dialog="CustomDialog" Control="Next" Event="NewDialog" Value="NextDialog">1</Publish>
</UI>
```

### Add Additional Files

Add to `ProductComponents` group:

```xml
<Component Id="MyFile" Guid="NEW-GUID-HERE">
  <File Id="MyFileId" Source="path\to\file.ext" KeyPath="yes" />
</Component>
```

### Conditional Installation

Use conditions on components:

```xml
<Component Id="OptionalFeature" Guid="GUID">
  <Condition>FEATUREENABLED = 1</Condition>
  <!-- component contents -->
</Component>
```

## Distribution Checklist

Before releasing the installer:

- [ ] Update version in `nexus.wxs`
- [ ] Test fresh installation
- [ ] Test upgrade from previous version
- [ ] Test uninstallation
- [ ] Sign MSI with code signing certificate (if available)
- [ ] Test on clean Windows 10 VM
- [ ] Test on Windows 11
- [ ] Create GitHub release with installer
- [ ] Update README with installation instructions

## Resources

- **WiX Toolset Documentation**: https://wixtoolset.org/documentation/
- **ICE Reference**: https://docs.microsoft.com/en-us/windows/win32/msi/internal-consistency-evaluators-ices
- **MSI Best Practices**: https://docs.microsoft.com/en-us/windows/win32/msi/windows-installer-best-practices

## Support

For issues with the installer, please open an issue at:
https://github.com/Drakaniia/nexus/issues
