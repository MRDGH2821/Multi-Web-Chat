# multi-web-chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Linux-only Tauri v2 + Svelte desktop app that embeds seven AI chat sites in child webviews, fans one shared prompt to all enabled panes in parallel, and persists per-provider logins and pane toggles.

**Architecture:** One main `Window` hosts a bottom chrome child webview (Svelte UI) plus zero-to-seven provider child webviews created/destroyed by Rust. Layout math runs in Rust on resize/toggle. Chrome talks to Rust via Tauri commands/events; Rust drives provider pages by `eval` of local adapter scripts that expose `window.__mwcAdapter`.

**Tech Stack:** Tauri 2.x (`unstable` for multiwebview), latest stable Rust, Svelte 5 + Vite + TypeScript, Bun, WebKitGTK on Linux; package AppImage / `.deb` / `.rpm`. Always install the latest published versions of npm packages, crates, and rustc — plan snippets that show older version numbers are shape examples, not pins.

## Global Constraints

- Spec source of truth: `docs/superpowers/specs/2026-09-05-multi-web-chat-design.md` — do not expand v1 scope.
- Shell: Tauri v2 only (not Electron, not Dioxus); chrome UI: Svelte only.
- Chat transport: DOM automation via injected JS only — no provider chat APIs.
- Target OS: Linux only; no macOS/Windows targets in CI or bundle config.
- Packages: AppImage, `.deb`, `.rpm`; document WebKitGTK runtime dependency in README.
- Providers (fixed order): `chatgpt`, `claude`, `gemini`, `copilot`, `zai`, `deepseek`, `grok`.
- Layout A: bottom chrome default **96px**, cap **140px**; pane gap **2px**; all 7 → 4+3 grid; fewer → equal-column reflow (`n≤4` one row; 5→3+2; 6→3+3; 0→empty).
- Disable destroys webview immediately; session dir on disk retained.
- Prefs: `app_config_dir/prefs.json` enabled map only; missing keys default `true`.
- Sessions: `app_data_dir/sessions/<provider_id>/` as each provider webview `data_directory`.
- Adapter global: `window.__mwcAdapter` with `setPrompt`, `submit`, `newChat`.
- Send: empty/whitespace trimmed text → no-op; Enter sends, Shift+Enter newline; Clear is chrome-local only.
- Multiwebview requires Cargo feature `unstable` on `tauri`; child webviews via `Window::add_child`.
- Linux risk: child webview absolute positioning has historically been broken (GTK `Box` packing). Task 5 includes a hard positioning smoke gate before building features on top.
- Commits: Conventional Commits; AI commits include `Co-authored-by: Composer via Cursor <cursoragent@cursor.com>`; log work in `.agents/logs/YYYY-MM-DD.md`.
- Package manager: Bun (existing repo). Prefer `mise` for tool versions.
- Dependency versions: use the latest published stable of every npm package, crate, and rustc. Never install a lower major/minor because a plan snippet shows an older number.

---

## File Structure

```text
/
├── package.json                          # scripts: tauri:dev, tauri:build, test:unit (frontend)
├── vite.config.ts                        # Vite + Svelte; build → dist/
├── svelte.config.js
├── tsconfig.json
├── index.html                            # chrome entry HTML
├── src/                                  # Svelte chrome UI only (never loads provider URLs)
│   ├── main.ts
│   ├── App.svelte                        # toggles, prompt, Send/New/Clear, status row
│   ├── lib/
│   │   ├── providers.ts                  # mirrored registry ids/labels (display only)
│   │   ├── types.ts                      # PaneStatus, PrefsEnabled, command payloads
│   │   └── tauri.ts                      # invoke + listen wrappers
│   └── styles.css
├── adapters/                             # plain JS IFEs evaluated in provider webviews
│   ├── chatgpt.js
│   ├── claude.js
│   ├── gemini.js
│   ├── copilot.js
│   ├── zai.js
│   ├── deepseek.js
│   └── grok.js
├── src-tauri/
│   ├── Cargo.toml                        # tauri = { version = "2", features = ["unstable", ...] }
│   ├── tauri.conf.json                   # Linux bundle targets only; single main window config unused for children
│   ├── capabilities/default.json         # chrome webview permissions for invoke/events
│   ├── build.rs
│   ├── icons/                            # default Tauri icons
│   └── src/
│       ├── main.rs                       # entry → lib::run()
│       ├── lib.rs                        # Builder, setup, manage AppState, handlers
│       ├── registry.rs                   # ordered providers + adapter asset paths
│       ├── prefs.rs                      # load/save prefs.json
│       ├── layout.rs                     # bounds math (unit-tested)
│       ├── shell.rs                      # create/destroy/reflow webviews
│       ├── commands.rs                   # send_prompt, new_chat_all, set_provider_enabled, get_prefs, get_providers
│       ├── adapter_runtime.rs            # load adapter source, eval setPrompt/submit/newChat
│       └── status.rs                     # PaneStatus enum + emit pane_status
├── docs/superpowers/specs/2026-09-05-multi-web-chat-design.md
└── README.md                             # WebKitGTK packages + run/build
```

**Responsibility rules:** `layout.rs` has no Tauri types (pure math). `prefs.rs` only touches disk JSON. `shell.rs` owns webview lifecycle. `adapter_runtime.rs` owns eval strings. Svelte never imports adapter JS.

---

### Task 1: Scaffold Tauri v2 + Svelte chrome in the existing repo

**Files:**

- Create: `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/App.svelte`, `src/styles.css`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`
- Modify: `package.json`, `mise.toml`, `cog.toml` (add scopes: `tauri`, `chrome`, `adapters`, `shell`, `prefs`, `layout`), `.gitignore`, `.config/cspell.json` as needed
- Test: `bun run build` (Vite) and `cargo check` in `src-tauri` (may fail until Linux WebKit deps installed — note in step)

**Interfaces:**

- Consumes: existing Bun/`package.json` scaffold
- Produces: runnable chrome-only app skeleton (`bun run tauri:dev` starts window; later tasks add child webviews)
- [ ] **Step 1: Add Rust toolchain via mise and verify**

```toml
# append to mise.toml [tools]
rust = "1.85"
```

Run:

```bash
mise install
rustc --version
cargo --version
```

Expected: Rust ≥ 1.77 prints; cargo available.

- [ ] **Step 2: Extend root package.json for Vite + Svelte + Tauri CLI**

Replace/extend `package.json` scripts and deps:

```json
{
  "name": "multi-web-chat",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "@tauri-apps/cli": "^2.5.0",
    "prettier": "^3.9.6",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "typescript": "^5.7.0",
    "vite": "^6.0.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.5.0"
  }
}
```

Run: `bun install`  
Expected: lockfile updates; no install errors.

- [ ] **Step 3: Add Vite + Svelte entry files**

`vite.config.ts`:

```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] }
  }
});
```

`index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>multi-web-chat</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

`src/main.ts`:

```ts
import "./styles.css";
import { mount } from "svelte";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
```

`src/App.svelte` (placeholder chrome):

```svelte
<main class="chrome">
  <p>multi-web-chat chrome scaffold</p>
</main>
```

`src/styles.css`:

