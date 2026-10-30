<!--
  SyncDialog.svelte - Sync preview and confirmation dialog

  Displays a preview of changes when re-importing from a source file.
  Users can selectively accept additions and changes before applying.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CheckCheck, CircleAlert, Loader2 } from "lucide-svelte";
  import { SvelteSet } from "svelte/reactivity";
  import type { SyncPreview, SyncAddition, ReimportSummary, SyncChange } from "../types";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { saveProseBefore } from "../utils/proseFlush";
  import { t } from "../i18n";

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

  // Conflicts may replace a kindling edit, so they are only ever chosen one by one.
  function selectAllChanges() {
    selectedChanges.clear();
    for (const change of syncPreview.changes) {
      if (!change.conflict) selectedChanges.add(change.id);
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

  const selectedCount = $derived(selectedAdditions.size + selectedChanges.size);
  const conflictCount = $derived(syncPreview.changes.filter((c) => c.conflict).length);
  // Applying settles unticked conflicts as "keep kindling", so that alone is worth applying.
  const keptCount = $derived(
    syncPreview.changes.filter((c) => c.conflict && !selectedChanges.has(c.id)).length
  );
  const hasNothingToSync = $derived(
    syncPreview.additions.length === 0 && syncPreview.changes.length === 0
  );

  const FIELD_LABELS: Record<SyncChange["field"], string> = {
    title: "fieldTitle",
    synopsis: "fieldSynopsis",
    content: "fieldContent",
    prose: "fieldProse",
  };

  const KIND_LABELS: Record<SyncAddition["item_type"], string> = {
    chapter: "kindChapter",
    scene: "kindScene",
    beat: "kindBeat",
  };

  function fieldLabel(field: SyncChange["field"]): string {
    return t(`references.syncDialog.${FIELD_LABELS[field] ?? field}`);
  }

  function kindLabel(itemType: SyncAddition["item_type"]): string {
    return t(`references.syncDialog.${KIND_LABELS[itemType] ?? itemType}`);
  }

  const selectionSummary = $derived.by(() => {
    const selected = t(
      selectedCount === 1
        ? "references.syncDialog.selectionOne"
        : "references.syncDialog.selectionMany",
      { count: selectedCount }
    );
    if (keptCount === 0) return selected;
    const kept = t(
      keptCount === 1 ? "references.syncDialog.keptOne" : "references.syncDialog.keptMany",
      { count: keptCount }
    );
    return `${selected}, ${kept}`;
  });

  async function applySync() {
    syncing = true;
    error = null;
    try {
      await saveProseBefore(projectId, "syncing");
      const summary = await invoke<ReimportSummary>("apply_sync", {
        projectId,
        acceptedChangeIds: Array.from(selectedChanges),
        acceptedAdditionIds: Array.from(selectedAdditions),
        // Only conflicts shown here are settled as "keep kindling".
        keptConflictIds: syncPreview.changes
          .filter((c) => c.conflict && !selectedChanges.has(c.id))
          .map((c) => c.id),
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
  class="dialog-scrim"
  use:modalFocus={{ onEscape: () => !syncing && onClose() }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="sync-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell sync">
    <DialogHeader
      title={t("references.syncDialog.title")}
      titleId="sync-dialog-title"
      subtitle={t("references.syncDialog.subtitle")}
      {onClose}
      closeLabel={t("references.syncDialog.close")}
      closeTestId="sync-dialog-close"
      disabled={syncing}
    />

    <div class="ka-dialog-body sync-body">
      {#if hasNothingToSync}
        <div class="ka-empty od-stack">
          <CheckCheck class="w-7 h-7" aria-hidden="true" />
          <h4>{t("references.syncDialog.allSynced")}</h4>
          <p>{t("references.syncDialog.allSyncedMessage")}</p>
        </div>
      {:else}
        <p class="sync-intro">
          {t("references.syncDialog.intro")}
        </p>

        <div class="sync-summary" aria-label={t("references.syncDialog.summaryLabel")}>
          {#if syncPreview.additions.length > 0}
            <span class="ka-badge ka-badge--success"
              >{t("references.syncDialog.addedCount", {
                count: syncPreview.additions.length,
              })}</span
            >
          {/if}
          {#if syncPreview.changes.length > 0}
            <span class="ka-badge"
              >{t("references.syncDialog.changedCount", {
                count: syncPreview.changes.length,
              })}</span
            >
          {/if}
          {#if conflictCount > 0}
            <span class="ka-badge ka-badge--warning" data-testid="sync-conflict-count"
              >{t(
                conflictCount === 1
                  ? "references.syncDialog.conflictOne"
                  : "references.syncDialog.conflictMany",
                { count: conflictCount }
              )}</span
            >
          {/if}
        </div>
        {#if conflictCount > 0}
          <p class="ka-help" data-testid="sync-conflict-note">
            {t("references.syncDialog.conflictNote")}
          </p>
        {/if}

        {#if syncPreview.additions.length > 0}
          <section class="ka-group sync-group" aria-labelledby="sync-additions-title">
            <div class="sync-group-head">
              <h3 class="ka-group-title" id="sync-additions-title">
                {t("references.syncDialog.newItems")}
              </h3>
              <span class="sync-count"
                >{t("references.syncDialog.selectedOf", {
                  selected: selectedAdditions.size,
                  total: syncPreview.additions.length,
                })}</span
              >
              <span class="sync-bulk">
                <button
                  type="button"
                  onclick={selectAllAdditions}
                  class="ka-button ka-button--ghost"
                  aria-label={t("references.syncDialog.selectAllNewItems")}
                  title={t("references.syncDialog.selectAll")}
                  >{t("references.syncDialog.allButton")}</button
                >
                <button
                  type="button"
                  onclick={deselectAllAdditions}
                  class="ka-button ka-button--ghost"
                  aria-label={t("references.syncDialog.deselectAllNewItems")}
                  title={t("references.syncDialog.deselectAll")}
                  >{t("references.syncDialog.noneButton")}</button
                >
              </span>
            </div>
            <ul class="sync-list">
              {#each syncPreview.additions as addition (addition.id)}
                <li>
                  <label class="sync-row">
                    <input
                      type="checkbox"
                      checked={selectedAdditions.has(addition.id)}
                      onchange={() => toggleAddition(addition.id)}
                      disabled={syncing}
                    />
                    <span class="sync-type"
                      ><span class="ka-badge ka-badge--success"
                        >{t("references.syncDialog.addedBadge")}</span
                      ></span
                    >
                    <span class="sync-text">
                      <span class="ka-label sync-name"
                        >{t("references.syncDialog.kindTitle", {
                          kind: kindLabel(addition.item_type),
                          title: addition.title,
                        })}</span
                      >
                      {#if addition.parent_title}
                        <span class="sync-meta"
                          >{t("references.syncDialog.parentIn", {
                            parent: addition.parent_title,
                          })}</span
                        >
                      {/if}
                    </span>
                  </label>
                </li>
              {/each}
            </ul>
          </section>
        {/if}

        {#if syncPreview.changes.length > 0}
          <section class="ka-group sync-group" aria-labelledby="sync-changes-title">
            <div class="sync-group-head">
              <h3 class="ka-group-title" id="sync-changes-title">
                {t("references.syncDialog.changesTitle")}
              </h3>
              <span class="sync-count"
                >{t("references.syncDialog.selectedOf", {
                  selected: selectedChanges.size,
                  total: syncPreview.changes.length,
                })}</span
              >
              <span class="sync-bulk">
                <button
                  type="button"
                  onclick={selectAllChanges}
                  class="ka-button ka-button--ghost"
                  aria-label={t("references.syncDialog.selectAllChangesLabel")}
                  title={t("references.syncDialog.selectAllExceptConflicts")}
                  >{t("references.syncDialog.allButton")}</button
                >
                <button
                  type="button"
                  onclick={deselectAllChanges}
                  class="ka-button ka-button--ghost"
                  aria-label={t("references.syncDialog.deselectAllChangesLabel")}
                  title={t("references.syncDialog.deselectAll")}
                  >{t("references.syncDialog.noneButton")}</button
                >
              </span>
            </div>
            <ul class="sync-list">
              {#each syncPreview.changes as change (change.id)}
                <li>
                  <label class="sync-row">
                    <input
                      type="checkbox"
                      data-testid="sync-change-checkbox"
                      data-change-id={change.id}
                      checked={selectedChanges.has(change.id)}
                      onchange={() => toggleChange(change.id)}
                      disabled={syncing}
                    />
                    <span class="sync-type">
                      {#if change.conflict}
                        <span class="ka-badge ka-badge--warning"
                          >{t("references.syncDialog.conflictBadge")}</span
                        >
                      {:else}
                        <span class="ka-badge">{t("references.syncDialog.changedBadge")}</span>
                      {/if}
                    </span>
                    <span class="sync-text">
                      <span class="ka-label sync-name"
                        >{t("references.syncDialog.kindTitle", {
                          kind: kindLabel(change.item_type),
                          title: change.item_title,
                        })}</span
                      >
                      <span class="sync-meta">{fieldLabel(change.field)}</span>
                      {#if change.conflict}
                        <span class="ka-help" data-testid="sync-conflict-help">
                          {selectedChanges.has(change.id)
                            ? t("references.syncDialog.conflictTicked")
                            : t("references.syncDialog.conflictUnticked")}
                        </span>
                      {/if}
                      {#if change.field === "prose"}
                        <span class="sync-prose" data-testid="sync-prose-diff">
                          <span class="ka-help">
                            {t("references.syncDialog.proseNote")}
                          </span>
                          <span class="sync-version">
                            <span class="sync-meta">{t("references.syncDialog.currentProse")}</span>
                            <span class="prose-review"
                              >{change.current_value || t("references.syncDialog.emptyValue")}</span
                            >
                          </span>
                          <span class="sync-version">
                            <span class="sync-meta">{t("references.syncDialog.incomingProse")}</span
                            >
                            <span class="prose-review"
                              >{change.new_value || t("references.syncDialog.emptyValue")}</span
                            >
                          </span>
                        </span>
                      {:else}
                        <span class="sync-values">
                          <span class="sync-value">
                            <span class="sync-value-label"
                              >{t("references.syncDialog.current")}</span
                            >
                            <span class="sync-value-text"
                              >{change.current_value || t("references.syncDialog.emptyValue")}</span
                            >
                          </span>
                          <span class="sync-value">
                            <span class="sync-value-label"
                              >{t("references.syncDialog.incoming")}</span
                            >
                            <span class="sync-value-text"
                              >{change.new_value || t("references.syncDialog.emptyValue")}</span
                            >
                          </span>
                        </span>
                      {/if}
                    </span>
                  </label>
                </li>
              {/each}
            </ul>
          </section>
        {/if}
      {/if}
    </div>

    {#if error}
      <div class="sync-alert">
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <CircleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>{t("references.syncDialog.syncFailed", { error: String(error) })}</strong>
            <p>
              {t("references.syncDialog.syncFailedNote")}
            </p>
          </div>
        </div>
      </div>
    {/if}

    <footer class="ka-dialog-footer">
      {#if hasNothingToSync}
        <button type="button" onclick={onClose} class="ka-button ka-button--secondary">
          {t("references.syncDialog.close")}
        </button>
      {:else}
        <p class="ka-help ka-dialog-footer-start">
          {selectionSummary}
        </p>
        <button
          type="button"
          onclick={onClose}
          class="ka-button ka-button--secondary"
          disabled={syncing}
        >
          {t("references.syncDialog.cancel")}
        </button>
        <button
          type="button"
          data-testid="sync-confirm"
          onclick={applySync}
          disabled={syncing || (selectedCount === 0 && keptCount === 0)}
          aria-busy={syncing || undefined}
          class="ka-button"
        >
          {#if syncing}
            <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
            {t("references.syncDialog.applying")}
          {:else if selectedCount === 0 && keptCount > 0}
            {t(
              keptCount === 1
                ? "references.syncDialog.keepKindling"
                : "references.syncDialog.keepKindlingPlural"
            )}
          {:else if selectedCount === 0}
            {t("references.syncDialog.applyChanges")}
          {:else}
            {t(
              selectedCount === 1
                ? "references.syncDialog.applyOne"
                : "references.syncDialog.applyMany",
              { count: selectedCount }
            )}
          {/if}
        </button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .sync {
    height: min(760px, calc(100dvh - 48px));
  }
  .sync-body {
    display: grid;
    align-content: start;
    gap: var(--space-m);
  }
  .sync-body > .ka-group + .ka-group {
    margin-top: 0;
  }
  .sync-intro {
    margin: 0;
    max-width: 60ch;
    font: var(--text-ui) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
  .sync-summary {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .ka-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
  }
  .ka-empty p {
    margin: 0;
  }
  .sync-group {
    gap: var(--space-2xs);
    padding-top: 0;
    border-top: 0;
  }
  .sync-group-head {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .sync-group-head .ka-group-title {
    margin: 0;
  }
  .sync-count {
    flex: 1 1 auto;
    font: 400 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .sync-bulk {
    display: flex;
    gap: var(--space-3xs);
    flex: none;
  }
  .sync-list {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .sync-list > li {
    border-bottom: var(--border-hair);
  }
  .sync-row {
    display: grid;
    grid-template-columns: 20px 96px minmax(0, 1fr);
    align-items: start;
    gap: var(--space-xs);
    min-height: var(--control-target);
    padding: var(--space-xs) var(--space-2xs);
    cursor: pointer;
  }
  .sync-row > input {
    margin-top: 2px;
  }
  .sync-type {
    display: flex;
  }
  .sync-text {
    display: grid;
    gap: var(--space-3xs);
    min-width: 0;
  }
  .sync-name {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .sync-meta {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .sync-values {
    display: grid;
    gap: var(--space-3xs);
  }
  .sync-value {
    display: grid;
    grid-template-columns: 72px minmax(0, 1fr);
    gap: var(--space-2xs);
    font: var(--text-small) / 1.5 var(--font-ui);
  }
  .sync-value-label {
    color: var(--color-text-muted);
  }
  .sync-value-text {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .sync-prose {
    display: grid;
    gap: var(--space-xs);
    margin-top: var(--space-2xs);
  }
  .sync-prose .ka-help {
    display: block;
  }
  .sync-version {
    display: grid;
    gap: var(--space-3xs);
  }
  .prose-review {
    display: block;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
    max-width: var(--measure);
  }
  .sync-alert {
    flex: none;
    padding: 0 var(--space-m) var(--space-s);
  }
  .sync-alert p {
    margin: 0;
  }
</style>
