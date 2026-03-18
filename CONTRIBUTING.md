# Contributing to Dirloom

Thank you for considering contributing to Dirloom! This document outlines the process for contributing to this project.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all backgrounds and experience levels.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/dirloom.git`
3. Add the upstream remote: `git remote add upstream https://github.com/jchultarsky101/dirloom.git`
4. Initialize Git Flow: `git flow init -d`

## Development Workflow

This project uses [Git Flow](https://nvie.com/posts/a-successful-git-branching-model/):

### Feature Development

```bash
# Start a new feature
git flow feature start <feature-name>

# Make your changes and commit
git add .
git commit -m "feat: add new feature"

# Finish the feature
git flow feature finish <feature-name>
```

### Bug Fixes

```bash
# For development fixes
git flow feature start fix-<issue-number>

# For production hotfixes
git flow hotfix start <fix-name>
```

## Making Changes

1. **Create an issue** for bugs or feature requests
2. **Create a branch** using Git Flow
3. **Make your changes** following code style guidelines
4. **Write tests** for new functionality
5. **Run checks** before submitting:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```

## Commit Messages

We use conventional commits:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Test additions or modifications
- `chore:` - Maintenance tasks

Example: `feat: add progress bar to backup operation`

## Pull Requests

1. Ensure your branch is up to date: `git flow feature finish <name>`
2. Push your branch to your fork
3. Open a Pull Request against `develop`
4. Describe your changes and reference any related issues
5. Respond to review feedback

## Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Address all `clippy` warnings
- Write documentation comments for public items
- Keep functions focused and small

## Questions?

Feel free to open an issue with the `question` label for any questions about contributing.