```css
:root {
  color-scheme: light;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
}
html,
body,
#app {
  margin: 0;
  height: 100%;
}
.chrome {
  box-sizing: border-box;
  height: 100%;
  padding: 8px 12px;
  background: #1a1d21;
  color: #e8eaed;
}
```

Add minimal `svelte.config.js` and `tsconfig.json` matching `@sveltejs/vite-plugin-svelte` defaults for Svelte 5.

- [ ] **Step 4: Create src-tauri crate with unstable multiwebview**

`src-tauri/Cargo.toml`:

```toml
[package]
name = "multi-web-chat"
version = "0.0.0"
description = "Linux multi-webview AI chat browser wrapper"
authors = ["multi-web-chat"]
edition = "2021"

[lib]
name = "multi_web_chat_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["unstable"] }
tauri-plugin-log = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "macros", "rt"] }

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
```

`src-tauri/tauri.conf.json` (Linux bundles only; empty windows — created in Rust):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "multi-web-chat",
  "version": "0.0.0",
  "identifier": "dev.multiwebchat.app",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "security": {
      "csp": null
    },
    "windows": []
  },
  "bundle": {
    "active": true,
    "targets": ["appimage", "deb", "rpm"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/src/lib.rs`:

```rust
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _window = tauri::window::WindowBuilder::new(app, "main")
                .title("multi-web-chat")
                .inner_size(1280.0, 800.0)
                .build()?;

            // Chrome webview added in Task 5; placeholder title-only window for now.
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running multi-web-chat");
}
```

`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    multi_web_chat_lib::run();
}
```

`src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

Generate default icons with `bunx tauri icon` from any 1024 PNG, or copy defaults from `create-tauri-app` template into `src-tauri/icons/`.

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Chrome webview capabilities",
  "windows": ["main"],
  "webviews": ["chrome"],
  "permissions": ["core:default", "core:event:default", "core:webview:default"]
}
```

- [ ] **Step 5: Update .gitignore for Rust/Vite**

Append if missing:

```gitignore
dist/
src-tauri/target/
src-tauri/gen/
*.AppImage
```

- [ ] **Step 6: Add cog scopes used by later commits**

In `cog.toml` `scopes` array, add alphabetically:

```toml
"adapters",
"chrome",
"layout",
"prefs",
"shell",
"tauri",
```

- [ ] **Step 7: Verify frontend build**

Run: `bun run build`  
Expected: `dist/` produced; exit 0.

- [ ] **Step 8: Verify Rust compiles (or record WebKit dep gap)**

Run:

```bash
cd src-tauri && cargo check
```

Expected: success if WebKitGTK/dev packages present; if linker/pkg-config fails for `webkit2gtk`, install distro packages (Task 12 documents them) then re-run until `cargo check` passes.

- [ ] **Step 9: Commit**

```bash
git add package.json bun.lock vite.config.ts svelte.config.js tsconfig.json index.html src src-tauri mise.toml cog.toml .gitignore
git commit -m "$(cat <<'EOF'
feat(tauri): scaffold Tauri v2 + Svelte chrome app

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 2: Provider registry

**Files:**

- Create: `src-tauri/src/registry.rs`, `src/lib/providers.ts`, `src/lib/types.ts`
- Modify: `src-tauri/src/lib.rs` (mod registry)
- Test: `src-tauri/src/registry.rs` unit tests (`cargo test registry::`)

**Interfaces:**

- Consumes: none
- Produces:
  - `ProviderId` enum / `&'static str` ids
  - `pub struct Provider { pub id: &'static str, pub label: &'static str, pub start_url: &'static str, pub adapter_file: &'static str }`
  - `pub fn all_providers() -> &'static [Provider]`
  - `pub fn provider(id: &str) -> Option<&'static Provider>`
  - TS: `export const PROVIDERS: { id: string; label: string }[]` same order/ids
- [ ] **Step 1: Write failing Rust registry test**

In `registry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_seven_providers_in_spec_order() {
        let ids: Vec<&str> = all_providers().iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            vec![
                "chatgpt", "claude", "gemini", "copilot", "zai", "deepseek", "grok"
            ]
        );
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(provider("nope").is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test registry::tests::registry_has_seven_providers_in_spec_order -- --nocapture`  
Expected: FAIL (module/functions missing).

- [ ] **Step 3: Implement registry**

```rust
#[derive(Debug, Clone, Copy)]
pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub start_url: &'static str,
    pub adapter_file: &'static str,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "chatgpt",
        label: "ChatGPT",
        start_url: "https://chatgpt.com",
        adapter_file: "chatgpt.js",
    },
    Provider {
        id: "claude",
        label: "Claude",
        start_url: "https://claude.ai",
        adapter_file: "claude.js",
    },
    Provider {
        id: "gemini",
        label: "Gemini",
        start_url: "https://gemini.google.com",
        adapter_file: "gemini.js",
    },
    Provider {
        id: "copilot",
        label: "Copilot",
        start_url: "https://copilot.microsoft.com",
        adapter_file: "copilot.js",
    },
    Provider {
        id: "zai",
        label: "Z.AI",
        start_url: "https://chat.z.ai",
        adapter_file: "zai.js",
    },
    Provider {
        id: "deepseek",
        label: "DeepSeek",
        start_url: "https://chat.deepseek.com",
        adapter_file: "deepseek.js",
    },
    Provider {
        id: "grok",
        label: "Grok",
        start_url: "https://grok.x.ai",
        adapter_file: "grok.js",
    },
];

pub fn all_providers() -> &'static [Provider] {
    PROVIDERS
}

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}
```

Mirror ids/labels in `src/lib/providers.ts`. Add `src/lib/types.ts`:

```ts
export type PaneStatus = "idle" | "sending" | "ok" | "error";
export type ProviderId =
  "chatgpt" | "claude" | "gemini" | "copilot" | "zai" | "deepseek" | "grok";
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cd src-tauri && cargo test registry:: -- --nocapture`  
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/registry.rs src-tauri/src/lib.rs src/lib/providers.ts src/lib/types.ts
git commit -m "$(cat <<'EOF'
feat(shell): add ordered provider registry

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 3: Prefs store (load/save with defaults)

**Files:**

- Create: `src-tauri/src/prefs.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: unit tests inside `prefs.rs`

**Interfaces:**

- Consumes: `registry::all_providers`
- Produces:
  - `pub struct Prefs { pub enabled: HashMap<String, bool> }`
  - `pub fn default_prefs() -> Prefs`
  - `pub fn load_prefs(path: &Path) -> Result<Prefs, PrefsError>`
  - `pub fn save_prefs(path: &Path, prefs: &Prefs) -> Result<(), PrefsError>`
  - `pub fn is_enabled(prefs: &Prefs, id: &str) -> bool` — missing key → `true`
  - `pub fn set_enabled(prefs: &mut Prefs, id: &str, enabled: bool)`
  - Path helper: `prefs_path(app: &AppHandle) -> PathBuf` = `app.path().app_config_dir()/prefs.json`
- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn missing_file_yields_all_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prefs.json");
    let prefs = load_prefs(&path).unwrap();
    for p in all_providers() {
        assert!(is_enabled(&prefs, p.id));
    }
}

#[test]
fn unknown_keys_ignored_missing_keys_default_true() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prefs.json");
    std::fs::write(
        &path,
        r#"{"enabled":{"chatgpt":false,"unknown":true}}"#,
    )
    .unwrap();
    let prefs = load_prefs(&path).unwrap();
    assert!(!is_enabled(&prefs, "chatgpt"));
    assert!(is_enabled(&prefs, "claude")); // missing → true
}

#[test]
fn round_trip_save_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prefs.json");
    let mut prefs = default_prefs();
    set_enabled(&mut prefs, "grok", false);
    save_prefs(&path, &prefs).unwrap();
    let loaded = load_prefs(&path).unwrap();
    assert!(!is_enabled(&loaded, "grok"));
    assert!(is_enabled(&loaded, "chatgpt"));
}
```

