<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { runImport, type ImportType } from "../utils/import";
  import {
    BookOpen,
    CircleHelp,
    FileText,
    FolderOpen,
    Loader2,
    Plus,
    TriangleAlert,
    Trash2,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project } from "../types";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    recentProjects: Project[];
    onImportLongform?: () => void;
    onImportComplete?: (project: Project, type: ImportType) => void;
    onOpenQuickStart?: () => void;
    onNewProject?: () => void;
    onOpenEditorial?: () => void;
  }

  let {
    recentProjects = $bindable(),
    onImportLongform,
    onImportComplete,
    onOpenQuickStart,
    onNewProject,
    onOpenEditorial,
  }: Props = $props();

  let deletingProjectId = $state<string | null>(null);
  let openingProjectId = $state<string | null>(null);
  let startingSample = $state(false);
  let openError = $state<{ name: string; message: string } | null>(null);
  let projectToDelete = $state<Project | null>(null);
  let showAllProjects = $state(false);
  let allProjects = $state<Project[] | null>(null);

  /** The start screen lists the latest few; "View all" appears only when there are more. */
  const RECENT_SHOWN = 5;
  const listedProjects = $derived(
    showAllProjects && allProjects ? allProjects : recentProjects.slice(0, RECENT_SHOWN)
  );
  const hasMoreProjects = $derived(recentProjects.length > RECENT_SHOWN);

  async function handleImport(type: ImportType) {
    const project = await runImport(type);
    if (!project) return;
    currentProject.setProject(project);
    ui.setView("editor");
    onImportComplete?.(project, type);
  }

  const importPlottr = () => handleImport("plottr");
  const importMarkdown = () => handleImport("markdown");
  const importYWriter = () => handleImport("ywriter");
  const importLongform = () => handleImport("longform");
  const importScrivener = () => handleImport("scrivener");
  const importNovelWriter = () => handleImport("novelwriter");

  function handleLongformImport() {
    const handler = onImportLongform ?? importLongform;
    handler();
  }

  async function trySampleProject() {
    startingSample = true;
    ui.startImport("Opening sample project", "Creating the sample project…");
    try {
      const project = await invoke<Project>("create_sample_project");
      currentProject.setProject(project);
      ui.setView("editor");
    } catch (e) {
      console.error("Failed to create sample project:", e);
      ui.showError(`Failed to create sample project: ${e}`);
    } finally {
      ui.finishImport();
      startingSample = false;
    }
  }

  async function openProject(project: Project) {
    if (openingProjectId) return;
    openingProjectId = project.id;
    openError = null;
    try {
      const loaded = await invoke<Project>("get_project", { id: project.id });
      currentProject.setProject(loaded);
      ui.setView("editor");
      // Sidebar $effect will run loadChapters
    } catch (e) {
      console.error("Failed to open project:", e);
      openError = { name: project.name, message: String(e) };
    } finally {
      openingProjectId = null;
    }
  }

  // The importer id reads as plain metadata, spelled as each tool spells itself.
  const SOURCE_LABELS: Record<string, string> = {
    blank: "Blank",
    markdown: "Markdown",
    plottr: "Plottr",
    ywriter: "yWriter",
    longform: "Longform",
    scrivener: "Scrivener",
    novelwriter: "novelWriter",
  };
  function sourceLabel(sourceType: string) {
    return SOURCE_LABELS[sourceType.toLowerCase()] ?? sourceType;
  }

  function modifiedDate(value: string) {
    const date = new Date(value);
    return {
      short: date.toLocaleDateString(undefined, {
        day: "numeric",
        month: "short",
        year: "numeric",
      }),
      long: date.toLocaleDateString(undefined, { dateStyle: "long" }),
    };
  }

  const IMPORTS: { label: string; hint: string; run: () => void }[] = [
    { label: "Plottr", hint: ".pltr", run: () => importPlottr() },
    { label: "yWriter", hint: ".yw7", run: () => importYWriter() },
    { label: "Markdown", hint: ".md", run: () => importMarkdown() },
    { label: "Longform", hint: "Index or vault", run: () => handleLongformImport() },
    { label: "Scrivener", hint: ".scriv", run: () => importScrivener() },
    { label: "novelWriter", hint: "Project folder", run: () => importNovelWriter() },
  ];

  let loadingProjectList = $state(false);

  async function toggleProjectList() {
    if (loadingProjectList) return;
    openError = null;
    if (showAllProjects) {
      showAllProjects = false;
      return;
    }
    loadingProjectList = true;
    try {
      allProjects = await invoke<Project[]>("get_all_projects");
      showAllProjects = true;
    } catch (e) {
      console.error("Failed to load projects:", e);
      ui.showError(`Failed to load projects: ${e}`);
    } finally {
      loadingProjectList = false;
    }
  }

  function showDeleteConfirmation(event: MouseEvent, project: Project) {
    event.stopPropagation();
    projectToDelete = project;
  }

  async function confirmDeleteProject() {
    if (!projectToDelete) return;

    const project = projectToDelete;
    projectToDelete = null;
    deletingProjectId = project.id;

    try {
      await invoke("delete_project", { projectId: project.id });
      recentProjects = recentProjects.filter((p) => p.id !== project.id);
      allProjects = allProjects?.filter((p) => p.id !== project.id) ?? null;
    } catch (e) {
      console.error("Failed to delete project:", e);
      ui.showError(`Failed to delete project: ${e}`);
    } finally {
      deletingProjectId = null;
    }
  }

  function cancelDeleteProject() {
    projectToDelete = null;
  }
