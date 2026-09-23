<!--
  SyncSummaryDialog.svelte - Shows sync results after completion

  Displays a summary of what was added, updated, and preserved during sync.
-->
<script lang="ts">
  import { CheckCheck } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
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

  const totalAdded = $derived(summary.chapters_added + summary.scenes_added + summary.beats_added);
  const totalUpdated = $derived(
    summary.chapters_updated +
      summary.scenes_updated +
      summary.beats_updated +
      summary.prose_updated
  );

  const rows = $derived(
    [
      { kind: "Chapters", added: summary.chapters_added, updated: summary.chapters_updated },
      { kind: "Scenes", added: summary.scenes_added, updated: summary.scenes_updated },
      { kind: "Beats", added: summary.beats_added, updated: summary.beats_updated },
    ].filter((row) => row.added > 0 || row.updated > 0)
  );

  function plural(count: number, word: string): string {
    return `${count} ${word}${count !== 1 ? "s" : ""}`;
  }
</script>

<div
  data-testid="sync-summary-dialog"
  class="dialog-scrim"
  role="dialog"
  aria-modal="true"
  aria-labelledby="sync-summary-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title="Sync complete"
      titleId="sync-summary-title"
      {onClose}
      closeTestId="dialog-close"
    />
    <div class="ka-dialog-body">
      <div data-testid="sync-summary" class="summary">
        {#if !hasChanges}
          <div class="ka-empty od-stack">
            <CheckCheck class="w-7 h-7" aria-hidden="true" />
            <h4>Nothing changed</h4>
            <p>No changes were applied.</p>
          </div>
        {:else}
          <div class="summary-badges">
            {#if totalAdded > 0}
              <span class="ka-badge ka-badge--success">{totalAdded} added</span>
            {/if}
            {#if totalUpdated > 0}
              <span class="ka-badge">{totalUpdated} updated</span>
            {/if}
            {#if summary.prose_preserved > 0}
              <span class="ka-badge">{summary.prose_preserved} preserved</span>
            {/if}
          </div>
          <ul class="summary-list">
            {#each rows as row (row.kind)}
              <li class="summary-row">
                <span class="ka-label">{row.kind}</span>
                <span class="summary-meta">{row.added} added, {row.updated} updated</span>
              </li>
            {/each}
            {#if summary.prose_updated > 0 || summary.prose_preserved > 0}
              <li class="summary-row">
                <span class="ka-label">Prose</span>
                <span class="summary-meta">
                  {#if summary.prose_updated > 0}
                    <span>{plural(summary.prose_updated, "prose item")} updated</span>
                  {/if}
                  {#if summary.prose_preserved > 0}
                    <span>{plural(summary.prose_preserved, "prose item")} preserved</span>
                  {/if}
                </span>
              </li>
            {/if}
          </ul>
        {/if}
      </div>
    </div>
    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button">Done</button>
    </footer>
  </div>
</div>

<style>
  .summary {
    display: grid;
    gap: var(--space-s);
  }
  .summary-badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .summary-list {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .summary-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--space-3xs) var(--space-s);
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-2xs);
    border-bottom: var(--border-hair);
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .summary-meta {
    display: grid;
    justify-items: end;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .ka-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
  }
  .ka-empty p {
    margin: 0;
  }
</style>
