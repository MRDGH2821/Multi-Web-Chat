<script lang="ts">
  import { onMount } from "svelte";
  import {
    getPrefs,
    getProviders,
    newChatAll,
    onPaneStatus,
    sendPrompt,
    setProviderEnabled,
    type PaneStatusPayload,
    type Prefs,
    type ProviderDto
  } from "./lib/tauri";
  import type { PaneStatus } from "./lib/types";

  let providers: ProviderDto[] = $state([]);
  let prefs: Prefs = $state({ enabled: {} });
  let prompt = $state("");
  let statuses: Record<string, { status: PaneStatus; message?: string }> =
    $state({});

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      providers = await getProviders();
      prefs = await getPrefs();
      unlisten = await onPaneStatus((p: PaneStatusPayload) => {
        statuses[p.id] = {
          status: p.status,
          message: p.message ?? undefined
        };
        statuses = statuses;
      });
    })();
    return () => unlisten?.();
  });

  const enabledCount = () =>
    providers.filter((p) => prefs.enabled[p.id] !== false).length;

  async function toggle(id: string, enabled: boolean) {
    await setProviderEnabled(id, enabled);
    prefs.enabled[id] = enabled;
    prefs = prefs;
  }

  async function onSend() {
    await sendPrompt(prompt);
  }

  function onClear() {
    prompt = "";
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void onSend();
    }
  }
</script>

<main class="chrome">
  <div class="toggles">
    {#each providers as p}
      <label>
        <input
          type="checkbox"
          checked={prefs.enabled[p.id] !== false}
          onchange={(e) => toggle(p.id, e.currentTarget.checked)}
        />
        {p.label}
      </label>
    {/each}
  </div>
  <div class="row">
    <textarea
      bind:value={prompt}
      onkeydown={onKeydown}
      placeholder="Prompt all enabled providers"
      rows="2"></textarea>
    <button type="button" onclick={onSend}>Send</button>
    <button type="button" onclick={() => newChatAll()}>New</button>
    <button type="button" onclick={onClear}>Clear</button>
  </div>
  <div class="status">
    {#if enabledCount() === 0}
      <span>Enable at least one provider</span>
    {:else}
      {#each providers as p}
        {#if prefs.enabled[p.id] !== false}
          <span
            >{p.label}: {statuses[p.id]?.status ?? "idle"}{statuses[p.id]
              ?.message
              ? ` — ${statuses[p.id].message}`
              : ""}</span
          >
        {/if}
      {/each}
    {/if}
  </div>
</main>
