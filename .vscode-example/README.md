# VS Code Configuration

This folder contains recommended VS Code settings for Kindling development.

## Quick Setup

Copy these files to your `.vscode/` folder:

```bash
# From the project root
cp -r .vscode-example/. .vscode/
```

Or manually copy individual files as needed.

## What's Included

### `extensions.json`
Recommended extensions for the project:

| Extension | Purpose |
|-----------|---------|
| **svelte.svelte-vscode** | Svelte language support, syntax highlighting, intellisense |
| **rust-lang.rust-analyzer** | Rust language support, code completion, inline errors |
| **tauri-apps.tauri-vscode** | Tauri-specific commands and snippets |
| **esbenp.prettier-vscode** | Code formatting for TypeScript/Svelte |
| **dbaeumer.vscode-eslint** | Linting for TypeScript/Svelte |
| **bradlc.vscode-tailwindcss** | Tailwind CSS intellisense |
| **streetsidesoftware.code-spell-checker** | Catch typos in code and comments |
| **usernamehw.errorlens** | Inline error/warning display |
| **eamodio.gitlens** | Enhanced Git integration |

VS Code will prompt you to install these when you open the project.

### `settings.json`
Project-specific editor settings:

- **Auto-format on save** for TypeScript, Svelte, and Rust
- **ESLint auto-fix** on save
- **Rust Analyzer** configured to use Clippy
- **Tailwind CSS** class recognition in Svelte files
- **Search exclusions** for faster searches (ignores node_modules, target, etc.)
- **File nesting** for cleaner explorer (groups related config files)

### `tasks.json`
Pre-configured tasks accessible via `Cmd/Ctrl+Shift+P` → "Tasks: Run Task":

| Task | Description | Shortcut |
|------|-------------|----------|
| **Dev: Start Application** | Run full Tauri app | `Cmd+Shift+B` (default build) |
| **Dev: Frontend Only** | Run Vite dev server only | |
| **Test: All** | Run all tests (frontend + Rust) | |
| **Test: Frontend** | Run frontend tests only | |
| **Test: Frontend (Watch)** | Run frontend tests in watch mode | |
| **Test: Rust** | Run Rust tests only | |
| **Lint: All** | Run all linters | |
| **Lint: Fix All** | Auto-fix linting issues | |
| **Format: All** | Format all code | |
| **Check: Full CI** | Run everything CI checks | |
| **Build: Production** | Create production build | |
| **Rust: Check/Build/Clippy** | Rust-specific commands | |

### `launch.json`
Debug configurations for the Run and Debug panel (`Cmd+Shift+D`):

| Configuration | Purpose |
|---------------|---------|
| **Tauri Development** | Debug the full application (requires CodeLLDB extension) |
| **Rust Backend Debug** | Debug just the Rust backend |
| **Debug Rust Tests** | Debug Rust tests |
| **Attach to Frontend DevTools** | Attach to Chrome DevTools for frontend debugging |

**Note:** Rust debugging requires the [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension.

## Customization

These are starting points. Feel free to modify them for your workflow:

- Add your own keyboard shortcuts in `keybindings.json`
- Adjust formatting preferences in `settings.json`
- Add custom tasks for your workflow in `tasks.json`

Your `.vscode/` folder is gitignored, so your customizations won't affect others.

## Troubleshooting

### Rust Analyzer not working
1. Ensure Rust is installed: `rustc --version`
2. Reload VS Code window: `Cmd+Shift+P` → "Developer: Reload Window"
3. Check Output panel → "Rust Analyzer" for errors

### Svelte intellisense not working
1. Ensure the Svelte extension is installed
2. Check that `svelte.enable-ts-plugin` is `true` in settings
3. Restart the TypeScript server: `Cmd+Shift+P` → "TypeScript: Restart TS Server"

### Tasks not appearing
1. Ensure `tasks.json` is in `.vscode/` folder
2. Run `Cmd+Shift+P` → "Tasks: Run Task" to see available tasks
