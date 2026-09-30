#!/usr/bin/env bash
# Bundle the desktop app. On Linux, linuxdeploy's bundled strip cannot parse
# RELR (.relr.dyn) sections produced by newer binutils, and appimagetool aborts
# on Fedora when ARCH is unset. See AGENTS.md.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if [ "$(uname -s)" = "Linux" ]; then
  : "${NO_STRIP:=1}"
  export NO_STRIP

  if [ -z "${ARCH:-}" ]; then
    case "$(uname -m)" in
      x86_64 | aarch64)
        ARCH="$(uname -m)"
        export ARCH
        ;;
      arm64)
        ARCH=aarch64
        export ARCH
        ;;
      *) ;;
    esac
  fi
fi

# ubuntu-24.04 CI bundles libwayland 1.22. Mesa 25+ (Fedora 44) then aborts in
# eglGetDisplay with EGL_BAD_PARAMETER. Host libwayland matches the running
# EGL stack; the shared-library name has been stable for years.
strip_bundled_libwayland() {
  local image="$1"
  local plugin="${XDG_CACHE_HOME:-$HOME/.cache}/tauri/linuxdeploy-plugin-appimage.AppImage"
  local work count backup

  if [ ! -f "$plugin" ]; then
    echo "tauri-build: $plugin missing; cannot drop bundled libwayland" >&2
    return 1
  fi

  work="$(mktemp -d)"
  chmod +x "$image"
  (
    cd "$work"
    "$image" --appimage-extract
  )
  count="$(find "$work/squashfs-root" -name 'libwayland-*.so*' -printf '.' | wc -c)"
  if [ "$count" -eq 0 ]; then
    rm -rf "$work"
    return 0
  fi

  find "$work/squashfs-root" -name 'libwayland-*.so*' -delete
  backup="${image}.before-wayland-strip"
  mv "$image" "$backup"
  if ! LDAI_OUTPUT="$image" LDAI_NO_APPSTREAM=1 APPIMAGE_EXTRACT_AND_RUN=1 \
    "$plugin" --appimage-extract-and-run --appdir="$work/squashfs-root"; then
    mv "$backup" "$image"
    rm -rf "$work"
    echo "tauri-build: failed to repack $(basename "$image") without libwayland" >&2
    return 1
  fi
  rm -f "$backup"
  rm -rf "$work"
  chmod +x "$image"
  echo "tauri-build: removed $count bundled libwayland objects from $(basename "$image")"
}

bun run tauri -- build "$@"

if [ "$(uname -s)" = "Linux" ]; then
  while IFS= read -r -d '' image; do
    strip_bundled_libwayland "$image"
  done < <(find "$root/src-tauri/target" -path '*/bundle/appimage/*.AppImage' -print0)
fi
