<!--
  ExportDialog.svelte - Export configuration dialog

  Allows users to configure and initiate project exports:
  - Format selection (Markdown only for now, DOCX coming soon)
  - Scope selection based on context (project/chapter/scene)
  - Options like beat markers
  - Destination folder picker
-->
<script lang="ts">
  /* eslint-disable no-undef */
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { X, Loader2, FolderOpen } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import type { ExportResult, MarkdownExportOptions, ExportScope } from "../types";
  import Tooltip from "./Tooltip.svelte";

  const LAST_EXPORT_PATH_KEY = "kindling:lastExportPath";

  let {
    scope,
    scopeId,
    scopeTitle,
    onClose,
    onSuccess,
  }: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    scopeTitle: string;
    onClose: () => void;
    onSuccess: (result: ExportResult) => void;
  } = $props();

  let exportFormat = $state<"markdown" | "docx">("markdown");
  let includeBeatMarkers = $state(true);
  let deleteExisting = $state(false);
  let createSnapshot = $state(false);
  let outputPath = $state("");
  let exportName = $state("");
  let exporting = $state(false);
  let error = $state<string | null>(null);

  // Initialize export name from project name
  $effect(() => {
    if (currentProject.value && !exportName) {
      exportName = currentProject.value.name;
    }
  });

  // Load last export path from localStorage on mount
  $effect(() => {
    const savedPath = localStorage.getItem(LAST_EXPORT_PATH_KEY);
    if (savedPath && !outputPath) {
      outputPath = savedPath;
    }
  });

  const canExport = $derived(exportFormat === "markdown" && outputPath.length > 0);

  async function selectDestination() {
    const path = await open({
      directory: true,
      title: "Select Export Destination",
      defaultPath: outputPath || undefined,
    });

    if (path) {
      outputPath = path;
      error = null;
    }
  }

  async function handleExport() {
    if (!canExport) return;

    exporting = true;
    error = null;

    try {
      // Build the scope for the export options
      let exportScope: ExportScope;
      if (scope === "project") {
        exportScope = "project";
      } else if (scope === "chapter" && scopeId) {
        exportScope = { chapter: scopeId };
      } else if (scope === "scene" && scopeId) {
        exportScope = { scene: scopeId };
      } else {
        exportScope = "project";
      }

      const options: MarkdownExportOptions = {
        scope: exportScope,
        include_beat_markers: includeBeatMarkers,
        output_path: outputPath,
        delete_existing: deleteExisting,
        export_name: exportName.trim() || undefined,
        create_snapshot: createSnapshot,
      };

      if (!currentProject.value) {
        throw new Error("No project selected");
      }

      const result = await invoke<ExportResult>("export_to_markdown", {
        projectId: currentProject.value.id,
        options,
      });

      // Save the export path for next time
      localStorage.setItem(LAST_EXPORT_PATH_KEY, outputPath);

      onSuccess(result);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      exporting = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && canExport && !exporting) {
      handleExport();
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
  aria-labelledby="export-dialog-title"
>
  <!-- Dialog -->
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <h2 id="export-dialog-title" class="text-lg font-medium text-text-primary">
        Export {scopeTitle}
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
      <!-- Format Selection -->
      <fieldset>
        <legend class="block text-sm font-medium text-text-secondary mb-2">Format</legend>
        <div class="space-y-2">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="format"
              value="markdown"
              bind:group={exportFormat}
              class="w-4 h-4 text-accent bg-bg-card border-bg-card focus:ring-accent"
            />
            <span class="text-text-primary">Markdown (.md files)</span>
          </label>
          <label class="flex items-center gap-2 cursor-not-allowed opacity-50">
            <input
              type="radio"
              name="format"
              value="docx"
              bind:group={exportFormat}
              disabled
              class="w-4 h-4 text-accent bg-bg-card border-bg-card"
            />
            <span class="text-text-secondary">Word Document (.docx)</span>
            <span class="text-xs text-text-secondary italic">Coming soon</span>
          </label>
        </div>
      </fieldset>

      <!-- Options -->
      <fieldset>
        <legend class="block text-sm font-medium text-text-secondary mb-2">Options</legend>
        <div class="space-y-2">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={includeBeatMarkers}
              class="w-4 h-4 text-accent bg-bg-card border-bg-card rounded focus:ring-accent"
            />
            <span class="text-text-primary">Include beat markers as headings</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={deleteExisting}
              class="w-4 h-4 text-accent bg-bg-card border-bg-card rounded focus:ring-accent"
            />
            <span class="text-text-primary">Delete existing export folder</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={createSnapshot}
              class="w-4 h-4 text-accent bg-bg-card border-bg-card rounded focus:ring-accent"
            />
            <span class="text-text-primary">Create snapshot before exporting</span>
          </label>
        </div>
      </fieldset>

      <!-- Export Name -->
      <div>
        <label for="export-name" class="block text-sm font-medium text-text-secondary mb-2">
          Export Name
        </label>
        <input
          id="export-name"
          type="text"
          bind:value={exportName}
          placeholder="Enter export folder name..."
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent"
        />
        <p class="text-xs text-text-secondary mt-1">
          Folder will be created as: {exportName.trim() || currentProject.value?.name || "Project"}
        </p>
      </div>

      <!-- Destination -->
      <div>
        <label for="destination" class="block text-sm font-medium text-text-secondary mb-2">
          Destination
        </label>
        <div class="flex gap-2">
          <input
            id="destination"
            type="text"
            readonly
            value={outputPath}
            placeholder="Select a folder..."
            class="flex-1 bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent cursor-pointer"
            onclick={selectDestination}
          />
          <Tooltip text="Browse" position="top">
            <button
              type="button"
              onclick={selectDestination}
              class="px-3 py-2 bg-bg-card text-text-primary rounded-lg hover:bg-beat-header transition-colors"
              aria-label="Browse for folder"
            >
              <FolderOpen class="w-5 h-5" />
            </button>
          </Tooltip>
        </div>
      </div>

      <!-- Error Message -->
      {#if error}
        <p class="text-sm text-red-400">{error}</p>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        disabled={exporting}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleExport}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        disabled={!canExport || exporting}
      >
        {#if exporting}
          <Loader2 class="w-4 h-4 animate-spin" />
          Exporting...
        {:else}
          Export
        {/if}
      </button>
    </div>
  </div>
</div>
