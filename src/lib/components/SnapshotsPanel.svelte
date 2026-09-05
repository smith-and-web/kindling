<!--
  SnapshotsPanel.svelte - Snapshot management panel

  Allows users to view, create, restore, and delete project snapshots.
  Similar to ArchivePanel but for versioning/restore points.
-->
<script lang="ts">
  import {
    X,
    Clock,
    RotateCcw,
    Trash2,
    Loader2,
    Plus,
    Book,
    FileText,
    ListChecks,
    HardDrive,
    Calendar,
  } from "lucide-svelte";
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
  import Tooltip from "./Tooltip.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let loading = $state(true);
  let snapshots = $state<SnapshotMetadata[]>([]);
  let error = $state<string | null>(null);
  let restoringId = $state<string | null>(null);
  let deletingId = $state<string | null>(null);
  let creating = $state(false);

  // Create snapshot dialog state
  let showCreateDialog = $state(false);
  let newSnapshotName = $state("");
  let newSnapshotDescription = $state("");

  // Restore dialog state
  let showRestoreDialog = $state(false);
  let snapshotToRestore = $state<SnapshotMetadata | null>(null);
  let restoreMode = $state<"replace_current" | "create_new">("replace_current");
  let newProjectName = $state("");

  $effect(() => {
    loadSnapshots();
  });

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
      error = e instanceof Error ? e.message : "Failed to load snapshots";
    } finally {
      loading = false;
    }
  }

  function openCreateDialog() {
    newSnapshotName = `Snapshot ${new Date().toLocaleDateString()}`;
    newSnapshotDescription = "";
    showCreateDialog = true;
  }

  async function createSnapshot() {
    if (!currentProject.value || !newSnapshotName.trim()) return;

    creating = true;
    error = null;

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
      showCreateDialog = false;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create snapshot";
    } finally {
      creating = false;
    }
  }

  function openRestoreDialog(snapshot: SnapshotMetadata) {
    snapshotToRestore = snapshot;
    restoreMode = "replace_current";
    newProjectName = `${currentProject.value?.name || "Project"} (Restored)`;
    showRestoreDialog = true;
  }

  async function restoreSnapshot() {
    if (!snapshotToRestore) return;

    restoringId = snapshotToRestore.id;
    error = null;

    try {
      const project = await invoke<Project>("restore_snapshot", {
        snapshotId: snapshotToRestore.id,
        options: {
          mode: restoreMode,
          new_project_name: restoreMode === "create_new" ? newProjectName.trim() : undefined,
        },
      });

      showRestoreDialog = false;
      snapshotToRestore = null;

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
      error = e instanceof Error ? e.message : "Failed to restore snapshot";
    } finally {
      restoringId = null;
    }
  }

  async function deleteSnapshot(snapshot: SnapshotMetadata) {
    if (!confirm(`Delete snapshot "${snapshot.name}"? This cannot be undone.`)) {
      return;
    }

    deletingId = snapshot.id;

    try {
      await invoke("delete_snapshot", { snapshotId: snapshot.id });
      snapshots = snapshots.filter((s) => s.id !== snapshot.id);
    } catch (e) {
      console.error("Failed to delete snapshot:", e);
    } finally {
      deletingId = null;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      if (showCreateDialog || showRestoreDialog) {
        showCreateDialog = false;
        showRestoreDialog = false;
      } else {
        onClose();
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (showCreateDialog) {
        showCreateDialog = false;
      } else if (showRestoreDialog) {
        showRestoreDialog = false;
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

<!-- Backdrop -->
<div
  class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="snapshots-panel-title"
  tabindex="-1"
>
  <!-- Panel - wider for more breathing room -->
  <div
    class="app-dialog-surface bg-press-surface rounded-xl shadow-press-overlay w-full max-w-4xl mx-4 max-h-[85vh] flex flex-col overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-5 border-b border-press-border">
      <div class="flex items-center gap-3">
        <Clock class="w-6 h-6 text-press-accent-text" />
        <h2 id="snapshots-panel-title" class="text-press-h3 font-semibold text-press-text">
          Snapshots
        </h2>
      </div>
      <div class="flex items-center gap-3">
        <button
          data-testid="snapshot-create-button"
          type="button"
          onclick={openCreateDialog}
          disabled={creating}
          class="flex items-center gap-2 px-4 py-2 text-press-ui font-medium bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors"
        >
          <Plus class="w-4 h-4" />
          <span>Create Snapshot</span>
        </button>
        <Tooltip text="Close" position="left">
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
            aria-label="Close"
            data-testid="snapshots-close"
          >
            <X class="w-5 h-5" />
          </button>
        </Tooltip>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      {#if loading}
        <div class="flex items-center justify-center py-16">
          <Loader2 class="w-8 h-8 animate-spin text-press-accent-text" />
        </div>
      {:else if error}
        <div class="text-center py-16">
          <p class="text-press-error">{error}</p>
        </div>
      {:else if snapshots.length === 0}
        <div class="text-center py-16">
          <Clock class="w-16 h-16 mx-auto text-press-muted mb-4" />
          <p class="text-press-text text-press-body-lg font-medium">No snapshots yet</p>
          <p class="text-press-muted text-press-ui mt-2 max-w-sm mx-auto">
            Snapshots let you save restore points of your project. Create one before making big
            changes.
          </p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each snapshots as snapshot (snapshot.id)}
            <div class="bg-press-sunken rounded-lg p-4 hover:bg-press-sunken transition-colors">
              <!-- Top row: Name + Badge + Actions -->
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <h3 class="text-press-text font-medium text-press-base">{snapshot.name}</h3>
                    <span
                      class="px-2 py-0.5 text-press-eyebrow font-medium rounded-full bg-press-surface text-press-muted"
                    >
                      {getTriggerLabel(snapshot.trigger_type)}
                    </span>
                  </div>
                  {#if snapshot.description}
                    <p class="text-press-muted text-press-ui mt-1 line-clamp-2">
                      {snapshot.description}
                    </p>
                  {/if}
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-1 flex-shrink-0">
                  <Tooltip text="Restore snapshot" position="top">
                    <button
                      type="button"
                      onclick={() => openRestoreDialog(snapshot)}
                      disabled={restoringId === snapshot.id || deletingId === snapshot.id}
                      class="flex items-center gap-1.5 px-3 py-1.5 text-press-ui text-press-accent-text hover:bg-press-accent-wash rounded-lg transition-colors"
                      aria-label="Restore snapshot"
                    >
                      {#if restoringId === snapshot.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <RotateCcw class="w-4 h-4" />
                      {/if}
                      <span>Restore</span>
                    </button>
                  </Tooltip>
                  <Tooltip text="Delete snapshot" position="top">
                    <button
                      type="button"
                      onclick={() => deleteSnapshot(snapshot)}
                      disabled={restoringId === snapshot.id || deletingId === snapshot.id}
                      class="p-1.5 text-press-muted hover:text-press-error hover:bg-press-error-wash rounded-lg transition-colors"
                      aria-label="Delete snapshot"
                    >
                      {#if deletingId === snapshot.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <Trash2 class="w-4 h-4" />
                      {/if}
                    </button>
                  </Tooltip>
                </div>
              </div>

              <!-- Bottom row: Metadata with icons -->
              <div class="flex items-center gap-6 mt-3 text-press-ui text-press-muted">
                <div class="flex items-center gap-1.5">
                  <Calendar class="w-3.5 h-3.5" />
                  <span>{formatDate(snapshot.created_at)}</span>
                </div>
                <div class="flex items-center gap-1.5">
                  <HardDrive class="w-3.5 h-3.5" />
                  <span>{formatFileSize(snapshot.file_size)}</span>
                </div>
                <div class="flex items-center gap-4 ml-auto">
                  <Tooltip text="Chapters" position="top">
                    <div class="flex items-center gap-1">
                      <Book class="w-3.5 h-3.5" />
                      <span>{snapshot.chapter_count}</span>
                    </div>
                  </Tooltip>
                  <Tooltip text="Scenes" position="top">
                    <div class="flex items-center gap-1">
                      <FileText class="w-3.5 h-3.5" />
                      <span>{snapshot.scene_count}</span>
                    </div>
                  </Tooltip>
                  <Tooltip text="Beats" position="top">
                    <div class="flex items-center gap-1">
                      <ListChecks class="w-3.5 h-3.5" />
                      <span>{snapshot.beat_count}</span>
                    </div>
                  </Tooltip>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Create Snapshot Dialog -->
  {#if showCreateDialog}
    <div
      class="fixed inset-0 z-press-popover flex items-center justify-center bg-press-overlay"
      onclick={(e) => e.target === e.currentTarget && (showCreateDialog = false)}
      onkeydown={handleKeydown}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div
        class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
      >
        <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
          <h3 class="text-press-body-lg font-medium text-press-text">Create Snapshot</h3>
          <button
            type="button"
            onclick={() => (showCreateDialog = false)}
            class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 space-y-4">
          <div>
            <label
              for="snapshot-name"
              class="block text-press-ui font-medium text-press-muted mb-2"
            >
              Name
            </label>
            <input
              id="snapshot-name"
              type="text"
              bind:value={newSnapshotName}
              placeholder="Enter snapshot name..."
              class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
            />
          </div>
          <div>
            <label
              for="snapshot-description"
              class="block text-press-ui font-medium text-press-muted mb-2"
            >
              Description (optional)
            </label>
            <textarea
              id="snapshot-description"
              bind:value={newSnapshotDescription}
              placeholder="Enter a description..."
              rows="2"
              class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent resize-none"
            ></textarea>
          </div>
        </div>
        <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
          <button
            type="button"
            onclick={() => (showCreateDialog = false)}
            class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors"
          >
            Cancel
          </button>
          <button
            data-testid="snapshot-confirm-create"
            type="button"
            onclick={createSnapshot}
            disabled={!newSnapshotName.trim() || creating}
            class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors flex items-center gap-2"
          >
            {#if creating}
              <Loader2 class="w-4 h-4 animate-spin" />
              Creating...
            {:else}
              Create Snapshot
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Restore Snapshot Dialog -->
  {#if showRestoreDialog && snapshotToRestore}
    <div
      class="fixed inset-0 z-press-popover flex items-center justify-center bg-press-overlay"
      onclick={(e) => e.target === e.currentTarget && (showRestoreDialog = false)}
      onkeydown={handleKeydown}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div
        class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
      >
        <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
          <h3 class="text-press-body-lg font-medium text-press-text">Restore Snapshot</h3>
          <button
            type="button"
            onclick={() => (showRestoreDialog = false)}
            class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 space-y-4">
          <p class="text-press-muted">
            Restore snapshot <strong class="text-press-text">"{snapshotToRestore.name}"</strong>?
          </p>

          <fieldset>
            <legend class="block text-press-ui font-medium text-press-muted mb-2"
              >Restore Mode</legend
            >
            <div class="space-y-2">
              <label class="flex items-start gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="restore-mode"
                  value="replace_current"
                  bind:group={restoreMode}
                  class="mt-1 w-4 h-4 text-press-accent-text bg-press-sunken border-press-border focus:ring-press-focus"
                />
                <div>
                  <span class="text-press-text">Replace current project</span>
                  <p class="text-press-eyebrow text-press-muted">
                    Overwrite current project data with the snapshot
                  </p>
                </div>
              </label>
              <label class="flex items-start gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="restore-mode"
                  value="create_new"
                  bind:group={restoreMode}
                  class="mt-1 w-4 h-4 text-press-accent-text bg-press-sunken border-press-border focus:ring-press-focus"
                />
                <div>
                  <span class="text-press-text">Create new project</span>
                  <p class="text-press-eyebrow text-press-muted">
                    Create a copy of the project from this snapshot
                  </p>
                </div>
              </label>
            </div>
          </fieldset>

          {#if restoreMode === "create_new"}
            <div>
              <label
                for="new-project-name"
                class="block text-press-ui font-medium text-press-muted mb-2"
              >
                New Project Name
              </label>
              <input
                id="new-project-name"
                type="text"
                bind:value={newProjectName}
                placeholder="Enter project name..."
                class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
              />
            </div>
          {/if}

          {#if restoreMode === "replace_current"}
            <p class="text-press-ui text-press-warning">
              Warning: This will replace all current project data. Consider creating a snapshot
              first if you want to preserve the current state.
            </p>
          {/if}
        </div>
        <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
          <button
            type="button"
            onclick={() => (showRestoreDialog = false)}
            class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={restoreSnapshot}
            disabled={restoringId !== null ||
              (restoreMode === "create_new" && !newProjectName.trim())}
            class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors flex items-center gap-2"
          >
            {#if restoringId}
              <Loader2 class="w-4 h-4 animate-spin" />
              Restoring...
            {:else}
              Restore
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
