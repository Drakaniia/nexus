# Nexus - Windows Launcher

A lightweight, fast, and aesthetically pleasing Windows launcher application inspired by Spotlight on macOS. Built with Rust and Slint, Nexus provides quick access to applications, files, and system utilities with a modern Windows 11 aesthetic.

## Features

### 🔍 Smart Search
- **Fuzzy matching**: Find applications even with partial or approximate text
- **Most Recently Used (MRU)**: Frequently used applications appear higher in results
- **Multi-source indexing**: Searches Start Menu, Desktop, and system utilities

### ⌨️ Quick Actions
- **Global hotkey**: Activate with `Alt + Space` anywhere in Windows
- **Calculator**: Type mathematical expressions directly (e.g., `2+2`, `sqrt(16)`)
- **Web search**: Quick search with prefixes like `g search term` for Google or `yt video` for YouTube
- **System commands**: Access system actions like Lock, Sleep, Restart, Shutdown, Sign Out, and Empty Recycle Bin

### 🎨 Modern UI
- **Windows 11 aesthetics**: Glass-like transparency effects with Mica-inspired backdrop
- **Smooth animations**: Fluid transitions and hover effects
- **Intuitive navigation**: Arrow keys to navigate results, Enter to select
- **Visual categorization**: Different icons for apps, files, calculations, and web searches

### 🚀 Performance
- **Lightweight**: Minimal resource usage with fast startup times
- **Asynchronous indexing**: Apps are discovered in the background
- **Optimized builds**: Release builds are optimized for size and performance

## Installation

### Prerequisites
- Windows 10 or Windows 11
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/Drakaniia/nexus.git
cd nexus
```

2. Build the application:
```bash
cargo build --release
```

3. The executable will be located at:
```
target/release/winlauncher.exe
```

### Pre-built Binaries
Pre-built binaries will be available in the [Releases](https://github.com/Drakaniia/nexus/releases) section once published.

## Usage

### Activation
Press `Alt + Space` anywhere in Windows to activate the launcher.

### Searching
- Start typing to search for applications, files, or system utilities
- Use arrow keys (`↑` and `↓`) to navigate results
- Press `Enter` to launch the selected item
- Press `Escape` to close the launcher

### Special Functions

#### Calculator
Type mathematical expressions directly:
- `2+2`
- `10 * 5`
- `sqrt(16)`
- `3.14 * 2^2`

#### Web Search
Use prefixes to initiate web searches:
- `g search terms` - Google search
- `google search terms` - Google search
- `yt video title` - YouTube search
- `youtube video title` - YouTube search
- `gh repository name` - GitHub search
- `wiki topic` - Wikipedia search

#### System Actions
Type these commands to perform system actions:
- `lock` - Lock your computer
- `sleep` - Put computer to sleep
- `restart` or `reboot` - Restart your computer
- `shutdown` - Shut down your computer
- `logout` or `sign out` - Sign out of your account
- `empty trash` or `empty recycle bin` - Empty the Recycle Bin

#### Direct URLs
Paste or type URLs directly to open them in your default browser:
- `https://www.example.com`

## Configuration

The application automatically discovers applications from:
- User's Start Menu
- System's Start Menu
- Desktop folder
- Common system utilities

Configuration options will be available in future releases.

## Contributing

We welcome contributions to Nexus! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

### Development Setup

1. Install Rust and Cargo
2. Install [Slint](https://slint.dev/) dependencies
3. Clone the repository
4. Build and run in debug mode:
```bash
cargo run
```

## Architecture

Nexus is built with:
- **Rust**: Systems programming language for performance and safety
- **Slint**: Toolkit for developing native user interfaces
- **Windows API**: Direct integration with Windows systems
- **Tokio**: Asynchronous runtime for background operations

### Key Components

- `main.rs`: Application entry point and event loop management
- `app_discovery.rs`: Logic for discovering installed applications
- `search.rs`: Fuzzy matching and search algorithms
- `actions.rs`: Special commands and system actions
- `ui/main.slint`: User interface definition

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Slint](https://slint.dev/) for the UI framework
- [Rust](https://www.rust-lang.org/) for the programming language
- [Windows API](https://docs.microsoft.com/en-us/windows/win32/api/) for system integration
- Inspired by Spotlight on macOS and other launcher applications

## Support

If you encounter any issues or have suggestions for improvement, please [open an issue](https://github.com/Drakaniia/nexus/issues) on GitHub.