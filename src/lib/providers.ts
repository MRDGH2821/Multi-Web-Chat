import type { ProviderId } from "./types";

export const PROVIDERS: { id: ProviderId; label: string }[] = [
  { id: "chatgpt", label: "ChatGPT" },
  { id: "claude", label: "Claude" },
  { id: "gemini", label: "Gemini" },
  { id: "copilot", label: "Copilot" },
  { id: "zai", label: "Z.AI" },
  { id: "deepseek", label: "DeepSeek" },
  { id: "grok", label: "Grok" }
];
