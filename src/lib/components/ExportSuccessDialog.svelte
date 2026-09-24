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
      openError = `Couldn’t open the folder: ${e instanceof Error ? e.message : String(e)}`;
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
      title="Export complete"
      titleId="export-success-dialog-title"
      {onClose}
      closeLabel="Close"
    />

    <div class="ka-dialog-body export-success">
      <div class="ka-notice ka-notice--success od-row-top" role="status">
        <CircleCheck class="w-5 h-5" aria-hidden="true" />
        <div class="od-field od-fill">
          <strong>Successfully exported:</strong>
          <ul class="export-counts">
            {#if result.chapters_exported > 0}
              <li>{result.chapters_exported} chapter{result.chapters_exported === 1 ? "" : "s"}</li>
            {/if}
            {#if result.scenes_exported > 0}
              <li>{result.scenes_exported} scene{result.scenes_exported === 1 ? "" : "s"}</li>
            {/if}
            <li>{result.files_created} file{result.files_created === 1 ? "" : "s"} created</li>
          </ul>
        </div>
      </div>

      <div class="od-field export-location">
        <p class="ka-label">Saved to</p>
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
        Close
      </button>
      <button type="button" onclick={openFolder} class="ka-button">
        <FolderOpen class="w-5 h-5" aria-hidden="true" />
        Open folder
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
