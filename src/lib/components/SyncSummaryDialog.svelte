<!--
  SyncSummaryDialog.svelte - Shows sync results after completion

  Displays a summary of what was added, updated, and preserved during sync.
-->
<script lang="ts">
  import type { ReimportSummary } from "../types";

  interface Props {
    summary: ReimportSummary;
    onClose: () => void;
  }

  let { summary, onClose }: Props = $props();

  const hasChanges = $derived.by(
    () =>
      summary.chapters_added > 0 ||
      summary.chapters_updated > 0 ||
      summary.scenes_added > 0 ||
      summary.scenes_updated > 0 ||
      summary.beats_added > 0 ||
      summary.beats_updated > 0 ||
      summary.prose_updated > 0
  );
</script>

<div
  data-testid="sync-summary-dialog"
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-lg p-6 max-w-md w-full mx-4 shadow-press-overlay"
  >
    <h3 class="text-press-body-lg font-heading font-medium text-press-text mb-4">Sync Complete</h3>
    <div data-testid="sync-summary" class="text-press-muted text-press-ui space-y-2 mb-6">
      {#if !hasChanges}
        <p>No changes were applied.</p>
      {:else}
        {#if summary.chapters_added > 0 || summary.chapters_updated > 0}
          <p>Chapters: {summary.chapters_added} added, {summary.chapters_updated} updated</p>
        {/if}
        {#if summary.scenes_added > 0 || summary.scenes_updated > 0}
          <p>Scenes: {summary.scenes_added} added, {summary.scenes_updated} updated</p>
        {/if}
        {#if summary.beats_added > 0 || summary.beats_updated > 0}
          <p>Beats: {summary.beats_added} added, {summary.beats_updated} updated</p>
        {/if}
        {#if summary.prose_updated}<p class="text-press-ui text-press-text">
            {summary.prose_updated} prose item{summary.prose_updated !== 1 ? "s" : ""} updated
          </p>{/if}
        {#if summary.prose_preserved > 0}
          <p class="text-press-muted italic">
            {summary.prose_preserved} prose item{summary.prose_preserved !== 1 ? "s" : ""} preserved
          </p>
        {/if}
      {/if}
    </div>
    <button
      data-testid="dialog-close"
      onclick={onClose}
      class="w-full px-4 py-2 bg-press-accent text-press-on-accent rounded hover:bg-press-accent-text transition-colors"
    >
      Close
    </button>
  </div>
</div>
