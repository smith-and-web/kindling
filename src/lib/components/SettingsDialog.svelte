<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { TriangleAlert } from "lucide-svelte";
  import type { Project } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import KeyboardSettings from "./KeyboardSettings.svelte";
  import AppearanceSettings from "./AppearanceSettings.svelte";
  import AuthorSettings from "./AuthorSettings.svelte";
  import ProjectSettings from "./ProjectSettings.svelte";

  let { onClose }: { onClose: () => void } = $props();
  const groups = [
    {
      id: "kindling",
      label: "kindling",
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
  let pane = $state<HTMLElement | null>(null);

  let projects = $state<Project[]>([]);
  let selectedId = $state(untrack(() => currentProject.value?.id ?? ""));
  // Each area starts at its top; the pane is shared, so its scroll would carry over.
  $effect(() => {
    void area;
    void selectedId;
    if (pane) pane.scrollTop = 0;
  });
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

  let tourAfterClose = false;

  function requestClose() {
    if (busy) return;
    if (authorDirty || projectDirty) pending = { close: true };
    else onClose();
  }

  /** The tour runs over the app, so Settings closes first (asking about unsaved changes). */
  function startTour() {
    if (busy) return;
    if (authorDirty || projectDirty) {
      tourAfterClose = true;
      pending = { close: true };
      return;
    }
    onClose();
    beginTour();
  }

  function keepEditing() {
    pending = null;
    tourAfterClose = false;
  }

  function beginTour() {
    ui.startOnboarding();
    ui.goToStep("tour-sidebar");
  }

  function discard() {
    if (!pending || busy) return;
    if ("close" in pending) {
      onClose();
      if (tourAfterClose) beginTour();
    } else {
      applyProjectSelection(pending.projectId);
      projectDirty = false;
    }
    pending = null;
  }

  function projectSaved(project: Project) {
    projects = projects.map((item) => (item.id === project.id ? project : item));
    if (currentProject.value?.id === project.id) currentProject.setProject(project);
  }

  function focusOnMount(node: HTMLElement) {
    queueMicrotask(() => node.focus());
  }

  function openDialog(node: HTMLDialogElement) {
    node.showModal();
    // Start on the chosen area, not the close button.
    node.querySelector<HTMLElement>('nav [aria-current="page"]')?.focus();
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
      if (pending) keepEditing();
      else requestClose();
    }
  }
</script>

<dialog
  use:openDialog
  aria-labelledby="settings-dialog-title"
  data-testid="settings-dialog"
  class="app-dialog-surface ka-dialog-settings settings"
  oncancel={(event) => {
    event.preventDefault();
    requestClose();
  }}
  onkeydown={handleKeydown}
>
  <DialogHeader
    title="Settings"
    titleId="settings-dialog-title"
    onClose={requestClose}
    closeLabel="Close settings"
    closeTestId="settings-close"
    disabled={busy}
  />
  {#if pending}
    <div role="alert" class="ka-dialog-alert">
      <TriangleAlert class="w-5 h-5 ka-icon" aria-hidden="true" />
      <p>
        {"close" in pending
          ? "Discard unsaved settings changes?"
          : "Discard unsaved changes for this project before switching?"}
      </p>
      <div class="ka-row">
        <button
          type="button"
          onclick={keepEditing}
          class="ka-button ka-button--secondary"
          use:focusOnMount>Keep editing</button
        >
        <button type="button" onclick={discard} class="ka-button ka-button--danger"
          >Discard changes</button
        >
      </div>
    </div>
  {/if}
  <div class="ka-settings" inert={pending !== null}>
    <nav aria-label="Settings areas" class="ka-settings-nav">
      {#each groups as group (group.id)}
        <section aria-labelledby={`settings-${group.id}-heading`}>
          <h3 id={`settings-${group.id}-heading`} class="settings-group">
            {group.label}
          </h3>
          {#if group.id === "projects"}
            {#if loading}
              <p role="status" class="ka-help">Loading projects…</p>
            {:else if error}
              <p role="alert" class="ka-error">{error}</p>
              <button type="button" onclick={loadProjects} class="ka-button ka-button--secondary"
                >Retry loading projects</button
              >
            {:else if projects.length === 0}
              <p class="ka-help">No projects available.</p>
            {:else}
              <div class="ka-field od-field">
                <label for="settings-project" class="ka-sr">Project</label>
                <select
                  id="settings-project"
                  value={selectedId}
                  onchange={selectProject}
                  disabled={busy}
                  title={selected?.name}
                  aria-describedby="settings-project-help"
                >
                  {#each projects as project (project.id)}<option value={project.id}
                      >{project.name}</option
                    >{/each}
                </select>
                <p id="settings-project-help" class="ka-help">
                  Settings for this project. Your open manuscript stays in place.
                </p>
              </div>
            {/if}
          {:else}
            <p class="ka-help">Applies to all projects.</p>
          {/if}
          {#each group.sections as section (section.id)}
            <div class="settings-section">
              {#if group.sections.length > 1}<h4 class="ka-eyebrow">{section.label}</h4>{/if}
              <ul class="ka-tree-list settings-areas">
                {#each section.areas as item (item.id)}
                  <li class="ka-tree">
                    <button
                      type="button"
                      onclick={() => (area = item.id)}
                      disabled={busy}
                      aria-current={area === item.id ? "page" : undefined}
                      class:ka-tree-selected={area === item.id}>{item.label}</button
                    >
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </section>
      {/each}
    </nav>
    <div class="ka-settings-pane" bind:this={pane}>
      <div>
        <header class="ka-settings-head">
          <p class="ka-crumbs" data-testid="settings-location">{locationLabel}</p>
          <h3 class="settings-title">{activeArea?.label}</h3>
        </header>
        {#if area === "keyboard"}<KeyboardSettings bind:busy={keyboardBusy} />{/if}
        <div hidden={area !== "appearance"}><AppearanceSettings onStartTour={startTour} /></div>
        <div hidden={area !== "author"}>
          <AuthorSettings bind:dirty={authorDirty} bind:busy={authorBusy} />
        </div>
        <div hidden={!isProjectArea}>
          {#if !loading && !error && projects.length === 0}
            <p class="ka-help">
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
  .settings {
    margin: auto;
    padding: 0;
    overflow: hidden;
  }
  .settings[open] {
    display: flex;
    flex-direction: column;
  }
  .settings::backdrop {
    background: var(--color-overlay-scrim);
  }
  .settings-group {
    margin: 0;
    padding: 0 var(--space-xs);
    font: 600 var(--text-base) / 1.5 var(--font-ui);
    letter-spacing: 0;
    color: var(--color-text);
  }
  .ka-settings-nav :global(.ka-help) {
    margin: 0;
  }
  .settings-section {
    display: grid;
    gap: var(--space-3xs);
  }
  .settings-section .ka-eyebrow {
    margin: 0;
  }
  .settings-areas {
    gap: var(--space-3xs);
  }
  .settings-areas .ka-tree button {
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .settings-title {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
</style>
