use crate::prefs::{save_prefs, set_enabled, Prefs};
use crate::registry::{all_providers, provider};
use crate::shell::{self, AppState};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

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
