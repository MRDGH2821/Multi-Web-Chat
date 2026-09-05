#!/usr/bin/env bash
# Idempotent Cloud Agent bootstrap for Multi Web Chat (mise toolchain scaffold).
set -euo pipefail

export PATH="${HOME}/.local/bin:${PATH}"

if ! command -v mise >/dev/null 2>&1; then
  curl -fsSL https://mise.run | sh
fi

# shellcheck disable=SC1091
eval "$(mise activate bash)"

mise trust --yes >/dev/null 2>&1 || mise trust || true
mise install
mise run prepare
mise run ai-setup
