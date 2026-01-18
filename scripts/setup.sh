#!/bin/bash
#
# Kindling Development Environment Setup
#
# This script automates the setup process for new contributors.
# Run with: ./scripts/setup.sh
#
# What it does:
#   1. Checks for required tools (Node.js, Rust, platform deps)
#   2. Installs npm dependencies
#   3. Verifies Rust toolchain
#   4. Runs initial build to verify setup
#   5. Runs tests to ensure everything works
#

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

# Header
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Kindling Development Environment Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
fi
info "Detected OS: $OS"

# Check Node.js
echo ""
info "Checking Node.js..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version | sed 's/v//')
    NODE_MAJOR=$(echo "$NODE_VERSION" | cut -d. -f1)
    if [ "$NODE_MAJOR" -ge 20 ]; then
        success "Node.js $NODE_VERSION (meets requirement: 20+)"
    else
        error "Node.js $NODE_VERSION found, but version 20+ is required"
        echo "  Install from: https://nodejs.org/"
        exit 1
    fi
else
    error "Node.js not found"
    echo "  Install from: https://nodejs.org/"
    exit 1
fi

# Check npm
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    success "npm $NPM_VERSION"
else
    error "npm not found"
    exit 1
fi

# Check Rust
echo ""
info "Checking Rust..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    success "Rust $RUST_VERSION"
else
    error "Rust not found"
    echo "  Install from: https://rustup.rs/"
    exit 1
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version | cut -d' ' -f2)
    success "Cargo $CARGO_VERSION"
else
    error "Cargo not found"
    exit 1
fi

# Platform-specific checks
echo ""
info "Checking platform dependencies..."
case $OS in
    macos)
        if xcode-select -p &> /dev/null; then
            success "Xcode Command Line Tools installed"
        else
            warn "Xcode Command Line Tools not found"
            echo "  Run: xcode-select --install"
            exit 1
        fi
        ;;
    linux)
        # Check for common Tauri dependencies on Linux
        MISSING_DEPS=""
        for pkg in libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev; do
            if ! dpkg -s "$pkg" &> /dev/null 2>&1; then
                MISSING_DEPS="$MISSING_DEPS $pkg"
            fi
        done
        if [ -n "$MISSING_DEPS" ]; then
            warn "Missing Linux dependencies:$MISSING_DEPS"
            echo "  Run: sudo apt-get install -y$MISSING_DEPS"
        else
            success "Linux dependencies installed"
        fi
        ;;
    windows)
        info "Windows detected - ensure you have:"
        echo "  - Visual Studio Build Tools"
        echo "  - WebView2 Runtime"
        ;;
esac

# Install npm dependencies
echo ""
info "Installing npm dependencies..."
npm install
success "npm dependencies installed"

# Install Rust dependencies (cargo will do this automatically on build)
echo ""
info "Checking Rust dependencies..."
cd src-tauri
cargo check --quiet 2>/dev/null || cargo check
cd ..
success "Rust dependencies verified"

# Run linting
echo ""
info "Running linters..."
npm run lint --silent && success "ESLint passed" || warn "ESLint found issues (run: npm run lint:fix)"
npm run lint:rust --silent 2>/dev/null && success "Clippy passed" || warn "Clippy found issues"

# Run tests
echo ""
info "Running tests..."
npm test --silent && success "Frontend tests passed" || warn "Some frontend tests failed"
npm run test:rust --silent 2>/dev/null && success "Rust tests passed" || warn "Some Rust tests failed"

# Git hooks setup
echo ""
info "Setting up git hooks..."
if [ -d ".githooks" ]; then
    git config core.hooksPath .githooks
    success "Git hooks configured (commitlint will validate commit messages)"
else
    warn "Git hooks directory not found"
fi

# VS Code setup
echo ""
if [ -d ".vscode-example" ]; then
    if [ -d ".vscode" ]; then
        info "VS Code config already exists at .vscode/"
    else
        info "Setting up VS Code configuration..."
        cp -r .vscode-example/. .vscode/
        success "VS Code config copied to .vscode/"
    fi
fi

# Final summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Setup Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
success "Your development environment is ready!"
echo ""
echo "Next steps:"
echo "  1. Start the app:        npm run tauri dev"
echo "  2. Run tests:            npm run test:all"
echo "  3. Check code:           npm run check:all"
echo ""
echo "VS Code users:"
echo "  - Open the project in VS Code"
echo "  - Install recommended extensions when prompted"
echo "  - Use Cmd/Ctrl+Shift+P > 'Tasks: Run Task' for common operations"
echo "  - See .vscode-example/README.md for full documentation"
echo ""
echo "See CONTRIBUTING.md for more details."
echo ""
