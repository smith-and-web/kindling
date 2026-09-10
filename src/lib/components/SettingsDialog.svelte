<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "lucide-svelte";
  import type { Project } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import AppearanceSettings from "./AppearanceSettings.svelte";
  import AuthorSettings from "./AuthorSettings.svelte";
  import ProjectSettings from "./ProjectSettings.svelte";

  let { onClose }: { onClose: () => void } = $props();
  const areas = [
    { id: "appearance", label: "Appearance & Guidance" },
    { id: "author", label: "Author & Contact" },
    { id: "details", label: "Project Details" },
    { id: "references", label: "Reference Types" },
    { id: "tags", label: "Tags" },
    { id: "fields", label: "Custom Fields" },
  ];
  let area = $state("appearance");
  let projects = $state<Project[]>([]);
  let selectedId = $state(untrack(() => currentProject.value?.id ?? ""));
  let loading = $state(true);
  let error = $state<string | null>(null);
  let authorDirty = $state(false);
  let projectDirty = $state(false);
  let authorBusy = $state(false);
  let projectBusy = $state(false);
  let pending = $state<{ projectId: string } | { close: true } | null>(null);
  const busy = $derived(authorBusy || projectBusy);
  const selected = $derived(projects.find((project) => project.id === selectedId));
  const isProjectArea = $derived(!["appearance", "author"].includes(area));

  onMount(() => {
    void loadProjects();
  });

  async function loadProjects() {
    loading = true;
    error = null;
    try {
      projects = await invoke<Project[]>("get_all_projects");
      if (!projects.some((project) => project.id === selectedId))
        selectedId = projects[0]?.id ?? "";
    } catch (e) {
      error = `Could not load projects: ${String(e)}`;
    } finally {
      loading = false;
    }
  }

  function selectProject(event: Event) {
    const select = event.target as HTMLSelectElement;
    const id = select.value;
    select.value = selectedId;
    if (id === selectedId || busy) return;
    if (projectDirty) pending = { projectId: id };
    else selectedId = id;
  }

  function requestClose() {
    if (busy) return;
    if (authorDirty || projectDirty) pending = { close: true };
    else onClose();
  }

  function discard() {
    if (!pending || busy) return;
    if ("close" in pending) onClose();
    else {
      selectedId = pending.projectId;
      projectDirty = false;
    }
    pending = null;
  }

  function projectSaved(project: Project) {
    projects = projects.map((item) => (item.id === project.id ? project : item));
    if (currentProject.value?.id === project.id) currentProject.setProject(project);
  }

  function openDialog(node: HTMLDialogElement) {
    node.showModal();
    return {
      destroy() {
        node.close();
      },
    };
  }

  function handleKeydown(event: KeyboardEvent) {
    // Keep editor/window shortcuts out of the settings controls.
    event.stopPropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      if (pending) pending = null;
      else requestClose();
    }
  }
</script>

<dialog
  use:openDialog
  aria-labelledby="settings-dialog-title"
  data-testid="settings-dialog"
  class="app-dialog-surface m-auto p-0 w-[calc(100%-2rem)] max-w-5xl h-[85vh] max-h-[85vh] bg-press-surface text-press-text border border-press-border rounded-lg shadow-press-overlay overflow-hidden"
  oncancel={(event) => {
    event.preventDefault();
    requestClose();
  }}
  onkeydown={handleKeydown}
>
  <div class="flex flex-col h-full">
    <header
      class="flex items-center justify-between px-6 py-4 border-b border-press-border shrink-0"
    >
      <h2 id="settings-dialog-title" class="font-heading text-press-body-lg">Settings</h2>
      <button
        type="button"
        onclick={requestClose}
        disabled={busy}
        aria-label="Close settings"
        data-testid="settings-close"
        class="p-2 rounded text-press-muted hover:text-press-text"><X class="w-5 h-5" /></button
      >
    </header>
    {#if pending}
      <div role="alert" class="px-6 py-4 border-b border-press-border bg-press-sunken space-y-3">
        <p class="text-press-ui">
          {"close" in pending
            ? "Discard unsaved settings changes?"
            : "Discard unsaved changes for this project before switching?"}
        </p>
        <div class="flex gap-3">
          <button
            type="button"
            onclick={() => (pending = null)}
            class="px-3 py-2 border border-press-border rounded">Keep editing</button
          >
          <button
            type="button"
            onclick={discard}
            class="px-3 py-2 bg-press-error text-press-on-accent rounded">Discard changes</button
          >
        </div>
      </div>
    {/if}
    <div class="flex min-h-0 flex-1" inert={pending !== null}>
      <nav
        aria-label="Settings areas"
        class="w-48 sm:w-60 shrink-0 p-3 border-r border-press-border bg-press-sunken overflow-y-auto space-y-1"
      >
        {#each areas as item (item.id)}
          {#if item.id === "details"}<p
              class="px-3 pt-6 pb-2 text-press-eyebrow text-press-muted uppercase tracking-wide"
            >
              Projects
            </p>{/if}
          <button
            type="button"
            onclick={() => (area = item.id)}
            disabled={busy}
            aria-current={area === item.id ? "page" : undefined}
            class="w-full text-left px-3 py-2 rounded text-press-ui {area === item.id
              ? 'bg-press-accent-wash text-press-accent-text font-medium'
              : 'text-press-muted hover:text-press-text hover:bg-press-surface'}"
            >{item.label}</button
          >
        {/each}
      </nav>
      <div class="flex-1 min-w-0 overflow-y-auto p-6 space-y-5">
        <h3 class="font-heading text-press-body-lg">
          {areas.find((item) => item.id === area)?.label}
        </h3>
        <div hidden={area !== "appearance"}><AppearanceSettings /></div>
        <div hidden={area !== "author"}>
          <AuthorSettings bind:dirty={authorDirty} bind:busy={authorBusy} />
        </div>
        <div hidden={!isProjectArea} class="space-y-5">
          {#if loading}<p role="status" class="text-press-ui text-press-muted">Loading projects…</p>
          {:else if error}
            <p role="alert" class="text-press-ui text-press-error">{error}</p>
            <button type="button" onclick={loadProjects}>Retry loading projects</button>
          {:else if projects.length === 0}
            <p class="text-press-ui text-press-muted">
              No projects yet. Create or import a project to configure its settings.
            </p>
          {:else}
            <div>
              <label for="settings-project" class="block text-press-ui text-press-muted mb-2"
                >Project</label
              >
              <select
                id="settings-project"
                value={selectedId}
                onchange={selectProject}
                disabled={busy}
                class="w-full bg-press-sunken border border-press-border rounded-lg px-3 py-2 text-press-text"
              >
                {#each projects as project (project.id)}<option value={project.id}
                    >{project.name}</option
                  >{/each}
              </select>
              <p class="text-press-eyebrow text-press-muted mt-2">
                Choose any project without changing the manuscript open in your editor.
              </p>
            </div>
            {#if selected}
              {#key selected.id}<ProjectSettings
                  project={selected}
                  section={area}
                  onSave={projectSaved}
                  bind:dirty={projectDirty}
                  bind:busy={projectBusy}
                />{/key}
            {/if}
          {/if}
        </div>
      </div>
    </div>
  </div>
</dialog>

<style>
  dialog::backdrop {
    background: var(--color-overlay-scrim);
  }
</style>
