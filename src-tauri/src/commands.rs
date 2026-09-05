use crate::adapter_runtime;
use crate::prefs::{enabled_provider_ids, save_prefs, set_enabled, Prefs};
use crate::registry::{all_providers, provider};
use crate::shell::{self, AppState};
use crate::status::{PaneStatusEvent, PaneStatusKind, EVENT_PANE_STATUS};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderDto {
    pub id: String,
    pub label: String,
}

/// Returns the full provider registry (id + label) in spec order.
#[tauri::command]
pub fn get_providers() -> Vec<ProviderDto> {
    all_providers()
        .iter()
        .map(|p| ProviderDto {
            id: p.id.to_string(),
            label: p.label.to_string(),
        })
        .collect()
}

/// Returns a snapshot of the current prefs.
#[tauri::command]
pub fn get_prefs(state: State<'_, AppState>) -> Result<Prefs, String> {
    let prefs = state.prefs.lock().map_err(|e| e.to_string())?;
    Ok(prefs.clone())
}

/// Enables or disables a provider's webview, persists prefs, and reflows
/// the window layout.
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
        let path =
            crate::prefs::prefs_path(app.path().app_config_dir().map_err(|e| e.to_string())?);
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

/// Sends `text` to every enabled pane's adapter in parallel. Empty/whitespace
/// prompts and a zero-pane selection are silent no-ops (no status events).
#[tauri::command]
pub async fn send_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let Some(text) = adapter_runtime::normalize_prompt(&text) else {
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

/// Starts a new chat on every enabled pane's adapter in parallel. A
/// zero-pane selection is a silent no-op (no status events).
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
