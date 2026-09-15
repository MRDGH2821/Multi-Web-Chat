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
