<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { BookOpen, Film, Layout, Loader2, X } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project, ProjectType, StoryTemplate } from "../types";
  import TemplateBrowser from "./TemplateBrowser.svelte";
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
  let selectedTemplate = $state<StoryTemplate | null>(null);
  let showTemplateBrowser = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let inputRef: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (inputRef && !showTemplateBrowser) {
      inputRef.focus();
      inputRef.select();
    }
  });

  function handleTemplateSelect(template: StoryTemplate) {
    selectedTemplate = template;
    showTemplateBrowser = false;
  }

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

      if (selectedTemplate) {
        await invoke("apply_template", {
          projectId: project.id,
          templateJson: JSON.stringify(selectedTemplate),
          clearExisting: true,
        });
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
    "w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent";
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="new-project-dialog-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
  >
    <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
      <h2 id="new-project-dialog-title" class="text-press-body-lg font-medium text-press-text">
        New Project
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          aria-label="Close"
          data-testid="new-project-close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <div class="p-4 space-y-4">
      <fieldset>
        <legend class="block text-press-ui font-medium text-press-muted mb-2">Project type</legend>
        <div class="flex gap-2">
          <button
            type="button"
            onclick={() => (projectType = "novel")}
            class="flex-1 flex items-center gap-2 p-3 rounded-lg border-2 transition-colors {projectType ===
            'novel'
              ? 'border-press-accent bg-press-accent-wash'
              : 'border-press-border hover:border-press-accent'}"
          >
            <BookOpen class="w-5 h-5 text-press-accent-text" />
            <span class="text-press-text font-medium">Novel</span>
          </button>
          <button
            type="button"
            onclick={() => (projectType = "screenplay")}
            class="flex-1 flex items-center gap-2 p-3 rounded-lg border-2 transition-colors {projectType ===
            'screenplay'
              ? 'border-press-accent bg-press-accent-wash'
              : 'border-press-border hover:border-press-accent'}"
          >
            <Film class="w-5 h-5 text-press-accent-text" />
            <span class="text-press-text font-medium">Screenplay</span>
          </button>
        </div>
      </fieldset>

      <div>
        <label for="new-project-name" class="block text-press-ui font-medium text-press-muted mb-2">
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
          <label for="target-length" class="block text-press-ui font-medium text-press-muted mb-2">
            Target length
          </label>
          <select id="target-length" bind:value={targetLength} class={inputClass} disabled={saving}>
            <option value="short">Short (&lt;30 pages)</option>
            <option value="feature">Feature (90–120 pages)</option>
            <option value="long_feature">Long feature (120–180 pages)</option>
          </select>
        </div>
      {/if}

      <fieldset>
        <legend class="block text-press-ui font-medium text-press-muted mb-2"
          >Structure template</legend
        >
        {#if selectedTemplate}
          <div
            class="flex items-center gap-2 px-3 py-2 bg-press-accent-wash border border-press-accent rounded-lg"
          >
            <Layout class="w-4 h-4 text-press-accent-text shrink-0" />
            <span class="text-press-ui text-press-text flex-1 truncate"
              >{selectedTemplate.name}</span
            >
            <button
              type="button"
              onclick={() => (selectedTemplate = null)}
              class="text-press-muted hover:text-press-text p-0.5"
              aria-label="Remove template"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        {:else}
          <button
            type="button"
            onclick={() => (showTemplateBrowser = true)}
            class="w-full text-left px-3 py-2 text-press-ui text-press-muted bg-press-sunken border border-press-border rounded-lg hover:border-press-accent transition-colors"
            disabled={saving}
          >
            Browse templates...
          </button>
        {/if}
      </fieldset>

      {#if error}
        <p class="text-press-ui text-press-error">{error}</p>
      {/if}
    </div>

    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        data-testid="new-project-create"
        type="button"
        onclick={handleCreate}
        class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors flex items-center gap-2"
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

{#if showTemplateBrowser}
  <TemplateBrowser
    {projectType}
    onSelect={handleTemplateSelect}
    onClose={() => (showTemplateBrowser = false)}
  />
{/if}