Add `tempfile` to `[dev-dependencies]` in `Cargo.toml`.

- [ ] **Step 2: Run tests — expect FAIL**

Run: `cd src-tauri && cargo test prefs:: -- --nocapture`  
Expected: FAIL (prefs module missing).

- [ ] **Step 3: Implement prefs.rs**

```rust
use crate::registry::all_providers;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Prefs {
    pub enabled: HashMap<String, bool>,
}

#[derive(Debug, Error)]
pub enum PrefsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn default_prefs() -> Prefs {
    let mut enabled = HashMap::new();
    for p in all_providers() {
        enabled.insert(p.id.to_string(), true);
    }
    Prefs { enabled }
}

pub fn is_enabled(prefs: &Prefs, id: &str) -> bool {
    prefs.enabled.get(id).copied().unwrap_or(true)
}

pub fn set_enabled(prefs: &mut Prefs, id: &str, enabled: bool) {
    prefs.enabled.insert(id.to_string(), enabled);
}

pub fn load_prefs(path: &Path) -> Result<Prefs, PrefsError> {
    if !path.exists() {
        return Ok(default_prefs());
    }
    let raw = fs::read_to_string(path)?;
    let parsed: Prefs = serde_json::from_str(&raw)?;
    let mut prefs = default_prefs();
    for p in all_providers() {
        if let Some(v) = parsed.enabled.get(p.id) {
            prefs.enabled.insert(p.id.to_string(), *v);
        }
    }
    Ok(prefs)
}

pub fn save_prefs(path: &Path, prefs: &Prefs) -> Result<(), PrefsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut clean = Prefs {
        enabled: HashMap::new(),
    };
    for p in all_providers() {
        clean
            .enabled
            .insert(p.id.to_string(), is_enabled(prefs, p.id));
    }
    let json = serde_json::to_string_pretty(&clean)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn prefs_path(config_dir: PathBuf) -> PathBuf {
    config_dir.join("prefs.json")
}
```

- [ ] **Step 4: Run tests — expect PASS**

Run: `cd src-tauri && cargo test prefs:: -- --nocapture`  
Expected: all prefs tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/prefs.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "$(cat <<'EOF'
feat(prefs): add prefs.json load/save with defaults

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 4: Layout math (Layout A)

**Files:**

- Create: `src-tauri/src/layout.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: exhaustive unit tests in `layout.rs`

**Interfaces:**

- Consumes: none (pure)
- Produces:
  - `pub const CHROME_HEIGHT_DEFAULT: f64 = 96.0;`
  - `pub const CHROME_HEIGHT_MAX: f64 = 140.0;`
  - `pub const PANE_GAP: f64 = 2.0;`
  - `pub struct Rect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }`
  - `pub struct LayoutPlan { pub chrome: Rect, pub panes: Vec<(String, Rect)> }`
  - `pub fn compute_layout(window_width: f64, window_height: f64, chrome_height: f64, enabled_ids_in_order: &[String]) -> LayoutPlan`
- [ ] **Step 1: Write failing layout tests**

```rust
#[test]
fn chrome_sits_at_bottom_with_default_height() {
    let plan = compute_layout(1000.0, 800.0, CHROME_HEIGHT_DEFAULT, &[]);
    assert_eq!(plan.chrome.y, 800.0 - 96.0);
    assert_eq!(plan.chrome.height, 96.0);
    assert_eq!(plan.chrome.width, 1000.0);
    assert!(plan.panes.is_empty());
}

#[test]
fn seven_panes_use_four_plus_three() {
    let ids: Vec<String> = ["chatgpt","claude","gemini","copilot","zai","deepseek","grok"]
        .into_iter().map(str::to_string).collect();
    let plan = compute_layout(1002.0, 800.0, 96.0, &ids);
    assert_eq!(plan.panes.len(), 7);
    // top row y=0, bottom row y = pane_h + gap
    let pane_region_h = 800.0 - 96.0;
    let row_h = (pane_region_h - PANE_GAP) / 2.0;
    assert!((plan.panes[0].1.y - 0.0).abs() < 0.01);
    assert!((plan.panes[4].1.y - (row_h + PANE_GAP)).abs() < 0.01);
    assert_eq!(plan.panes[0].1.width, plan.panes[1].1.width);
    // bottom row three equal columns
    assert_eq!(plan.panes[4].1.width, plan.panes[5].1.width);
    assert_eq!(plan.panes[5].1.width, plan.panes[6].1.width);
}

#[test]
fn five_panes_split_three_plus_two() {
    let ids: Vec<String> = ["chatgpt","claude","gemini","copilot","zai"]
        .into_iter().map(str::to_string).collect();
    let plan = compute_layout(1000.0, 800.0, 96.0, &ids);
    assert_eq!(plan.panes.len(), 5);
    let top: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y < 1.0).collect();
    let bottom: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y >= 1.0).collect();
    assert_eq!(top.len(), 3);
    assert_eq!(bottom.len(), 2);
}

#[test]
fn four_or_fewer_single_row() {
    let ids: Vec<String> = ["chatgpt","claude"].into_iter().map(str::to_string).collect();
    let plan = compute_layout(1000.0, 800.0, 96.0, &ids);
    assert!(plan.panes.iter().all(|(_, r)| r.y < 0.01));
    assert!((plan.panes[0].1.width - plan.panes[1].1.width).abs() < 0.01);
}

#[test]
fn chrome_height_clamped_to_max() {
    let plan = compute_layout(800.0, 600.0, 999.0, &[]);
    assert_eq!(plan.chrome.height, CHROME_HEIGHT_MAX);
}
```

- [ ] **Step 2: Run tests — expect FAIL**

Run: `cd src-tauri && cargo test layout:: -- --nocapture`  
Expected: FAIL.

- [ ] **Step 3: Implement layout.rs**

```rust
pub const CHROME_HEIGHT_DEFAULT: f64 = 96.0;
pub const CHROME_HEIGHT_MAX: f64 = 140.0;
pub const PANE_GAP: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct LayoutPlan {
    pub chrome: Rect,
    pub panes: Vec<(String, Rect)>,
}

