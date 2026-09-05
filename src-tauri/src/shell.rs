use crate::adapter_runtime::load_adapter_source;
use crate::layout::{compute_layout, CHROME_HEIGHT_DEFAULT};
use crate::prefs::{enabled_provider_ids, load_prefs, prefs_path, Prefs};
use crate::registry::provider;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl, WindowEvent,
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
        .inner_size(1600.0, 960.0)
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

    let startup_ids = enabled_provider_ids(&prefs);

    app.manage(AppState {
        prefs: Mutex::new(prefs),
        chrome_height: Mutex::new(CHROME_HEIGHT_DEFAULT),
        living: Mutex::new(Vec::new()),
    });

    for id in &startup_ids {
        ensure_provider_webview(app, id)?;
    }
    reflow(app)?;

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
        enabled_provider_ids(&prefs)
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

/// Creates (if not already living) the child webview for `provider_id`,
/// loading its start URL with a persistent per-provider `data_directory`,
/// and marks it as living. Does not reposition it — callers should follow
/// up with [`reflow`].
pub fn ensure_provider_webview(app: &AppHandle, provider_id: &str) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    {
        let living = state.living.lock().expect("living poisoned");
        if living.contains(&provider_id.to_string()) {
            return Ok(());
        }
    }

    let Some(window) = app.get_window("main") else {
        return Ok(());
    };

    let Some(p) = provider(provider_id) else {
        return Ok(());
    };

    let data_dir = session_dir(app, provider_id)?;
    let start_url = p
        .start_url
        .parse()
        .expect("registry start_url must be a valid URL");
    let builder = WebviewBuilder::new(provider_id, WebviewUrl::External(start_url))
        .data_directory(data_dir)
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                if let Ok(src) = load_adapter_source(webview.label()) {
                    let _ = webview.eval(src);
                }
            }
        });

    window.add_child(
        builder,
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(1.0, 1.0),
    )?;

    state
        .living
        .lock()
        .expect("living poisoned")
        .push(provider_id.to_string());

    Ok(())
}

/// Closes the child webview for `provider_id` (if living) and removes it
/// from the living set. Callers should follow up with [`reflow`].
pub fn destroy_provider_webview(app: &AppHandle, provider_id: &str) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let was_living = {
        let mut living = state.living.lock().expect("living poisoned");
        if let Some(pos) = living.iter().position(|id| id == provider_id) {
            living.remove(pos);
            true
        } else {
            false
        }
    };

    if was_living {
        if let Some(webview) = app.get_webview(provider_id) {
            webview.close()?;
        }
    }

    Ok(())
}
