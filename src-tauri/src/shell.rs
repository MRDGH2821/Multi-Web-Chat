use crate::layout::{compute_layout, CHROME_HEIGHT_DEFAULT};
use crate::prefs::{is_enabled, load_prefs, prefs_path, Prefs};
use crate::registry::all_providers;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl,
    WindowEvent,
};

/// Label used for the bottom chrome (toolbar) webview.
pub const CHROME_LABEL: &str = "chrome";

/// Shared application state managed by Tauri.
pub struct AppState {
    pub prefs: Mutex<Prefs>,
    pub chrome_height: Mutex<f64>,
    /// Provider ids for which a webview currently exists.
    pub living: Mutex<Vec<String>>,
}

/// Creates the main window plus the bottom chrome child webview, manages
/// `AppState`, and wires resize/scale events to `reflow`.
pub fn create_main_shell(app: &AppHandle) -> tauri::Result<()> {
    let window = tauri::window::WindowBuilder::new(app, "main")
        .title("multi-web-chat")
        .inner_size(1280.0, 800.0)
        .build()?;

    let (width, height) = window_logical_size(&window)?;
    let plan = compute_layout(width, height, CHROME_HEIGHT_DEFAULT, &[]);

    let chrome = WebviewBuilder::new(CHROME_LABEL, WebviewUrl::App("index.html".into()));
    window.add_child(
        chrome,
        LogicalPosition::new(plan.chrome.x, plan.chrome.y),
        LogicalSize::new(plan.chrome.width, plan.chrome.height),
    )?;

    let prefs = app
        .path()
        .app_config_dir()
        .ok()
        .map(|dir| load_prefs(&prefs_path(dir)))
        .transpose()
        .ok()
        .flatten()
        .unwrap_or_default();

    app.manage(AppState {
        prefs: Mutex::new(prefs),
        chrome_height: Mutex::new(CHROME_HEIGHT_DEFAULT),
        living: Mutex::new(Vec::new()),
    });

    let app_handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
        ) {
            let _ = reflow(&app_handle);
        }
    });

    Ok(())
}

/// Recomputes the layout for the current window size and repositions the
/// chrome webview plus every living provider webview.
pub fn reflow(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_window("main") else {
        return Ok(());
    };

    let state = app.state::<AppState>();
    let chrome_height = *state.chrome_height.lock().expect("chrome_height poisoned");
    let enabled_ids: Vec<String> = {
        let prefs = state.prefs.lock().expect("prefs poisoned");
        all_providers()
            .iter()
            .filter(|p| is_enabled(&prefs, p.id))
            .map(|p| p.id.to_string())
            .collect()
    };

    let (width, height) = window_logical_size(&window)?;
    let plan = compute_layout(width, height, chrome_height, &enabled_ids);

    if let Some(chrome) = app.get_webview(CHROME_LABEL) {
        // Set position and size atomically via `set_bounds`. Calling
        // `set_position`/`set_size` separately is racy on Linux: each call
        // reads the webview's *current* bounds via a live X11 query (which
        // only reflects the previous GTK size-allocate pass) and merges in
        // the new field, so the second call can clobber the first call's
        // change with a stale value before GTK has re-allocated the widget.
        chrome.set_bounds(Rect {
            position: LogicalPosition::new(plan.chrome.x, plan.chrome.y).into(),
            size: LogicalSize::new(plan.chrome.width, plan.chrome.height).into(),
        })?;
    }

    let living = state.living.lock().expect("living poisoned");
    for (id, rect) in &plan.panes {
        if !living.contains(id) {
            continue;
        }
        if let Some(webview) = app.get_webview(id) {
            webview.set_bounds(Rect {
                position: LogicalPosition::new(rect.x, rect.y).into(),
                size: LogicalSize::new(rect.width, rect.height).into(),
            })?;
        }
    }

    Ok(())
}

fn window_logical_size(window: &tauri::Window) -> tauri::Result<(f64, f64)> {
    let size = window.inner_size()?;
    let scale_factor = window.scale_factor()?;
    Ok((
        size.width as f64 / scale_factor,
        size.height as f64 / scale_factor,
    ))
}

/// Returns (creating if necessary) the on-disk session directory for a
/// provider's persisted webview data. Used by Task 6 when spawning provider
/// webviews.
pub fn session_dir(app: &AppHandle, provider_id: &str) -> tauri::Result<PathBuf> {
    let base = app
        .path()
        .app_data_dir()?
        .join("sessions")
        .join(provider_id);
    std::fs::create_dir_all(&base)?;
    Ok(base)
}
