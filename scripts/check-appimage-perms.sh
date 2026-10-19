#!/usr/bin/env bash
# Fail if any entry in an AppImage's squashfs is executable by its owner but
# not by everyone else.
#
# WHY
# The normal launch path (the runtime's FUSE mount) presents every file as
# owned by the user who started the AppImage, so owner-only modes go unnoticed.
# Anything that kernel-mounts the squashfs instead sees the stored root:root
# ownership: `firejail --appimage`, and the appimage.github.io catalog test that
# runs in it. There a 0770 launcher fails with "AppRun.wrapped: Permission
# denied" (Tauri bundler, tauri-apps/tauri#16155).
#
# Every u+x entry must also be o+rx: a directory needs o+x to be traversed, an
# ELF or script needs o+r as well as o+x to be executed.
#
# Usage: scripts/check-appimage-perms.sh <file.AppImage>
# Needs unsquashfs (squashfs-tools). Runs the AppImage only to ask its runtime
# for the payload offset.
set -euo pipefail

appimage="$(realpath "${1:?usage: $0 <file.AppImage>}")"
[[ -x "$appimage" ]] || chmod +x "$appimage"

offset="$("$appimage" --appimage-offset)"

# -lln: long listing with numeric uid/gid. First field is the mode string,
# e.g. -rwxrwx---. Owner x is char 4, other r and x are chars 8 and 10.
# Symlinks are always lrwxrwxrwx and are skipped.
bad="$(unsquashfs -lln -o "$offset" "$appimage" |
  awk 'substr($1,1,1) != "l" && substr($1,4,1) ~ /[xs]/ &&
       (substr($1,8,1) != "r" || substr($1,10,1) !~ /[xt]/)')"

if [[ -n "$bad" ]]; then
  echo "::error::$(basename "$appimage") has entries executable by owner but not by others:"
  echo "$bad"
  exit 1
fi

echo "$(basename "$appimage"): every owner-executable entry is also o+rx."
