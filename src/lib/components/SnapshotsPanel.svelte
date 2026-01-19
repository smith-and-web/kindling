<!--
  SnapshotsPanel.svelte - Snapshot management panel

  Allows users to view, create, restore, and delete project snapshots.
  Similar to ArchivePanel but for versioning/restore points.
-->
<script lang="ts">
  /* eslint-disable no-undef */
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
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="snapshots-panel-title"
>
  <!-- Panel - wider for more breathing room -->
  <div
    class="bg-bg-panel rounded-xl shadow-xl w-full max-w-4xl mx-4 max-h-[85vh] flex flex-col overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-5 border-b border-bg-card">
      <div class="flex items-center gap-3">
        <Clock class="w-6 h-6 text-accent" />
        <h2 id="snapshots-panel-title" class="text-xl font-semibold text-text-primary">
          Snapshots
        </h2>
      </div>
      <div class="flex items-center gap-3">
        <button
          type="button"
          onclick={openCreateDialog}
          disabled={creating}
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-accent text-white rounded-lg hover:bg-accent/90 transition-colors disabled:opacity-50"
        >
          <Plus class="w-4 h-4" />
          <span>Create Snapshot</span>
        </button>
        <Tooltip text="Close" position="left">
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-text-secondary hover:text-text-primary hover:bg-bg-card rounded-lg transition-colors"
            aria-label="Close"
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
          <Loader2 class="w-8 h-8 animate-spin text-accent" />
        </div>
      {:else if error}
        <div class="text-center py-16">
          <p class="text-red-400">{error}</p>
        </div>
      {:else if snapshots.length === 0}
        <div class="text-center py-16">
          <Clock class="w-16 h-16 mx-auto text-text-secondary/30 mb-4" />
          <p class="text-text-primary text-lg font-medium">No snapshots yet</p>
          <p class="text-text-secondary text-sm mt-2 max-w-sm mx-auto">
            Snapshots let you save restore points of your project. Create one before making big
            changes.
          </p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each snapshots as snapshot (snapshot.id)}
            <div class="bg-bg-card rounded-lg p-4 hover:bg-bg-card/80 transition-colors">
              <!-- Top row: Name + Badge + Actions -->
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <h3 class="text-text-primary font-medium text-base">{snapshot.name}</h3>
                    <span
                      class="px-2 py-0.5 text-[11px] font-medium rounded-full bg-bg-panel text-text-secondary"
                    >
                      {getTriggerLabel(snapshot.trigger_type)}
                    </span>
                  </div>
                  {#if snapshot.description}
                    <p class="text-text-secondary/70 text-sm mt-1 line-clamp-2">
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
                      class="flex items-center gap-1.5 px-3 py-1.5 text-sm text-accent hover:bg-accent/10 rounded-lg transition-colors disabled:opacity-50"
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
                      class="p-1.5 text-text-secondary hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors disabled:opacity-50"
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
              <div class="flex items-center gap-6 mt-3 text-sm text-text-secondary">
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
      class="fixed inset-0 z-60 flex items-center justify-center bg-black/50"
      onclick={(e) => e.target === e.currentTarget && (showCreateDialog = false)}
      role="dialog"
      aria-modal="true"
    >
      <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
          <h3 class="text-lg font-medium text-text-primary">Create Snapshot</h3>
          <button
            type="button"
            onclick={() => (showCreateDialog = false)}
            class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 space-y-4">
          <div>
            <label for="snapshot-name" class="block text-sm font-medium text-text-secondary mb-2">
              Name
            </label>
            <input
              id="snapshot-name"
              type="text"
              bind:value={newSnapshotName}
              placeholder="Enter snapshot name..."
              class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent"
            />
          </div>
          <div>
            <label
              for="snapshot-description"
              class="block text-sm font-medium text-text-secondary mb-2"
            >
              Description (optional)
            </label>
            <textarea
              id="snapshot-description"
              bind:value={newSnapshotDescription}
              placeholder="Enter a description..."
              rows="2"
              class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent resize-none"
            ></textarea>
          </div>
        </div>
        <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
          <button
            type="button"
            onclick={() => (showCreateDialog = false)}
            class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={createSnapshot}
            disabled={!newSnapshotName.trim() || creating}
            class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
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
      class="fixed inset-0 z-60 flex items-center justify-center bg-black/50"
      onclick={(e) => e.target === e.currentTarget && (showRestoreDialog = false)}
      role="dialog"
      aria-modal="true"
    >
      <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
          <h3 class="text-lg font-medium text-text-primary">Restore Snapshot</h3>
          <button
            type="button"
            onclick={() => (showRestoreDialog = false)}
            class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
        <div class="p-4 space-y-4">
          <p class="text-text-secondary">
            Restore snapshot <strong class="text-text-primary">"{snapshotToRestore.name}"</strong>?
          </p>

          <fieldset>
            <legend class="block text-sm font-medium text-text-secondary mb-2">Restore Mode</legend>
            <div class="space-y-2">
              <label class="flex items-start gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="restore-mode"
                  value="replace_current"
                  bind:group={restoreMode}
                  class="mt-1 w-4 h-4 text-accent bg-bg-card border-bg-card focus:ring-accent"
                />
                <div>
                  <span class="text-text-primary">Replace current project</span>
                  <p class="text-xs text-text-secondary/70">
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
                  class="mt-1 w-4 h-4 text-accent bg-bg-card border-bg-card focus:ring-accent"
                />
                <div>
                  <span class="text-text-primary">Create new project</span>
                  <p class="text-xs text-text-secondary/70">
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
                class="block text-sm font-medium text-text-secondary mb-2"
              >
                New Project Name
              </label>
              <input
                id="new-project-name"
                type="text"
                bind:value={newProjectName}
                placeholder="Enter project name..."
                class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent"
              />
            </div>
          {/if}

          {#if restoreMode === "replace_current"}
            <p class="text-sm text-amber-400">
              Warning: This will replace all current project data. Consider creating a snapshot
              first if you want to preserve the current state.
            </p>
          {/if}
        </div>
        <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
          <button
            type="button"
            onclick={() => (showRestoreDialog = false)}
            class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={restoreSnapshot}
            disabled={restoringId !== null ||
              (restoreMode === "create_new" && !newProjectName.trim())}
            class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
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
