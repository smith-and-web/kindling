# Kindling E2E Tests

End-to-end tests for Kindling using [WebdriverIO](https://webdriver.io/) and [Tauri's WebDriver support](https://v2.tauri.app/develop/tests/webdriver/).

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Supported | Uses `WebKitWebDriver` |
| Windows | ✅ Supported | Uses `msedgedriver` |
| macOS | ❌ Not supported | No WKWebView WebDriver |

**Note:** macOS does not have a WebDriver for WKWebView, so E2E tests cannot run locally on macOS. Tests run in CI on Linux instead.

## Running Tests

### In CI (Recommended)

E2E tests run automatically in GitHub Actions on Linux runners. See `.github/workflows/e2e.yml`.

### Locally on Linux

```bash
# From project root
cd e2e
npm install

# Build the app first
cd ..
npm run tauri build

# Run tests
cd e2e
npm test
```

### Locally on Windows

```powershell
# Install msedgedriver (must match your Edge version)
# Download from: https://developer.microsoft.com/microsoft-edge/tools/webdriver/

cd e2e
npm install

# Build the app
cd ..
npm run tauri build

# Run tests
cd e2e
npm test
```

## Test Structure

```
e2e/
├── package.json       # WebdriverIO dependencies
├── wdio.conf.js       # WebdriverIO configuration
├── specs/
│   ├── helpers.js     # Shared test utilities
│   ├── app-launch.spec.js      # Basic smoke tests
│   ├── beat-editing.spec.js    # Feature #38 tests
│   ├── create-content.spec.js  # Feature #15 tests
│   ├── drag-drop.spec.js       # Feature #14 tests
│   ├── delete-content.spec.js  # Feature #16 tests
│   └── reimport.spec.js        # Feature #40 tests
└── README.md
```

## Writing Tests

### Selectors

Tests use `data-testid` attributes for reliable element selection. Add these to your Svelte components:

```svelte
<button data-testid="new-chapter-button">+ New Chapter</button>
```

### Common Patterns

```javascript
import { waitForAppReady, selectChapter } from "./helpers.js";

describe("My Feature", () => {
  beforeEach(async () => {
    await waitForAppReady();
  });

  it("should do something", async () => {
    const button = await $('[data-testid="my-button"]');
    await button.click();
    expect(await button.isExisting()).toBe(true);
  });
});
```

## Required Test IDs

For tests to work, add these `data-testid` attributes to components:

### Sidebar
- `sidebar` - Main sidebar container
- `recent-projects` - Recent projects section
- `import-section` - Import options area
- `chapter-item` - Chapter row
- `chapter-title` - Chapter title text
- `scene-item` - Scene row
- `scene-title` - Scene title text
- `new-chapter-button` - "+ New Chapter" button
- `new-scene-button` - "+ New Scene" button
- `title-input` - Inline title input for creating items
- `drag-handle` - Drag grip icon
- `delete-button` - Trash icon button
- `reimport-button` - Refresh/reimport button

### Scene Panel
- `scene-panel` - Main scene content area
- `beat-header` - Clickable beat header
- `beat-prose-textarea` - Prose editing textarea
- `save-indicator` - "Saving..." / "Saved" indicator
- `empty-state` - Empty/welcome state

### Dialogs
- `confirm-dialog` - Confirmation dialog container
- `dialog-message` - Dialog message text
- `dialog-confirm` - Confirm/Delete button
- `dialog-cancel` - Cancel button
- `dialog-close` - Close button (X)
- `reimport-summary-dialog` - Reimport results dialog
- `reimport-summary` - Summary text content
- `reimport-spinner` - Loading spinner

## Debugging

### View test output
```bash
npm test -- --logLevel=debug
```

### Take screenshots on failure
Screenshots are automatically saved to `e2e/screenshots/` when tests fail.

### Interactive debugging
```bash
# On Linux with a display
npm test -- --watch
```

## Prerequisites

Before running E2E tests locally, ensure you have:

### Linux

```bash
# Install WebKit driver and display dependencies
sudo apt-get install -y webkit2gtk-driver xvfb

# Install tauri-driver (Rust WebDriver server for Tauri)
cargo install tauri-driver
```

### Windows

```powershell
# Install msedgedriver (must match your Edge version)
# Download from: https://developer.microsoft.com/microsoft-edge/tools/webdriver/

# Install tauri-driver
cargo install tauri-driver
```

## Troubleshooting

### "Could not find Tauri binary"

You need to build the Tauri app before running E2E tests:

```bash
npm run tauri build
```

For faster iteration during development, you can use debug builds:

```bash
npm run tauri build -- --debug
```

### "tauri-driver not found"

Install the tauri-driver:

```bash
cargo install tauri-driver
```

Make sure `~/.cargo/bin` is in your PATH.

### Tests hang or timeout

1. **On Linux without a display**: Use Xvfb for a virtual display:
   ```bash
   Xvfb :99 -screen 0 1920x1080x24 &
   export DISPLAY=:99
   npm test
   ```

2. **Connection refused errors**: Ensure no other tauri-driver instance is running on port 4444:
   ```bash
   lsof -i :4444  # Check what's using the port
   ```

### "macOS does not support WebDriver testing"

This is expected. macOS lacks a WebDriver for WKWebView. Run E2E tests:
- In CI (automatically runs on Linux)
- On a Linux machine or VM
- Using Docker (experimental)

### Test flakiness

If tests are flaky, try:
1. Increase timeouts in `wdio.conf.js`
2. Add explicit waits using `browser.waitUntil()`
3. Ensure unique `data-testid` attributes

## CI Configuration

The E2E workflow (`.github/workflows/e2e.yml`) does:

1. Builds the Tauri app in release mode
2. Installs `webkit2gtk-driver` and `tauri-driver`
3. Starts Xvfb for headless display
4. Runs WebdriverIO tests
5. Uploads screenshots and logs as artifacts

Tests run on every push to `main` and on pull requests.

## Running from Project Root

You can also run E2E tests from the project root using convenience scripts:

```bash
# From project root (after building)
npm run test:e2e

# For CI/headless mode
npm run test:e2e:ci
```
