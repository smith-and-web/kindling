#!/usr/bin/env bash
# Repack an AppImage so every owner-executable entry is also world-executable.
#
# WHY
# The Tauri bundler writes the AppImage launcher with mode 0770 and linuxdeploy
# keeps it as AppRun.wrapped (tauri-apps/tauri#16155). FUSE launches don't care,
# but a kernel mount of the squashfs (firejail --appimage, the
# appimage.github.io catalog test) refuses to exec it for anyone but root.
# scripts/check-appimage-perms.sh is the matching gate; drop this step once a
# Tauri CLI release produces an image that passes the check on its own.
#
# HOW
# Extract the squashfs, `chmod a+rx` each u+x entry that lacks o+rx, and repack
# with a pinned appimagetool using the original image's own runtime, so the
# type-2 runtime is byte-identical apart from the payload digest appimagetool
# writes into it. The AppImage is replaced in place; re-sign it afterwards.
#
# Usage: scripts/fix-appimage-perms.sh <file.AppImage>
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
