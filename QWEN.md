# Dirloom Development Guide

## Project Overview

Dirloom is a CLI TUI application for backing up directories and all subdirectories to another location on the local filesystem.

## Tech Stack

- **Language**: Rust (edition 2024)
- **CLI Framework**: `clap` for command-line argument parsing
- **TUI Framework**: `ratatui` for terminal user interface
- **Error Handling**: `thiserror` for custom error types
- **Logging**: `tracing` with `tracing-subscriber` for structured logging (RUST_LOG support)
- **Parallel Processing**: `rayon` for parallel file operations
- **Build Tool**: Cargo
- **Distribution**: `cargo-dist` for cross-platform releases

## Core Development Principles

### 1. Accuracy Over Speed

- Analyze requirements deeply before implementation
- Ask clarifying questions when anything is unclear
- Prefer correct, well-tested code over fast delivery
- Document assumptions and edge cases explicitly

### 2. Git Flow Best Practices

This project uses [Git Flow](https://nvie.com/posts/a-successful-git-branching-model/):

**Branch Structure:**
- `main` - Production-ready code only (auto-deployed via CI)
- `develop` - Integration branch for all features
- `feature/*` - New features (branch from `develop`, merge back to `develop`)
- `release/*` - Release preparation (branch from `develop`, merge to `main` and `develop`)
- `hotfix/*` - Urgent production fixes (branch from `main`, merge to `main` and `develop`)

**Common Commands:**
```bash
git flow init                    # Initialize (already done)
git flow feature start <name>    # Start new feature
git flow feature finish <name>   # Complete feature (runs tests, merges to develop)
git flow release start <version> # Prepare release
git flow release finish <version># Complete release (tags, merges to main)
git flow hotfix start <name>     # Start hotfix
```

**Best Practices:**
- Never commit directly to `main` or `develop`
- Complete one feature per branch
- Run full test suite before `feature finish`
- Keep feature branches short-lived (rebase often)

### 3. Testing Requirements

**All new features MUST include:**
- Unit tests for core logic and edge cases
- Integration tests for user-facing functionality
- Tests must pass before marking tasks complete

**Test Structure:**
```
src/
├── module.rs      # Implementation
└── module/
    └── tests.rs   # Unit tests (or inline #[cfg(test)] module)
tests/
└── integration/   # Integration tests
```

**Test Commands:**
```bash
cargo test                    # Run all tests
cargo test --lib              # Unit tests only
cargo test --test integration # Integration tests only
cargo test -- --nocapture     # Show test output
cargo test -- --test-threads=1 # Sequential execution
```

### 4. Code Quality Gate

**Before any commit:**
```bash
cargo fmt              # Format code
cargo clippy -- -D warnings  # Lint (warnings = errors)
cargo build            # Verify compilation (no warnings)
cargo test             # All tests must pass
```

**CI will reject:**
- Code that doesn't compile cleanly
- Any clippy warnings
- Failed tests
- Unformatted code

### 5. Documentation Standards

**Keep documentation always up to date:**

- Update README.md when CLI arguments change
- Update CHANGELOG.md for every user-facing change
- Update examples when behavior changes
- Add inline docs for complex logic
- Document public APIs with `///` comments

**Documentation Checklist:**
- [ ] README installation instructions work
- [ ] README examples reflect current behavior
- [ ] CHANGELOG follows Keep a Changelog format
- [ ] Public functions have doc comments
- [ ] Complex algorithms have explanatory comments

### 6. Clean Code Principles

- **No warnings**: Zero tolerance for compiler/clippy warnings
- **Meaningful names**: Variables, functions, types should be self-documenting
- **Small functions**: Each function does one thing well
- **Error handling**: Use `Result` and `Option` idiomatically
- **No panics**: Handle errors gracefully in user-facing code
- **Type safety**: Leverage Rust's type system to prevent bugs

## Development Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run -- <args>
cargo run -- --help

# Run with logging
RUST_LOG=debug cargo run -- /source /destination
RUST_LOG=dirloom=trace cargo run -- /source /destination

# Test
cargo test
cargo test --all-features
cargo test -- --nocapture

# Code Quality
cargo fmt
cargo fmt --check
cargo clippy
cargo clippy -- -D warnings

# Distribution
cargo dist build
cargo dist plan
```

## Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `clap` with derive macros for CLI arguments
- Use `ratatui` for TUI components
- Prefer composition over inheritance
- Use `newtype` patterns for type safety
- Implement `Debug`, `Clone`, `Copy` where appropriate
- Use `#[must_use]` for functions with side effects

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: resolve bug in backup logic
docs: update README examples
style: format code
refactor: improve error handling
test: add integration tests
chore: update dependencies
```

**Guidelines:**
- Subject line under 72 characters
- Use imperative mood ("add" not "added")
- Reference issues/PRs where applicable
- Include body for non-trivial changes

## Release Process

1. `git flow release start X.Y.Z`
2. Update `CHANGELOG.md` with version and date
3. Bump version in `Cargo.toml`
4. Run quality gate: `cargo fmt && cargo clippy -- -D warnings && cargo test`
5. Run `cargo dist build` to verify releases
6. `git flow release finish X.Y.Z` (creates tag)
7. `git push --follow-tags`
8. Publish to crates.io (if applicable)

## Project Structure

```
dirloom/
├── src/
│   ├── main.rs          # CLI entry point, tracing init
│   ├── lib.rs           # Library root
│   ├── cli.rs           # Clap argument definitions
│   ├── tui/             # TUI components
│   │   ├── mod.rs
│   │   ├── app.rs       # Application state & events
│   │   └── ui.rs        # UI rendering with ratatui
│   └── backup/          # Backup logic
│       ├── mod.rs
│       ├── error.rs     # Custom error types (thiserror)
│       ├── core.rs      # Core file operations
│       ├── sync.rs      # Sync strategies (parallel with rayon)
│       └── progress.rs  # Progress tracking
├── tests/
│   └── integration.rs   # Integration tests
├── data/                # Test data (gitignored)
│   ├── source/          # Test source files
│   └── target/          # Test backup target
├── Cargo.toml           # Dependencies and metadata
├── Cargo.lock           # Locked dependency versions
├── CHANGELOG.md         # Version history
├── README.md            # User documentation
├── CONTRIBUTING.md      # Contribution guidelines
├── LICENSE              # License file
├── QWEN.md              # This file (dev guide)
└── .github/
    └── workflows/
        └── release.yml  # CI/CD pipeline
```

## Quality Checklist

**Before marking any task complete:**

- [ ] Code compiles without warnings
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt` applied
- [ ] Unit tests written and passing
- [ ] Integration tests written and passing (if applicable)
- [ ] Documentation updated (README, CHANGELOG, inline docs)
- [ ] Examples tested and working
- [ ] No TODOs or FIXMEs left unresolved

## Questions & Clarifications

**Always ask when:**
- Requirements are ambiguous
- Trade-offs need discussion
- Edge cases are unclear
- Implementation approach has multiple valid options
- Error handling strategy is uncertain

**Better to ask early than refactor later.**

## Additional Best Practices

- **Dependency hygiene**: Keep dependencies updated, audit regularly (`cargo audit`)
- **Error messages**: User-facing errors should be helpful and actionable
- **Logging**: Use `tracing` for debuggable code
- **Benchmarks**: Add benchmarks for performance-critical code
- **Security**: Validate inputs, handle paths safely, no hardcoded secrets
