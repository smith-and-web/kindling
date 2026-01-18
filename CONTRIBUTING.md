# Contributing to Kindling

First off, thank you for considering contributing to Kindling! It's people like you that make Kindling such a great tool for fiction writers.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Style Guidelines](#style-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

- Make sure you have a [GitHub account](https://github.com/signup)
- Check the [issues](https://github.com/smith-and-web/kindling/issues) for existing bugs or feature requests
- Check the [discussions](https://github.com/smith-and-web/kindling/discussions) for questions and ideas

### Good First Issues

Looking for a place to start? Check out issues labeled [`good first issue`](https://github.com/smith-and-web/kindling/labels/good%20first%20issue) - these are great for newcomers to the project.

## How Can I Contribute?

### Reporting Bugs

- **Check existing issues** first to avoid duplicates
- Use the [bug report template](https://github.com/smith-and-web/kindling/issues/new?template=bug_report.yml)
- Include as much detail as possible
- Include steps to reproduce the issue

### Suggesting Features

- Check the [roadmap](https://github.com/smith-and-web/kindling#roadmap) first
- Start a [discussion](https://github.com/smith-and-web/kindling/discussions/categories/ideas) to gauge interest
- Use the [feature request template](https://github.com/smith-and-web/kindling/issues/new?template=feature_request.yml)

### Writing Code

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes (see [Commit Messages](#commit-messages))
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Writing Documentation

Documentation improvements are always welcome! This includes:
- README improvements
- Code comments
- Wiki pages
- Tutorials and guides

## Development Setup

### Quick Start (Recommended)

We provide a setup script that automates the entire process:

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/kindling.git
cd kindling

# Run the setup script
./scripts/setup.sh
```

The script will:
- Check for required tools (Node.js 20+, Rust)
- Verify platform-specific dependencies
- Install npm and Rust dependencies
- Run linters and tests to verify your setup

### Manual Setup

If you prefer manual setup or the script doesn't work for your system:

#### Prerequisites

- **Node.js** 20+
- **Rust** (stable toolchain)
- **Platform-specific dependencies** (see below)

#### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

#### Install and Run

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/kindling.git
cd kindling

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### VS Code Setup

We recommend VS Code for development. The project includes pre-configured settings in `.vscode-example/`:

```bash
# Copy the example configs to your local .vscode folder
cp -r .vscode-example/. .vscode/
```

This gives you:
- **Recommended extensions** - VS Code will prompt to install them
- **Auto-formatting** on save for TypeScript, Svelte, and Rust
- **Tasks** (Cmd/Ctrl+Shift+P > "Tasks: Run Task") for common operations:
  - `Dev: Start Application` - Start the full Tauri app
  - `Test: All` - Run all tests
  - `Check: Full CI` - Run everything CI would check
  - `Lint: Fix All` - Auto-fix linting issues
- **Debug configurations** for Rust and frontend debugging

See `.vscode-example/README.md` for full documentation of each config file.

### Running Tests

```bash
# All tests (frontend + Rust)
npm run test:all

# Frontend tests only
npm test

# Frontend tests in watch mode
npm run test:watch

# Frontend test coverage
npm run test:coverage

# Rust tests only
npm run test:rust

# E2E tests (Linux/Windows only - requires built app)
npm run tauri build
npm run test:e2e
```

### E2E Testing

E2E tests use WebdriverIO with Tauri's WebDriver support. See [`e2e/README.md`](./e2e/README.md) for detailed documentation.

**Important notes:**
- E2E tests require a built Tauri binary (`npm run tauri build`)
- macOS does not support local E2E testing (no WKWebView WebDriver)
- E2E tests run automatically in CI on Linux

```bash
# Quick E2E setup on Linux
sudo apt-get install -y webkit2gtk-driver xvfb
cargo install tauri-driver
npm run tauri build
npm run test:e2e
```

### Linting & Formatting

```bash
# Check everything (like CI does)
npm run check:all

# Lint everything (frontend + Rust)
npm run lint:all

# Fix all auto-fixable issues
npm run lint:fix && npm run format:rust

# Format everything
npm run format:all

# Frontend linting only
npm run lint

# Rust linting only
cd src-tauri && cargo clippy
```

## Style Guidelines

### TypeScript/Svelte

- We use ESLint and Prettier for code formatting
- Run `npm run lint:fix` to auto-fix issues
- Run `npm run format` to format code

### Rust

- We use `rustfmt` for formatting
- We use `clippy` for linting
- Run `cargo fmt` to format code
- Run `cargo clippy` to check for issues

### General

- Use meaningful variable and function names
- Write self-documenting code where possible
- Add comments for complex logic
- Keep functions focused and small

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code changes that neither fix bugs nor add features
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

### Examples

```
feat(import): add support for Scrivener 3 projects

fix(editor): resolve cursor position issue on paste

docs(readme): update installation instructions

chore(deps): update Tauri to 2.1.0
```

## Pull Request Process

1. **Update documentation** if you're changing behavior
2. **Add tests** for new functionality
3. **Ensure CI passes** - all checks must be green
4. **Request review** from maintainers
5. **Address feedback** promptly
6. **Squash commits** if requested

### PR Title

Use the same format as commit messages:

```
feat(import): add support for Scrivener 3 projects
```

### Review Process

- PRs require at least 1 approving review
- Maintainers may request changes
- Be patient - we review PRs as quickly as possible

## Recognition

Contributors are recognized in:
- GitHub's contributor graph
- Release notes for significant contributions
- The project README (for major contributors)

## Questions?

- Start a [discussion](https://github.com/smith-and-web/kindling/discussions)
- Check the [wiki](https://github.com/smith-and-web/kindling/wiki)

Thank you for contributing to Kindling!
