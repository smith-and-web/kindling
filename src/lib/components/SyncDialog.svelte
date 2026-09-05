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
  import Tooltip from "./Tooltip.svelte";

  interface Props {
    projectId: string;
    syncPreview: SyncPreview;
    onClose: () => void;
    onSyncComplete: (summary: ReimportSummary) => void;
  }

  let { projectId, syncPreview, onClose, onSyncComplete }: Props = $props();

  let syncing = $state(false);
  let error = $state<string | null>(null);
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
    error = null;
    try {
      const summary = await invoke<ReimportSummary>("apply_sync", {
        projectId,
        acceptedChangeIds: Array.from(selectedChanges),
        acceptedAdditionIds: Array.from(selectedAdditions),
      });
      onSyncComplete(summary);
    } catch (e) {
      console.error("Failed to apply sync:", e);
      error = String(e);
    } finally {
      syncing = false;
    }
  }
</script>

<div
  data-testid="sync-preview-dialog"
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal p-6 md:p-10"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-2xl w-full h-full max-w-7xl flex flex-col shadow-press-overlay border border-press-border overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-8 py-6 border-b border-press-border/50">
      <div>
        <h2 class="text-press-h2 font-heading font-semibold text-press-text">Sync from Source</h2>
        <p class="text-press-muted text-press-ui mt-1">Review and select items to import</p>
      </div>
      <Tooltip text="Close" position="left">
        <button
          data-testid="sync-dialog-close"
          onclick={onClose}
          class="p-2 text-press-muted hover:text-press-text rounded-lg hover:bg-press-sunken transition-colors"
          aria-label="Close"
        >
          <X class="w-6 h-6" />
        </button>
      </Tooltip>
    </div>

    <!-- Content - Two Column Layout -->
    {#if syncPreview.additions.length === 0 && syncPreview.changes.length === 0}
      <!-- No changes message -->
      <div class="flex-1 flex items-center justify-center">
        <div class="text-center py-12">
          <div
            class="w-16 h-16 rounded-full bg-press-success-wash flex items-center justify-center mx-auto mb-4"
          >
            <RefreshCw class="w-8 h-8 text-press-success" />
          </div>
          <p class="text-press-text text-press-body-lg font-medium">All synced!</p>
          <p class="text-press-muted text-press-ui mt-1">
            Your project is up to date with the source file.
          </p>
        </div>
      </div>
    {:else}
      <div
        class="flex-1 overflow-hidden grid grid-cols-1 lg:grid-cols-2 divide-y lg:divide-y-0 lg:divide-x divide-press-border"
      >
        <!-- Left Column: Additions -->
        <div class="flex flex-col min-h-0">
          <div class="flex items-center justify-between px-6 py-4 border-b border-press-border/30">
            <div class="flex items-center gap-3">
              <div
                class="w-8 h-8 rounded-lg bg-press-success-wash flex items-center justify-center"
              >
                <Plus class="w-4 h-4 text-press-success" />
              </div>
              <div>
                <h3 class="text-press-ui font-medium text-press-text">New Items</h3>
                <p class="text-press-eyebrow text-press-muted">
                  {selectedAdditions.size} of {syncPreview.additions.length} selected
                </p>
              </div>
            </div>
            {#if syncPreview.additions.length > 0}
              <div class="flex gap-2 text-press-eyebrow">
                <Tooltip text="Select all" position="bottom">
                  <button
                    onclick={selectAllAdditions}
                    class="text-press-muted hover:text-press-accent-text transition-colors"
                    >All</button
                  >
                </Tooltip>
                <span class="text-press-muted">|</span>
                <Tooltip text="Deselect all" position="bottom">
                  <button
                    onclick={deselectAllAdditions}
                    class="text-press-muted hover:text-press-accent-text transition-colors"
                    >None</button
                  >
                </Tooltip>
              </div>
            {/if}
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-2">
            {#if syncPreview.additions.length === 0}
              <div class="text-center py-12 text-press-muted">
                <p>No new items to import</p>
              </div>
            {:else}
              {#each syncPreview.additions as addition (addition.id)}
                <label
                  class="flex items-center gap-4 p-4 bg-press-sunken rounded-xl cursor-pointer hover:bg-press-sunken transition-colors group"
                >
                  <input
                    type="checkbox"
                    checked={selectedAdditions.has(addition.id)}
                    onchange={() => toggleAddition(addition.id)}
                    class="w-5 h-5 rounded border-2 border-press-border bg-transparent text-press-accent-text focus:ring-press-focus focus:ring-offset-0 cursor-pointer"
                  />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span
                        class="px-2 py-0.5 text-press-eyebrow font-medium rounded-full bg-press-success-wash text-press-success uppercase"
                      >
                        {addition.item_type}
                      </span>
                      <span class="text-press-text font-medium truncate">{addition.title}</span>
                    </div>
                    {#if addition.parent_title}
                      <p class="text-press-eyebrow text-press-muted mt-1">
                        in {addition.parent_title}
                      </p>
                    {/if}
                  </div>
                </label>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Right Column: Changes -->
        <div class="flex flex-col min-h-0">
          <div class="flex items-center justify-between px-6 py-4 border-b border-press-border/30">
            <div class="flex items-center gap-3">
              <div
                class="w-8 h-8 rounded-lg bg-press-warning-wash flex items-center justify-center"
              >
                <Pencil class="w-4 h-4 text-press-warning" />
              </div>
              <div>
                <h3 class="text-press-ui font-medium text-press-text">Changes</h3>
                <p class="text-press-eyebrow text-press-muted">
                  {selectedChanges.size} of {syncPreview.changes.length} selected
                </p>
              </div>
            </div>
            {#if syncPreview.changes.length > 0}
              <div class="flex gap-2 text-press-eyebrow">
                <Tooltip text="Select all" position="bottom">
                  <button
                    onclick={selectAllChanges}
                    class="text-press-muted hover:text-press-accent-text transition-colors"
                    >All</button
                  >
                </Tooltip>
                <span class="text-press-muted">|</span>
                <Tooltip text="Deselect all" position="bottom">
                  <button
                    onclick={deselectAllChanges}
                    class="text-press-muted hover:text-press-accent-text transition-colors"
                    >None</button
                  >
                </Tooltip>
              </div>
            {/if}
          </div>

          <div class="flex-1 overflow-y-auto p-4 space-y-2">
            {#if syncPreview.changes.length === 0}
              <div class="text-center py-12 text-press-muted">
                <p>No changes detected</p>
              </div>
            {:else}
              {#each syncPreview.changes as change (change.id)}
                <label
                  class="flex items-start gap-4 p-4 bg-press-sunken rounded-xl cursor-pointer hover:bg-press-sunken transition-colors"
                >
                  <input
                    type="checkbox"
                    data-testid="sync-change-checkbox"
                    data-change-id={change.id}
                    checked={selectedChanges.has(change.id)}
                    onchange={() => toggleChange(change.id)}
                    class="mt-0.5 w-5 h-5 rounded border-2 border-press-border bg-transparent text-press-accent-text focus:ring-press-focus focus:ring-offset-0 cursor-pointer"
                  />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <span
                        class="px-2 py-0.5 text-press-eyebrow font-medium rounded-full bg-press-warning-wash text-press-warning uppercase"
                      >
                        {change.item_type}
                      </span>
                      <span class="text-press-text font-medium truncate">{change.item_title}</span>
                      <span class="text-press-muted text-press-eyebrow">({change.field})</span>
                    </div>
                    {#if change.field === "prose"}
                      <div class="space-y-3" data-testid="sync-prose-diff">
                        <p class="text-press-small text-press-muted">
                          Accepting replaces the prose shown below. Scene replacements without beat
                          comments keep planning beats and put the incoming text in the first beat.
                        </p>
                        <div>
                          <p class="text-press-small text-press-muted">Current prose</p>
                          <div class="prose-review">{change.current_value || "(empty)"}</div>
                        </div>
                        <div>
                          <p class="text-press-small text-press-muted">Incoming prose</p>
                          <div class="prose-review">{change.new_value || "(empty)"}</div>
                        </div>
                      </div>
                    {:else}
                      <div class="text-press-ui space-y-1 font-mono">
                        <div class="flex gap-2 text-press-error">
                          <span class="flex-shrink-0">-</span>
                          <span class="line-through text-press-disabled-text truncate"
                            >{change.current_value || "(empty)"}</span
                          >
                        </div>
                        <div class="flex gap-2 text-press-success">
                          <span class="flex-shrink-0">+</span>
                          <span class="truncate">{change.new_value || "(empty)"}</span>
                        </div>
                      </div>
                    {/if}
                  </div>
                </label>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}

    {#if error}<p role="alert" class="px-8 text-press-error">Sync failed: {error}</p>{/if}

    <!-- Footer -->
    <div
      class="flex items-center justify-between px-8 py-5 border-t border-press-border/50 bg-press-sunken"
    >
      <p class="text-press-muted text-press-ui">
        {selectedAdditions.size + selectedChanges.size} item{selectedAdditions.size +
          selectedChanges.size !==
        1
          ? "s"
          : ""} selected
      </p>
      <div class="flex gap-4">
        <button
          onclick={onClose}
          class="px-6 py-2.5 text-press-muted hover:text-press-text rounded-lg hover:bg-press-sunken transition-colors"
        >
          Cancel
        </button>
        <button
          data-testid="sync-confirm"
          onclick={applySync}
          disabled={syncing || (selectedAdditions.size === 0 && selectedChanges.size === 0)}
          class="px-6 py-2.5 bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors flex items-center gap-2"
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

<style>
  .prose-review {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-family: var(--font-body);
    font-size: var(--text-body);
    color: var(--color-text);
    max-width: var(--measure);
  }
</style>