</script>

<div class="start">
  <div class="start-bar">
    {#if onOpenQuickStart}
      <button type="button" class="ka-button ka-button--ghost" onclick={onOpenQuickStart}>
        <CircleHelp class="w-5 h-5" aria-hidden="true" />
        Quick start
      </button>
    {/if}
  </div>

  <div class="start-body">
    <div class="start-intro">
      <h1 class="ka-sr">kindling</h1>
      <div class="signature">
        <figure class="brand">
          <img
            class="on-light"
            src="/brand/kindling-lockup-stacked.svg"
            alt=""
            width="160"
            height="132"
          />
          <img
            class="on-dark"
            src="/brand/kindling-lockup-stacked-reversed.svg"
            alt=""
            width="160"
            height="132"
          />
        </figure>
        <p class="tagline">Spark your draft</p>
      </div>

      {#if onNewProject || onOpenEditorial}
        <div class="start-actions" role="group" aria-label="Start">
          {#if onNewProject}
            <button
              type="button"
              data-testid="new-project-button"
              onclick={onNewProject}
              class="ka-button start-action"
            >
              <Plus class="w-5 h-5" aria-hidden="true" />
              <span>New project <small>Start from scratch</small></span>
            </button>
            <button
              type="button"
              onclick={trySampleProject}
              disabled={startingSample}
              aria-busy={startingSample || undefined}
              class="ka-button ka-button--secondary start-action"
            >
              <BookOpen class="w-5 h-5" aria-hidden="true" />
              <span>
                {startingSample ? "Opening sample…" : "Sample project"}
                <small>Explore kindling first</small>
              </span>
            </button>
          {/if}
          {#if onOpenEditorial}
            <button
              type="button"
              onclick={onOpenEditorial}
              class="ka-button ka-button--secondary start-action"
            >
              <FolderOpen class="w-5 h-5" aria-hidden="true" />
              <span>Open review package <small>Open a review or feedback file</small></span>
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <div class="start-main">
      {#if recentProjects.length > 0}
        <section data-testid="recent-projects" aria-labelledby="recent-projects-heading">
          <div class="section-head">
            <h2 id="recent-projects-heading" class="section-title">
              {showAllProjects ? "All projects" : "Recent projects"}
            </h2>
            {#if hasMoreProjects || showAllProjects}
              <button
                type="button"
                class="ka-button ka-button--ghost"
                aria-pressed={showAllProjects}
                aria-busy={loadingProjectList || undefined}
                onclick={toggleProjectList}
              >
                {showAllProjects ? "Show recent" : "View all"}
              </button>
            {/if}
          </div>
          {#if openError}
            <div class="ka-notice ka-notice--error od-row-top open-error" role="alert">
              <TriangleAlert class="w-5 h-5 shrink-0" aria-hidden="true" />
              <div class="od-field od-fill">
                <strong>Couldn’t open “{openError.name}”</strong>
                <p>{openError.message}</p>
              </div>
            </div>
          {/if}
          <ul class="recent-list">
            {#each listedProjects as project (project.id)}
              {@const modified = modifiedDate(project.modified_at)}
              <li class="recent-row" aria-busy={openingProjectId === project.id || undefined}>
                <button
                  type="button"
                  data-testid="project-card"
                  onclick={() => openProject(project)}
                  class="recent-open"
                >
                  <span class="recent-title">
                    <span class="recent-name">{project.name}</span>
                    <span class="recent-type">{sourceLabel(project.source_type)}</span>
                  </span>
                  {#if openingProjectId === project.id}
                    <span class="recent-date">Opening…</span>
                  {:else}
                    <time
                      class="recent-date"
                      datetime={project.modified_at}
                      aria-label={`Modified ${modified.long}`}>{modified.short}</time
                    >
                  {/if}
                </button>
                <button
                  type="button"
                  onclick={(e) => showDeleteConfirmation(e, project)}
                  disabled={deletingProjectId === project.id || openingProjectId === project.id}
                  class="ka-button ka-button--ghost ka-icon-button recent-delete"
                  class:is-busy={deletingProjectId === project.id}
                  aria-label={`Delete ${project.name}`}
                  title="Delete project"
                >
                  {#if deletingProjectId === project.id}
                    <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                  {:else}
                    <Trash2 class="w-5 h-5" aria-hidden="true" />
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        </section>
      {:else}
        <section aria-label="Projects">
          <div class="ka-empty od-stack">
            <FileText class="w-7 h-7" aria-hidden="true" />
            <h2 class="section-title">Your projects will appear here</h2>
            <p>Start a new project, try the sample, or import an outline you already have.</p>
          </div>
        </section>
      {/if}

      <section data-testid="import-section" class="import" aria-labelledby="import-heading">
        <h2 id="import-heading" class="section-title">Import an outline</h2>
        <p class="ka-help section-help">
          Chapters, scenes and beats from your outline become prompts in the writing view.
        </p>
        <ul class="import-list">
          {#each IMPORTS as format (format.label)}
            <li>
              <button
                type="button"
                class="ka-button ka-button--ghost import-format"
                onclick={format.run}
              >
                {format.label}
                <small>{format.hint}</small>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    </div>
  </div>
</div>

<!-- Delete Project Confirmation -->
{#if projectToDelete}
  <ConfirmDialog
    title={`Delete “${projectToDelete.name}”?`}
    message="This permanently deletes the project and all its chapters, scenes, beats and snapshots. It can’t be undone."
    confirmLabel="Delete project"
    onConfirm={confirmDeleteProject}
    onCancel={cancelDeleteProject}
  />
{/if}

<style>
  .start {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    overflow: hidden;
    container-type: size;
    container-name: start;
  }
  /* Quick start shares the page frame's right edge with the lists below. */
  .start-bar {
    display: flex;
    justify-content: flex-end;
    width: min(var(--page-frame), 100% - 2 * var(--page-gutter));
    margin-inline: auto;
    padding-top: var(--space-s);
    min-height: calc(var(--control-target) + var(--space-s));
  }
  .start-body {
    display: grid;
    grid-template-columns: 360px minmax(0, 1fr);
    column-gap: var(--space-3xl);
    min-height: 0;
    width: min(var(--page-frame), 100% - 2 * var(--page-gutter));
    margin-inline: auto;
    padding-block: var(--space-l) 0;
  }
  .start-intro {
    align-self: start;
  }
  /* The stacked lockup is composed on its centre line, so the tagline centres
     under it and the two read as one signature. Press asks for clear space of
     a quarter of the emblem height around the lockup; the tagline keeps it. */
  .signature {
    display: grid;
    justify-items: center;
    gap: var(--space-s);
    width: fit-content;
  }
  .brand {
    margin: 0;
  }
  .brand img {
    display: block;
    width: 160px;
    height: auto;
    aspect-ratio: 740.5 / 612.8;
  }
  .brand .on-dark {
    display: none;
  }
  :global([data-theme="dark"]) .brand .on-light {
    display: none;
  }
  :global([data-theme="dark"]) .brand .on-dark {
    display: block;
  }
  .tagline {
    margin: 0;
    text-align: center;
    font: italic 400 var(--text-body-lg) / var(--leading) var(--font-body);
    color: var(--color-text-muted);
  }
  .start-actions {
    display: grid;
    gap: var(--space-2xs);
    margin-top: var(--space-xl);
  }
  .start-action {
    justify-content: flex-start;
    width: 100%;
    min-height: 60px;
    padding: var(--space-xs) var(--space-s);
    gap: var(--space-xs);
    text-align: left;
  }
  .start-action span {
    display: grid;
  }
  .start-action small {
    font: 400 var(--text-small) / 1.4 var(--font-ui);
  }
  .ka-button--secondary.start-action small {
    color: var(--color-text-muted);
  }

  /* The main column scrolls on its own so the brand and start actions stay
     put. Inline padding keeps the 3px focus outline inside the scroll box. */
  .start-main {
    min-height: 0;
    overflow: auto;
    padding: 0 var(--space-3xs) var(--space-xl);
    margin-inline: calc(-1 * var(--space-3xs));
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    min-height: var(--control-target);
  }
  .section-title {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .section-help {
    margin-top: var(--space-3xs);
  }
  .open-error {
    margin-top: var(--space-2xs);
  }
  .open-error p {
    overflow-wrap: anywhere;
  }

  .recent-list {
    list-style: none;
    margin: var(--space-2xs) 0 0;
    padding: 0;
    border-top: var(--border-hair);
  }
  .recent-row {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    padding-right: var(--space-3xs);
    border-bottom: var(--border-hair);
    transition: background-color var(--ka-motion) ease-out;
  }
  .recent-row:hover,
  .recent-row:focus-within {
    background: var(--color-surface-sunken);
  }
  .recent-open {
    flex: 1 1 auto;
    min-width: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-m);
    min-height: 52px;
    padding: var(--space-xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .recent-title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    column-gap: var(--space-xs);
  }
  .recent-name {
    font: 500 var(--text-base) / 1.4 var(--font-ui);
    overflow-wrap: anywhere;
  }
  .recent-type,
  .recent-date {
    font: var(--text-small) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
  }
  .recent-date {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  /* Reserved even while hidden so the date never shifts; always reachable by
     keyboard, and always visible without hover. */
  .recent-delete {
    color: var(--color-text-muted);
    opacity: 0;
  }
  .recent-row:hover .recent-delete,
  .recent-row:focus-within .recent-delete,
  .recent-delete.is-busy {
    opacity: 1;
  }
  .recent-row[aria-busy="true"] .recent-delete {
    visibility: hidden;
  }
  @media (hover: none) {
    .recent-delete {
      opacity: 1;
    }
  }
  /* The ghost hover fill is the row's own hover fill, so the destructive
     control takes the error wash to stay distinguishable. */
  @media (hover: hover) {
    .recent-row .recent-delete:not(:disabled):hover {
      background: var(--color-error-wash);
      color: var(--color-error);
    }
  }

  .ka-empty {
    border-bottom: var(--border-hair);
    padding-top: var(--space-m);
  }
  .ka-empty :global(svg) {
    color: var(--color-text-muted);
  }

  .import {
    margin-top: var(--space-2xl);
  }
  .import-list {
    list-style: none;
    margin: var(--space-s) 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--space-m);
  }
  .import-list li {
    border-bottom: var(--border-hair);
  }
  .import-list li:nth-child(-n + 2) {
    border-top: var(--border-hair);
  }
  .import-format {
    width: 100%;
    justify-content: space-between;
    padding-inline: var(--space-xs);
    margin-block: var(--space-3xs);
    border-radius: var(--radius-xs);
  }
  .import-format small {
    font: 400 var(--text-small) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
  }

  @container start (max-width: 1280px) {
    .start-bar {
      width: calc(100% - 2 * var(--space-m));
      padding-top: var(--space-2xs);
    }
    .start-body {
      grid-template-columns: 300px minmax(0, 1fr);
      column-gap: var(--space-2xl);
      width: calc(100% - 2 * var(--space-m));
      padding-top: var(--space-2xs);
    }
    .brand img {
      width: 140px;
    }
    .start-actions {
      margin-top: var(--space-l);
    }
    .import {
      margin-top: var(--space-xl);
    }
  }
  @container start (max-height: 720px) {
    .start-body {
      padding-top: 0;
    }
    .import {
      margin-top: var(--space-l);
    }
  }
</style>
