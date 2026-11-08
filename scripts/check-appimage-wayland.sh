#!/usr/bin/env bash
# Fail if an AppImage still bundles libwayland-client.so.0.
#
# WHY
# The copy linuxdeploy picks up from the build machine is older than the wayland
# the host's Mesa links against, and it shadows the system library: eglGetDisplay
# then fails with EGL_BAD_PARAMETER, WebKitWebProcess aborts, and the window stays
# empty. libwayland-client.so.0 is on the AppImage excludelist for this reason
# (mesa#11316, pkg2appimage#559). scripts/repack-appimage.sh removes it; this gate
# catches it coming back through a Tauri CLI change, a wayland upgrade on the
# runner, or a hand-made bundle.
#
# Usage: scripts/check-appimage-wayland.sh <file.AppImage>
# Needs unsquashfs (squashfs-tools). Runs the AppImage only to ask its runtime
# for the payload offset.
set -euo pipefail

appimage="$(realpath "${1:?usage: $0 <file.AppImage>}")"
[[ -x "$appimage" ]] || chmod +x "$appimage"

offset="$("$appimage" --appimage-offset)"

# Match on the basename so a versioned copy (libwayland-client.so.0.20.0) and its
# symlink are both caught. Symlinks are included on purpose: the link alone still
# resolves inside the bundle.
bad="$(unsquashfs -lln -o "$offset" "$appimage" |
  awk '{ n = $NF; sub(/.*\//, "", n); if (n ~ /^libwayland-client\.so/) print }')"

if [[ -n "$bad" ]]; then
  echo "::error::$(basename "$appimage") still bundles libwayland-client, which shadows the host library and breaks EGL on newer Mesa hosts:"
  echo "$bad"
  exit 1
fi

echo "$(basename "$appimage"): no bundled libwayland-client, EGL can use the host's."
