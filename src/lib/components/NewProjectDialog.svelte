<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { BookOpen, Film, Loader2, X } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project, ProjectType } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    onClose,
    onComplete,
  }: {
    onClose: () => void;
    onComplete?: (project: Project) => void;
  } = $props();

  let projectType = $state<ProjectType>("novel");
  let name = $state("My Project");
  let targetLength = $state<"short" | "feature" | "long_feature">("feature");
  let saving = $state(false);
  let error = $state<string | null>(null);
  let inputRef: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (inputRef) {
      inputRef.focus();
      inputRef.select();
    }
  });

  async function handleCreate() {
    const trimmedName = name.trim();
    if (!trimmedName) {
      error = "Name cannot be empty";
      return;
    }

    saving = true;
    error = null;

    try {
      let project: Project;
      if (projectType === "screenplay") {
        project = await invoke<Project>("create_screenplay_project", {
          name: trimmedName,
          target_length: targetLength,
        });
      } else {
        project = await invoke<Project>("create_blank_project", { name: trimmedName });
      }
      currentProject.setProject(null);
      currentProject.setProject(project);
      ui.setView("editor");
      onComplete?.(project);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create project";
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && !saving) {
      handleCreate();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  const inputClass =
    "w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent";
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="new-project-dialog-title"
  tabindex="-1"
>
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <h2 id="new-project-dialog-title" class="text-lg font-medium text-text-primary">
        New Project
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

    <div class="p-4 space-y-4">
      <div>
        <label class="block text-sm font-medium text-text-secondary mb-2">Project type</label>
        <div class="flex gap-2">
          <button
            type="button"
            onclick={() => (projectType = "novel")}
            class="flex-1 flex items-center gap-2 p-3 rounded-lg border-2 transition-colors {projectType ===
            'novel'
              ? 'border-accent bg-accent/10'
              : 'border-bg-card hover:border-accent/50'}"
          >
            <BookOpen class="w-5 h-5 text-accent" />
            <span class="text-text-primary font-medium">Novel</span>
          </button>
          <button
            type="button"
            onclick={() => (projectType = "screenplay")}
            class="flex-1 flex items-center gap-2 p-3 rounded-lg border-2 transition-colors {projectType ===
            'screenplay'
              ? 'border-accent bg-accent/10'
              : 'border-bg-card hover:border-accent/50'}"
          >
            <Film class="w-5 h-5 text-accent" />
            <span class="text-text-primary font-medium">Screenplay</span>
          </button>
        </div>
      </div>

      <div>
        <label for="new-project-name" class="block text-sm font-medium text-text-secondary mb-2">
          Name
        </label>
        <input
          id="new-project-name"
          bind:this={inputRef}
          bind:value={name}
          type="text"
          class={inputClass}
          placeholder="Enter project name..."
          disabled={saving}
        />
      </div>

      {#if projectType === "screenplay"}
        <div>
          <label for="target-length" class="block text-sm font-medium text-text-secondary mb-2">
            Target length
          </label>
          <select id="target-length" bind:value={targetLength} class={inputClass} disabled={saving}>
            <option value="short">Short (&lt;30 pages)</option>
            <option value="feature">Feature (90–120 pages)</option>
            <option value="long_feature">Long feature (120–180 pages)</option>
          </select>
        </div>
      {/if}

      {#if error}
        <p class="text-sm text-red-400">{error}</p>
      {/if}
    </div>

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
        onclick={handleCreate}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        disabled={saving || !name.trim()}
      >
        {#if saving}
          <Loader2 class="w-4 h-4 animate-spin" />
        {/if}
        Create
      </button>
    </div>
  </div>
</div>
