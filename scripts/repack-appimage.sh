#!/usr/bin/env bash
# Repack a Tauri AppImage with the two fixes linuxdeploy leaves out:
#   1. drop the bundled libwayland-client.so.0 so the host's own is used
#   2. widen owner-only executable modes (tauri-apps/tauri#16155)
#
# WHY 1 — EGL inside the image has to talk to the host compositor
# linuxdeploy copies the build machine's libwayland-client into the AppImage, so
# on a host whose Mesa is newer than the build machine's wayland the bundled copy
# (built on Ubuntu 22.04) shadows the system one and the EGL display cannot be
# created:
#
#     Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
#
# WebKitWebProcess aborts on that line, so the GTK window never gets content and
# the app looks like it "does not start". libwayland-client.so.0 is on the
# AppImage excludelist for exactly this reason (mesa#11316, pkg2appimage#559),
# while the .deb/.rpm builds link the host library and are unaffected. Stripping
# it is necessary and sufficient: with the system copy in the search path EGL
# comes up on Mesa 25/26 hosts and the bundled WebKit renders normally.
#
# The host must provide libwayland-client.so.0. Every distro that ships the GTK 3
# runtime ships that library with it, so this drops a copy that breaks EGL in
# favour of one that is already present. The remaining bundled
# libwayland-{cursor,egl,server} stay: they reach the host client through a
# stable soname and dropping them would only add new hard dependencies.
#
# WHY 2 — permissions
# The Tauri bundler writes the AppImage launcher with mode 0770 and linuxdeploy
# keeps it as AppRun.wrapped (tauri-apps/tauri#16155). FUSE launches don't care,
# but a kernel mount of the squashfs (firejail --appimage, the
# appimage.github.io catalog test) refuses to exec it for anyone but root.
# scripts/check-appimage-perms.sh and scripts/check-appimage-wayland.sh are the
# matching gates; drop the corresponding half of this script once a Tauri CLI
# release produces an image that passes those checks on its own.
#
# HOW
# Extract the squashfs, remove the bundled libwayland-client, `chmod a+rx` each
# u+x entry that lacks o+rx, and repack with a pinned appimagetool using the
# original image's own runtime, so the type-2 runtime is byte-identical apart
# from the payload digest appimagetool writes into it. The AppImage is replaced
# in place; re-sign it afterwards.
#
# Usage: scripts/repack-appimage.sh <file.AppImage>
# Needs: unsquashfs, curl, sha256sum, and x86_64 Linux.
set -euo pipefail

APPIMAGETOOL_VERSION="1.9.1"
APPIMAGETOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"

appimage="$(realpath "${1:?usage: $0 <file.AppImage>}")"
[[ -x "$appimage" ]] || chmod +x "$appimage"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

tool="$work/appimagetool.AppImage"
curl -fsSL -o "$tool" \
  "https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-x86_64.AppImage"
echo "${APPIMAGETOOL_SHA256}  $tool" | sha256sum -c --quiet
chmod +x "$tool"

offset="$("$appimage" --appimage-offset)"
runtime_version="$("$appimage" --appimage-version 2>&1)"
updateinfo="$("$appimage" --appimage-updateinfo)"
head -c "$offset" "$appimage" >"$work/runtime"

unsquashfs -q -no-xattrs -d "$work/AppDir" -o "$offset" "$appimage" >/dev/null

echo "Dropping the bundled libwayland-client (the host Mesa needs its own):"
mapfile -t wayland_client < <(find "$work/AppDir" -name 'libwayland-client.so*' -printf '%P\n')
if (( ${#wayland_client[@]} == 0 )); then
  echo "  already absent"
else
  printf '  %s\n' "${wayland_client[@]}"
  find "$work/AppDir" -name 'libwayland-client.so*' -delete
fi

echo "Widening owner-only executable modes:"
find "$work/AppDir" ! -type l -perm -u+x \( ! -perm -o+r -o ! -perm -o+x \) \
  -printf '  %M %P\n' -exec chmod a+rx {} +

# A clean environment: no GitHub variables for appimagetool to read update
# information from, and no FUSE needed to run it on the runner.
args=(--no-appstream --comp zstd --runtime-file "$work/runtime")
[[ -n "$updateinfo" ]] && args+=(--updateinformation "$updateinfo")
env -i PATH="$PATH" HOME="$work" ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 \
  "$tool" "${args[@]}" "$work/AppDir" "$work/out.AppImage"

new_offset="$("$work/out.AppImage" --appimage-offset)"
new_runtime_version="$("$work/out.AppImage" --appimage-version 2>&1)"
if [[ "$new_offset" != "$offset" || "$new_runtime_version" != "$runtime_version" ]]; then
  echo "::error::Repacked AppImage has a different runtime (offset $offset -> $new_offset; '$runtime_version' -> '$new_runtime_version')"
  exit 1
fi

mv "$work/out.AppImage" "$appimage"
chmod 0755 "$appimage"
echo "Repacked $(basename "$appimage") with its original runtime ($runtime_version)."
