<!--
  ProjectSettingsDialog.svelte - Project-specific settings dialog

  Allows users to configure project-specific metadata:
  - Pen name (overrides app-level author name for this project)
  - Genre
  - Description
  - Word target
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X, Loader2, BookOpen } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import type { Project } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    onClose,
    onSave,
  }: {
    onClose: () => void;
    onSave: (project: Project) => void;
  } = $props();

  // Form fields initialized from current project
  let authorPenName = $state(currentProject.value?.author_pen_name ?? "");
  let genre = $state(currentProject.value?.genre ?? "");
  let description = $state(currentProject.value?.description ?? "");
  let wordTarget = $state(
    currentProject.value?.word_target !== null && currentProject.value?.word_target !== undefined
      ? String(currentProject.value.word_target)
      : ""
  );

  let saving = $state(false);
  let error = $state<string | null>(null);

  async function handleSave() {
    if (!currentProject.value) return;

    saving = true;
    error = null;

    try {
      const parsedWordTarget = wordTarget.trim().length ? Number(wordTarget.trim()) : null;
      if (parsedWordTarget !== null && Number.isNaN(parsedWordTarget)) {
        throw new Error("Word target must be a number");
      }

      // Convert empty strings to null for optional fields
      const settings = {
        author_pen_name: authorPenName.trim() || null,
        genre: genre.trim() || null,
        description: description.trim() || null,
        word_target: parsedWordTarget,
      };

      const updatedProject = await invoke<Project>("update_project_settings", {
        projectId: currentProject.value.id,
        settings,
      });

      onSave(updatedProject);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && !saving) {
      handleSave();
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
  onkeydown={(e) => e.key === "Enter" && handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <div class="flex items-center gap-2">
        <BookOpen class="w-5 h-5 text-accent" />
        <h2 id="settings-dialog-title" class="text-lg font-medium text-text-primary">
          Project Settings
        </h2>
      </div>
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
      <p class="text-sm text-text-secondary">
        These settings are specific to <strong class="text-text-primary"
          >{currentProject.value?.name}</strong
        >.
      </p>

      <!-- Pen Name -->
      <div>
        <label for="author-pen-name" class="block text-sm text-text-secondary mb-1">
          Pen Name <span class="text-text-secondary/60">(optional)</span>
        </label>
        <input
          id="author-pen-name"
          type="text"
          bind:value={authorPenName}
          placeholder="Leave blank to use your author name"
          disabled={saving}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
        />
        <p class="text-xs text-text-secondary mt-1">
          If provided, this will be used as the byline on title pages instead of your author name.
        </p>
      </div>

      <!-- Genre -->
      <div>
        <label for="genre" class="block text-sm text-text-secondary mb-1">
          Genre <span class="text-text-secondary/60">(optional)</span>
        </label>
        <input
          id="genre"
          type="text"
          bind:value={genre}
          placeholder="e.g., Literary Fiction, Science Fiction, Mystery"
          disabled={saving}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
        />
        <p class="text-xs text-text-secondary mt-1">
          Genre will be displayed on manuscript title pages.
        </p>
      </div>

      <!-- Description -->
      <div>
        <label for="project-description" class="block text-sm text-text-secondary mb-1">
          Project Description <span class="text-text-secondary/60">(optional)</span>
        </label>
        <textarea
          id="project-description"
          rows="4"
          bind:value={description}
          placeholder="Short summary or notes about this project"
          disabled={saving}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50 resize-none"
        ></textarea>
      </div>

      <!-- Word Target -->
      <div>
        <label for="word-target" class="block text-sm text-text-secondary mb-1">
          Word Target <span class="text-text-secondary/60">(optional)</span>
        </label>
        <input
          id="word-target"
          type="number"
          min="0"
          inputmode="numeric"
          bind:value={wordTarget}
          placeholder="e.g., 80000"
          disabled={saving}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent disabled:opacity-50"
        />
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
        disabled={saving}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSave}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        disabled={saving}
      >
        {#if saving}
          <Loader2 class="w-4 h-4 animate-spin" />
          Saving...
        {:else}
          Save
        {/if}
      </button>
    </div>
  </div>
</div>
