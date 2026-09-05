import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PaneStatus, ProviderId } from "./types";

export type Prefs = { enabled: Record<string, boolean> };
export type ProviderDto = { id: ProviderId; label: string };
export type PaneStatusPayload = {
  id: string;
  status: PaneStatus;
  message?: string | null;
};

export const getProviders = () => invoke<ProviderDto[]>("get_providers");
export const getPrefs = () => invoke<Prefs>("get_prefs");
export const setProviderEnabled = (id: string, enabled: boolean) =>
  invoke<void>("set_provider_enabled", { id, enabled });
export const sendPrompt = (text: string) =>
  invoke<void>("send_prompt", { text });
export const newChatAll = () => invoke<void>("new_chat_all");

export function onPaneStatus(
  cb: (p: PaneStatusPayload) => void
): Promise<UnlistenFn> {
  return listen<PaneStatusPayload>("pane_status", (e) => cb(e.payload));
}
