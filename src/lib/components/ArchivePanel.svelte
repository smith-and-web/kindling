<script lang="ts">
  import { Archive, CircleAlert, Loader2, TriangleAlert } from "lucide-svelte";
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { Chapter, Scene, ArchivedItems } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";

  let { onClose }: { onClose: () => void } = $props();

  let loading = $state(true);
  let archivedChapters = $state<Chapter[]>([]);
  let archivedScenes = $state<Scene[]>([]);
  let error = $state<string | null>(null);
  let restoringId = $state<string | null>(null);
  let deletingId = $state<string | null>(null);
  // Inline delete confirmation and per-row failure message.
  let confirmingId = $state<string | null>(null);
  let rowError = $state<{ id: string; message: string } | null>(null);
  const deleteTriggers: Record<string, HTMLElement | null> = {};

  $effect(() => {
    loadArchivedItems();
  });

  function messageOf(e: unknown, fallback: string): string {
    if (e instanceof Error) return e.message;
    if (typeof e === "string" && e) return e;
    return fallback;
  }

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

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
      error = messageOf(e, "Failed to load archived items");
    } finally {
      loading = false;
    }
  }

  async function restoreChapter(chapter: Chapter) {
    restoringId = chapter.id;
    rowError = null;
    try {
      const restored = await invoke<Chapter>("restore_chapter", {
        chapterId: chapter.id,
      });
      archivedChapters = archivedChapters.filter((c) => c.id !== chapter.id);
      currentProject.addChapter(restored);
    } catch (e) {
      rowError = { id: chapter.id, message: messageOf(e, "Could not restore this chapter.") };
    } finally {
      restoringId = null;
    }
  }

  async function restoreScene(scene: Scene) {
    restoringId = scene.id;
    rowError = null;
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
      rowError = { id: scene.id, message: messageOf(e, "Could not restore this scene.") };
    } finally {
      restoringId = null;
    }
  }

  function askDelete(id: string) {
    if (confirmingId === id) {
      void cancelDelete();
      return;
    }
    rowError = null;
    confirmingId = id;
  }

  async function cancelDelete() {
    if (deletingId || !confirmingId) return;
    const id = confirmingId;
    confirmingId = null;
    rowError = null;
    await tick();
    deleteTriggers[id]?.focus();
  }

  async function permanentDeleteChapter(chapter: Chapter) {
    deletingId = chapter.id;
    rowError = null;
    try {
      await invoke("delete_chapter", { chapterId: chapter.id });
      archivedChapters = archivedChapters.filter((c) => c.id !== chapter.id);
      confirmingId = null;
    } catch (e) {
      rowError = { id: chapter.id, message: messageOf(e, "Could not delete this chapter.") };
    } finally {
      deletingId = null;
    }
  }

  async function permanentDeleteScene(scene: Scene) {
    deletingId = scene.id;
    rowError = null;
    try {
      await invoke("delete_scene", {
        sceneId: scene.id,
        chapterId: scene.chapter_id,
      });
      archivedScenes = archivedScenes.filter((s) => s.id !== scene.id);
      confirmingId = null;
    } catch (e) {
      rowError = { id: scene.id, message: messageOf(e, "Could not delete this scene.") };
    } finally {
      deletingId = null;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      if (confirmingId) void cancelDelete();
      else onClose();
    }
  }

  // Escape backs out one layer: the open delete confirmation first, then the panel.
  function handleEscape() {
    if (confirmingId) void cancelDelete();
    else onClose();
  }

  // Get parent chapter title for a scene
  function getParentChapterTitle(scene: Scene): string {
    const chapter = currentProject.chapters.find((c) => c.id === scene.chapter_id);
    return chapter?.title || "Unknown Chapter";
  }
</script>

