#[derive(Debug, Clone, Copy)]
pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub start_url: &'static str,
}

/// International AI providers in screenshot display order.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "chatgpt",
        label: "ChatGPT",
        start_url: "https://chatgpt.com",
    },
    Provider {
        id: "claude",
        label: "Claude",
        start_url: "https://claude.ai",
    },
    Provider {
        id: "copilot",
        label: "Copilot",
        start_url: "https://copilot.microsoft.com",
    },
    Provider {
        id: "copilot-gh",
        label: "Copilot (GH)",
        start_url: "https://github.com/copilot",
    },
    Provider {
        id: "felo",
        label: "Felo",
        start_url: "https://chat.felo.ai",
    },
    Provider {
        id: "gemini",
        label: "Gemini",
        start_url: "https://gemini.google.com",
    },
    Provider {
        id: "genspark",
        label: "Genspark",
        start_url: "https://www.genspark.ai",
    },
    Provider {
        id: "grok",
        label: "Grok",
        start_url: "https://grok.x.ai",
    },
    Provider {
        id: "liner",
        label: "Liner",
        start_url: "https://liner.com",
    },
    Provider {
        id: "meta",
        label: "Meta AI",
        start_url: "https://www.meta.ai",
    },
    Provider {
        id: "mistral",
        label: "Mistral",
        start_url: "https://chat.mistral.ai",
    },
    Provider {
        id: "perplexity",
        label: "Perplexity",
        start_url: "https://www.perplexity.ai",
    },
    Provider {
        id: "poe",
        label: "Poe",
        start_url: "https://poe.com",
    },
    Provider {
        id: "qwen",
        label: "Qwen Chat",
        start_url: "https://chat.qwen.ai",
    },
    Provider {
        id: "zai",
        label: "Z.ai",
        start_url: "https://chat.z.ai",
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
    fn registry_has_international_providers_in_screenshot_order() {
        let ids: Vec<&str> = all_providers().iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            vec![
                "chatgpt",
                "claude",
                "copilot",
                "copilot-gh",
                "felo",
                "gemini",
                "genspark",
                "grok",
                "liner",
                "meta",
                "mistral",
                "perplexity",
                "poe",
                "qwen",
                "zai",
            ]
        );
        assert_eq!(all_providers().len(), 15);
        assert!(provider("deepseek").is_none());
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(provider("nope").is_none());
    }
}
