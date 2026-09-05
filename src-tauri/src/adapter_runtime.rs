use serde::Deserialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
}

pub fn load_adapter_source(provider_id: &str) -> Result<&'static str, AdapterError> {
    Ok(match provider_id {
        "chatgpt" => include_str!("../../adapters/chatgpt.js"),
        "claude" => include_str!("../../adapters/claude.js"),
        "copilot" => include_str!("../../adapters/copilot.js"),
        "copilot-gh" => include_str!("../../adapters/copilot-gh.js"),
        "felo" => include_str!("../../adapters/felo.js"),
        "gemini" => include_str!("../../adapters/gemini.js"),
        "genspark" => include_str!("../../adapters/genspark.js"),
        "grok" => include_str!("../../adapters/grok.js"),
        "liner" => include_str!("../../adapters/liner.js"),
        "meta" => include_str!("../../adapters/meta-ai.js"),
        "mistral" => include_str!("../../adapters/mistral.js"),
        "perplexity" => include_str!("../../adapters/perplexity.js"),
        "poe" => include_str!("../../adapters/poe.js"),
        "qwen" => include_str!("../../adapters/qwen-chat.js"),
        "zai" => include_str!("../../adapters/zai.js"),
        _ => return Err(AdapterError::UnknownProvider(provider_id.to_string())),
    })
}

/// Returns the adapter IIFE source ready for `webview.eval` (source is already an IIFE).
pub fn inject_adapter_script(source: &str) -> String {
    source.to_string()
}

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

/// Trims `text` and returns `None` for an empty (or whitespace-only) prompt,
/// so callers can treat it as a no-op without touching pane status.
pub fn normalize_prompt(text: &str) -> Option<String> {
    let t = text.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct LastResult {
    pub ok: bool,
    pub error: Option<String>,
}

/// Runs `setPrompt` + `submit` on the pane's adapter and awaits the
/// `window.__mwcLastResult` bridge for success/failure.
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

/// Runs `newChat` on the pane's adapter and awaits the
/// `window.__mwcLastResult` bridge for success/failure.
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

/// Linux: read `window.__mwcLastResult` via webkit2gtk `evaluate_javascript`,
/// polling until the adapter bridge sets it (or `timeout` elapses).
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
                        w.inner().evaluate_javascript(
                            "window.__mwcLastResult ? JSON.stringify(window.__mwcLastResult) : null",
                            None,
                            None,
                            None::<&webkit2gtk::gio::Cancellable>,
                            move |res| {
                                let mapped = res
                                    .map_err(|e| e.to_string())
                                    .map(|value| value.to_string());
                                let _ = tx.send(mapped.map(|s| {
                                    if s == "null" || s.is_empty() {
                                        None
                                    } else {
                                        Some(s)
                                    }
                                }));
                            },
                        );
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        let _ = tx.send(Err("multi-web-chat v1 supports Linux only".into()));
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        let recv_result = tokio::task::spawn_blocking(move || {
            rx.recv_timeout(std::time::Duration::from_millis(200))
        })
        .await
        .map_err(|e| e.to_string())?;

        match recv_result {
            Ok(Ok(Some(raw))) => {
                let parsed = serde_json::from_str::<LastResult>(&raw).map_err(|e| e.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_prompt_is_noop_guard() {
        assert!(normalize_prompt("  \n\t ").is_none());
        assert_eq!(normalize_prompt("hello").as_deref(), Some("hello"));
    }

    #[test]
    fn set_prompt_js_escapes_quotes_and_newlines() {
        let js = call_set_prompt_js("say \"hi\"\nthere");
        assert!(js.contains("say \\\"hi\\\""));
        assert!(js.contains("\\n"));
    }

    #[test]
    fn load_adapter_source_returns_all_providers() {
        for p in crate::registry::all_providers() {
            let src = load_adapter_source(p.id).expect("known provider");
            assert!(src.contains("window.__mwcAdapter"));
        }
    }

    #[test]
    fn load_adapter_source_real_providers_have_no_stub_marker() {
        for p in crate::registry::all_providers() {
            let src = load_adapter_source(p.id).expect("known provider");
            assert!(!src.contains("stub:"));
        }
    }

    #[test]
    fn load_adapter_source_unknown_provider() {
        assert_eq!(
            load_adapter_source("nope"),
            Err(AdapterError::UnknownProvider("nope".into()))
        );
    }
}