{#snippet itemRow(
  id: string,
  title: string,
  origin: string,
  kind: "chapter" | "scene",
  onRestore: () => void,
  onDelete: () => void
)}
  {@const busy = restoringId === id || deletingId === id}
  <li class="archive-row">
    <div class="archive-main">
      <div class="archive-text">
        <p class="ka-label archive-name" id={`archive-${id}-name`}>{title}</p>
        <p class="archive-meta">{origin}</p>
      </div>
      <div class="archive-actions">
        <button
          type="button"
          onclick={onRestore}
          disabled={busy}
          aria-busy={restoringId === id || undefined}
          aria-describedby={`archive-${id}-name`}
          class="ka-button ka-button--secondary"
          data-testid="archive-restore"
        >
          {#if restoringId === id}
            <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
            Restoring…
          {:else}
            Restore
          {/if}
        </button>
        <button
          bind:this={deleteTriggers[id]}
          type="button"
          onclick={() => askDelete(id)}
          disabled={busy}
          aria-describedby={`archive-${id}-name`}
          aria-expanded={confirmingId === id}
          aria-controls={confirmingId === id ? `archive-${id}-confirm` : undefined}
          class="ka-button ka-button--danger"
        >
          Delete
        </button>
      </div>
    </div>
    {#if confirmingId === id}
      <div
        id={`archive-${id}-confirm`}
        class="ka-notice ka-notice--warning od-row-top archive-confirm"
        role="alertdialog"
        aria-labelledby={`archive-${id}-confirm-title`}
        aria-describedby={`archive-${id}-confirm-body`}
      >
        <TriangleAlert class="w-5 h-5" aria-hidden="true" />
        <div class="od-field od-fill archive-confirm-body">
          <strong id={`archive-${id}-confirm-title`}>Permanently delete “{title}”?</strong>
          <p id={`archive-${id}-confirm-body`}>
            This permanently deletes the {kind}. It cannot be undone.
          </p>
          {#if rowError?.id === id}<p class="ka-error" role="alert">{rowError.message}</p>{/if}
          <div class="archive-actions-end">
            <button
              type="button"
              onclick={cancelDelete}
              disabled={deletingId !== null}
              class="ka-button ka-button--ghost"
              use:focusOnMount
            >
              Cancel
            </button>
            <button
              type="button"
              onclick={onDelete}
              disabled={deletingId !== null}
              aria-busy={deletingId === id || undefined}
              class="ka-button ka-button--danger"
            >
              {#if deletingId === id}
                <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                Deleting…
              {:else}
                Delete permanently
              {/if}
            </button>
          </div>
        </div>
      </div>
    {:else if rowError?.id === id}
      <p class="ka-error" role="alert">{rowError.message}</p>
    {/if}
  </li>
{/snippet}

<!-- Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: handleEscape }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="archive-panel-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell">
    <DialogHeader
      title="Archive"
      titleId="archive-panel-title"
      subtitle={currentProject.value?.name}
      {onClose}
      closeTestId="archive-close"
    />

    <div class="ka-dialog-body archive">
      {#if loading}
        <p class="ka-help archive-loading" role="status">
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Loading archived items…
        </p>
      {:else if error}
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <CircleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong>Could not load the archive</strong>
            <p>{error}</p>
          </div>
        </div>
      {:else if archivedChapters.length === 0 && archivedScenes.length === 0}
        <div class="ka-empty od-stack archive-empty">
          <Archive class="w-7 h-7" aria-hidden="true" />
          <h4>No archived items</h4>
          <p>Archived chapters and scenes will appear here.</p>
        </div>
      {:else}
        <p class="ka-help archive-intro">
          Restore puts an item back where it was. Delete removes it for good.
        </p>
        {#if archivedChapters.length > 0}
          <section class="ka-group archive-group" aria-labelledby="archive-chapters-title">
            <h3 id="archive-chapters-title" class="ka-group-title">
              Archived chapters ({archivedChapters.length})
            </h3>
            <ul class="archive-list">
              {#each archivedChapters as chapter (chapter.id)}
                {@render itemRow(
                  chapter.id,
                  chapter.title,
                  "Chapter",
                  "chapter",
                  () => restoreChapter(chapter),
                  () => permanentDeleteChapter(chapter)
                )}
              {/each}
            </ul>
          </section>
        {/if}

        {#if archivedScenes.length > 0}
          <section class="ka-group archive-group" aria-labelledby="archive-scenes-title">
            <h3 id="archive-scenes-title" class="ka-group-title">
              Archived scenes ({archivedScenes.length})
            </h3>
            <ul class="archive-list">
              {#each archivedScenes as scene (scene.id)}
                {@render itemRow(
                  scene.id,
                  scene.title,
                  `Scene in ${getParentChapterTitle(scene)}`,
                  "scene",
                  () => restoreScene(scene),
                  () => permanentDeleteScene(scene)
                )}
              {/each}
            </ul>
          </section>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .archive {
    display: grid;
    align-content: start;
    gap: var(--space-s);
  }
  .archive-intro,
  .archive-loading {
    margin: 0;
  }
  .archive-loading {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .archive-empty {
    padding-block: var(--space-m);
  }
  .archive-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .archive-empty p {
    margin: 0;
  }
  .archive-group .ka-group-title {
    margin: 0 0 var(--space-2xs);
  }
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: var(--border-hair);
  }
  .archive-row {
    display: grid;
    gap: var(--space-xs);
    padding-block: var(--space-xs);
    border-bottom: var(--border-hair);
  }
  .archive-row > .ka-error {
    margin: 0;
  }
  .archive-main {
    display: flex;
    align-items: center;
    gap: var(--space-s);
    min-height: var(--control-target);
  }
  .archive-text {
    flex: 1 1 auto;
    min-width: 0;
  }
  .archive-text p {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .archive-name {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .archive-meta {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .archive-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex: none;
  }
  .archive-confirm-body {
    gap: var(--space-xs);
  }
  .archive-confirm-body p {
    margin: 0;
  }
  .archive-actions-end {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
</style>
