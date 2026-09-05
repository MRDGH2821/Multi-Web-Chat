# multi-web-chat — Design Spec

**Date:** 2026-09-05  
**Status:** Approved for implementation planning (pending user review of this written spec)  
**Product working name:** multi-web-chat

## 1. Problem and goal

Users compare answers from several AI chat websites by copy-pasting the same prompt into each site. That is slow and error-prone.

**Goal:** A desktop “browser wrapper” (Linux, Windows, and macOS) that embeds the real AI chat websites (not APIs) and fans one prompt out to all enabled panes in parallel — the same product idea as [GodMode](https://github.com/smol-ai/GodMode) and [llm-god](https://github.com/czhou578/llm-god), implemented with Tauri instead of Electron.

Success for v1: a user can log in once per provider, toggle panes on/off, type one prompt, send it to every enabled site, start new chats across panes, and keep sessions across app restarts.

## 2. Non-goals (v1)

Explicitly out of scope for v1:

- Image / screenshot paste into the shared prompt
- Global hotkey / system tray launcher
- Prompt templates or saved prompt library
- Response scraping, side-by-side compare, or auto-merge of answers
- Signed Apple / Microsoft store distribution (CI produces unsigned Windows and macOS installers)
- API / key-based chat mode (no provider APIs for sending chat)
- Cloud sync of prefs or sessions
- Local history of prompts sent from the chrome UI

## 3. Product decisions

| Decision          | Choice                                                                                                                        |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Product shape     | Desktop browser wrapper; real chat websites in webviews                                                                       |
| Chat transport    | DOM automation via injected JS only — no chat APIs                                                                            |
| UI chrome         | Svelte                                                                                                                        |
| Shell             | Tauri v2 (not Electron, not Dioxus)                                                                                           |
| Target OS         | Linux, Windows, and macOS                                                                                                     |
| Packages          | Linux: AppImage, `.deb`, `.rpm`; Windows: `.msi` and NSIS; macOS: `.dmg`                                                      |
| System dependency | Linux: WebKitGTK; Windows: WebView2; macOS: WKWebView                                                                         |
| Layout            | **Layout A:** bottom chrome; when all 7 panes enabled use a 4+3 grid; when fewer are enabled, reflow into equal-width columns |

## 4. Providers (v1)

Fixed ordered registry (display order = grid fill order, left-to-right, top-to-bottom):

| ID         | Label    | Start URL                       |
| ---------- | -------- | ------------------------------- |
| `chatgpt`  | ChatGPT  | `https://chatgpt.com`           |
| `claude`   | Claude   | `https://claude.ai`             |
| `gemini`   | Gemini   | `https://gemini.google.com`     |
| `copilot`  | Copilot  | `https://copilot.microsoft.com` |
| `zai`      | Z.AI     | `https://chat.z.ai`             |
| `deepseek` | DeepSeek | `https://chat.deepseek.com`     |
| `grok`     | Grok     | `https://grok.x.ai`             |

Provider IDs are stable keys for prefs, data directories, and IPC. Start URLs may be updated in the registry if a vendor changes its primary chat entry point; adapters own any post-load navigation quirks.

## 5. Features (v1)

Minimal surface:

1. **Pane toggles** — enable/disable each provider independently; prefs persist on disk.
2. **Multi-send** — one shared prompt; Send fans out to all currently enabled panes in parallel.
3. **New chat** — invoke each enabled pane’s `newChat` adapter action (fresh conversation on that site).
4. **Clear** — clear the shared chrome prompt field only (does not wipe provider conversations). Label in UI: **Clear**.
5. **Persist logins** — each provider webview uses its own `data_directory` so cookies/local storage survive restarts.
6. **Per-pane status** — chrome shows idle / sending / success / error per provider for the latest send (and load failures when creating a pane).

No other chrome features in v1.

## 6. Architecture

### 6.1 High-level

```text
┌─────────────────────────────────────────────────────────────┐
│ Tauri main window (Rust shell owns layout)                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ ChatGPT  │ │ Claude   │ │ Gemini   │ │ Copilot  │  row 1 │
│  │ webview  │ │ webview  │ │ webview  │ │ webview  │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │ Z.AI webview │ │ DeepSeek     │ │ Grok         │  row 2 │
│  └──────────────┘ └──────────────┘ └──────────────┘         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Chrome webview (Svelte): toggles, prompt, Send,     │    │
│  │ New, Clear, per-pane status                         │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

- **Chrome webview:** one dedicated webview hosting the Svelte UI. It never loads a provider site.
- **Child webviews:** one per _enabled_ provider. Rust creates, positions, and destroys them.
- **IPC:** Svelte ↔ Rust via Tauri commands/events. Rust ↔ provider pages via `eval` of injected adapter scripts.

### 6.2 Why multi-webview (not one iframe grid)

Separate webviews give isolated cookie jars (`data_directory` per provider), independent navigation, and the ability to destroy a disabled pane to reclaim RAM. Provider sites often block or break inside cross-origin iframes; child webviews match how GodMode / llm-god embed sites.

### 6.3 Layout rules (Layout A)

- Chrome strip is fixed to the **bottom** of the main window. Default chrome height: **96px** (enough for toggle row + prompt row); may grow slightly for status text wrapping, capped at **140px**.
- Remaining client area is the pane region.
- **Gap** between panes: **2px**. No app-drawn pane titles or borders in v1 — the loaded site UI is the identity.
- **All 7 enabled:** two rows — top row 4 equal columns, bottom row 3 equal columns (same row height for both rows: pane region height ÷ 2).
- **Fewer than 7 enabled:** single-row or multi-row **equal-column reflow**:
  - Let `n` = number of enabled panes.
  - Prefer one row when `n ≤ 4`.
  - When `n` is 5 or 6, use two rows with as even a split as possible (e.g. 3+2 for 5, 3+3 for 6), equal column widths within each row.
  - When `n` is 0, pane region is empty; Send is a no-op (see errors).
- On window resize or toggle, Rust recomputes bounds for every living child webview and the chrome webview.

### 6.4 Enable / disable lifecycle

- **Enable:** create webview with that provider’s URL and `data_directory`; inject adapter after load (or on `dom-ready` equivalent); reflow.
- **Disable:** destroy the webview immediately to free RAM; keep prefs and on-disk session data so re-enable restores login; reflow.
- Destroyed panes do not receive Send / New.

## 7. Components

| Component             | Owner                                        | Responsibility                                                                                                      |
| --------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **Shell**             | Rust (Tauri)                                 | Main window; create/destroy/bounds of chrome + provider webviews; layout math; command handlers; emit status events |
| **Chrome UI**         | Svelte                                       | Prompt textarea, provider toggles, Send / New / Clear, per-pane status display; calls Tauri commands                |
| **Provider registry** | Rust (+ shared constants usable from Svelte) | Ordered list of id, label, start URL, adapter script path                                                           |
| **Inject adapters**   | Per-provider JS                              | DOM helpers: `setPrompt`, `submit`, `newChat`; loaded into that provider’s webview only                             |
| **Session store**     | OS filesystem via webview `data_directory`   | Cookies / site storage per provider — not a custom DB                                                               |
| **Prefs store**       | Rust, on-disk JSON                           | Which panes are enabled only; chrome height is not persisted (runtime default 96px, cap 140px); no prompt history   |

### 7.1 Prefs file

Path: application config dir (Tauri `app_config_dir`) / `prefs.json`.

```json
{
  "enabled": {
    "chatgpt": true,
    "claude": true,
    "gemini": true,
    "copilot": true,
    "zai": true,
    "deepseek": true,
    "grok": true
  }
}
```

Missing keys default to `true` (all on for first launch). Unknown keys ignored. No cloud; local file only.

### 7.2 Session directories

Path pattern: application data dir / `sessions` / `<provider_id>/`.

Each provider webview is configured with that directory as its data directory so logins persist independently.

## 8. Inject adapter contract

Each adapter is a JS module (or IIFE) evaluated in the provider webview. It must attach `window.__mwcAdapter` implementing:

```ts
type PaneStatus = "idle" | "sending" | "ok" | "error";

interface MwcAdapter {
  /** Put text into the site’s composer. Must not submit. */
  setPrompt(text: string): Promise<void> | void;
  /** Submit the current composer contents (click send / Enter as appropriate). */
  submit(): Promise<void> | void;
  /** Navigate UI to a new blank conversation. */
  newChat(): Promise<void> | void;
}
```

Rules:

- Adapters must not call home to our servers; they only manipulate the loaded site.
- Selectors and heuristics live only in the adapter file for that provider.
- If `setPrompt` / `submit` / `newChat` throws or rejects, Rust records that pane as `error` with a short message and continues other panes.
- Adapters may no-op `newChat` with a rejected promise if the site UI cannot be driven yet (still counts as pane error for that action).

Rust orchestration always: `setPrompt(text)` then `submit()` for Send; only `newChat()` for New.

## 9. Data flow

### 9.1 Send

1. User edits prompt in chrome; clicks **Send** (or presses Enter in the prompt when not shifted — **Enter sends; Shift+Enter inserts newline**).
2. Chrome calls `send_prompt({ text })`.
3. Rust: if `text` trimmed is empty → return immediately (no-op, no status churn).
4. Rust: for each enabled webview **in parallel**:
   - set pane status `sending`
   - `eval` adapter `setPrompt` then `submit`
   - on success → `ok`; on failure → `error` with message
5. Rust emits `pane_status` events; chrome updates UI. Failures are isolated — one pane error does not cancel others.

### 9.2 New chat

1. Chrome calls `new_chat_all`.
2. Rust fans out `newChat()` to enabled panes in parallel with the same status event pattern.

### 9.3 Clear

1. Chrome clears its local prompt state only. No Rust command required.

### 9.4 Toggle

1. Chrome calls `set_provider_enabled({ id, enabled })`.
2. Rust updates prefs on disk, creates or destroys webview, reflows, emits layout/status as needed.

## 10. Error handling and edge cases

| Case                           | Behavior                                                                                     |
| ------------------------------ | -------------------------------------------------------------------------------------------- |
| Empty / whitespace-only prompt | Send is a no-op                                                                              |
| Zero panes enabled             | Send / New are no-ops; chrome may show a single hint string (“Enable at least one provider”) |
| Adapter failure                | That pane → `error`; others continue                                                         |
| Webview crash / load failure   | Pane → `error`; user can toggle off/on to recreate                                           |
| Site DOM change breaks adapter | Same as adapter failure; fix is an adapter update (expected maintenance)                     |
| Window resize                  | Recalculate all bounds                                                                       |
| Disable pane                   | Destroy webview; save RAM; session files remain on disk                                      |
| Re-enable pane                 | New webview + same `data_directory` → login restored if cookies valid                        |

No modal dialogs required in v1; status text in chrome is enough.

## 11. Packaging and runtime (Linux)

- Ship **AppImage**, **`.deb`**, and **`.rpm`** via Tauri bundler configuration.
- Prefer single-file / AppImage distribution for portable use; `.deb` / `.rpm` for distro integration.
- Document in README: WebKitGTK (webkit2gtk) and related packages must be installed on the host — Tauri Linux webviews depend on system WebKit. Exact package names vary by distro (e.g. Debian/Ubuntu `webkit2gtk` / `libwebkit2gtk-*`); the README will list verified package names at implementation time based on the Tauri v2 Linux docs for the pinned Tauri version.
- No macOS/Windows targets in CI or bundle config for v1.

## 12. Testing strategy

| Layer         | What                                                                                      |
| ------------- | ----------------------------------------------------------------------------------------- |
| Unit          | Layout math (given window size + enabled set → expected bounds); prefs load/save defaults |
| Adapter smoke | Manual checklist per provider: login persists, setPrompt, submit, newChat                 |
| Integration   | Rust command tests with mock/eval harness where feasible; otherwise manual                |
| Packaging     | Build AppImage on CI or release job; smoke-launch where the runner allows                 |

Automated end-to-end against live provider sites is not required for v1 (sites change and need auth). Manual smoke after adapter changes is the acceptance bar.

## 13. Security and privacy notes

- No chat APIs and no shipping of prompts to infrastructure we control — prompts go only into the user’s loaded provider pages.
- Separate data directories reduce cross-provider cookie leakage.
- Injected scripts are local app assets, not remote-loaded.
- Users should treat the app with the same trust as a browser profile holding AI-site logins.

## 14. Implementation boundaries (for later planning)

This spec intentionally stops at design. The next artifact should be an implementation plan covering: Tauri v2 + Svelte project scaffold, registry + prefs, multi-webview shell + layout, adapter stubs then per-provider adapters, desktop bundle targets (Linux / Windows / macOS), and README webview runtime notes.

Do not expand v1 scope during planning without revisiting this document.

## 15. References

- [smol-ai/GodMode](https://github.com/smol-ai/GodMode) — Electron multi-webview AI chat browser; shared prompt fan-out
- [czhou578/llm-god](https://github.com/czhou578/llm-god) — similar multi-prompt desktop app (Electron)

## 16. Decision log (approved)

- Desktop browser wrapper; no chat APIs
- Tauri v2 + Svelte chrome (not Electron, not Dioxus)
- Providers: ChatGPT, Claude, Gemini, Copilot, Z.AI, DeepSeek, Grok
- Desktop packages: Linux AppImage/deb/rpm, Windows MSI/NSIS, macOS DMG; document WebKitGTK, WebView2, and WKWebView
- Features: toggles, New, Clear (prompt only), persist logins, multi-send
- Non-goals: image paste, global hotkey, templates, scrape/compare, signed store distribution, API mode
- Architecture: chrome webview + per-provider child webviews; Rust bounds; JS inject adapters; per-provider data dirs
- Layout A: bottom chrome; 4+3 when all seven on; equal-column reflow otherwise
- Destroy webview when disabled; empty prompt no-op; isolated per-pane errors
