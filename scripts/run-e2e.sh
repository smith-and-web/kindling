#!/bin/bash

# E2E Test Runner for Kindling
# This script helps developers run E2E tests with proper setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Kindling E2E Test Runner                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}⚠️  macOS does not support local E2E testing${NC}"
    echo ""
    echo "   WebDriver for WKWebView is not available on macOS."
    echo "   E2E tests run automatically in CI on Linux."
    echo ""
    echo "   Options:"
    echo "   - Push your changes and let CI run the tests"
    echo "   - Use a Linux VM or container"
    echo ""
    exit 0
fi

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"
echo ""

ERRORS=0

# Check for tauri-driver
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
if ! command -v tauri-driver &> /dev/null && [ ! -f "$CARGO_BIN/tauri-driver" ]; then
    echo -e "${RED}❌ tauri-driver not found${NC}"
    echo "   Install with: cargo install tauri-driver"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}✅ tauri-driver found${NC}"
fi

# Check for Tauri binary
BINARY_PATH=""
if [ -f "$PROJECT_ROOT/src-tauri/target/release/kindling" ]; then
    BINARY_PATH="$PROJECT_ROOT/src-tauri/target/release/kindling"
    echo -e "${GREEN}✅ Tauri binary found (release)${NC}"
elif [ -f "$PROJECT_ROOT/src-tauri/target/debug/kindling" ]; then
    BINARY_PATH="$PROJECT_ROOT/src-tauri/target/debug/kindling"
    echo -e "${GREEN}✅ Tauri binary found (debug)${NC}"
else
    echo -e "${RED}❌ Tauri binary not found${NC}"
    echo "   Build with: npm run tauri build"
    echo "   Or for faster builds: npm run tauri build -- --debug"
    ERRORS=$((ERRORS + 1))
fi

# Check for E2E dependencies
if [ ! -d "$PROJECT_ROOT/e2e/node_modules" ]; then
    echo -e "${YELLOW}⚠️  E2E dependencies not installed${NC}"
    echo "   Installing now..."
    cd "$PROJECT_ROOT/e2e" && npm ci
    echo -e "${GREEN}✅ E2E dependencies installed${NC}"
else
    echo -e "${GREEN}✅ E2E dependencies installed${NC}"
fi

# Check for test data
if [ ! -d "$PROJECT_ROOT/test-data" ]; then
    echo -e "${RED}❌ test-data directory not found${NC}"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}✅ test-data directory found${NC}"
fi

echo ""

# Exit if there are errors
if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}Prerequisites not met. Please fix the issues above.${NC}"
    exit 1
fi

# Check if we need Xvfb (headless Linux)
NEED_XVFB=false
if [[ "$OSTYPE" == "linux"* ]] && [ -z "$DISPLAY" ]; then
    NEED_XVFB=true
fi

# Parse arguments
HEADLESS=false
DEBUG=false
SPEC=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --headless|-h)
            HEADLESS=true
            shift
            ;;
        --debug|-d)
            DEBUG=true
            shift
            ;;
        --spec|-s)
            SPEC="$2"
            shift 2
            ;;
        *)
            echo -e "${YELLOW}Unknown option: $1${NC}"
            shift
            ;;
    esac
done

# Run the tests
echo -e "${BLUE}Running E2E tests...${NC}"
echo ""

cd "$PROJECT_ROOT/e2e"

# Build the test command
TEST_CMD="npm test"

if [ "$HEADLESS" = true ] || [ "$NEED_XVFB" = true ]; then
    # Start Xvfb for headless mode
    if command -v Xvfb &> /dev/null; then
        echo "Starting Xvfb for headless display..."
        Xvfb :99 -screen 0 1920x1080x24 &
        XVFB_PID=$!
        export DISPLAY=:99
        sleep 1

        # Cleanup Xvfb on exit
        trap "kill $XVFB_PID 2>/dev/null" EXIT
    else
        echo -e "${YELLOW}⚠️  Xvfb not installed. Tests may fail without a display.${NC}"
        echo "   Install with: sudo apt-get install xvfb"
    fi
fi

if [ "$DEBUG" = true ]; then
    TEST_CMD="$TEST_CMD -- --logLevel=debug"
fi

if [ -n "$SPEC" ]; then
    TEST_CMD="$TEST_CMD -- --spec $SPEC"
fi

# Run tests
eval $TEST_CMD
TEST_EXIT_CODE=$?

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  ✅ All E2E tests passed!                              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
else
    echo -e "${RED}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ Some E2E tests failed                               ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Check e2e/screenshots/ for failure screenshots"
fi

exit $TEST_EXIT_CODE
