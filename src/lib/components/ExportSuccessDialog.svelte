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
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="export-success-dialog-title"
>
  <!-- Dialog -->
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <h2 id="export-success-dialog-title" class="text-lg font-medium text-text-primary">
        Export Complete
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
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
        <CheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
        <div>
          <p class="text-text-primary font-medium">Successfully exported:</p>
          <ul class="mt-2 space-y-1 text-text-secondary text-sm">
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
        <p class="text-sm font-medium text-text-secondary mb-1">Location:</p>
        <p class="text-sm text-text-primary break-all bg-bg-card rounded px-2 py-1.5">
          {result.output_path}
        </p>
      </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
      <button
        type="button"
        onclick={openFolder}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors flex items-center gap-2"
      >
        <FolderOpen class="w-4 h-4" />
        Open Folder
      </button>
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors"
      >
        Close
      </button>
    </div>
  </div>
</div>
