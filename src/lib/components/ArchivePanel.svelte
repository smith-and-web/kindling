<script lang="ts">
  import { X, Archive, RotateCcw, Trash2, Loader2, Book, FileText } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { Chapter, Scene, ArchivedItems } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import Tooltip from "./Tooltip.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let loading = $state(true);
  let archivedChapters = $state<Chapter[]>([]);
  let archivedScenes = $state<Scene[]>([]);
  let error = $state<string | null>(null);
  let restoringId = $state<string | null>(null);
  let deletingId = $state<string | null>(null);

  $effect(() => {
    loadArchivedItems();
  });

  async function loadArchivedItems() {
    if (!currentProject.value) return;

    loading = true;
    error = null;

    try {
      const items = await invoke<ArchivedItems>("get_archived_items", {
        projectId: currentProject.value.id,
      });
      archivedChapters = items.chapters;
      archivedScenes = items.scenes;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load archived items";
    } finally {
      loading = false;
    }
  }

  async function restoreChapter(chapter: Chapter) {
    restoringId = chapter.id;
    try {
      const restored = await invoke<Chapter>("restore_chapter", {
        chapterId: chapter.id,
      });
      archivedChapters = archivedChapters.filter((c) => c.id !== chapter.id);
      currentProject.addChapter(restored);
    } catch (e) {
      console.error("Failed to restore chapter:", e);
    } finally {
      restoringId = null;
    }
  }

  async function restoreScene(scene: Scene) {
    restoringId = scene.id;
    try {
      const restored = await invoke<Scene>("restore_scene", {
        sceneId: scene.id,
      });
      archivedScenes = archivedScenes.filter((s) => s.id !== scene.id);
      // If the chapter is the current chapter, add the scene to the list
      if (currentProject.currentChapter?.id === restored.chapter_id) {
        currentProject.addScene(restored);
      }
    } catch (e) {
      console.error("Failed to restore scene:", e);
    } finally {
      restoringId = null;
    }
  }

  async function permanentDeleteChapter(chapter: Chapter) {
    if (!confirm(`Permanently delete "${chapter.title}"? This cannot be undone.`)) {
      return;
    }

    deletingId = chapter.id;
    try {
      await invoke("delete_chapter", { chapterId: chapter.id });
      archivedChapters = archivedChapters.filter((c) => c.id !== chapter.id);
    } catch (e) {
      console.error("Failed to delete chapter:", e);
    } finally {
      deletingId = null;
    }
  }

  async function permanentDeleteScene(scene: Scene) {
    if (!confirm(`Permanently delete "${scene.title}"? This cannot be undone.`)) {
      return;
    }

    deletingId = scene.id;
    try {
      await invoke("delete_scene", {
        sceneId: scene.id,
        chapterId: scene.chapter_id,
      });
      archivedScenes = archivedScenes.filter((s) => s.id !== scene.id);
    } catch (e) {
      console.error("Failed to delete scene:", e);
    } finally {
      deletingId = null;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }

  // Get parent chapter title for a scene
  function getParentChapterTitle(scene: Scene): string {
    const chapter = currentProject.chapters.find((c) => c.id === scene.chapter_id);
    return chapter?.title || "Unknown Chapter";
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
  aria-labelledby="archive-panel-title"
  tabindex="-1"
>
  <!-- Panel -->
  <div
    class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-press-border">
      <div class="flex items-center gap-3">
        <Archive class="w-5 h-5 text-press-accent-text" />
        <h2 id="archive-panel-title" class="text-press-h3 font-medium text-press-text">Archive</h2>
      </div>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <Loader2 class="w-8 h-8 animate-spin text-press-accent-text" />
        </div>
      {:else if error}
        <div class="text-center py-12">
          <p class="text-press-error">{error}</p>
        </div>
      {:else if archivedChapters.length === 0 && archivedScenes.length === 0}
        <div class="text-center py-12">
          <Archive class="w-12 h-12 mx-auto text-press-muted mb-4" />
          <p class="text-press-muted">No archived items</p>
          <p class="text-press-muted text-press-ui mt-1">
            Archived chapters and scenes will appear here
          </p>
        </div>
      {:else}
        <!-- Archived Chapters -->
        {#if archivedChapters.length > 0}
          <section class="mb-8">
            <h3 class="text-press-ui font-medium text-press-muted uppercase tracking-wide mb-4">
              Archived Chapters ({archivedChapters.length})
            </h3>
            <div class="space-y-2">
              {#each archivedChapters as chapter (chapter.id)}
                <div class="flex items-center justify-between bg-press-sunken rounded-lg px-4 py-3">
                  <div class="flex items-center gap-3">
                    <Book class="w-4 h-4 text-press-muted" />
                    <span class="text-press-text">{chapter.title}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => restoreChapter(chapter)}
                      disabled={restoringId === chapter.id || deletingId === chapter.id}
                      class="flex items-center gap-1 px-2 py-1 text-press-ui text-press-accent-text hover:text-press-accent-text transition-colors"
                      title="Restore"
                    >
                      {#if restoringId === chapter.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <RotateCcw class="w-4 h-4" />
                      {/if}
                      <span>Restore</span>
                    </button>
                    <button
                      type="button"
                      onclick={() => permanentDeleteChapter(chapter)}
                      disabled={restoringId === chapter.id || deletingId === chapter.id}
                      class="flex items-center gap-1 px-2 py-1 text-press-ui text-press-error hover:text-press-error transition-colors"
                      title="Delete permanently"
                    >
                      {#if deletingId === chapter.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <Trash2 class="w-4 h-4" />
                      {/if}
                      <span>Delete</span>
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <!-- Archived Scenes -->
        {#if archivedScenes.length > 0}
          <section>
            <h3 class="text-press-ui font-medium text-press-muted uppercase tracking-wide mb-4">
              Archived Scenes ({archivedScenes.length})
            </h3>
            <div class="space-y-2">
              {#each archivedScenes as scene (scene.id)}
                <div class="flex items-center justify-between bg-press-sunken rounded-lg px-4 py-3">
                  <div class="flex items-center gap-3 min-w-0">
                    <FileText class="w-4 h-4 text-press-muted flex-shrink-0" />
                    <div class="min-w-0">
                      <p class="text-press-text truncate">{scene.title}</p>
                      <p class="text-press-muted text-press-eyebrow truncate">
                        in {getParentChapterTitle(scene)}
                      </p>
                    </div>
                  </div>
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <button
                      type="button"
                      onclick={() => restoreScene(scene)}
                      disabled={restoringId === scene.id || deletingId === scene.id}
                      class="flex items-center gap-1 px-2 py-1 text-press-ui text-press-accent-text hover:text-press-accent-text transition-colors"
                      title="Restore"
                    >
                      {#if restoringId === scene.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <RotateCcw class="w-4 h-4" />
                      {/if}
                      <span>Restore</span>
                    </button>
                    <button
                      type="button"
                      onclick={() => permanentDeleteScene(scene)}
                      disabled={restoringId === scene.id || deletingId === scene.id}
                      class="flex items-center gap-1 px-2 py-1 text-press-ui text-press-error hover:text-press-error transition-colors"
                      title="Delete permanently"
                    >
                      {#if deletingId === scene.id}
                        <Loader2 class="w-4 h-4 animate-spin" />
                      {:else}
                        <Trash2 class="w-4 h-4" />
                      {/if}
                      <span>Delete</span>
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}
      {/if}
    </div>
  </div>
</div>
