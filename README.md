# Nexus - Windows Application Launcher

A lightweight, modern Windows launcher that runs as a background service with advanced configuration and auto-update capabilities.

## Features

✨ **Lightning Fast**: Instant application search and launch  
🎨 **Modern UI**: Clean, dark-themed interface with customizable opacity  
⚙️ **Highly Configurable**: Comprehensive settings for every aspect  
🔄 **Auto-Update**: Built-in update system with GitHub integration  
🚀 **Professional Installer**: WiX-based MSI installer with upgrade support  
🔧 **Background Service**: Runs persistently in system tray with crash recovery  

## Installation

### Option 1: MSI Installer (Recommended)
1. Download `Nexus-Setup.msi` from [Releases](https://github.com/Qwenzy/nexus/releases)
2. Double-click to install
3. Launch from Start Menu or Desktop shortcut

### Option 2: Build from Source
```bash
# Build release binary
cargo build --release

# Build MSI installer
cd installer
.\build.bat
```

## Usage

- **Launch**: Press `Alt+Space` (configurable) to show launcher
- **Search**: Start typing to find applications, files, or use calculator/web search
- **Execute**: Press Enter or click to launch (window hides automatically after launch)
- **Settings**: Right-click tray icon → Settings
- **Exit**: Right-click tray icon → Exit

The launcher runs as a background service and automatically hides after each use, staying ready in the system tray for the next activation.

## Configuration

Settings are stored in: `%APPDATA%\Nexus\config.json`

### Available Settings

**General:**
- Theme (Dark/Light)
- Window Opacity
- Font Size
- Window Size (Compact/Normal/Large)

**Search:**
- Max Results count
- Fuzzy Matching toggle
- Search Delay (debounce)
- Folder Exclusions

**Startup:**
- Run on Windows startup
- Show window on startup

**Updates:**
- Auto-check for updates
- Check frequency
- Beta channel toggle

**Service:**
- Run on Windows startup (with 2-second delay for system stability)
- Crash recovery and automatic restart
- Single instance management
- Background operation with system tray presence

## Development

### Prerequisites
- Rust 1.70+ ([rustup.rs](https://rustup.rs))
- WiX Toolset 3.11+ (for installer)
- Python 3.x with PIL/Pillow (for asset preparation, optional)

### Assets

The application uses a custom logo located in `docs/logoNexus.png`. The build process automatically:
- Copies the logo to `installerassets/icon.png` for the application UI
- Converts it to `installerassets/icon.ico` for the installer and system tray

### Build Commands
```bash
# Check code
cargo check

# Run in development
cargo run

# Build release
cargo build --release

# Build installer
cd installer && build.bat
```

### Project Structure
```
nexus/
├── src/           # Rust source code
├── ui/            # Slint UI definitions
├── installer/     # WiX installer config
├── installerassets/ # Application icons and assets
├── docs/          # Documentation and logos
└── Cargo.toml     # Project manifest
```

## Architecture

- **UI Framework**: Slint (declarative, compiled)
- **Platform**: Windows-native APIs
- **Hotkey**: global-hotkey crate
- **Tray**: tray-icon crate
- **Updates**: GitHub Releases API
- **Installer**: WiX Toolset
- **Service**: Background process with crash recovery and startup management

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE.rtf)

## Changelog

### v0.1.1 (2025-12-25)
- **Service Behavior**: Application now runs as a persistent background service
- **Improved Stability**: Added crash recovery and event loop error handling
- **Better Startup**: 2-second startup delay prevents system race conditions
- **Auto-Hide**: Launcher window automatically hides after launching applications
- **Process Monitoring**: Automatic restart if application crashes
- **Enhanced Installer**: Improved service registration and startup handling

### v0.1.0 (2025-12-23)
- Initial release
- Core launcher functionality
- Settings UI with 4 tabs
- MSI installer
- Auto-update system (planned)
- Comprehensive configuration

## Credits

Developed by **Qwenzy**  
Built with Rust 🦀 and Slint