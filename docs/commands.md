# Project Commands

This document contains all the commands you can use to work with the WinLauncher project.

## Development Commands

### Build Commands
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode (optimized)
- `cargo check` - Check for compilation errors without building
- `cargo clean` - Remove generated files and build artifacts

### Run Commands
- `cargo run` - Run the project in debug mode
- `cargo run --release` - Run the project in release mode
- `cargo run --bin winlauncher` - Run the specific binary

### Test Commands
- `cargo test` - Run all tests
- `cargo test --lib` - Run only library tests
- `cargo test <test_name>` - Run a specific test
- `cargo test --release` - Run tests in release mode

### Code Quality Commands
- `cargo fmt` - Format all code according to Rust style guidelines
- `cargo fmt --check` - Check if code is properly formatted
- `cargo clippy` - Run linting to find common mistakes and style issues
- `cargo clippy --fix` - Automatically fix some clippy issues

### Documentation Commands
- `cargo doc` - Build documentation
- `cargo doc --open` - Build and open documentation in browser
- `cargo doc --document-private-items` - Include private items in docs

## Git Commands
- `git status` - Show current status of the repository
- `git add .` - Add all changes to staging
- `git add <file>` - Add specific file to staging
- `git commit -m "message"` - Commit staged changes
- `git push origin main` - Push changes to remote repository
- `git pull origin main` - Pull latest changes from remote repository
- `git log --oneline` - Show commit history in compact format
- `git diff` - Show unstaged changes
- `git diff --staged` - Show staged changes

## Dependency Management
- `cargo add <dependency>` - Add a new dependency
- `cargo add <dependency>@<version>` - Add specific version of dependency
- `cargo remove <dependency>` - Remove a dependency
- `cargo update` - Update dependencies to latest compatible versions
- `cargo update -p <package>` - Update specific package
- `cargo tree` - Show dependency tree

## Clean Build Process
1. Clean previous builds: `cargo clean`
2. Update dependencies: `cargo update`
3. Check for errors: `cargo check`
4. Format code: `cargo fmt`
5. Run lints: `cargo clippy`
6. Run tests: `cargo test`
7. Build release: `cargo build --release`

## Debugging Commands
- `cargo run --verbose` - Run with verbose output
- `RUST_BACKTRACE=1 cargo run` - Run with backtrace enabled
- `cargo build --verbose` - Build with verbose output
- `cargo tree --format "{p} {v} {f}"` - Detailed dependency view

## Release Process
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite: `cargo test --release`
4. Build release binary: `cargo build --release`
5. Test release build: `./target/release/winlauncher`
6. Commit and tag: `git tag v<x.y.z>`
7. Push with tags: `git push origin main --tags`
8. Create GitHub release with built binary

## Windows-Specific Commands
- `cargo build --target x86_64-pc-windows-msvc` - Build specifically for Windows x86_64
- `cargo run --target x86_64-pc-windows-msvc` - Run on Windows target
- `cargo install --force --path .` - Install the binary locally

## Useful Aliases
You can add these to your shell profile:

```bash
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cf='cargo fmt'
alias cc='cargo clippy'
alias crr='cargo run --release'
alias cbr='cargo build --release'
```