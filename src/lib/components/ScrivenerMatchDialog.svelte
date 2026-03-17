<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Loader2, Link2, AlertCircle, X } from "lucide-svelte";
  import type { ScrivenerMatchPreview } from "../types";

  let {
    projectId,
    scrivPath,
    onConfirm,
    onCancel,
  }: {
    projectId: string;
    scrivPath: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let matches = $state<ScrivenerMatchPreview[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  const matchedCount = $derived(matches.filter((m) => m.matched_scriv_title).length);
  const unmatchedCount = $derived(matches.filter((m) => !m.matched_scriv_title).length);

  onMount(async () => {
    try {
      matches = await invoke<ScrivenerMatchPreview[]>("preview_scrivener_matches", {
        projectId,
        scrivPath,
      });
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
  onclick={onCancel}
  onkeydown={(e) => e.key === "Escape" && onCancel()}
  role="presentation"
  tabindex="-1"
>
  <div
    class="bg-bg-panel border border-bg-card rounded-2xl shadow-2xl w-xl max-h-[80vh] flex flex-col"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="flex items-center justify-between px-6 py-4 border-b border-bg-card">
      <div>
        <h2 class="text-lg font-semibold text-text-primary">Scrivener Match Preview</h2>
        <p class="text-xs text-text-secondary mt-0.5">
          Review how scenes will map to Scrivener documents
        </p>
      </div>
      <button
        onclick={onCancel}
        class="p-1 text-text-secondary hover:text-text-primary"
        aria-label="Close"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-4">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <Loader2 class="w-6 h-6 animate-spin text-accent" />
          <span class="ml-2 text-text-secondary">Analyzing matches...</span>
        </div>
      {:else if loadError}
        <div class="flex items-center gap-2 text-red-400 py-4">
          <AlertCircle class="w-5 h-5 shrink-0" />
          <p class="text-sm">{loadError}</p>
        </div>
      {:else}
        <div class="flex items-center gap-4 mb-4">
          <span class="text-xs text-text-secondary">
            <span class="text-green-400 font-medium">{matchedCount}</span> matched
          </span>
          {#if unmatchedCount > 0}
            <span class="text-xs text-text-secondary">
              <span class="text-amber-400 font-medium">{unmatchedCount}</span> unmatched (will be created)
            </span>
          {/if}
        </div>

        <div class="space-y-1">
          {#each matches as m}
            {@const isMatched = !!m.matched_scriv_title}
            {@const MatchIcon = isMatched ? Link2 : AlertCircle}
            <div
              class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm {isMatched
                ? 'bg-green-500/5'
                : 'bg-amber-500/5'}"
            >
              <MatchIcon
                class="w-3.5 h-3.5 {isMatched ? 'text-green-400' : 'text-amber-400'} shrink-0"
              />
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-text-secondary text-xs truncate">{m.chapter_title}</span>
                  <span class="text-text-secondary/40">/</span>
                  <span class="text-text-primary truncate">{m.scene_title}</span>
                </div>
                <p class="text-xs mt-0.5 {isMatched ? 'text-text-secondary' : 'text-amber-400/70'}">
                  {isMatched
                    ? `→ ${m.matched_scriv_title} (via ${m.match_method === "source_id" ? "ID" : "title"})`
                    : "New document"}
                </p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-bg-card">
      <button
        onclick={onCancel}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={onConfirm}
        disabled={loading || !!loadError}
        class="px-4 py-2 bg-accent text-white text-sm font-medium rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50"
      >
        Proceed with Export
      </button>
    </div>
  </div>
</div>
