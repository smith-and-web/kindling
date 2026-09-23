<!--
  SnapshotsPanel.svelte - Snapshot management panel

  Allows users to view, create, restore, and delete project snapshots.
  Similar to ArchivePanel but for versioning/restore points. Restore and
  delete are confirmed inline in the affected row.
-->
<script lang="ts">
  import {
    CheckCircle2,
    CircleAlert,
    History,
    Loader2,
    Plus,
    Trash2,
    TriangleAlert,
  } from "lucide-svelte";
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type {
    SnapshotMetadata,
    CreateSnapshotOptions,
    Project,
    Chapter,
    Character,
    Location,
  } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { proseSaves } from "../utils/proseSaves";
  import { synopsisSaves } from "../stores/synopsisSaves.svelte";

  let {
    onClose,
    prepareRestore,
  }: {
    onClose: () => void;
    prepareRestore: () => Promise<void>;
  } = $props();

  let loading = $state(true);
  let snapshots = $state<SnapshotMetadata[]>([]);
  let error = $state<string | null>(null);
  let restoringId = $state<string | null>(null);
  let deletingId = $state<string | null>(null);
  let creating = $state(false);

  // Inline create form state
  let showCreateDialog = $state(false);
  let newSnapshotName = $state("");
  let newSnapshotDescription = $state("");
  let createError = $state<string | null>(null);
  let createdName = $state<string | null>(null);

  // Inline row confirmation (restore or delete) state
  let confirming = $state<{ id: string; kind: "restore" | "delete" } | null>(null);
  let restoreMode = $state<"replace_current" | "create_new">("replace_current");
  let newProjectName = $state("");
  let rowError = $state<string | null>(null);

  const restoreTriggers: Record<string, HTMLElement | null> = {};
  const deleteTriggers: Record<string, HTMLElement | null> = {};
  let createTrigger = $state<HTMLElement | null>(null);

  $effect(() => {
    loadSnapshots();
  });

  function messageOf(e: unknown, fallback: string): string {
    if (e instanceof Error) return e.message;
    if (typeof e === "string" && e) return e;
    return fallback;
  }

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

  async function loadSnapshots() {
    if (!currentProject.value) return;

    loading = true;
    error = null;

    try {
      const items = await invoke<SnapshotMetadata[]>("list_snapshots", {
        projectId: currentProject.value.id,
      });
      snapshots = items;
    } catch (e) {
      error = messageOf(e, "Failed to load snapshots");
    } finally {
      loading = false;
    }
  }

  function openCreateDialog() {
    newSnapshotName = `Snapshot ${new Date().toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" })}`;
    newSnapshotDescription = "";
    createError = null;
    createdName = null;
    confirming = null;
    showCreateDialog = true;
  }

  async function closeCreateDialog() {
    if (creating) return;
    showCreateDialog = false;
    createError = null;
    await tick();
    createTrigger?.focus();
  }

  async function createSnapshot() {
    if (!currentProject.value || !newSnapshotName.trim() || creating) return;

    creating = true;
    createError = null;

    try {
      const options: CreateSnapshotOptions = {
        name: newSnapshotName.trim(),
        description: newSnapshotDescription.trim() || undefined,
        trigger_type: "manual",
      };

      const snapshot = await invoke<SnapshotMetadata>("create_snapshot", {
        projectId: currentProject.value.id,
        options,
      });

      snapshots = [snapshot, ...snapshots];
      createdName = snapshot.name;
      showCreateDialog = false;
    } catch (e) {
      createError = messageOf(e, "Failed to create snapshot");
    } finally {
      creating = false;
    }
  }

  function openRestoreDialog(snapshot: SnapshotMetadata) {
    if (confirming?.id === snapshot.id && confirming.kind === "restore") {
      void cancelConfirm();
      return;
    }
    restoreMode = "replace_current";
    newProjectName = `${currentProject.value?.name || "Project"} (Restored)`;
    rowError = null;
    createdName = null;
    confirming = { id: snapshot.id, kind: "restore" };
  }

  function openDeleteConfirm(snapshot: SnapshotMetadata) {
    if (confirming?.id === snapshot.id && confirming.kind === "delete") {
      void cancelConfirm();
      return;
    }
    rowError = null;
    createdName = null;
    confirming = { id: snapshot.id, kind: "delete" };
  }

  async function cancelConfirm() {
    if (restoringId || deletingId || !confirming) return;
    const { id, kind } = confirming;
    confirming = null;
    rowError = null;
    await tick();
    (kind === "restore" ? restoreTriggers[id] : deleteTriggers[id])?.focus();
  }

  async function restoreSnapshot() {
    if (!confirming || confirming.kind !== "restore" || restoringId) return;

    const snapshotId = confirming.id;
    const options = {
      mode: restoreMode,
      new_project_name: restoreMode === "create_new" ? newProjectName.trim() : undefined,
    };
    restoringId = snapshotId;
    rowError = null;

    try {
      await prepareRestore();
      await synopsisSaves.flush(currentProject.value?.id);
      await proseSaves.flush(currentProject.value?.id);
      if (proseSaves.draftsForRecovery(currentProject.value?.id).length) {
        throw new Error("Save or recover unsaved prose before restoring a snapshot.");
      }
      const project = await invoke<Project>("restore_snapshot", {
        snapshotId,
        options,
      });

      confirming = null;

      // Reload the project data to reflect restored state
      currentProject.setProject(project);

      // Load chapters
      const chapters = await invoke<Chapter[]>("get_chapters", {
        projectId: project.id,
      });
      currentProject.setChapters(chapters);

      // Load characters and locations
      const [characters, locations] = await Promise.all([
        invoke<Character[]>("get_characters", { projectId: project.id }),
        invoke<Location[]>("get_locations", { projectId: project.id }),
      ]);
      currentProject.setCharacters(characters);
      currentProject.setLocations(locations);

      // Reset current selections
      currentProject.setCurrentChapter(null);
      currentProject.setCurrentScene(null);
      currentProject.setBeats([]);

      // Close the panel after successful restore
      onClose();
    } catch (e) {
      rowError = messageOf(e, "Failed to restore snapshot");
    } finally {
      restoringId = null;
    }
  }

  async function deleteSnapshot(snapshot: SnapshotMetadata) {
    if (deletingId) return;
    deletingId = snapshot.id;
    rowError = null;

    try {
      await invoke("delete_snapshot", { snapshotId: snapshot.id });
      snapshots = snapshots.filter((s) => s.id !== snapshot.id);
      confirming = null;
    } catch (e) {
      rowError = messageOf(e, "Failed to delete snapshot");
    } finally {
      deletingId = null;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (restoringId) return;
    if (event.target === event.currentTarget) {
      if (confirming) {
        void cancelConfirm();
      } else if (showCreateDialog) {
        void closeCreateDialog();
      } else {
        onClose();
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // The scrim and the window both listen; handle each keypress once.
    if (event.defaultPrevented || restoringId) return;
    if (event.key === "Escape") {
      event.preventDefault();
      if (confirming) {
        void cancelConfirm();
      } else if (showCreateDialog) {
        void closeCreateDialog();
      } else {
        onClose();
      }
    }
  }

  // Format file size for display
  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Format date for display
  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function plural(count: number | undefined, noun: string): string {
    const n = count ?? 0;
    return `${n} ${noun}${n === 1 ? "" : "s"}`;
  }

  // Get trigger type display
  function getTriggerLabel(trigger: string): string {
    switch (trigger) {
      case "manual":
        return "Manual";
      case "export":
        return "Export";
      case "auto":
        return "Auto";
      default:
        return trigger;
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
  aria-labelledby="snapshots-panel-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell">
    <DialogHeader
      title="Snapshots"
      titleId="snapshots-panel-title"
      subtitle={currentProject.value?.name}
      {onClose}
      closeTestId="snapshots-close"
      disabled={restoringId !== null}
    />

    <div class="ka-dialog-body snapshots">
      <div class="snapshots-intro">
        <p class="ka-help">
          A snapshot is a restore point for the whole project. Create one before making big changes.
        </p>
        {#if !showCreateDialog}
          <button
            bind:this={createTrigger}
            data-testid="snapshot-create-button"
            type="button"
            onclick={openCreateDialog}
            disabled={creating || restoringId !== null}
            class="ka-button"
          >
            <Plus class="w-5 h-5" aria-hidden="true" />
            Create snapshot
          </button>
        {/if}
      </div>

      {#if showCreateDialog}
        <section class="ka-tagedit snapshot-create" aria-labelledby="snapshot-create-title">
          <h3 id="snapshot-create-title" class="ka-group-title">New snapshot</h3>
          <div class="ka-field od-field">
            <label for="snapshot-name">Name</label>
            <input
              id="snapshot-name"
              type="text"
              bind:value={newSnapshotName}
              use:focusOnMount
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  void createSnapshot();
                }
              }}
              placeholder="e.g. Before the second-act rewrite"
              disabled={creating}
            />
          </div>
          <div class="ka-field od-field">
            <label for="snapshot-description"
              >Description <span class="ka-optional">(optional)</span></label
            >
            <textarea
              id="snapshot-description"
              bind:value={newSnapshotDescription}
              placeholder="What changed, or why you're saving this point"
              rows="2"
              disabled={creating}
            ></textarea>
          </div>
          {#if createError}<p class="ka-error" role="alert">{createError}</p>{/if}
          <div class="snapshot-actions-end">
            <button
              type="button"
              onclick={closeCreateDialog}
              disabled={creating}
              class="ka-button ka-button--secondary"
            >
              Cancel
            </button>
            <button
              data-testid="snapshot-confirm-create"
              type="button"
              onclick={createSnapshot}
              disabled={!newSnapshotName.trim() || creating}
              aria-busy={creating || undefined}
              class="ka-button"
            >
              {#if creating}
                <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                Creating snapshot…
              {:else}
                Create snapshot
              {/if}
            </button>
          </div>
        </section>
      {/if}

      {#if createdName}
        <div class="ka-notice ka-notice--success od-row-top" role="status">
          <CheckCircle2 class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>Snapshot created</strong>
            <p>“{createdName}” is now the newest restore point.</p>
          </div>
        </div>
      {/if}

      {#if loading}
        <p class="ka-help snapshots-loading" role="status">
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Loading snapshots…
        </p>
      {:else if error}
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <CircleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>Could not load snapshots</strong>
            <p>{error}</p>
          </div>
        </div>
      {:else if snapshots.length === 0 && !showCreateDialog}
        <div class="ka-empty od-stack snapshots-empty">
          <History class="w-7 h-7" aria-hidden="true" />
          <h4>No snapshots yet</h4>
          <p>Your first snapshot will appear here, with its date and what it contains.</p>
        </div>
      {:else}
        <ul class="snapshot-list" aria-label="Snapshots">
          {#each snapshots as snapshot (snapshot.id)}
            {@const date = formatDate(snapshot.created_at)}
            {@const isRestoring = confirming?.id === snapshot.id && confirming.kind === "restore"}
            {@const isDeleting = confirming?.id === snapshot.id && confirming.kind === "delete"}
            <li class="snapshot-row">
              <div class="snapshot-main">
                <div class="snapshot-text">
                  <p class="ka-label snapshot-name" id={`snapshot-${snapshot.id}-name`}>
                    {snapshot.name}
                  </p>
                  <p class="snapshot-meta">
                    <time datetime={snapshot.created_at}>{date}</time> · {getTriggerLabel(
                      snapshot.trigger_type
                    )} · {formatFileSize(snapshot.file_size)}
                  </p>
                  <p class="snapshot-meta">
                    {plural(snapshot.chapter_count, "chapter")} · {plural(
                      snapshot.scene_count,
                      "scene"
                    )} · {plural(snapshot.beat_count, "beat")}
                  </p>
                  {#if snapshot.description}
                    <p class="snapshot-description">{snapshot.description}</p>
                  {/if}
                </div>
                <div class="snapshot-actions">
                  <button
                    bind:this={restoreTriggers[snapshot.id]}
                    type="button"
                    onclick={() => openRestoreDialog(snapshot)}
                    disabled={restoringId !== null || deletingId === snapshot.id}
                    class="ka-button ka-button--secondary"
                    aria-label="Restore snapshot"
                    aria-describedby={`snapshot-${snapshot.id}-name`}
                    aria-expanded={isRestoring}
                    aria-controls={isRestoring ? `snapshot-${snapshot.id}-confirm` : undefined}
                  >
                    Restore
                  </button>
                  <button
                    bind:this={deleteTriggers[snapshot.id]}
                    type="button"
                    onclick={() => openDeleteConfirm(snapshot)}
                    disabled={restoringId !== null || deletingId === snapshot.id}
                    class="ka-button ka-button--ghost ka-icon-button snapshot-delete"
                    aria-label={`Delete snapshot from ${date}`}
                    title={`Delete snapshot from ${date}`}
                    aria-expanded={isDeleting}
                    aria-controls={isDeleting ? `snapshot-${snapshot.id}-confirm` : undefined}
                  >
                    <Trash2 class="w-5 h-5" aria-hidden="true" />
                  </button>
                </div>
              </div>

              {#if isRestoring}
                <div
                  id={`snapshot-${snapshot.id}-confirm`}
                  class="ka-notice ka-notice--warning od-row-top snapshot-confirm"
                  role="alertdialog"
                  aria-labelledby={`snapshot-${snapshot.id}-confirm-title`}
                >
                  <TriangleAlert class="w-5 h-5" aria-hidden="true" />
                  <div class="od-field od-fill snapshot-confirm-body">
                    <strong id={`snapshot-${snapshot.id}-confirm-title`}
                      >Restore this snapshot?</strong
                    >
                    <fieldset class="snapshot-modes" disabled={restoringId !== null}>
                      <legend class="ka-sr">Restore mode</legend>
                      <label class="ka-check snapshot-mode">
                        <input
                          type="radio"
                          name="restore-mode"
                          value="replace_current"
                          bind:group={restoreMode}
                          use:focusOnMount
                        />
                        <span class="od-field">
                          <span>Replace current project</span>
                          <span class="ka-help"
                            >Overwrites this project. Create a snapshot first if you want to keep
                            the current state.</span
                          >
                        </span>
                      </label>
                      <label class="ka-check snapshot-mode">
                        <input
                          type="radio"
                          name="restore-mode"
                          value="create_new"
                          bind:group={restoreMode}
                        />
                        <span class="od-field">
                          <span>Create new project</span>
                          <span class="ka-help"
                            >Create a copy of the project from this snapshot.</span
                          >
                        </span>
                      </label>
                    </fieldset>
                    {#if restoreMode === "create_new"}
                      <div class="ka-field od-field">
                        <label for="snapshot-restore-project-name">New project name</label>
                        <input
                          id="snapshot-restore-project-name"
                          type="text"
                          bind:value={newProjectName}
                          disabled={restoringId !== null}
                          aria-describedby={!newProjectName.trim()
                            ? "snapshot-restore-project-name-help"
                            : undefined}
                        />
                        {#if !newProjectName.trim()}
                          <p id="snapshot-restore-project-name-help" class="ka-help">
                            Enter a name for the new project.
                          </p>
                        {/if}
                      </div>
                    {/if}
                    {#if rowError}<p class="ka-error" role="alert">{rowError}</p>{/if}
                    <div class="snapshot-actions-end">
                      <button
                        type="button"
                        onclick={cancelConfirm}
                        disabled={restoringId !== null}
                        class="ka-button ka-button--ghost"
                      >
                        Cancel
                      </button>
                      <button
                        type="button"
                        onclick={restoreSnapshot}
                        disabled={restoringId !== null ||
                          (restoreMode === "create_new" && !newProjectName.trim())}
                        aria-busy={restoringId !== null || undefined}
                        class="ka-button ka-button--danger"
                      >
                        {#if restoringId}
                          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                          Restoring…
                        {:else}
                          Restore
                        {/if}
                      </button>
                    </div>
                  </div>
                </div>
              {:else if isDeleting}
                <div
                  id={`snapshot-${snapshot.id}-confirm`}
                  class="ka-notice ka-notice--warning od-row-top snapshot-confirm"
                  role="alertdialog"
                  aria-labelledby={`snapshot-${snapshot.id}-confirm-title`}
                  aria-describedby={`snapshot-${snapshot.id}-confirm-body`}
                >
                  <TriangleAlert class="w-5 h-5" aria-hidden="true" />
                  <div class="od-field od-fill snapshot-confirm-body">
                    <strong id={`snapshot-${snapshot.id}-confirm-title`}
                      >Delete this snapshot?</strong
                    >
                    <p id={`snapshot-${snapshot.id}-confirm-body`}>
                      This restore point will be removed. Your project is not changed.
                    </p>
                    {#if rowError}<p class="ka-error" role="alert">{rowError}</p>{/if}
                    <div class="snapshot-actions-end">
                      <button
                        type="button"
                        onclick={cancelConfirm}
                        disabled={deletingId !== null}
                        class="ka-button ka-button--ghost"
                        use:focusOnMount
                      >
                        Cancel
                      </button>
                      <button
                        type="button"
                        onclick={() => deleteSnapshot(snapshot)}
                        disabled={deletingId !== null}
                        aria-busy={deletingId === snapshot.id || undefined}
                        class="ka-button ka-button--danger"
                      >
                        {#if deletingId === snapshot.id}
                          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                          Deleting…
                        {:else}
                          Delete
                        {/if}
                      </button>
                    </div>
                  </div>
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>

<style>
  .snapshots {
    display: grid;
    align-content: start;
    gap: var(--space-s);
  }
  .snapshots-intro {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--space-s);
  }
  .snapshots-intro .ka-help {
    flex: 1 1 16rem;
    margin: 0;
  }
  /* The form sits on the surface; only its inputs are sunken. */
  .snapshot-create {
    margin: 0;
    padding: var(--space-s);
    border: var(--border-hair);
    border-radius: var(--radius-m);
    background: var(--color-surface);
  }
  .snapshot-create .ka-group-title {
    margin: 0;
  }
  .snapshots-loading {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin: 0;
  }
  .snapshots-empty {
    padding-block: var(--space-m);
  }
  .snapshots-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .snapshots-empty p {
    margin: 0;
  }
  .snapshot-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: var(--border-hair);
  }
  .snapshot-row {
    display: grid;
    gap: var(--space-xs);
    padding-block: var(--space-xs);
    border-bottom: var(--border-hair);
  }
  .snapshot-main {
    display: flex;
    align-items: center;
    gap: var(--space-s);
    min-height: var(--control-target);
  }
  .snapshot-text {
    flex: 1 1 auto;
    min-width: 0;
  }
  .snapshot-text p {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .snapshot-name {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .snapshot-meta {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .snapshot-description {
    margin-top: var(--space-3xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    max-width: var(--measure);
  }
  .snapshot-actions {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    flex: none;
  }
  .snapshot-delete {
    color: var(--color-text-muted);
  }
  @media (hover: hover) {
    .snapshot-delete:not(:disabled):hover {
      color: var(--color-error);
    }
  }
  .snapshot-confirm-body {
    gap: var(--space-xs);
  }
  .snapshot-confirm-body p {
    margin: 0;
  }
  .snapshot-modes {
    display: grid;
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }
  .snapshot-mode {
    display: flex;
    align-items: flex-start;
    padding-block: var(--space-2xs);
    color: var(--color-text);
    cursor: pointer;
  }
  .snapshot-mode input {
    margin-top: 2px;
  }
  .snapshot-mode .ka-help {
    max-width: none;
  }
  .snapshot-actions-end {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
</style>
