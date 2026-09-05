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
        "gemini" => include_str!("../../adapters/gemini.js"),
        "copilot" => include_str!("../../adapters/copilot.js"),
        "zai" => include_str!("../../adapters/zai.js"),
        "deepseek" => include_str!("../../adapters/deepseek.js"),
        "grok" => include_str!("../../adapters/grok.js"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_prompt_js_escapes_quotes_and_newlines() {
        let js = call_set_prompt_js("say \"hi\"\nthere");
        assert!(js.contains("say \\\"hi\\\""));
        assert!(js.contains("\\n"));
    }

    #[test]
    fn load_adapter_source_returns_all_providers() {
        for id in [
            "chatgpt", "claude", "gemini", "copilot", "zai", "deepseek", "grok",
        ] {
            let src = load_adapter_source(id).expect("known provider");
            assert!(src.contains("window.__mwcAdapter"));
            assert!(src.contains(&format!("stub: {id} selectors not implemented")));
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