pub fn compute_layout(
    window_width: f64,
    window_height: f64,
    chrome_height: f64,
    enabled_ids_in_order: &[String],
) -> LayoutPlan {
    let chrome_h = chrome_height.clamp(0.0, CHROME_HEIGHT_MAX);
    let chrome = Rect {
        x: 0.0,
        y: (window_height - chrome_h).max(0.0),
        width: window_width.max(0.0),
        height: chrome_h,
    };
    let pane_region_h = (window_height - chrome_h).max(0.0);
    let n = enabled_ids_in_order.len();
    if n == 0 {
        return LayoutPlan {
            chrome,
            panes: vec![],
        };
    }

    let row_counts: Vec<usize> = if n == 7 {
        vec![4, 3]
    } else if n <= 4 {
        vec![n]
    } else if n == 5 {
        vec![3, 2]
    } else if n == 6 {
        vec![3, 3]
    } else {
        // n > 7 should not happen with fixed registry; fall back to single row
        vec![n]
    };

    let row_count = row_counts.len();
    let total_gap_y = PANE_GAP * (row_count.saturating_sub(1) as f64);
    let row_h = if row_count == 0 {
        0.0
    } else {
        (pane_region_h - total_gap_y) / row_count as f64
    };

    let mut panes = Vec::with_capacity(n);
    let mut idx = 0usize;
    let mut y = 0.0;
    for cols in row_counts {
        let total_gap_x = PANE_GAP * (cols.saturating_sub(1) as f64);
        let col_w = if cols == 0 {
            0.0
        } else {
            (window_width - total_gap_x) / cols as f64
        };
        for c in 0..cols {
            let id = enabled_ids_in_order[idx].clone();
            panes.push((
                id,
                Rect {
                    x: c as f64 * (col_w + PANE_GAP),
                    y,
                    width: col_w,
                    height: row_h,
                },
            ));
            idx += 1;
        }
        y += row_h + PANE_GAP;
    }

    LayoutPlan { chrome, panes }
}
```

- [ ] **Step 4: Run tests — expect PASS**

Run: `cd src-tauri && cargo test layout:: -- --nocapture`  
Expected: all layout tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/layout.rs src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(layout): implement Layout A pane bounds math

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 5: Multi-webview shell + Linux positioning smoke gate

**Files:**

- Create: `src-tauri/src/shell.rs`, `src-tauri/src/status.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`
- Test: manual Linux smoke (required); optional unit test for label helpers

**Interfaces:**

- Consumes: `layout::compute_layout`, `prefs::*`, `registry::*`
- Produces:
  - `AppState` with `prefs: Mutex<Prefs>`, `chrome_height: Mutex<f64>`, living provider labels
  - `pub fn create_main_shell(app: &AppHandle) -> tauri::Result<()>`
  - `pub fn reflow(app: &AppHandle) -> tauri::Result<()>`
  - Chrome webview label: `"chrome"`; provider labels: provider id (`"chatgpt"`, …)
  - Session path: `app.path().app_data_dir()?.join("sessions").join(provider_id)`
- [ ] **Step 1: Implement status types**

`status.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaneStatusKind {
    Idle,
    Sending,
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaneStatusEvent {
    pub id: String,
    pub status: PaneStatusKind,
    pub message: Option<String>,
}

pub const EVENT_PANE_STATUS: &str = "pane_status";
```

- [ ] **Step 2: Implement shell create chrome + reflow helpers**

In `shell.rs`, create main window, add chrome child at bottom loading `WebviewUrl::App("index.html".into())`, store `AppState`, on `WindowEvent::Resized` / `ScaleFactorChanged` call `reflow`.

Core create pattern (no `auto_resize` — manual bounds):

```rust
use tauri::{
    LogicalPosition, LogicalSize, Manager, WebviewUrl,
    webview::{PageLoadEvent, WebviewBuilder},
};

let window = tauri::window::WindowBuilder::new(app, "main")
    .title("multi-web-chat")
    .inner_size(1280.0, 800.0)
    .build()?;

let (w, h) = {
    let s = window.inner_size()?;
    let sf = window.scale_factor()?;
    (s.width as f64 / sf, s.height as f64 / sf)
};
let plan = compute_layout(w, h, CHROME_HEIGHT_DEFAULT, &[]);

let chrome = WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into()));
window.add_child(
    chrome,
    LogicalPosition::new(plan.chrome.x, plan.chrome.y),
    LogicalSize::new(plan.chrome.width, plan.chrome.height),
)?;
```

`reflow` must: read enabled ids in registry order; `compute_layout`; for each living provider webview `set_position` + `set_size`; same for chrome.

Provider create (used by Task 6; stub function now):

```rust
pub fn session_dir(app: &AppHandle, provider_id: &str) -> tauri::Result<PathBuf> {
    let base = app.path().app_data_dir()?.join("sessions").join(provider_id);
    std::fs::create_dir_all(&base)?;
    Ok(base)
}
```

- [ ] **Step 3: Wire setup in lib.rs**

Replace placeholder setup with `create_main_shell`, manage state, listen for resize:

```rust
window.on_window_event(|event| {
    // use app handle clone to call reflow on Resized
});
```

- [ ] **Step 4: Linux positioning smoke gate (manual)**

Run: `bun run tauri:dev`

Verification checklist (must pass before Task 6):

1. Window opens on Linux.
2. Chrome webview visible at bottom (~96px), not full-window stacked incorrectly.
3. Temporarily hard-code adding one external child (e.g. `https://example.com`) at `LogicalPosition(0,0)` with size `(400,300)` in setup; confirm it occupies top-left, not stacked under chrome via GTK Box.
4. Resize window; after `reflow`, chrome stays bottom and test pane keeps intended bounds.

If positioning fails (known `gtk::Box` issue): stop feature work; bump to newest `tauri`/`tao`/`wry` that pack children in `gtk::Fixed`, or vendor the Fixed-container fix. Record the chosen pin in `Cargo.toml` / plan notes in the agents log. Do not proceed to toggles until two webviews respect absolute bounds.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/shell.rs src-tauri/src/status.rs src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(shell): add main window chrome webview and reflow

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 6: Enable/disable provider webviews + prefs command

**Files:**

- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/shell.rs`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`
- Test: manual toggle smoke; unit test `enabled_ids` ordering helper

**Interfaces:**

- Consumes: `shell`, `prefs`, `registry`, `layout`
- Produces commands:
  - `get_providers() -> Vec<ProviderDto { id, label }>`
  - `get_prefs() -> Prefs`
  - `set_provider_enabled(id: String, enabled: bool) -> Result<(), String>`
  - On enable: `WebviewBuilder::new(id, WebviewUrl::External(start_url)).data_directory(session_dir).on_page_load(...)` then `add_child` + `reflow`
  - On disable: `webview.close()` + `reflow` + save prefs
  - Startup: load prefs; create webviews for each enabled provider
- [ ] **Step 1: Write unit test for enabled id ordering**

```rust
#[test]
fn enabled_ids_follow_registry_order() {
    let mut prefs = default_prefs();
    set_enabled(&mut prefs, "chatgpt", false);
    set_enabled(&mut prefs, "grok", true);
    let ids = enabled_provider_ids(&prefs);
    assert_eq!(ids.first().map(String::as_str), Some("claude"));
    assert_eq!(ids.last().map(String::as_str), Some("grok"));
    assert!(!ids.iter().any(|i| i == "chatgpt"));
}
```

Implement:

```rust
pub fn enabled_provider_ids(prefs: &Prefs) -> Vec<String> {
    all_providers()
        .iter()
        .filter(|p| is_enabled(prefs, p.id))
        .map(|p| p.id.to_string())
        .collect()
}
```

- [ ] **Step 2: Run test — FAIL then implement — PASS**

Run: `cd src-tauri && cargo test enabled_provider_ids -- --nocapture`

- [ ] **Step 3: Implement set_provider_enabled + startup create**

