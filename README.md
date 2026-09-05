# Multi Web Chat

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)

Desktop app (Tauri v2) that embeds multiple AI chat websites and sends one
prompt to all enabled panes.

International AI panes (screenshot order): ChatGPT, Claude, Copilot,
Copilot (GH), Felo, Gemini, Genspark, Grok, Liner, Meta AI, Mistral,
Perplexity, Poe, Qwen Chat, Z.ai. All 15 enabled uses a 5+5+5 grid.

## Runtime dependencies

### Linux

Tauri webviews on Linux are backed by WebKitGTK, so the following system
packages must be installed before running `tauri:dev` or `tauri:build`. These
are the exact packages verified to make `cargo check` and `tauri build`
succeed on Ubuntu 24.04 (Tauri v2 Linux prerequisites,
<https://v2.tauri.app/start/prerequisites/#linux>):

On Debian/Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev \
  libssl-dev \
  librsvg2-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file
```

On Fedora:

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel gtk3-devel \
  librsvg2-devel libappindicator-gtk3-devel patchelf
```

Building an AppImage additionally requires a working FUSE mount
(`/dev/fuse` present, `fusermount3` setuid), since `tauri-bundler` downloads
and runs `linuxdeploy`/`appimagetool` at build time. If FUSE is unavailable
(common in containers), the deb and rpm bundles still build successfully.

### Windows

Webviews use the system WebView2 runtime (preinstalled on current Windows 10
and 11). The packaged installer downloads the Evergreen WebView2 bootstrapper
if the runtime is missing. Build from a Windows host (or the Windows GitHub
Actions runner) with the MSVC toolchain; this Linux workspace cannot emit
`.msi` / NSIS artifacts.

### macOS

Webviews use WKWebView. Build from a macOS host (or the macOS GitHub Actions
runner) for Apple Silicon and Intel. Unsigned local and CI builds will show a
Gatekeeper warning until the app is signed with an Apple Developer
certificate. This Linux workspace cannot emit `.dmg` / `.app` artifacts.

## Develop

```bash
mise install
bun install
bun run tauri:dev
```

## Build packages

```bash
bun run tauri:build
```

`src-tauri/tauri.conf.json` `bundle.targets` is
`["appimage", "deb", "dmg", "msi", "nsis", "rpm"]`. Each host builds only the
targets it can produce:

- Linux: `src-tauri/target/release/bundle/{appimage,deb,rpm}/`
- Windows: `src-tauri/target/release/bundle/{msi,nsis}/`
- macOS: `src-tauri/target/release/bundle/{dmg,macos}/`

Cross-platform installers are produced by `.github/workflows/tauri-build.yml`
(pull request, `main`, version tags, or `workflow_dispatch`). Download the
`multi-web-chat-windows-x64`, `multi-web-chat-macos-arm64`,
`multi-web-chat-macos-x64`, and `multi-web-chat-linux-x64` artifacts from the
workflow run.

## Recommendations

### Configuration directory

If this project is a tool, CLI, or library that reads its own configuration,
support resolving it from a project-level `.config/` directory
(e.g. `.config/multi-web-chat.toml`) alongside
any other locations you accept. It keeps consumers' repo roots tidy and follows
an emerging cross-ecosystem convention:

- <https://github.com/numtide/prj-spec> — project directory specification
- <https://dot-config.github.io/> — the `.config/` directory convention
- <https://github.com/pi0/config-dir> — reference implementation for resolving it
