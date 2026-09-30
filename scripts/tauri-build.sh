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

exec bun run tauri -- build "$@"
