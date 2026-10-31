<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CircleAlert } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import type { ScrivenerMatchPreview } from "../types";
  import { t } from "../i18n";

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

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onCancel();
  }
</script>

<!-- Opened over the export dialog: modalFocus makes this the topmost, so Escape
     and focus stay here rather than reaching the export dialog behind. -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onCancel }}
  onclick={handleBackdropClick}
  role="presentation"
  tabindex="-1"
>
  <div
    class="app-dialog-surface ka-dialog-default dialog-shell"
    role="dialog"
    aria-modal="true"
    aria-labelledby="scrivener-match-title"
    tabindex="-1"
  >
    <DialogHeader
      title={t("references.scrivenerMatchDialog.title")}
      titleId="scrivener-match-title"
      subtitle={t("references.scrivenerMatchDialog.subtitle")}
      onClose={onCancel}
      closeLabel={t("references.scrivenerMatchDialog.close")}
    />

    <div class="ka-dialog-body match">
      {#if loading}
        <div class="ka-progress od-field" role="status">
          <span>{t("references.scrivenerMatchDialog.analyzing")}</span>
          <progress aria-label={t("references.scrivenerMatchDialog.analyzingLabel")}></progress>
        </div>
      {:else if loadError}
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <CircleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>{t("references.scrivenerMatchDialog.previewFailed")}</strong>
            <p>{loadError}</p>
          </div>
        </div>
      {:else if matches.length === 0}
        <div class="ka-empty od-stack">
          <h4>{t("references.scrivenerMatchDialog.noScenes")}</h4>
          <p>{t("references.scrivenerMatchDialog.noScenesMessage")}</p>
        </div>
      {:else}
        <div class="match-summary">
          <span class="ka-badge ka-badge--success"
            >{t("references.scrivenerMatchDialog.matchedCount", { count: matchedCount })}</span
          >
          {#if unmatchedCount > 0}
            <span class="ka-badge ka-badge--warning"
              >{t("references.scrivenerMatchDialog.unmatchedCount", {
                count: unmatchedCount,
              })}</span
            >
          {/if}
        </div>

        <ul class="match-list">
          {#each matches as m}
            {@const isMatched = !!m.matched_scriv_title}
            <li class="match-row">
              <span class="match-type">
                {#if isMatched}
                  <span class="ka-badge ka-badge--success"
                    >{t("references.scrivenerMatchDialog.matchedBadge")}</span
                  >
                {:else}
                  <span class="ka-badge ka-badge--warning"
                    >{t("references.scrivenerMatchDialog.newBadge")}</span
                  >
                {/if}
              </span>
              <span class="match-text">
                <span class="ka-label match-name">{m.chapter_title} · {m.scene_title}</span>
                <span class="match-meta">
                  {isMatched
                    ? t("references.scrivenerMatchDialog.via", {
                        title: m.matched_scriv_title ?? "",
                        method:
                          m.match_method === "source_id"
                            ? t("references.scrivenerMatchDialog.methodId")
                            : t("references.scrivenerMatchDialog.methodTitle"),
                      })
                    : t("references.scrivenerMatchDialog.newDocument")}
                </span>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <button type="button" onclick={onCancel} class="ka-button ka-button--secondary">
        {t("references.scrivenerMatchDialog.cancel")}
      </button>
      <button type="button" onclick={onConfirm} disabled={loading || !!loadError} class="ka-button">
        {t("references.scrivenerMatchDialog.proceed")}
      </button>
    </footer>
  </div>
</div>

<style>
  .match {
    display: grid;
    align-content: start;
    gap: var(--space-s);
  }
  .match-summary {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .match-list {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .match-row {
    display: grid;
    grid-template-columns: 96px minmax(0, 1fr);
    align-items: start;
    gap: var(--space-xs);
    min-height: var(--control-target);
    padding: var(--space-xs) var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .match-type {
    display: flex;
  }
  .match-text {
    display: grid;
    gap: var(--space-3xs);
    min-width: 0;
  }
  .match-name {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .match-meta {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }
  .ka-notice p {
    margin: 0;
  }
  .ka-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
  }
  .ka-empty p {
    margin: 0;
  }
</style>
