#!/usr/bin/env bash
# Sync the canonical Press source into this repo as one-way, read-only mirrors.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE="$REPO_ROOT/../brand-assets/design-system"
DEST="$REPO_ROOT/src/styles/press"
BRAND_SVG="$REPO_ROOT/../brand-assets/svg"
FAVICON_DIR="$REPO_ROOT/../brand-assets/favicon"
APP_ICON_DIR="$REPO_ROOT/../brand-assets/app-icons"
APP_ICON="$APP_ICON_DIR/app-icon-1024.png"

if [ ! -d "$SOURCE" ]; then
  echo "error: canonical design system not found at $SOURCE" >&2
  echo "       clone brand-assets as a sibling of this repo, then re-run." >&2
  exit 1
fi

node "$SOURCE/generate-tokens.mjs" --check
mkdir -p "$DEST" "$REPO_ROOT/static/brand"

for file in tokens.css tokens.json components.css generate-tokens.mjs; do
  cp "$SOURCE/$file" "$DEST/$file"
  echo "synced src/styles/press/$file"
done

cp "$SOURCE/DESIGN_GUIDE.md" "$REPO_ROOT/DESIGN_GUIDE.md"
cp "$BRAND_SVG/kindling-mark.svg" "$REPO_ROOT/static/brand/kindling-mark.svg"
cp "$BRAND_SVG/kindling-mark-reversed.svg" "$REPO_ROOT/static/brand/kindling-mark-reversed.svg"
cp "$BRAND_SVG/kindling-favicon.svg" "$REPO_ROOT/static/brand/kindling-flame.svg"
cp "$BRAND_SVG/kindling-favicon-reversed.svg" "$REPO_ROOT/static/brand/kindling-flame-reversed.svg"
cp "$BRAND_SVG/kindling-favicon.svg" "$REPO_ROOT/static/favicon.svg"
cp "$BRAND_SVG/kindling-favicon.svg" "$REPO_ROOT/static/app-icon.svg"
cp "$FAVICON_DIR/favicon-32.png" "$REPO_ROOT/static/favicon.png"
echo "synced DESIGN_GUIDE.md, theme-aware brand marks, and favicon"

node "$REPO_ROOT/scripts/generate-press-theme.mjs"

if [ "${1:-}" = "--with-app-icons" ]; then
  npm run tauri -- icon "$APP_ICON" --output "$REPO_ROOT/src-tauri/icons"
  cp "$APP_ICON_DIR/kindling.icns" "$REPO_ROOT/src-tauri/icons/icon.icns"
  echo "regenerated Tauri icons from the canonical Press app icon"
fi

echo "done. edit canonical files in brand-assets, never the mirrors in this repo."
