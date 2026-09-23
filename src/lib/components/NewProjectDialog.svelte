<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { BookOpen, ChevronRight, Film, Layout, Loader2, X } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project, ProjectType, StoryTemplate } from "../types";
  import TemplateBrowser from "./TemplateBrowser.svelte";
  import DialogHeader from "./DialogHeader.svelte";

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
    const onCreated = window.__KINDLING_TEST__?.onProjectCreated;

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

      onCreated?.(
        projectType === "screenplay" ? "create_screenplay_project" : "create_blank_project",
        project
      );

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
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="dialog-scrim"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="new-project-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title="New project"
      titleId="new-project-dialog-title"
      {onClose}
      closeLabel="Close"
      closeTestId="new-project-close"
    />

    <div class="ka-dialog-body new-project">
      <fieldset class="ka-segments">
        <legend>Project type</legend>
        <div class="ka-segment-track">
          <label class="ka-segment" class:ka-selected={projectType === "novel"}>
            <input type="radio" name="project-type" value="novel" bind:group={projectType} />
            <BookOpen class="w-5 h-5" aria-hidden="true" />
            <span>Novel</span>
          </label>
          <label class="ka-segment" class:ka-selected={projectType === "screenplay"}>
            <input type="radio" name="project-type" value="screenplay" bind:group={projectType} />
            <Film class="w-5 h-5" aria-hidden="true" />
            <span>Screenplay</span>
          </label>
        </div>
      </fieldset>

      <div class="ka-field od-field">
        <label for="new-project-name">Name</label>
        <input
          id="new-project-name"
          bind:this={inputRef}
          bind:value={name}
          type="text"
          placeholder="Enter project name…"
          disabled={saving}
          aria-invalid={error ? true : undefined}
          aria-describedby={error ? "new-project-error" : undefined}
        />
      </div>

      {#if projectType === "screenplay"}
        <div class="ka-field od-field">
          <label for="target-length">Target length</label>
          <select id="target-length" bind:value={targetLength} disabled={saving}>
            <option value="short">Short (&lt;30 pages)</option>
            <option value="feature">Feature (90–120 pages)</option>
            <option value="long_feature">Long feature (120–180 pages)</option>
          </select>
        </div>
      {/if}

      <fieldset class="new-project-template">
        <legend>Structure template <span class="ka-optional">(optional)</span></legend>
        {#if selectedTemplate}
          <div class="new-project-chosen">
            <Layout class="w-5 h-5" aria-hidden="true" />
            <span>{selectedTemplate.name}</span>
            <button
              type="button"
              onclick={() => (selectedTemplate = null)}
              class="ka-button ka-button--ghost ka-icon-button"
              aria-label="Remove template"
              title="Remove template"
            >
              <X class="w-5 h-5" aria-hidden="true" />
            </button>
          </div>
        {:else}
          <button
            type="button"
            onclick={() => (showTemplateBrowser = true)}
            class="ka-button ka-button--secondary new-project-browse"
            disabled={saving}
          >
            <Layout class="w-5 h-5" aria-hidden="true" />
            <span>Browse templates…</span>
            <ChevronRight class="w-5 h-5 ka-icon" aria-hidden="true" />
          </button>
        {/if}
      </fieldset>

      {#if error}
        <p id="new-project-error" class="ka-error" role="alert">{error}</p>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <button
        type="button"
        onclick={onClose}
        class="ka-button ka-button--secondary"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        data-testid="new-project-create"
        type="button"
        onclick={handleCreate}
        class="ka-button"
        disabled={saving || !name.trim()}
        aria-busy={saving || undefined}
      >
        {#if saving}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Creating…
        {:else}
          Create
        {/if}
      </button>
    </footer>
  </div>
</div>

{#if showTemplateBrowser}
  <TemplateBrowser
    {projectType}
    onSelect={handleTemplateSelect}
    onClose={() => (showTemplateBrowser = false)}
  />
{/if}

<style>
  .new-project {
    display: grid;
    gap: 20px;
  }
  .new-project-template {
    display: grid;
    justify-items: start;
    gap: var(--space-2xs);
    margin: 0;
    padding: 0;
    border: 0;
  }
  .new-project-template .ka-optional {
    font-weight: 400;
    color: var(--color-text-muted);
  }
  .new-project-browse {
    width: 100%;
    justify-content: flex-start;
  }
  .new-project-browse > span {
    flex: 1;
    text-align: left;
  }
  .new-project-template legend {
    margin-bottom: var(--space-2xs);
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .new-project-chosen {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    padding: 0 0 0 var(--space-xs);
    border: var(--border-hair);
    border-radius: var(--radius-m);
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .new-project-chosen span {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
