<!--
  SyncDialog.svelte - Sync preview and confirmation dialog

  Displays a preview of changes when re-importing from a source file.
  Users can selectively accept additions and changes before applying.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X, Plus, Pencil, RefreshCw, Loader2 } from "lucide-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import type { SyncPreview, ReimportSummary } from "../types";

  interface Props {
    projectId: string;
    syncPreview: SyncPreview;
    onClose: () => void;
    onSyncComplete: (summary: ReimportSummary) => void;
  }

  let { projectId, syncPreview, onClose, onSyncComplete }: Props = $props();

  let syncing = $state(false);
  let selectedChanges = new SvelteSet<string>();
  let selectedAdditions = new SvelteSet<string>();

  // Default: all additions selected, no changes selected
  $effect(() => {
    selectedChanges.clear();
    selectedAdditions.clear();
    for (const addition of syncPreview.additions) {
      selectedAdditions.add(addition.id);
    }
  });

  function toggleChange(changeId: string) {
    if (selectedChanges.has(changeId)) {
      selectedChanges.delete(changeId);
    } else {
      selectedChanges.add(changeId);
    }
  }

  function selectAllChanges() {
    selectedChanges.clear();
    for (const change of syncPreview.changes) {
      selectedChanges.add(change.id);
    }
  }

  function deselectAllChanges() {
    selectedChanges.clear();
  }

  function toggleAddition(additionId: string) {
    if (selectedAdditions.has(additionId)) {
      selectedAdditions.delete(additionId);
    } else {
      selectedAdditions.add(additionId);
    }
  }

  function selectAllAdditions() {
    selectedAdditions.clear();
    for (const addition of syncPreview.additions) {
      selectedAdditions.add(addition.id);
    }
  }

  function deselectAllAdditions() {
    selectedAdditions.clear();
  }

  async function applySync() {
    syncing = true;
    try {
      const summary = await invoke<ReimportSummary>("apply_sync", {
        projectId,
        acceptedChangeIds: Array.from(selectedChanges),
        acceptedAdditionIds: Array.from(selectedAdditions),
      });
      onSyncComplete(summary);
    } catch (e) {
      console.error("Failed to apply sync:", e);
    } finally {
      syncing = false;
    }
  }
</script>

<div
  data-testid="sync-preview-dialog"
  class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-6 md:p-10"
  role="dialog"
  aria-modal="true"
