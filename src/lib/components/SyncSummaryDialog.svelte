<!--
  SyncSummaryDialog.svelte - Shows sync results after completion

  Displays a summary of what was added, updated, and preserved during sync.
-->
<script lang="ts">
  import { CheckCheck } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import type { ReimportSummary } from "../types";
  import { t } from "../i18n";

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

  const KIND_LABEL_KEYS: Record<string, string> = {
    Chapters: "kindChapters",
    Scenes: "kindScenes",
    Beats: "kindBeats",
  };

  function kindLabel(kind: string): string {
    return t(`references.syncSummaryDialog.${KIND_LABEL_KEYS[kind]}`);
  }

  function proseItemLabel(count: number, updated: boolean): string {
    if (updated) {
      return t(
        count === 1
          ? "references.syncSummaryDialog.proseItemUpdatedOne"
          : "references.syncSummaryDialog.proseItemUpdatedMany",
        { count }
      );
    }
    return t(
      count === 1
        ? "references.syncSummaryDialog.proseItemPreservedOne"
        : "references.syncSummaryDialog.proseItemPreservedMany",
      { count }
    );
  }
</script>

<div
  data-testid="sync-summary-dialog"
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="sync-summary-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title={t("references.syncSummaryDialog.title")}
      titleId="sync-summary-title"
      {onClose}
      closeTestId="dialog-close"
    />
    <div class="ka-dialog-body">
      <div data-testid="sync-summary" class="summary">
        {#if !hasChanges}
          <div class="ka-empty od-stack">
            <CheckCheck class="w-7 h-7" aria-hidden="true" />
            <h4>{t("references.syncSummaryDialog.nothingChanged")}</h4>
            <p>{t("references.syncSummaryDialog.noChangesApplied")}</p>
          </div>
        {:else}
          <div class="summary-badges">
            {#if totalAdded > 0}
              <span class="ka-badge ka-badge--success"
                >{t("references.syncSummaryDialog.addedCount", { count: totalAdded })}</span
              >
            {/if}
            {#if totalUpdated > 0}
              <span class="ka-badge"
                >{t("references.syncSummaryDialog.updatedCount", { count: totalUpdated })}</span
              >
            {/if}
            {#if summary.prose_preserved > 0}
              <span class="ka-badge"
                >{t("references.syncSummaryDialog.preservedCount", {
                  count: summary.prose_preserved,
                })}</span
              >
            {/if}
          </div>
          <ul class="summary-list">
            {#each rows as row (row.kind)}
              <li class="summary-row">
                <span class="ka-label">{kindLabel(row.kind)}</span>
                <span class="summary-meta"
                  >{t("references.syncSummaryDialog.rowMeta", {
                    added: row.added,
                    updated: row.updated,
                  })}</span
                >
              </li>
            {/each}
            {#if summary.prose_updated > 0 || summary.prose_preserved > 0}
              <li class="summary-row">
                <span class="ka-label">{t("references.syncSummaryDialog.prose")}</span>
                <span class="summary-meta">
                  {#if summary.prose_updated > 0}
                    <span>{proseItemLabel(summary.prose_updated, true)}</span>
                  {/if}
                  {#if summary.prose_preserved > 0}
                    <span>{proseItemLabel(summary.prose_preserved, false)}</span>
                  {/if}
                </span>
              </li>
            {/if}
          </ul>
        {/if}
      </div>
    </div>
    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button"
        >{t("references.syncSummaryDialog.done")}</button
      >
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
