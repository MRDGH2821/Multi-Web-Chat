import type { ProviderId } from "./types";

export const PROVIDERS: { id: ProviderId; label: string }[] = [
  { id: "chatgpt", label: "ChatGPT" },
  { id: "claude", label: "Claude" },
  { id: "copilot", label: "Copilot" },
  { id: "copilot-gh", label: "Copilot (GH)" },
  { id: "felo", label: "Felo" },
  { id: "gemini", label: "Gemini" },
  { id: "genspark", label: "Genspark" },
  { id: "grok", label: "Grok" },
  { id: "liner", label: "Liner" },
  { id: "meta", label: "Meta AI" },
  { id: "mistral", label: "Mistral" },
  { id: "perplexity", label: "Perplexity" },
  { id: "poe", label: "Poe" },
  { id: "qwen", label: "Qwen Chat" },
  { id: "zai", label: "Z.ai" }
];
