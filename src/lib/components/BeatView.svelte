<script lang="ts">
  import {
    ChevronRight,
    ChevronDown,
    Loader2,
    Plus,
    Lock,
    GripVertical,
    MoreVertical,
    Trash2,
    Pencil,
    ArrowUp,
    ArrowDown,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { proseSaves, type ProseSave } from "../utils/proseSaves";
  import { tick, untrack } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import type { Beat } from "../types";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import NovelEditor from "./NovelEditor.svelte";

  let {
    beats,
    isLocked = false,
  }: {
    beats: Beat[];
    isLocked?: boolean;
  } = $props();

  let beatRefs = new SvelteMap<string, HTMLElement>();

  function registerBeatRef(node: HTMLElement, beatId: string) {
    beatRefs.set(beatId, node);
    return {
      destroy() {
        beatRefs.delete(beatId);
      },
    };
  }

  let addingBeat = $state(false);
  let newBeatContent = $state("");
  let creatingBeat = $state(false);
  let localSaveStatus = $state<"idle" | "saving" | "error">("idle");
  let novelEditorRef: { getSplitBeforeParagraph: () => number | null } | null = $state(null);

  let draggedBeatId: string | null = $state(null);
  let dragOverBeatId: string | null = $state(null);
  let isDraggingBeat = $state(false);
  let draggedBeatElement: HTMLElement | null = null;
  let currentDragOverBeatElement: HTMLElement | null = null;

  let beatContextMenu: { beat: Beat; x: number; y: number } | null = $state(null);
  let deleteBeatDialog: Beat | null = $state(null);
  let deletingBeat = $state(false);
  let changingBeats = $state(false);
  let editingBeatId: string | null = $state(null);
  let editingBeatContent = $state("");
  // Screen-reader announcement for keyboard moves (polite live region).
  let moveAnnouncement = $state("");

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let pendingSaveBeatId: string | null = null;
  let pendingProseUpdates = new SvelteMap<string, string>();
  let draftProse = new SvelteMap<string, ProseSave>();

  function syncPendingProse(beatId: string) {
    const pendingProse = pendingProseUpdates.get(beatId);
    if (pendingProse !== undefined) {
      currentProject.updateBeatProse(beatId, pendingProse);
      pendingProseUpdates.delete(beatId);
    }
  }

  function flushPendingSave(beatId?: string) {
    const targetBeatId = beatId ?? pendingSaveBeatId;
    if (!targetBeatId) return saveQueue;
    if (saveTimeout && pendingSaveBeatId === targetBeatId) {
      clearTimeout(saveTimeout);
      saveTimeout = null;
      pendingSaveBeatId = null;
    }
    const draft = draftProse.get(targetBeatId);
    if (draft !== undefined) {
      return saveBeatProse(draft);
    }
    return saveQueue;
  }

  export function flushOnSceneChange() {
    flushPendingSave(ui.expandedBeatId ?? undefined);
    if (ui.expandedBeatId) {
      syncPendingProse(ui.expandedBeatId);
    }
    pendingProseUpdates.clear();
    draftProse.clear();
    ui.setExpandedBeat(null);
  }

  export function discardFailedDrafts(drafts: ProseSave[]) {
    let discardedLocalDraft = false;
    for (const draft of drafts) {
      const local = draftProse.get(draft.id);
      if (
        draft.kind !== "beat" ||
        local?.prose !== draft.prose ||
        local.projectId !== draft.projectId
      )
        continue;
      discardedLocalDraft = true;
      draftProse.delete(draft.id);
      pendingProseUpdates.delete(draft.id);
      if (pendingSaveBeatId === draft.id) {
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = null;
        pendingSaveBeatId = null;
      }
      // Remount only the editor whose draft was discarded, preserving unrelated editors.
      if (currentProject.value?.id === draft.projectId && ui.expandedBeatId === draft.id) {
        ui.setExpandedBeat(null);
      }
    }
    if (discardedLocalDraft) localSaveStatus = "idle";
  }

  export function handleEscape() {
    if (editingBeatId) {
      editingBeatId = null;
      editingBeatContent = "";
      return true;
    }
    if (addingBeat) {
      addingBeat = false;
      newBeatContent = "";
      return true;
    }
    return false;
  }

  let saveQueue: Promise<void> = Promise.resolve();

  function saveBeatProse(draft: ProseSave) {
    const writing = proseSaves.save(draft);
    saveQueue = persistBeatProse(draft, writing);
    return saveQueue;
  }

  export async function flushForSearch() {
    const projectId = currentProject.value?.id ?? "";
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = null;
    pendingSaveBeatId = null;
    await saveQueue;
    for (const draft of [...draftProse.values()]) {
      await saveBeatProse(draft);
    }
    await proseSaves.flush(projectId, (save) => {
      if (currentProject.value?.id !== save.projectId) return;
      if (save.kind === "beat") {
        const latestDraft = draftProse.get(save.id);
        if (!latestDraft || latestDraft.prose === save.prose) {
          currentProject.updateBeatProse(save.id, save.prose);
          draftProse.delete(save.id);
          pendingProseUpdates.delete(save.id);
        }
      } else currentProject.updateScene(save.id, { prose: save.prose });
    });
  }

  async function persistBeatProse(draft: ProseSave, writing: Promise<void>) {
    const { id: beatId, prose, projectId } = draft;
    if (currentProject.value?.id === projectId) localSaveStatus = "saving";
    try {
      await writing;
      if (currentProject.value?.id !== projectId) {
        if (draftProse.get(beatId) === draft) draftProse.delete(beatId);
        pendingProseUpdates.delete(beatId);
        return;
      }
      if (!beats.some((beat) => beat.id === beatId)) {
        draftProse.delete(beatId);
        localSaveStatus = "idle";
        return;
      }
      const latestDraft = draftProse.get(beatId)?.prose;
      if (latestDraft === undefined || latestDraft === prose) {
        currentProject.updateBeatProse(beatId, prose);
      }
      pendingProseUpdates.delete(beatId);
      if (draftProse.get(beatId)?.prose === prose) draftProse.delete(beatId);
      setTimeout(() => {
        localSaveStatus = "idle";
      }, 1000);
    } catch (e) {
      console.error("Failed to save beat prose:", e);
      if (currentProject.value?.id === projectId) localSaveStatus = "error";
    }
  }

  function handleProseInput(beatId: string, value: string) {
    // Capture ownership for this edit, not for the lifetime of the component.
    // A debounce can finish after an in-place project switch.
    draftProse.set(beatId, {
      projectId: currentProject.value?.id ?? "",
      kind: "beat",
      id: beatId,
      prose: value,
    });
    if (saveTimeout) clearTimeout(saveTimeout);
    pendingSaveBeatId = beatId;
    saveTimeout = setTimeout(() => {
      saveTimeout = null;
      pendingSaveBeatId = null;
      const draft = draftProse.get(beatId);
      if (draft !== undefined) {
        saveBeatProse(draft);
      }
    }, 500);
  }

  // beat.prose only advances on a successful save. Mount and preview from the newest unsaved
  // text instead (this editor's draft, then a failed or in-flight save), as Page View does, so
  // reopening a beat after a failed save never shows stale prose that the next keystroke saves.
  // Built once per queue change rather than searched per beat on every render. Observing the
  // queue keeps a collapsed preview current when a save fails, succeeds or is discarded.
  const queuedBeatProse: ReadonlyMap<string, string> = $derived.by(() => {
    proseSaves.observe();
    return new Map(
      proseSaves
        .draftsForRecovery(currentProject.value?.id ?? "")
        .filter((draft) => draft.kind === "beat")
        .map((draft) => [draft.id, draft.prose])
    );
  });

  function unsavedBeatProse(beat: Beat): string {
    const local = draftProse.get(beat.id);
    if (local?.projectId === (currentProject.value?.id ?? "")) return local.prose;
    return queuedBeatProse.get(beat.id) ?? beat.prose ?? "";
  }

  // The open editor already holds its own draft; only a changed beat may push new content in.
  // Tracking draftProse here would reset the editor whenever a saved draft is cleared.
  function editorProse(beat: Beat): string {
    return untrack(() => unsavedBeatProse(beat));
  }

  function handleEditorUpdate(beatId: string) {
    return (html: string) => {
      handleProseInput(beatId, html);
    };
  }

  async function toggleBeat(beatId: string) {
    if (ui.expandedBeatId === beatId) {
      flushPendingSave(beatId);
      syncPendingProse(beatId);
      ui.setExpandedBeat(null);
    } else {
      if (ui.expandedBeatId) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
      }
      ui.setExpandedBeat(beatId);
      await tick();
      const beatElement = beatRefs.get(beatId);
      if (beatElement) {
        beatElement.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
  }

  function startAddingBeat() {
    addingBeat = true;
    newBeatContent = "";
  }

  async function createBeat() {
    if (!currentProject.currentScene || !newBeatContent.trim()) return;
    creatingBeat = true;
    try {
      const beat = await invoke<Beat>("create_beat", {
        sceneId: currentProject.currentScene.id,
        content: newBeatContent.trim(),
      });
      currentProject.addBeat(beat);
      addingBeat = false;
      newBeatContent = "";
      ui.setExpandedBeat(beat.id);
    } catch (e) {
      console.error("Failed to create beat:", e);
      ui.showError(`Failed to create beat: ${String(e)}`);
    } finally {
      creatingBeat = false;
    }
  }

  function handleNewBeatKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      createBeat();
    }
  }

  function startRenamingBeat(beat: Beat) {
    editingBeatId = beat.id;
    editingBeatContent = beat.content;
    tick().then(() => {
      const input = document.querySelector<HTMLInputElement>(`[data-rename-beat="${beat.id}"]`);
      input?.focus();
      input?.select();
    });
  }

  async function saveRenameBeat() {
    if (!editingBeatId) return;
    const content = editingBeatContent.trim();
    if (!content) {
      editingBeatId = null;
      editingBeatContent = "";
      return;
    }
    const beatId = editingBeatId;
    try {
      await invoke("rename_beat", { beatId, content });
    } catch (e) {
      // Keep the field open with the typed title so the writer can retry.
      console.error("Failed to rename beat:", e);
      ui.showError(`Failed to rename beat: ${String(e)}`);
      return;
    }
    currentProject.setBeats(
      currentProject.beats.map((b) => (b.id === beatId ? { ...b, content } : b))
    );
    if (editingBeatId === beatId) {
      editingBeatId = null;
      editingBeatContent = "";
    }
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      saveRenameBeat();
    } else if (e.key === "Escape") {
      editingBeatId = null;
      editingBeatContent = "";
    }
  }

  function getBeatContextMenuItems(beat: Beat) {
    const beatIndex = beats.findIndex((b) => b.id === beat.id);
    const nextBeat = beatIndex >= 0 && beatIndex < beats.length - 1 ? beats[beatIndex + 1] : null;
    const canSplit =
      beat.prose?.trim() &&
      ui.expandedBeatId === beat.id &&
      novelEditorRef &&
      (novelEditorRef.getSplitBeforeParagraph() ?? 0) >= 1;

    return [
      {
        label: "Rename",
        icon: Pencil,
        action: () => startRenamingBeat(beat),
        disabled: false,
      },
      {
        label: "Split at cursor",
        icon: ChevronRight,
        action: () => executeSplitBeat(beat),
        disabled: !canSplit,
      },
      {
        label: "Merge with next",
        icon: ChevronDown,
        action: () => {
          if (nextBeat) executeMergeBeats(beat, nextBeat);
        },
        disabled: !nextBeat,
      },
      {
        label: "Move up",
        icon: ArrowUp,
        action: () => moveBeatOneStep(beat, -1),
        disabled: beatIndex <= 0,
      },
      {
        label: "Move down",
        icon: ArrowDown,
        action: () => moveBeatOneStep(beat, 1),
        disabled: !nextBeat,
      },
      { label: "", divider: true, action: () => {} },
      {
        label: "Delete",
        icon: Trash2,
        action: () => {
          deleteBeatDialog = beat;
        },
        danger: true,
      },
    ];
  }

  async function executeDeleteBeat() {
    const beat = deleteBeatDialog;
    if (!beat || deletingBeat) return;
    deletingBeat = true;
    try {
      await invoke("delete_beat", { beatId: beat.id });
      currentProject.removeBeat(beat.id);
      if (ui.expandedBeatId === beat.id) {
        ui.setExpandedBeat(null);
      }
    } catch (e) {
      console.error("Failed to delete beat:", e);
      ui.showError(`Failed to delete beat: ${String(e)}`);
    } finally {
      deletingBeat = false;
      deleteBeatDialog = null;
    }
  }

  async function executeSplitBeat(beat: Beat) {
    if (changingBeats || isLocked) return;
    const paraIndex = novelEditorRef?.getSplitBeforeParagraph();
    if (paraIndex == null || paraIndex < 1) return;
    const sceneId = currentProject.currentScene?.id;
    const projectId = currentProject.value?.id;
    if (!sceneId || !projectId) return;
    changingBeats = true;
    try {
      await prepareBeatMutation([beat.id], projectId);
      if (
        currentProject.value?.id !== projectId ||
        currentProject.currentScene?.id !== sceneId ||
        isLocked
      )
        return;
      syncPendingProse(beat.id);
      const newBeat = await invoke<Beat>("split_beat", {
        beatId: beat.id,
        splitAt: null,
        splitBeforeParagraph: paraIndex,
      });
      const freshBeats = await invoke<Beat[]>("get_beats", {
        sceneId,
      });
      if (currentProject.value?.id !== projectId || currentProject.currentScene?.id !== sceneId)
        return;
      currentProject.setBeats(freshBeats);
      ui.setExpandedBeat(newBeat.id);
    } catch (e) {
      console.error("Failed to split beat:", e);
      ui.showError(`Failed to split beat: ${String(e)}`);
    } finally {
      changingBeats = false;
    }
  }

  async function executeMergeBeats(first: Beat, second: Beat) {
    if (changingBeats || isLocked) return;
    const sceneId = currentProject.currentScene?.id;
    const projectId = currentProject.value?.id;
    if (!sceneId || !projectId) return;
    changingBeats = true;
    try {
      await prepareBeatMutation([first.id, second.id], projectId);
      if (
        currentProject.value?.id !== projectId ||
        currentProject.currentScene?.id !== sceneId ||
        isLocked
      )
        return;
      if (ui.expandedBeatId === first.id || ui.expandedBeatId === second.id) {
        syncPendingProse(ui.expandedBeatId);
      }
      await invoke("merge_beats", {
        firstBeatId: first.id,
        secondBeatId: second.id,
      });
      const freshBeats = await invoke<Beat[]>("get_beats", {
        sceneId,
      });
      if (currentProject.value?.id !== projectId || currentProject.currentScene?.id !== sceneId)
        return;
      currentProject.setBeats(freshBeats);
      ui.setExpandedBeat(first.id);
    } catch (e) {
      console.error("Failed to merge beats:", e);
      ui.showError(`Failed to merge beats: ${String(e)}`);
    } finally {
      changingBeats = false;
    }
  }

  async function prepareBeatMutation(ids: string[], projectId: string) {
    for (const id of ids) await flushPendingSave(id);
    // A caught/terminal save failure must also prevent transforming stale database prose.
    if (proseSaves.draftsForRecovery(projectId).some((draft) => ids.includes(draft.id))) {
      throw new Error(
        "Save the affected beats or recover their unsaved drafts in Find and Replace first."
      );
    }
  }

  function onBeatDragHandleMouseDown(e: MouseEvent, beatId: string) {
    if (isLocked) return;
    e.preventDefault();
    e.stopPropagation();
    draggedBeatId = beatId;
    isDraggingBeat = true;
    const target = e.currentTarget as HTMLElement;
    draggedBeatElement = target.closest("[data-drag-beat]") as HTMLElement;
    if (draggedBeatElement) {
      draggedBeatElement.style.opacity = "0.5";
    }
    document.addEventListener("mousemove", onBeatDragMouseMove);
    document.addEventListener("mouseup", onBeatDragMouseUp);
    document.body.style.cursor = "grabbing";
    document.body.style.userSelect = "none";
  }

  function onBeatDragMouseMove(e: MouseEvent) {
    if (!isDraggingBeat || !draggedBeatId) return;
    if (currentDragOverBeatElement) {
      currentDragOverBeatElement.style.outline = "";
    }
    const itemElements = document.querySelectorAll("[data-drag-beat]");
    let foundId: string | null = null;
    let foundElement: HTMLElement | null = null;
    for (const el of itemElements) {
      const rect = el.getBoundingClientRect();
      const id = el.getAttribute("data-drag-beat");
      if (id && id !== draggedBeatId && e.clientY >= rect.top && e.clientY <= rect.bottom) {
        foundId = id;
        foundElement = el as HTMLElement;
        break;
      }
    }
    dragOverBeatId = foundId;
    currentDragOverBeatElement = foundElement;
    if (foundElement) {
      foundElement.style.outline = "2px solid var(--color-accent)";
    }
  }

  async function onBeatDragMouseUp() {
    document.removeEventListener("mousemove", onBeatDragMouseMove);
    document.removeEventListener("mouseup", onBeatDragMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    if (draggedBeatElement) {
      draggedBeatElement.style.opacity = "";
    }
    if (currentDragOverBeatElement) {
      currentDragOverBeatElement.style.outline = "";
    }
    if (draggedBeatId && dragOverBeatId && draggedBeatId !== dragOverBeatId) {
      const toIndex = beats.findIndex((b) => b.id === dragOverBeatId);
      if (toIndex !== -1) await moveBeat(draggedBeatId, toIndex);
    }
    isDraggingBeat = false;
    draggedBeatId = null;
    dragOverBeatId = null;
    draggedBeatElement = null;
    currentDragOverBeatElement = null;
  }

  /** Moves a beat to `toIndex` within the scene; resolves true once saved. */
  async function moveBeat(beatId: string, toIndex: number): Promise<boolean> {
    const sceneId = currentProject.currentScene?.id;
    const fromIndex = beats.findIndex((b) => b.id === beatId);
    if (!sceneId || isLocked || fromIndex === -1 || toIndex < 0 || toIndex >= beats.length)
      return false;
    if (fromIndex === toIndex) return false;
    const newOrder = [...beats];
    const [moved] = newOrder.splice(fromIndex, 1);
    newOrder.splice(toIndex, 0, moved);
    const newIds = newOrder.map((b) => b.id);
    try {
      await invoke("reorder_beats", { sceneId, beatIds: newIds });
      currentProject.reorderBeats(newIds);
      return true;
    } catch (e) {
      console.error("Failed to reorder beats:", e);
      ui.showError(`Failed to reorder beats: ${String(e)}`);
      return false;
    }
  }

  /** Keyboard alternative to dragging: move one place, announce it, keep focus on the beat. */
  async function moveBeatOneStep(beat: Beat, step: -1 | 1) {
    const toIndex = beats.findIndex((b) => b.id === beat.id) + step;
    const total = beats.length;
    if (!(await moveBeat(beat.id, toIndex))) return;
    moveAnnouncement = `Moved beat “${beat.content}” ${step < 0 ? "up" : "down"}, to position ${toIndex + 1} of ${total}.`;
    await tick();
    document
      .querySelector<HTMLElement>(`[data-drag-beat="${beat.id}"] [data-testid="beat-menu-button"]`)
      ?.focus({ preventScroll: true });
  }

  /** Opens the beat menu at the pointer, or under the button when opened from the keyboard. */
  function openBeatMenu(e: MouseEvent, beat: Beat) {
    let x = e.clientX;
    let y = e.clientY;
    if (x === 0 && y === 0 && e.currentTarget instanceof HTMLElement) {
      const rect = e.currentTarget.getBoundingClientRect();
      x = rect.left;
      y = rect.bottom;
    }
    beatContextMenu = { beat, x, y };
  }

  function stripHtml(html: string): string {
    return html
      .replace(/<[^>]*>/g, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

  function getBeatWordCount(prose: string | null): number {
    if (!prose) return 0;
    return stripHtml(prose)
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
  }
</script>

<section class="beats" aria-labelledby="beats-title">
  <p class="ka-sr" role="status" aria-live="polite">{moveAnnouncement}</p>
  <div class="beats-head">
    <h3 id="beats-title">Beats</h3>
    {#if beats.length > 0 && !addingBeat && !isLocked}
      <button
        type="button"
        onclick={startAddingBeat}
        class="ka-button ka-button--ghost beats-add-top"
      >
        <Plus class="w-5 h-5" aria-hidden="true" />
        Add beat
      </button>
    {/if}
  </div>
  {#if beats.length > 0}
    <div class="beats-list">
      {#each beats as beat, index (beat.id)}
        {@const isExpanded = ui.expandedBeatId === beat.id}
        {@const prose = unsavedBeatProse(beat)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <article
          data-drag-beat={beat.id}
          data-testid="beat-item"
          class="ka-beat beat"
          class:is-open={isExpanded}
          class:is-drop-target={dragOverBeatId === beat.id}
          use:registerBeatRef={beat.id}
        >
          <!-- Beat Header -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="beat-summary"
            oncontextmenu={(e) => {
              if (isLocked) return;
              beatContextMenu = { beat, x: e.clientX, y: e.clientY };
              e.preventDefault();
            }}
          >
            {#if !isLocked}
              <div
                data-testid="beat-drag-handle"
                onmousedown={(e) => onBeatDragHandleMouseDown(e, beat.id)}
                class="beat-grip"
                aria-hidden="true"
                title="Drag to reorder"
              >
                <GripVertical class="w-4 h-4" aria-hidden="true" />
              </div>
            {/if}
            {#if editingBeatId === beat.id}
              <div class="beat-toggle">
                <ChevronRight class="w-5 h-5 beat-chev" aria-hidden="true" />
                <span class="ka-beat-number">{index + 1}</span>
                <input
                  data-rename-beat={beat.id}
                  type="text"
                  bind:value={editingBeatContent}
                  onkeydown={handleRenameKeydown}
                  onblur={saveRenameBeat}
                  aria-label="Beat title"
                  class="beat-rename"
                />
              </div>
            {:else}
              <button
                data-testid="beat-header"
                onclick={() => toggleBeat(beat.id)}
                aria-expanded={isExpanded}
                class="beat-toggle"
              >
                <ChevronRight class="w-5 h-5 beat-chev" aria-hidden="true" />
                <span class="ka-beat-number">{index + 1}</span>
                <span class="beat-title" title={beat.content}>{beat.content}</span>
                {#if prose}
                  <small class="beat-count">{getBeatWordCount(prose)} words</small>
                {/if}
              </button>
            {/if}
            {#if !isLocked}
              <button
                data-testid="beat-menu-button"
                onclick={(e) => {
                  e.stopPropagation();
                  openBeatMenu(e, beat);
                }}
                class="ka-button ka-button--ghost ka-icon-button beat-menu"
                aria-label="Beat menu"
                aria-haspopup="menu"
              >
                <MoreVertical class="w-5 h-5" aria-hidden="true" />
              </button>
            {/if}
          </div>

          <!-- Expanded Beat Content -->
          {#if isExpanded}
            <div class="beat-editor">
              <NovelEditor
                bind:this={novelEditorRef}
                projectId={currentProject.value?.id}
                sceneId={beat.scene_id}
                beatId={beat.id}
                content={editorProse(beat)}
                placeholder={isLocked ? "Scene is locked" : "Write your prose for this beat…"}
                readonly={isLocked || changingBeats}
                saveStatus={localSaveStatus}
                onUpdate={handleEditorUpdate(beat.id)}
              />
            </div>
          {:else if prose}
            <div
              class="beat-preview"
              onclick={() => toggleBeat(beat.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  toggleBeat(beat.id);
                }
              }}
              role="button"
              tabindex="0"
              aria-label={`Open beat ${index + 1} prose`}
            >
              <p>{stripHtml(prose)}</p>
            </div>
          {/if}
        </article>
      {/each}
      {#if !addingBeat && !isLocked}
        <button
          type="button"
          onclick={startAddingBeat}
          class="ka-button ka-button--secondary beats-add"
        >
          <Plus class="w-5 h-5" aria-hidden="true" />
          Add beat
        </button>
      {/if}
    </div>
  {:else if !addingBeat && !isLocked}
    <div class="ka-empty od-stack beats-empty">
      <h4>No beats yet</h4>
      <p>Beats come from an imported outline, or you can add them here one moment at a time.</p>
      <button type="button" onclick={startAddingBeat} class="ka-button">
        <Plus class="w-5 h-5" aria-hidden="true" />
        Add your first beat
      </button>
    </div>
  {:else if !addingBeat && isLocked}
    <p class="ka-help beats-locked">
      <Lock class="w-4 h-4" aria-hidden="true" />
      Scene is locked
    </p>
  {/if}

  <!-- Add Beat Input -->
  {#if addingBeat && !isLocked}
    <div class="ka-field od-field beats-new">
      <label for="new-beat-content">New beat</label>
      <input
        id="new-beat-content"
        type="text"
        aria-describedby="new-beat-help"
        placeholder="Describe what happens in this beat…"
        bind:value={newBeatContent}
        onkeydown={handleNewBeatKeydown}
        disabled={creatingBeat}
      />
      <div class="ka-between">
        <p id="new-beat-help" class="ka-help">Press Enter to create, Escape to cancel.</p>
        <div class="ka-row">
          <button
            type="button"
            onclick={() => {
              addingBeat = false;
              newBeatContent = "";
            }}
            class="ka-button ka-button--ghost"
            disabled={creatingBeat}
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={createBeat}
            disabled={creatingBeat || !newBeatContent.trim()}
            aria-busy={creatingBeat || undefined}
            class="ka-button"
          >
            {#if creatingBeat}
              <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
              Creating…
            {:else}
              Create beat
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>

{#if beatContextMenu}
  <ContextMenu
    items={getBeatContextMenuItems(beatContextMenu.beat)}
    x={beatContextMenu.x}
    y={beatContextMenu.y}
    onClose={() => (beatContextMenu = null)}
  />
{/if}

{#if deleteBeatDialog}
  <ConfirmDialog
    title="Delete beat"
    message={beats[0]?.id === deleteBeatDialog.id
      ? "Are you sure you want to delete this beat? This is the first beat, so its prose will be permanently deleted."
      : "Are you sure you want to delete this beat? Any prose will be merged into the previous beat."}
    onConfirm={executeDeleteBeat}
    onCancel={() => (deleteBeatDialog = null)}
  />
{/if}

<style>
  .beats {
    margin-top: var(--space-m);
    padding-top: var(--space-s);
    border-top: var(--border-hair);
  }
  .beats-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    min-height: var(--control-target);
    margin-bottom: var(--space-2xs);
  }
  .beats-head h3 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .beats-list {
    display: grid;
    gap: var(--space-xs);
  }

  /* Press BeatItem: a sunken row; only the open beat carries the accent. */
  .beat {
    position: relative;
    user-select: none;
  }
  .beat.is-drop-target::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 2px;
    z-index: var(--z-raised);
    background: var(--color-accent-text);
  }
  .beat-summary {
    position: relative;
    display: flex;
    align-items: center;
    min-height: 52px;
    padding: 0 var(--space-3xs) 0 var(--space-2xs);
  }
  .beat {
    overflow: visible;
  }
  .beat.is-open .beat-summary {
    border-bottom: var(--border-hair);
  }
  .beat-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex: 1;
    min-width: 0;
    min-height: 52px;
    padding: var(--space-2xs) var(--space-2xs) var(--space-2xs) var(--space-3xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  .beat-toggle :global(.beat-chev) {
    flex: none;
    color: var(--color-text-muted);
    transition: transform var(--ka-motion) ease-out;
  }
  .beat.is-open .beat-toggle :global(.beat-chev) {
    transform: rotate(90deg);
  }
  .beat:not(.is-open) .ka-beat-number {
    background: transparent;
    color: var(--color-text-muted);
    box-shadow: inset 0 0 0 1px var(--color-control-border-hover);
  }
  .ka-beat-number {
    font: 500 var(--text-small) / 1 var(--font-ui);
  }
  .beat-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }
  .beat-count {
    flex: none;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .beat-rename {
    flex: 1;
    min-width: 0;
    min-height: var(--control-target);
    padding: var(--space-3xs) var(--space-2xs);
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
  }
  .beat-grip {
    position: absolute;
    left: -18px;
    top: 50%;
    display: flex;
    width: 16px;
    margin-top: -8px;
    color: var(--color-text-muted);
    cursor: grab;
    opacity: 0;
  }
  .beat-grip:active {
    cursor: grabbing;
  }
  .beat-menu {
    flex: none;
    color: var(--color-text-muted);
    opacity: 0;
  }
  .beat-summary:hover .beat-grip,
  .beat-summary:focus-within .beat-grip,
  .beat-summary:hover .beat-menu,
  .beat-summary:focus-within .beat-menu {
    opacity: 1;
  }
  @media (hover: none) {
    .beat-menu {
      opacity: 1;
    }
  }
  /* Grows with the prose up to a window-relative cap, then scrolls inside. */
  .beat-editor {
    position: relative;
    display: flex;
    flex-direction: column;
    max-height: min(50rem, calc(100vh - 20rem));
    border-radius: 0 0 var(--radius-m) var(--radius-m);
    overflow: hidden;
  }

  /* A collapsed beat's prose is manuscript: a strip of light paper in either
     theme, three lines deep, opening the beat when chosen. */
  .beat-preview {
    margin: 0 var(--space-s) var(--space-s);
    padding: var(--space-s) var(--space-m);
    max-height: 7.5rem;
    overflow: hidden;
    border-radius: var(--radius-xs);
    background: var(--color-prose-bg);
    color: var(--color-prose-text);
    box-shadow: 0 0 0 1px var(--color-prose-border);
    cursor: pointer;
  }
  .beat-preview p {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    margin: 0;
    max-width: var(--measure);
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
  }
  @media (hover: hover) {
    .beat-preview:hover {
      box-shadow: 0 0 0 1px var(--color-control-border-hover);
    }
  }

  .beats-add {
    justify-self: start;
    margin-top: var(--space-3xs);
  }
  .beats-empty {
    align-items: flex-start;
    padding: var(--space-m) 0;
  }
  .beats-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    color: var(--color-text);
  }
  .beats-locked {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .beats-new {
    margin-top: var(--space-s);
    gap: var(--space-2xs);
  }
</style>
