<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "lucide-svelte";
  import type { Project } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import KeyboardSettings from "./KeyboardSettings.svelte";
  import AppearanceSettings from "./AppearanceSettings.svelte";
  import AuthorSettings from "./AuthorSettings.svelte";
  import ProjectSettings from "./ProjectSettings.svelte";

  let { onClose }: { onClose: () => void } = $props();
  const groups = [
    {
      id: "kindling",
      label: "Kindling",
      sections: [
        {
          id: "preferences",
          label: "Preferences",
          areas: [
            { id: "appearance", label: "Appearance & Guidance" },
            { id: "author", label: "Author & Contact" },
            { id: "keyboard", label: "Keyboard Shortcuts" },
          ],
        },
      ],
    },
    {
      id: "projects",
      label: "Projects",
      sections: [
        {
          id: "manuscript",
          label: "Manuscript",
          areas: [{ id: "details", label: "Project Details" }],
        },
        {
          id: "reference-library",
          label: "Reference Library",
          areas: [
            { id: "references", label: "Reference Types" },
            { id: "tags", label: "Tags" },
            { id: "fields", label: "Custom Fields" },
          ],
        },
      ],
    },
  ];
  let area = $state("appearance");
  let projects = $state<Project[]>([]);
  let selectedId = $state(untrack(() => currentProject.value?.id ?? ""));
  let loading = $state(true);
  let error = $state<string | null>(null);
  let authorDirty = $state(false);
  let projectDirty = $state(false);
  let keyboardBusy = $state(false);
  let authorBusy = $state(false);
  let projectBusy = $state(false);
  let pending = $state<{ projectId: string } | { close: true } | null>(null);
  const busy = $derived(authorBusy || projectBusy || keyboardBusy);
  const selected = $derived(projects.find((project) => project.id === selectedId));
  const activeGroup = $derived(
    groups.find((group) =>
      group.sections.some((section) => section.areas.some((item) => item.id === area))
    ) ?? groups[0]
  );
  const activeSection = $derived(
    activeGroup.sections.find((section) => section.areas.some((item) => item.id === area)) ??
      activeGroup.sections[0]
  );
  const activeArea = $derived(activeSection.areas.find((item) => item.id === area));
  const isProjectArea = $derived(activeGroup.id === "projects");
  const locationLabel = $derived(
    [activeGroup.label, isProjectArea ? selected?.name : undefined, activeSection.label]
      .filter(Boolean)
      .join(" / ")
  );

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
    else applyProjectSelection(id);
  }

  function applyProjectSelection(id: string) {
    selectedId = id;
    if (!isProjectArea) area = "details";
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
      applyProjectSelection(pending.projectId);
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
        class="w-56 sm:w-72 shrink-0 p-4 border-r border-press-border bg-press-sunken overflow-y-auto space-y-6"
      >
        {#each groups as group (group.id)}
          <section aria-labelledby={`settings-${group.id}-heading`} class="space-y-3">
            <h3
              id={`settings-${group.id}-heading`}
              class="text-press-ui font-semibold text-press-text"
            >
              {group.label}
            </h3>
            {#if group.id === "projects"}
              {#if loading}
                <p role="status" class="text-press-ui text-press-muted">Loading projects…</p>
              {:else if error}
                <p role="alert" class="text-press-ui text-press-error">{error}</p>
                <button
                  type="button"
                  onclick={loadProjects}
                  class="text-press-ui text-press-accent-text underline"
                  >Retry loading projects</button
                >
              {:else if projects.length === 0}
                <p class="text-press-ui text-press-muted">No projects available.</p>
              {:else}
                <div>
                  <label
                    for="settings-project"
                    class="block text-press-eyebrow text-press-muted mb-1">Project</label
                  >
                  <select
                    id="settings-project"
                    value={selectedId}
                    onchange={selectProject}
                    disabled={busy}
                    title={selected?.name}
                    aria-describedby="settings-project-help"
                    class="w-full min-w-0 bg-press-surface border border-press-border rounded-lg px-2 py-2 text-press-text"
                  >
                    {#each projects as project (project.id)}<option value={project.id}
                        >{project.name}</option
                      >{/each}
                  </select>
                  <p id="settings-project-help" class="text-press-eyebrow text-press-muted mt-2">
                    Settings for this project. Your open manuscript stays in place.
                  </p>
                </div>
              {/if}
            {:else}
              <p class="text-press-eyebrow text-press-muted">Applies to all projects.</p>
            {/if}
            {#each group.sections as section (section.id)}
              <div class="space-y-1">
                <h4 class="text-press-eyebrow font-medium text-press-muted">{section.label}</h4>
                <ul class="ml-1 border-l border-press-border pl-2 space-y-1">
                  {#each section.areas as item (item.id)}
                    <li>
                      <button
                        type="button"
                        onclick={() => (area = item.id)}
                        disabled={busy}
                        aria-current={area === item.id ? "page" : undefined}
                        class="w-full text-left px-2 py-2 rounded text-press-ui {area === item.id
                          ? 'bg-press-accent-wash text-press-accent-text font-medium'
                          : 'text-press-muted hover:text-press-text hover:bg-press-surface'}"
                        >{item.label}</button
                      >
                    </li>
                  {/each}
                </ul>
              </div>
            {/each}
          </section>
        {/each}
      </nav>
      <div class="flex-1 min-w-0 overflow-y-auto p-6 space-y-5">
        <header class="space-y-2">
          <p class="text-press-eyebrow text-press-muted" data-testid="settings-location">
            {locationLabel}
          </p>
          <h3 class="font-heading text-press-body-lg">{activeArea?.label}</h3>
        </header>
        {#if area === "keyboard"}<KeyboardSettings bind:busy={keyboardBusy} />{/if}
        <div hidden={area !== "appearance"}><AppearanceSettings /></div>
        <div hidden={area !== "author"}>
          <AuthorSettings bind:dirty={authorDirty} bind:busy={authorBusy} />
        </div>
        <div hidden={!isProjectArea} class="space-y-5">
          {#if !loading && !error && projects.length === 0}
            <p class="text-press-ui text-press-muted">
              No projects yet. Create or import a project to configure its settings.
            </p>
          {:else if selected}
            {#key selected.id}<ProjectSettings
                project={selected}
                section={area}
                onSave={projectSaved}
                bind:dirty={projectDirty}
                bind:busy={projectBusy}
              />{/key}
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
