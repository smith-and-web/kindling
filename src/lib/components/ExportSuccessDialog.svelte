<!--
  ExportSuccessDialog.svelte - Export success confirmation

  Shows the results of a successful export operation:
  - Number of chapters/scenes/files exported
  - Output path with option to open in file browser
-->
<script lang="ts">
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { CircleCheck, FolderOpen } from "lucide-svelte";
  import type { ExportResult } from "../types";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus, ownsEnterKey } from "../utils/modalFocus";
  import { t } from "../i18n";

  let {
    result,
    onClose,
  }: {
    result: ExportResult;
    onClose: () => void;
  } = $props();

  let openError = $state<string | null>(null);

  async function openFolder() {
    openError = null;
    try {
      await revealItemInDir(result.output_path);
    } catch (e) {
      console.error("Failed to open folder:", e);
      openError = t("export.exportSuccessDialog.openFolderFailed", {
        error: e instanceof Error ? e.message : String(e),
      });
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Enter on a focused control activates that control (e.g. Open folder).
    if (event.key === "Enter" && !event.isComposing && !ownsEnterKey(event.target)) {
      event.preventDefault();
      onClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<!-- Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, onKeydown: handleKeydown }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-success-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title={t("export.exportSuccessDialog.title")}
      titleId="export-success-dialog-title"
      {onClose}
      closeLabel={t("export.exportSuccessDialog.close")}
    />

    <div class="ka-dialog-body export-success">
      <div class="ka-notice ka-notice--success od-row-top" role="status">
        <CircleCheck class="w-5 h-5" aria-hidden="true" />
        <div class="od-field od-fill">
          <strong>{t("export.exportSuccessDialog.successHeading")}</strong>
          <ul class="export-counts">
            {#if result.chapters_exported > 0}
              <li>
                {t(
                  result.chapters_exported === 1
                    ? "export.exportSuccessDialog.chaptersOne"
                    : "export.exportSuccessDialog.chaptersMany",
                  { count: result.chapters_exported }
                )}
              </li>
            {/if}
            {#if result.scenes_exported > 0}
              <li>
                {t(
                  result.scenes_exported === 1
                    ? "export.exportSuccessDialog.scenesOne"
                    : "export.exportSuccessDialog.scenesMany",
                  { count: result.scenes_exported }
                )}
              </li>
            {/if}
            <li>
              {t(
                result.files_created === 1
                  ? "export.exportSuccessDialog.filesOne"
                  : "export.exportSuccessDialog.filesMany",
                { count: result.files_created }
              )}
            </li>
          </ul>
        </div>
      </div>

      <div class="od-field export-location">
        <p class="ka-label">{t("export.exportSuccessDialog.savedTo")}</p>
        <div class="ka-code">
          <code>{result.output_path}</code>
        </div>
        {#if openError}
          <p class="ka-error" role="alert">{openError}</p>
        {/if}
      </div>
    </div>

    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button ka-button--secondary">
        {t("export.exportSuccessDialog.close")}
      </button>
      <button type="button" onclick={openFolder} class="ka-button">
        <FolderOpen class="w-5 h-5" aria-hidden="true" />
        {t("export.exportSuccessDialog.openFolder")}
      </button>
    </footer>
  </div>
</div>

<style>
  .export-success {
    display: grid;
    gap: var(--space-m);
  }
  .export-counts {
    margin: 0;
    padding: 0;
    list-style: none;
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .export-location {
    gap: var(--space-2xs);
  }
  .export-location p {
    margin: 0;
  }
  .export-location .ka-label {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
</style>