```rust
#[tauri::command]
pub async fn set_provider_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    if provider(&id).is_none() {
        return Err(format!("unknown provider: {id}"));
    }
    {
        let mut prefs = state.prefs.lock().map_err(|e| e.to_string())?;
        set_enabled(&mut prefs, &id, enabled);
        let path = prefs_path(app.path().app_config_dir().map_err(|e| e.to_string())?);
        save_prefs(&path, &prefs).map_err(|e| e.to_string())?;
    }
    if enabled {
        shell::ensure_provider_webview(&app, &id).map_err(|e| e.to_string())?;
    } else {
        shell::destroy_provider_webview(&app, &id).map_err(|e| e.to_string())?;
    }
    shell::reflow(&app).map_err(|e| e.to_string())?;
    Ok(())
}
```

`ensure_provider_webview` must set `.data_directory(session_dir(&app, &id)?)`.

Register handlers in `lib.rs` with `generate_handler![...]`.

- [ ] **Step 4: Manual verification**

Run `bun run tauri:dev`. Using temporary invoke from chrome console or a temporary button:

1. Start with prefs all true → seven panes + chrome layout (4+3).
2. Disable three providers → reflow to fewer columns; webviews gone (RAM drop is best-effort observation).
3. Re-enable → webview recreates; cookies path under `sessions/<id>/` exists on disk.
4. Restart app → same enable map from `prefs.json`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/shell.rs src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "$(cat <<'EOF'
feat(shell): create and destroy provider webviews from prefs

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 7: Adapter runtime + stub adapters

**Files:**

- Create: `src-tauri/src/adapter_runtime.rs`, `adapters/*.js` (seven stubs), ensure adapters copied/embedded
- Modify: `src-tauri/src/shell.rs` (inject on `PageLoadEvent::Finished`), `src-tauri/tauri.conf.json` / `Cargo.toml` resources
- Test: unit test for eval script builder; manual eval smoke on `example.com` optional

**Interfaces:**

- Consumes: registry `adapter_file`
- Produces:
  - `pub fn load_adapter_source(provider_id: &str) -> Result<String, AdapterError>`
  - `pub fn inject_adapter_script(source: &str) -> String` — wraps source (already IIFE)
  - `pub fn call_set_prompt_js(text: &str) -> String`
  - `pub fn call_submit_js() -> String`
  - `pub fn call_new_chat_js() -> String`
  - On page load finished for provider webview: `webview.eval(&source)`

Embed adapters via `include_str!` mapped by id (reliable; no runtime path issues):

```rust
pub fn load_adapter_source(provider_id: &str) -> Result<&'static str, AdapterError> {
    Ok(match provider_id {
        "chatgpt" => include_str!("../../adapters/chatgpt.js"),
        "claude" => include_str!("../../adapters/claude.js"),
        "gemini" => include_str!("../../adapters/gemini.js"),
        "copilot" => include_str!("../../adapters/copilot.js"),
        "zai" => include_str!("../../adapters/zai.js"),
        "deepseek" => include_str!("../../adapters/deepseek.js"),
        "grok" => include_str!("../../adapters/grok.js"),
        _ => return Err(AdapterError::UnknownProvider(provider_id.to_string())),
    })
}
```

Eval helpers (JSON-serialize text for safe embedding):

```rust
pub fn call_set_prompt_js(text: &str) -> String {
    let t = serde_json::to_string(text).unwrap();
    format!(
        r#"(async()=>{{
  const a=window.__mwcAdapter;
  if(!a) throw new Error('adapter missing');
  await a.setPrompt({t});
}})()"#
    )
}

pub fn call_submit_js() -> String {
    r#"(async()=>{
  const a=window.__mwcAdapter;
  if(!a) throw new Error('adapter missing');
  await a.submit();
})()"#
    .into()
}

pub fn call_new_chat_js() -> String {
    r#"(async()=>{
  const a=window.__mwcAdapter;
  if(!a) throw new Error('adapter missing');
  await a.newChat();
})()"#
    .into()
}
```

- [ ] **Step 1: Write failing test for JS helper escaping**

```rust
#[test]
fn set_prompt_js_escapes_quotes_and_newlines() {
    let js = call_set_prompt_js("say \"hi\"\nthere");
    assert!(js.contains("say \\\"hi\\\""));
    assert!(js.contains("\\n"));
}
```

- [ ] **Step 2: Run — FAIL; implement adapter_runtime — PASS**

- [ ] **Step 3: Create stub adapter file template for each provider**

Each `adapters/<id>.js` is an IIFE assigning `window.__mwcAdapter`. Stub behavior: `setPrompt`/`submit`/`newChat` reject with `"stub: <id> selectors not implemented"` until Task 9–10 fill selectors. Example `adapters/chatgpt.js`:

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 8000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(_text) {
      throw new Error("stub: chatgpt selectors not implemented");
    },
    async submit() {
      throw new Error("stub: chatgpt selectors not implemented");
    },
    async newChat() {
      throw new Error("stub: chatgpt selectors not implemented");
    }
  };
})();
```

Create the same stub (with matching provider name in error strings) for `claude`, `gemini`, `copilot`, `zai`, `deepseek`, `grok`.

- [ ] **Step 4: Inject on PageLoadEvent::Finished in ensure_provider_webview**

```rust
.on_page_load(|webview, payload| {
    if payload.event() == PageLoadEvent::Finished {
        if let Ok(src) = load_adapter_source(webview.label()) {
            let _ = webview.eval(src);
        }
    }
})
```

- [ ] **Step 5: Manual check**

Open a provider pane; in WebKit inspector if available, or by calling `send_prompt` early, confirm adapter missing vs stub error. After load, `eval("!!window.__mwcAdapter")` should be true.

- [ ] **Step 6: Commit**

```bash
git add adapters src-tauri/src/adapter_runtime.rs src-tauri/src/shell.rs src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(adapters): add inject runtime and stub adapters

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 8: send_prompt, new_chat_all, pane_status events

**Files:**

- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/adapter_runtime.rs`, `src-tauri/src/shell.rs`
- Test: unit tests for empty-prompt / zero-pane early return helpers; manual parallel fan-out

**Interfaces:**

- Consumes: adapter_runtime, shell, status
- Produces:
  - `send_prompt(text: String) -> Result<(), String>`
  - `new_chat_all() -> Result<(), String>`
  - Emits `pane_status` with `{ id, status, message? }` per pane
  - Parallelism: `tokio::spawn` one task per enabled webview; await all joins
  - Send path per pane: status `sending` → `run_send` → `ok` / `error`
  - New path: status `sending` → `run_new_chat` → `ok` / `error`
  - Result bridge (Linux-only, locked): async adapter IIFE writes `window.__mwcLastResult = { ok, error? }`; Rust polls it via `webview.with_webview` + `webkit2gtk` `evaluate_javascript` until set or timeout (30s)
- [ ] **Step 1: Write unit tests for guard helpers**

```rust
#[test]
fn empty_prompt_is_noop_guard() {
    assert!(normalize_prompt("  \n\t ").is_none());
    assert_eq!(normalize_prompt("hello").as_deref(), Some("hello"));
}
```

```rust
pub fn normalize_prompt(text: &str) -> Option<String> {
    let t = text.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
```

- [ ] **Step 2: Add Linux webkit2gtk dep for result polling**

In `src-tauri/Cargo.toml`:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
webkit2gtk = { version = "2.0.1", features = ["v2_40"] }
```

- [ ] **Step 3: Implement poll_last_result + run_send / run_new_chat**

```rust
#[derive(Deserialize)]
struct LastResult {
    ok: bool,
    error: Option<String>,
}

pub async fn run_send(app: &AppHandle, id: &str, text: &str) -> Result<(), String> {
    let webview = app
        .get_webview(id)
        .ok_or_else(|| format!("webview {id} missing"))?;
    webview
        .eval("window.__mwcLastResult = null;")
        .map_err(|e| e.to_string())?;
    let js = format!(
        r#"(async()=>{{
  try {{
    const a=window.__mwcAdapter;
    if(!a) throw new Error('adapter missing');
    await a.setPrompt({text});
    await a.submit();
    window.__mwcLastResult = {{ ok: true }};
  }} catch(e) {{
    window.__mwcLastResult = {{ ok: false, error: String(e && e.message || e) }};
  }}
}})()"#,
        text = serde_json::to_string(text).unwrap()
    );
    webview.eval(&js).map_err(|e| e.to_string())?;
    let result = poll_last_result(&webview, std::time::Duration::from_secs(30)).await?;
    if result.ok {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "unknown adapter error".into()))
    }
}

pub async fn run_new_chat(app: &AppHandle, id: &str) -> Result<(), String> {
    let webview = app
        .get_webview(id)
        .ok_or_else(|| format!("webview {id} missing"))?;
    webview
        .eval("window.__mwcLastResult = null;")
        .map_err(|e| e.to_string())?;
    webview
        .eval(
            r#"(async()=>{
  try {
    const a=window.__mwcAdapter;
    if(!a) throw new Error('adapter missing');
    await a.newChat();
    window.__mwcLastResult = { ok: true };
  } catch(e) {
    window.__mwcLastResult = { ok: false, error: String(e && e.message || e) };
  }
})()"#,
        )
        .map_err(|e| e.to_string())?;
    let result = poll_last_result(&webview, std::time::Duration::from_secs(30)).await?;
    if result.ok {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "unknown adapter error".into()))
    }
}

/// Linux: read window.__mwcLastResult via webkit2gtk evaluate_javascript.
pub async fn poll_last_result(
    webview: &tauri::Webview,
    timeout: std::time::Duration,
) -> Result<LastResult, String> {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err("adapter timed out".into());
        }
        let (tx, rx) = std::sync::mpsc::channel::<Result<Option<String>, String>>();
        webview
            .with_webview({
                let tx = tx;
                move |w| {
                    #[cfg(target_os = "linux")]
                    {
                        use webkit2gtk::WebViewExt;
                        let tx = tx.clone();
                        w.evaluate_javascript(
                            "window.__mwcLastResult ? JSON.stringify(window.__mwcLastResult) : null",
                            None,
                            None,
                            None::<&gtk::gio::Cancellable>,
                            move |res| {
                                let mapped = res
                                    .map_err(|e| e.to_string())
                                    .map(|value| value.to_string());
                                let _ = tx.send(mapped.map(|s| {
                                    if s == "null" || s.is_empty() {
                                        None
                                    } else {
                                        Some(s.trim_matches('"').to_string())
                                    }
                                }));
                            },
                        );
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        let _ = tx.send(Err(
                            "multi-web-chat v1 supports Linux only".into(),
                        ));
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        match rx.recv_timeout(std::time::Duration::from_millis(200)) {
            Ok(Ok(Some(raw))) => {
                // evaluate_javascript may return a JSON string value already;
                // parse as LastResult, falling back to unquoting once.
                let parsed = serde_json::from_str::<LastResult>(&raw)
                    .or_else(|_| {
                        let unquoted: String = serde_json::from_str(&raw)?;
                        serde_json::from_str(&unquoted)
                    })
                    .map_err(|e| e.to_string())?;
                return Ok(parsed);
            }
            Ok(Ok(None)) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
    }
}
```

Note: `webkit2gtk` `evaluate_javascript` callback signatures vary slightly by crate version — adjust imports (`gtk::gio::Cancellable`) to match the resolved `webkit2gtk 2.x` API during implementation, without changing this overall poll design.

- [ ] **Step 4: Implement send_prompt and new_chat_all commands**

```rust
#[tauri::command]
pub async fn send_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let Some(text) = normalize_prompt(&text) else {
        return Ok(());
    };
    let ids = {
        let prefs = state.prefs.lock().map_err(|e| e.to_string())?;
        enabled_provider_ids(&prefs)
    };
    if ids.is_empty() {
        return Ok(());
    }
    let mut handles = Vec::new();
    for id in ids {
        let app = app.clone();
        let text = text.clone();
        handles.push(tokio::spawn(async move {
            emit_status(&app, &id, PaneStatusKind::Sending, None);
            match adapter_runtime::run_send(&app, &id, &text).await {
                Ok(()) => emit_status(&app, &id, PaneStatusKind::Ok, None),
                Err(e) => emit_status(&app, &id, PaneStatusKind::Error, Some(e)),
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

#[tauri::command]
pub async fn new_chat_all(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let ids = {
        let prefs = state.prefs.lock().map_err(|e| e.to_string())?;
        enabled_provider_ids(&prefs)
    };
    if ids.is_empty() {
        return Ok(());
    }
    let mut handles = Vec::new();
    for id in ids {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            emit_status(&app, &id, PaneStatusKind::Sending, None);
            match adapter_runtime::run_new_chat(&app, &id).await {
                Ok(()) => emit_status(&app, &id, PaneStatusKind::Ok, None),
                Err(e) => emit_status(&app, &id, PaneStatusKind::Error, Some(e)),
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

fn emit_status(app: &AppHandle, id: &str, status: PaneStatusKind, message: Option<String>) {
    let _ = app.emit(
        EVENT_PANE_STATUS,
        PaneStatusEvent {
            id: id.to_string(),
            status,
            message,
        },
    );
}
```

- [ ] **Step 5: cargo test guards; manual fan-out**

Run: `cd src-tauri && cargo test normalize_prompt -- --nocapture`  
Expected: PASS.

Manual: with stubs, Send sets all enabled panes to `error` with stub messages in parallel. Empty prompt leaves statuses unchanged. Zero enabled panes: no events.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/adapter_runtime.rs src-tauri/src/status.rs src-tauri/Cargo.toml
git commit -m "$(cat <<'EOF'
feat(shell): fan out send_prompt and new_chat_all with pane status

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 9: Chrome Svelte UI (toggles, prompt, Send, New, Clear, status)

**Files:**

- Create: `src/lib/tauri.ts`
- Modify: `src/App.svelte`, `src/styles.css`, `src/lib/providers.ts`
- Test: `bun run check`; manual UI smoke

**Interfaces:**

- Consumes: commands `get_providers`, `get_prefs`, `set_provider_enabled`, `send_prompt`, `new_chat_all`; event `pane_status`
- Produces: full chrome UX per spec
- [ ] **Step 1: Add typed invoke wrappers**

`src/lib/tauri.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PaneStatus, ProviderId } from "./types";

export type Prefs = { enabled: Record<string, boolean> };
export type ProviderDto = { id: ProviderId; label: string };
export type PaneStatusPayload = {
  id: string;
  status: PaneStatus;
  message?: string | null;
};

export const getProviders = () => invoke<ProviderDto[]>("get_providers");
export const getPrefs = () => invoke<Prefs>("get_prefs");
export const setProviderEnabled = (id: string, enabled: boolean) =>
  invoke<void>("set_provider_enabled", { id, enabled });
export const sendPrompt = (text: string) =>
  invoke<void>("send_prompt", { text });
export const newChatAll = () => invoke<void>("new_chat_all");

export function onPaneStatus(
  cb: (p: PaneStatusPayload) => void
): Promise<UnlistenFn> {
  return listen<PaneStatusPayload>("pane_status", (e) => cb(e.payload));
}
```

- [ ] **Step 2: Implement App.svelte**

Behavior requirements in the component:

- On mount: load providers + prefs; subscribe to `pane_status`.
- Toggle row: one checkbox/button per provider label; calls `setProviderEnabled`.
- Prompt textarea: Enter → `sendPrompt` (prevent default); Shift+Enter → newline.
- Buttons: **Send**, **New**, **Clear** (Clear only clears local `prompt` state).
- Status line: per-pane `idle|sending|ok|error` + short message; if zero enabled, show `Enable at least one provider`.
- Keep chrome compact for ~96px height (flex wrap ok up to 140px).

Concrete structure:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import {
    getPrefs,
    getProviders,
    newChatAll,
    onPaneStatus,
    sendPrompt,
    setProviderEnabled,
    type PaneStatusPayload,
    type Prefs,
    type ProviderDto
  } from "./lib/tauri";
  import type { PaneStatus } from "./lib/types";

  let providers: ProviderDto[] = $state([]);
  let prefs: Prefs = $state({ enabled: {} });
  let prompt = $state("");
  let statuses: Record<string, { status: PaneStatus; message?: string }> =
    $state({});

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      providers = await getProviders();
      prefs = await getPrefs();
      unlisten = await onPaneStatus((p: PaneStatusPayload) => {
        statuses[p.id] = {
          status: p.status,
          message: p.message ?? undefined
        };
        statuses = statuses;
      });
    })();
    return () => unlisten?.();
  });

  const enabledCount = () =>
    providers.filter((p) => prefs.enabled[p.id] !== false).length;

  async function toggle(id: string, enabled: boolean) {
    await setProviderEnabled(id, enabled);
    prefs.enabled[id] = enabled;
    prefs = prefs;
  }

  async function onSend() {
    await sendPrompt(prompt);
  }

  function onClear() {
    prompt = "";
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void onSend();
    }
  }
