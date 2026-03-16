#!/usr/bin/env bash
# Clean build caches and run Tauri dev - use when project was moved or caches are stale
set -e
cd "$(dirname "$0")/.."

echo "Cleaning caches..."
# 1. Cargo/Rust: removes compiled artifacts (contains absolute paths)
(cd src-tauri && cargo clean)

# 2. Vite: removes pre-bundled dependency cache
rm -rf node_modules/.vite 2>/dev/null || true

# 3. Optional: dist folder from previous builds
rm -rf dist 2>/dev/null || true

echo "Running tauri dev..."
npx tauri dev
