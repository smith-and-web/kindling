<!--
  ExportSuccessDialog.svelte - Export success confirmation

  Shows the results of a successful export operation:
  - Number of chapters/scenes/files exported
  - Output path with option to open in file browser
-->
<script lang="ts">
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { X, CheckCircle, FolderOpen } from "lucide-svelte";
  import type { ExportResult } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    result,
    onClose,
  }: {
    result: ExportResult;
    onClose: () => void;
  } = $props();

  async function openFolder() {
    try {
      await revealItemInDir(result.output_path);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" || event.key === "Enter") {
      onClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-success-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
      <h2 id="export-success-dialog-title" class="text-press-body-lg font-medium text-press-text">
        Export Complete
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="p-4 space-y-4">
      <!-- Success Message -->
      <div class="flex items-start gap-3">
        <CheckCircle class="w-6 h-6 text-press-success flex-shrink-0 mt-0.5" />
        <div>
          <p class="text-press-text font-medium">Successfully exported:</p>
          <ul class="mt-2 space-y-1 text-press-muted text-press-ui">
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

      <!-- Location -->
      <div>
        <p class="text-press-ui font-medium text-press-muted mb-1">Location:</p>
        <p class="text-press-ui text-press-text break-all bg-press-sunken rounded px-2 py-1.5">
          {result.output_path}
        </p>
      </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
      <button
        type="button"
        onclick={openFolder}
        class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors flex items-center gap-2"
      >
        <FolderOpen class="w-4 h-4" />
        Open Folder
      </button>
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</div>