</script>

<main class="chrome">
  <div class="toggles">
    {#each providers as p}
      <label>
        <input
          type="checkbox"
          checked={prefs.enabled[p.id] !== false}
          onchange={(e) => toggle(p.id, e.currentTarget.checked)}
        />
        {p.label}
      </label>
    {/each}
  </div>
  <div class="row">
    <textarea
      bind:value={prompt}
      onkeydown={onKeydown}
      placeholder="Prompt all enabled providers"
      rows="2"></textarea>
    <button type="button" onclick={onSend}>Send</button>
    <button type="button" onclick={() => newChatAll()}>New</button>
    <button type="button" onclick={onClear}>Clear</button>
  </div>
  <div class="status">
    {#if enabledCount() === 0}
      <span>Enable at least one provider</span>
    {:else}
      {#each providers as p}
        {#if prefs.enabled[p.id] !== false}
          <span
            >{p.label}: {statuses[p.id]?.status ?? "idle"}{statuses[p.id]
              ?.message
              ? ` — ${statuses[p.id].message}`
              : ""}</span
          >
        {/if}
      {/each}
    {/if}
  </div>
</main>
```

Style with compact flex CSS in `styles.css` (dark chrome bar, no card chrome).

- [ ] **Step 3: Verify**

Run: `bun run check` → no TS/Svelte errors.  
Run: `bun run tauri:dev` → toggles persist; Clear does not call Rust; Enter sends; Shift+Enter newline; statuses update on Send with stub errors.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte src/styles.css src/lib/tauri.ts src/lib/types.ts src/lib/providers.ts
git commit -m "$(cat <<'EOF'
feat(chrome): add toggles, prompt, Send/New/Clear, status

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 10: Real adapters for ChatGPT, Claude, Gemini, Copilot

**Files:**

- Modify: `adapters/chatgpt.js`, `adapters/claude.js`, `adapters/gemini.js`, `adapters/copilot.js`
- Test: manual smoke checklist per provider (login persist, setPrompt, submit, newChat)

**Interfaces:**

- Consumes: stub IIFE + `waitFor` helper pattern from Task 7
- Produces: working `window.__mwcAdapter` for four providers
- [ ] **Step 1: Implement chatgpt.js**

Replace stub methods with DOM logic. Starting selectors (adjust during smoke if site changed):

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  function setNativeValue(el, value) {
    const proto = window.HTMLTextAreaElement.prototype;
    const desc = Object.getOwnPropertyDescriptor(proto, "value");
    desc?.set?.call(el, value);
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el =
        document.querySelector("#prompt-textarea") ||
        document.querySelector("textarea[data-id='root']") ||
        document.querySelector("div[contenteditable='true']#prompt-textarea") ||
        (await waitFor("textarea, div[contenteditable='true']"));
      if (el.tagName === "TEXTAREA") setNativeValue(el, text);
      else {
        el.focus();
        el.textContent = text;
        el.dispatchEvent(new InputEvent("input", { bubbles: true }));
      }
    },
    async submit() {
      const btn =
        document.querySelector('[data-testid="send-button"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (btn && !btn.disabled) btn.click();
      else {
        const el = document.querySelector(
          "#prompt-textarea, textarea, div[contenteditable='true']"
        );
        el?.dispatchEvent(
          new KeyboardEvent("keydown", { key: "Enter", bubbles: true })
        );
      }
    },
    async newChat() {
      const link =
        document.querySelector('a[href="/"]') ||
        document.querySelector('[data-testid="create-new-chat-button"]') ||
        document.querySelector('button[aria-label*="New chat"]');
      if (!link) throw new Error("chatgpt: new chat control not found");
      link.click();
    }
  };
})();
```

- [ ] **Step 2: Implement claude.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor(
        'div[contenteditable="true"].ProseMirror, div[contenteditable="true"][aria-label*="Write"], textarea'
      );
      el.focus();
      if (el.tagName === "TEXTAREA") {
        el.value = text;
      } else {
        el.textContent = text;
      }
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label="Send Message"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("claude: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('a[href="/new"]') ||
        document.querySelector('button[aria-label*="New chat"]');
      if (!btn) throw new Error("claude: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 3: Implement gemini.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor(
        'rich-textarea textarea, div[contenteditable="true"], textarea'
      );
      el.focus();
      if ("value" in el) {
        el.value = text;
      } else {
        el.textContent = text;
      }
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label*="Send"]') ||
        document.querySelector("button.send-button");
      if (!btn) throw new Error("gemini: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('button[aria-label*="New chat"]') ||
        document.querySelector('a[href*="new"]');
      if (!btn) throw new Error("gemini: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 4: Implement copilot.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor(
        '#userInput, textarea[aria-label*="Message"], textarea, div[contenteditable="true"]'
      );
      el.focus();
      if ("value" in el) el.value = text;
      else el.textContent = text;
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label*="Submit"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("copilot: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('button[aria-label*="New"]') ||
        document.querySelector('a[href*="new"]');
      if (!btn) throw new Error("copilot: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 5: Manual smoke (each of 4)**

For each provider: log in → restart app → still logged in → setPrompt fills composer without sending → submit sends → newChat opens blank conversation. Fix selectors in-file when DOM differs; do not add APIs.

- [ ] **Step 6: Commit**

```bash
git add adapters/chatgpt.js adapters/claude.js adapters/gemini.js adapters/copilot.js
git commit -m "$(cat <<'EOF'
feat(adapters): implement ChatGPT Claude Gemini Copilot DOM adapters

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 11: Real adapters for Z.AI, DeepSeek, Grok

**Files:**

- Modify: `adapters/zai.js`, `adapters/deepseek.js`, `adapters/grok.js`
- Test: same manual smoke checklist as Task 10

**Interfaces:**

- Consumes: adapter contract
- Produces: working adapters for remaining three providers
- [ ] **Step 1: Implement zai.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor('textarea, div[contenteditable="true"]');
      el.focus();
      if ("value" in el) el.value = text;
      else el.textContent = text;
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[type="submit"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("zai: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('button[aria-label*="New"]') ||
        document.querySelector('a[href="/"]');
      if (!btn) throw new Error("zai: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 2: Implement deepseek.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor('textarea, div[contenteditable="true"]');
      el.focus();
      if ("value" in el) el.value = text;
      else el.textContent = text;
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('div[role="button"][aria-label*="Send"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("deepseek: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('div[role="button"][aria-label*="New"]') ||
        document.querySelector('button[aria-label*="New"]');
      if (!btn) throw new Error("deepseek: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 3: Implement grok.js**

```js
(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor('textarea, div[contenteditable="true"]');
      el.focus();
      if ("value" in el) el.value = text;
      else el.textContent = text;
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label*="Send"]') ||
        document.querySelector('button[type="submit"]');
      if (!btn) throw new Error("grok: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('a[href*="new"]') ||
        document.querySelector('button[aria-label*="New"]');
      if (!btn) throw new Error("grok: new chat not found");
      btn.click();
    }
  };
})();
```

- [ ] **Step 4: Manual smoke for zai, deepseek, grok** (login persist, setPrompt, submit, newChat)

- [ ] **Step 5: Commit**

```bash
git add adapters/zai.js adapters/deepseek.js adapters/grok.js
git commit -m "$(cat <<'EOF'
feat(adapters): implement Z.AI DeepSeek Grok DOM adapters

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 12: Linux packaging + README WebKitGTK notes

**Files:**

- Modify: `src-tauri/tauri.conf.json` (confirm targets), `README.md`, optionally `.github/workflows/` release job (Linux only) if adding CI package smoke
- Test: `bun run tauri:build` produces AppImage/deb/rpm artifacts where tooling allows

**Interfaces:**

- Consumes: working app
- Produces: documented runtime deps + bundle outputs
- [ ] **Step 1: Confirm bundle targets**

Ensure `tauri.conf.json` has:

```json
"bundle": {
  "active": true,
  "targets": ["appimage", "deb", "rpm"]
}
```

No `msi`/`dmg` targets.

- [ ] **Step 2: Document WebKitGTK in README**

Replace/extend `README.md` with run instructions and dependency section. Exact package names verified against Tauri v2 Linux prerequisites for the pinned version during implementation. Template content to fill with verified names:

````markdown
# multi-web-chat

Linux desktop app that embeds multiple AI chat websites and sends one prompt to all enabled panes.

## Runtime dependencies (Linux)

Tauri webviews need WebKitGTK. On Debian/Ubuntu (verify versions for your pinned Tauri):

```bash
sudo apt install libwebkit2gtk-4.1-0 libwebkit2gtk-4.1-dev \
  librsvg2-dev patchelf libssl-dev libgtk-3-dev libayatana-appindicator3-dev
```
````

On Fedora:

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel gtk3-devel librsvg2-devel
```

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

Artifacts under `src-tauri/target/release/bundle/` (AppImage, deb, rpm).

````text

During implementation, replace package lists with the ones that actually made `cargo check` / `tauri build` succeed on the build host, citing the Tauri v2 docs URL for that version.

- [ ] **Step 3: Run release build**

Run: `bun run tauri:build`
Expected: exit 0; AppImage (and deb/rpm if tooling present) under bundle dir. Smoke-launch AppImage if runner has display/WebKit.

- [ ] **Step 4: Commit**

```bash
git add README.md src-tauri/tauri.conf.json
git commit -m "$(cat <<'EOF'
docs: document WebKitGTK deps and Linux bundle targets

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
````

---

### Task 13: End-to-end acceptance pass + agents log

**Files:**

- Modify: `.agents/logs/YYYY-MM-DD.md` (implementation notes)
- Test: full manual acceptance against design §5 and §10

**Interfaces:**

- Consumes: completed Tasks 1–12
- Produces: verified v1 acceptance; no new scope
- [ ] **Step 1: Run automated tests**

```bash
cd src-tauri && cargo test
bun run check
```

Expected: all unit tests PASS; svelte-check clean.

- [ ] **Step 2: Manual acceptance checklist**
- [ ] All seven providers listed in order; toggles persist across restart
- [ ] Layout 4+3 with all enabled; reflow for 1–6; empty panes show hint
- [ ] Login once per provider survives restart (session dirs)
- [ ] Send fans out in parallel; per-pane ok/error isolated
- [ ] Empty prompt no-op; Clear only clears chrome textarea
- [ ] New invokes newChat on enabled panes only
- [ ] Disable destroys webview; re-enable restores session
- [ ] Resize recalculates bounds
- [ ] AppImage/deb/rpm build config Linux-only; README lists WebKit packages
- [ ] **Step 3: Update agents log with implementation summary**
- [ ] **Step 4: Final commit if checklist fixes remain; otherwise done**

---

## Self-Review Notes (author)

1. **Spec coverage:** §§3–11,14 mapped to Tasks 1–12; §5 features → Tasks 6–9; §8 adapters → Tasks 7/10/11; §11 packaging → Task 12; §12 testing → unit tasks + Task 13 checklist; non-goals (§2) excluded; security (§13) inherited (local adapters, no APIs).
2. **Placeholder scan:** Removed unused `clamp_chrome` and ambiguous eval bridges; locked Linux `webkit2gtk` poll for adapter results. Remaining “during implementation” notes only cover distro package name verification (explicitly deferred by the design spec §11).
3. **Type consistency:** Provider ids, `PaneStatus`/`PaneStatusKind`, prefs `{ enabled }`, commands `get_providers` / `get_prefs` / `set_provider_enabled` / `send_prompt` / `new_chat_all`, event `pane_status`, webview labels `chrome` + provider id are aligned across Rust and Svelte tasks.
4. **Scope:** Single plan (one shippable Linux app). Adapters are sequenced after shell so each task still yields a testable increment.
5. **Risk gate:** Task 5 Linux absolute-position smoke must pass before toggles/adapters; documents known GTK multiwebview packing issue.