>
  <div
    class="bg-bg-panel rounded-2xl w-full h-full max-w-7xl flex flex-col shadow-2xl border border-white/5 overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-8 py-6 border-b border-bg-card/50">
      <div>
        <h2 class="text-2xl font-heading font-semibold text-text-primary">Sync from Source</h2>
        <p class="text-text-secondary text-sm mt-1">Review and select items to import</p>
      </div>
      <button
        onclick={onClose}
        class="p-2 text-text-secondary hover:text-text-primary rounded-lg hover:bg-bg-card transition-colors"
      >
        <X class="w-6 h-6" />
      </button>
    </div>

    <!-- Content - Two Column Layout -->
    {#if syncPreview.additions.length === 0 && syncPreview.changes.length === 0}
      <!-- No changes message -->
      <div class="flex-1 flex items-center justify-center">
        <div class="text-center py-12">
          <div
            class="w-16 h-16 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4"
          >
            <RefreshCw class="w-8 h-8 text-green-500" />
          </div>
          <p class="text-text-primary text-lg font-medium">All synced!</p>
          <p class="text-text-secondary text-sm mt-1">
            Your project is up to date with the source file.
          </p>
        </div>
      </div>
    {:else}
      <div
        class="flex-1 overflow-hidden grid grid-cols-1 lg:grid-cols-2 divide-y lg:divide-y-0 lg:divide-x divide-bg-card/50"
      >
        <!-- Left Column: Additions -->
        <div class="flex flex-col min-h-0">
          <div class="flex items-center justify-between px-6 py-4 border-b border-bg-card/30">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-green-500/10 flex items-center justify-center">
                <Plus class="w-4 h-4 text-green-500" />
              </div>
              <div>
                <h3 class="text-sm font-medium text-text-primary">New Items</h3>
                <p class="text-xs text-text-secondary">
                  {selectedAdditions.size} of {syncPreview.additions.length} selected
                </p>
              </div>
            </div>
            {#if syncPreview.additions.length > 0}
              <div class="flex gap-2 text-xs">
                <button
                  onclick={selectAllAdditions}
                  class="text-text-secondary hover:text-accent transition-colors">All</button
                >
                <span class="text-text-secondary/30">|</span>
                <button
                  onclick={deselectAllAdditions}
                  class="text-text-secondary hover:text-accent transition-colors">None</button
                >
              </div>
            {/if}
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-2">
            {#if syncPreview.additions.length === 0}
              <div class="text-center py-12 text-text-secondary">
                <p>No new items to import</p>
              </div>
            {:else}
              {#each syncPreview.additions as addition (addition.id)}
                <label
                  class="flex items-center gap-4 p-4 bg-bg-card/50 rounded-xl cursor-pointer hover:bg-bg-card transition-colors group"
                >
                  <input
                    type="checkbox"
                    checked={selectedAdditions.has(addition.id)}
                    onchange={() => toggleAddition(addition.id)}
                    class="w-5 h-5 rounded border-2 border-bg-card bg-transparent text-accent focus:ring-accent focus:ring-offset-0 cursor-pointer"
                  />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span
                        class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-500/10 text-green-500 uppercase"
                      >
                        {addition.item_type}
                      </span>
                      <span class="text-text-primary font-medium truncate">{addition.title}</span>
                    </div>
                    {#if addition.parent_title}
                      <p class="text-xs text-text-secondary mt-1">in {addition.parent_title}</p>
                    {/if}
                  </div>
                </label>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Right Column: Changes -->
        <div class="flex flex-col min-h-0">
          <div class="flex items-center justify-between px-6 py-4 border-b border-bg-card/30">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center">
                <Pencil class="w-4 h-4 text-amber-500" />
              </div>
              <div>
                <h3 class="text-sm font-medium text-text-primary">Changes</h3>
                <p class="text-xs text-text-secondary">
                  {selectedChanges.size} of {syncPreview.changes.length} selected
                </p>
              </div>
            </div>
            {#if syncPreview.changes.length > 0}
              <div class="flex gap-2 text-xs">
                <button
                  onclick={selectAllChanges}
                  class="text-text-secondary hover:text-accent transition-colors">All</button
                >
                <span class="text-text-secondary/30">|</span>
                <button
                  onclick={deselectAllChanges}
                  class="text-text-secondary hover:text-accent transition-colors">None</button
                >
              </div>
            {/if}
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-2">
            {#if syncPreview.changes.length === 0}
              <div class="text-center py-12 text-text-secondary">
                <p>No changes detected</p>
              </div>
            {:else}
              {#each syncPreview.changes as change (change.id)}
                <label
                  class="flex items-start gap-4 p-4 bg-bg-card/50 rounded-xl cursor-pointer hover:bg-bg-card transition-colors"
                >
                  <input
                    type="checkbox"
                    checked={selectedChanges.has(change.id)}
                    onchange={() => toggleChange(change.id)}
                    class="mt-0.5 w-5 h-5 rounded border-2 border-bg-card bg-transparent text-accent focus:ring-accent focus:ring-offset-0 cursor-pointer"
                  />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <span
                        class="px-2 py-0.5 text-xs font-medium rounded-full bg-amber-500/10 text-amber-500 uppercase"
                      >
                        {change.item_type}
                      </span>
                      <span class="text-text-primary font-medium truncate">{change.item_title}</span
                      >
                      <span class="text-text-secondary/60 text-xs">({change.field})</span>
                    </div>
                    <div class="text-sm space-y-1 font-mono">
                      <div class="flex gap-2 text-red-400/80">
                        <span class="flex-shrink-0">-</span>
                        <span class="line-through opacity-60 truncate"
                          >{change.current_value || "(empty)"}</span
                        >
                      </div>
                      <div class="flex gap-2 text-green-400/80">
                        <span class="flex-shrink-0">+</span>
                        <span class="truncate">{change.new_value || "(empty)"}</span>
                      </div>
                    </div>
                  </div>
                </label>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Footer -->
    <div
      class="flex items-center justify-between px-8 py-5 border-t border-bg-card/50 bg-bg-card/20"
    >
      <p class="text-text-secondary text-sm">
        {selectedAdditions.size + selectedChanges.size} item{selectedAdditions.size +
          selectedChanges.size !==
        1
          ? "s"
          : ""} selected
      </p>
      <div class="flex gap-4">
        <button
          onclick={onClose}
          class="px-6 py-2.5 text-text-secondary hover:text-text-primary rounded-lg hover:bg-bg-card transition-colors"
        >
          Cancel
        </button>
        <button
          data-testid="sync-confirm"
          onclick={applySync}
          disabled={syncing || (selectedAdditions.size === 0 && selectedChanges.size === 0)}
          class="px-6 py-2.5 bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        >
          {#if syncing}
            <Loader2 class="w-4 h-4 animate-spin" />
            Syncing...
          {:else}
            Apply Sync
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>
