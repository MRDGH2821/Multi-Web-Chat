# Multi Web Chat

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)

Linux desktop app (Tauri v2) that embeds multiple AI chat websites and sends
one prompt to all enabled panes.

## Runtime dependencies (Linux)

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

`src-tauri/tauri.conf.json` restricts `bundle.targets` to
`["appimage", "deb", "rpm"]` (Linux only; no `msi`/`dmg`). Artifacts are
written under `src-tauri/target/release/bundle/{appimage,deb,rpm}/`.

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
