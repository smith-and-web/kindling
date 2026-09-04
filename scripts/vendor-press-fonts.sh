#!/usr/bin/env bash
# Vendor Press fonts from pinned Fontsource packages. The built app makes no
# runtime font request and remains fully offline.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$REPO_ROOT/static/fonts"
LICENSE_DEST="$DEST/licenses"

mkdir -p "$DEST" "$LICENSE_DEST"
cp "$REPO_ROOT/node_modules/@fontsource-variable/fraunces/files/fraunces-latin-opsz-normal.woff2" "$DEST/Fraunces-Variable-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource-variable/newsreader/files/newsreader-latin-opsz-normal.woff2" "$DEST/Newsreader-Variable-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource-variable/newsreader/files/newsreader-latin-opsz-italic.woff2" "$DEST/Newsreader-Variable-Italic-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource/inter/files/inter-latin-400-normal.woff2" "$DEST/Inter-Regular-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource/inter/files/inter-latin-500-normal.woff2" "$DEST/Inter-Medium-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource/inter/files/inter-latin-600-normal.woff2" "$DEST/Inter-SemiBold-latin.woff2"
cp "$REPO_ROOT/node_modules/@fontsource-variable/fraunces/LICENSE" "$LICENSE_DEST/Fraunces-OFL-1.1.txt"
cp "$REPO_ROOT/node_modules/@fontsource-variable/newsreader/LICENSE" "$LICENSE_DEST/Newsreader-OFL-1.1.txt"
cp "$REPO_ROOT/node_modules/@fontsource/inter/LICENSE" "$LICENSE_DEST/Inter-OFL-1.1.txt"

echo "vendored Press fonts and licenses into static/fonts/"
