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

#[cfg(test)]
mod tests {
    use crate::prefs::{default_prefs, is_enabled, load_prefs, save_prefs, set_enabled};
    use crate::registry::all_providers;

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
}
