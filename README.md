# Dirloom

[![Crates.io](https://img.shields.io/crates/v/dirloom.svg)](https://crates.io/crates/dirloom)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/jchultarsky101/dirloom/actions/workflows/release.yml/badge.svg)](https://github.com/jchultarsky101/dirloom/actions)

A CLI TUI application for backing up directories and all subdirectories to another location on your local filesystem.

## Features

- 🖥️ **Interactive TUI** - Beautiful terminal interface built with [`ratatui`](https://ratatui.rs)
- ⚡ **Fast & Reliable** - Built with Rust for performance and safety
- 📋 **Easy Configuration** - Simple CLI arguments powered by [`clap`](https://docs.rs/clap)
- 🔄 **Multiple Sync Modes**:
  - **🪞 Mirror**: Destination matches source exactly (deletes extra files)
  - **📈 Incremental**: Only copy changed or new files (preserves extra files)
  - **🔄 Update**: Copy newer files, don't delete anything
  - **⚡ Force**: Overwrite ALL files regardless of existing content
- 🚫 **Exclude Patterns** - Skip files matching glob patterns (e.g., `*.tmp`, `node_modules`)
- 🔍 **Dry-run Mode** - Preview changes without modifying files
- 📊 **Progress Tracking** - Real-time progress display during backup operations
- 📁 **Directory Picker** - Interactive file browser for selecting source/destination
- ❓ **Help Screen** - Built-in help with keyboard shortcuts and mode descriptions

## Installation

### From Source

```bash
git clone https://github.com/jchultarsky101/dirloom.git
cd dirloom
cargo install --path .
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/jchultarsky101/dirloom/releases) page.

### Via Cargo

```bash
cargo install dirloom
```

## Usage

```bash
# Basic backup (opens TUI)
dirloom /path/to/source /path/to/destination

# Mirror sync (default - destination matches source exactly)
dirloom /source /destination
dirloom -m mirror /source /destination

# Incremental backup (only changed/new files, keeps extra files)
dirloom -m incremental /source /destination

# Update mode (copy newer files, don't delete)
dirloom -m update /source /destination

# Exclude patterns
dirloom -e "*.tmp" -e "node_modules" /source /destination

# Dry run (preview without changes)
dirloom --dry-run /source /destination

# Combined example
dirloom -m incremental -e "*.log" -e ".git" --dry-run /source /destination

# With debug logging
RUST_LOG=debug dirloom /source /destination
RUST_LOG=dirloom=trace dirloom /source /destination
```

### TUI Controls

Once the TUI opens:

**General:**
- **Space** - Start/restart backup
- **m** - Cycle sync mode (Mirror → Incremental → Update → Force)
- **1** or **s** - Open source directory picker
- **2** or **d** - Open destination directory picker
- **?** or **h** - Show help screen
- **q** or **Esc** - Cancel (during backup) or Quit
- **r** - Reset after completion

**Directory Picker:**
- **↑/↓** or **j/k** - Navigate list
- **Enter** or **→** - Open selected directory
- **Tab** or **Space** - Select current directory
- **Backspace** or **←** - Go to parent directory
- **Esc** - Cancel picker

### Command Line Options

```
dirloom [OPTIONS] <SOURCE> <DESTINATION>

Arguments:
  <SOURCE>        Source directory to backup
  <DESTINATION>   Destination directory for backup

Options:
  -m, --mode <MODE>  Backup synchronization mode [default: mirror]
                     [possible values: mirror, incremental, update]
  -e, --exclude <PAT>  Exclude patterns (can be used multiple times)
  -n, --dry-run        Show what would be done without making changes
  -h, --help           Print help
  -V, --version        Print version
```

## Development

### Prerequisites

- Rust 1.70 or later
- Git (for Git Flow workflow)

### Setup

```bash
# Clone the repository
git clone https://github.com/jchultarsky101/dirloom.git
cd dirloom

# Initialize Git Flow (if not already done)
git flow init -d

# Build the project
cargo build

# Run in development
cargo run -- /source /destination
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Project Structure

```
dirloom/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # Clap argument definitions
│   ├── tui/             # TUI components
│   │   ├── mod.rs
│   │   ├── app.rs       # Application state
│   │   └── ui.rs        # UI rendering
│   └── backup/          # Backup logic
│       ├── mod.rs
│       ├── error.rs     # Error types
│       ├── core.rs      # Core file operations
│       ├── sync.rs      # Sync strategies
│       └── progress.rs  # Progress tracking
├── tests/
│   └── integration.rs   # Integration tests
├── data/                # Test data (gitignored)
│   ├── source/          # Test source files
│   └── target/          # Test backup target
├── Cargo.toml           # Project configuration
├── CHANGELOG.md         # Version history
├── CONTRIBUTING.md      # Contribution guidelines
├── QWEN.md              # Development guide
└── LICENSE              # MIT License
```

## Roadmap

- [x] Core backup functionality
- [x] Interactive TUI implementation
- [x] File filtering and exclusion patterns
- [x] Progress bar and statistics
- [x] Unit and integration tests
- [ ] Restore functionality
- [ ] Scheduled backups
- [ ] Configuration file support
- [ ] Compression support
- [ ] Remote backup targets

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git flow feature start <name>`)
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

- [`ratatui`](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [`clap`](https://github.com/clap-rs/clap) - CLI argument parser
- [`cargo-dist`](https://github.com/axodotdev/cargo-dist) - Distribution tool
