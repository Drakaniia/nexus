# Nexus - Windows Application Launcher

A lightweight, modern Windows launcher with advanced configuration and auto-update capabilities.

## Features

✨ **Lightning Fast**: Instant application search and launch  
🎨 **Modern UI**: Clean, dark-themed interface with customizable opacity  
⚙️ **Highly Configurable**: Comprehensive settings for every aspect  
🔄 **Auto-Update**: Built-in update system with GitHub integration  
🚀 **Professional Installer**: WiX-based MSI installer with upgrade support  

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

- **Launch**: Press `Alt+Space` (configurable)
- **Search**: Start typing to find applications
- **Execute**: Press Enter or click to launch
- **Settings**: Right-click tray icon → Settings

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

## Development

### Prerequisites
- Rust 1.70+ ([rustup.rs](https://rustup.rs))
- WiX Toolset 3.11+ (for installer)

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
├── docs/          # Documentation
└── Cargo.toml     # Project manifest
```

## Architecture

- **UI Framework**: Slint (declarative, compiled)
- **Platform**: Windows-native APIs
- **Hotkey**: global-hotkey crate
- **Tray**: tray-icon crate  
- **Updates**: GitHub Releases API
- **Installer**: WiX Toolset

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE.rtf)

## Changelog

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