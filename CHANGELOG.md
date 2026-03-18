# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup
- Git Flow workflow configuration
- cargo-dist for cross-platform releases
- CLI argument parsing with clap
- Three sync modes: mirror (default), incremental, update
- Exclude patterns support
- Dry-run mode
- Interactive TUI with ratatui
- Real-time progress tracking with percentage and statistics
- Parallel file processing with rayon
- Structured logging with tracing (RUST_LOG support)
- Custom error types with thiserror
- Comprehensive unit tests (24 tests)
- Integration tests (9 tests)
- Test data directory structure

### Changed
- Replaced anyhow with thiserror for better error handling
- Enhanced TUI with improved styling and emoji indicators
- Added parallel processing for better performance
- Improved logging with emojis for better readability

### Deprecated

### Removed

### Fixed

### Security

---

## [0.1.0] - YYYY-MM-DD

### Added
- Initial release

[Unreleased]: https://github.com/jchultarsky101/dirloom/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jchultarsky101/dirloom/releases/tag/v0.1.0
