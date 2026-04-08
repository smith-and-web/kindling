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
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { tick } from "svelte";
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
  let hoveredBeatId: string | null = $state(null);

  let beatContextMenu: { beat: Beat; x: number; y: number } | null = $state(null);
  let deleteBeatDialog: Beat | null = $state(null);
  let deletingBeat = $state(false);
  let editingBeatId: string | null = $state(null);
  let editingBeatContent = $state("");

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let pendingSaveBeatId: string | null = null;
  let pendingProseUpdates = new SvelteMap<string, string>();
  let draftProse = new SvelteMap<string, string>();

  function syncPendingProse(beatId: string) {
    const pendingProse = pendingProseUpdates.get(beatId);
    if (pendingProse !== undefined) {
      currentProject.updateBeatProse(beatId, pendingProse);
      pendingProseUpdates.delete(beatId);
    }
  }

  function flushPendingSave(beatId?: string) {
    const targetBeatId = beatId ?? pendingSaveBeatId;
    if (!targetBeatId) return;
    if (saveTimeout && pendingSaveBeatId === targetBeatId) {
      clearTimeout(saveTimeout);
      saveTimeout = null;
      pendingSaveBeatId = null;
    }
    const draft = draftProse.get(targetBeatId);
    if (draft !== undefined) {
      saveBeatProse(targetBeatId, draft);
    }
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

  async function saveBeatProse(beatId: string, prose: string) {
    localSaveStatus = "saving";
    try {
      await invoke("save_beat_prose", { beatId, prose });
      if (!beats.some((beat) => beat.id === beatId)) {
        draftProse.delete(beatId);
        localSaveStatus = "idle";
        return;
      }
      currentProject.updateBeatProse(beatId, prose);
      pendingProseUpdates.delete(beatId);
      draftProse.delete(beatId);
      setTimeout(() => {
        localSaveStatus = "idle";
      }, 1000);
    } catch (e) {
      console.error("Failed to save beat prose:", e);
      localSaveStatus = "error";
    }
  }

  function handleProseInput(beatId: string, value: string) {
    draftProse.set(beatId, value);
    if (saveTimeout) clearTimeout(saveTimeout);
    pendingSaveBeatId = beatId;
    saveTimeout = setTimeout(() => {
      saveTimeout = null;
      pendingSaveBeatId = null;
      const draft = draftProse.get(beatId);
      if (draft !== undefined) {
        saveBeatProse(beatId, draft);
      }
    }, 500);
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
    try {
      await invoke("rename_beat", { beatId: editingBeatId, content });
      const freshBeats = await invoke<Beat[]>("get_beats", {
        sceneId: currentProject.currentScene!.id,
      });
      currentProject.setBeats(freshBeats);
    } catch (e) {
      console.error("Failed to rename beat:", e);
    }
    editingBeatId = null;
    editingBeatContent = "";
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
    } finally {
      deletingBeat = false;
      deleteBeatDialog = null;
    }
  }

  async function executeSplitBeat(beat: Beat) {
    const paraIndex = novelEditorRef?.getSplitBeforeParagraph();
    if (paraIndex == null || paraIndex < 1) return;
    if (!currentProject.currentScene) return;
    try {
      flushPendingSave(beat.id);
      syncPendingProse(beat.id);
      const newBeat = await invoke<Beat>("split_beat", {
        beatId: beat.id,
        splitAt: null,
        splitBeforeParagraph: paraIndex,
      });
      const freshBeats = await invoke<Beat[]>("get_beats", {
        sceneId: currentProject.currentScene.id,
      });
      currentProject.setBeats(freshBeats);
      ui.setExpandedBeat(newBeat.id);
    } catch (e) {
      console.error("Failed to split beat:", e);
    }
  }

  async function executeMergeBeats(first: Beat, second: Beat) {
    if (!currentProject.currentScene) return;
    try {
      if (ui.expandedBeatId === first.id || ui.expandedBeatId === second.id) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
      }
      await invoke("merge_beats", {
        firstBeatId: first.id,
        secondBeatId: second.id,
      });
      const freshBeats = await invoke<Beat[]>("get_beats", {
        sceneId: currentProject.currentScene.id,
      });
      currentProject.setBeats(freshBeats);
      ui.setExpandedBeat(first.id);
    } catch (e) {
      console.error("Failed to merge beats:", e);
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
    if (
      draggedBeatId &&
      dragOverBeatId &&
      draggedBeatId !== dragOverBeatId &&
      currentProject.currentScene
    ) {
      const fromIndex = beats.findIndex((b) => b.id === draggedBeatId);
      const toIndex = beats.findIndex((b) => b.id === dragOverBeatId);
      if (fromIndex !== -1 && toIndex !== -1) {
        const newOrder = [...beats];
        const [moved] = newOrder.splice(fromIndex, 1);
        newOrder.splice(toIndex, 0, moved);
        const newIds = newOrder.map((b) => b.id);
        try {
          await invoke("reorder_beats", {
            sceneId: currentProject.currentScene.id,
            beatIds: newIds,
          });
          currentProject.reorderBeats(newIds);
        } catch (e) {
          console.error("Failed to reorder beats:", e);
        }
      }
    }
    isDraggingBeat = false;
    draggedBeatId = null;
    dragOverBeatId = null;
    draggedBeatElement = null;
    currentDragOverBeatElement = null;
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

<section>
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide">Beats</h2>
    {#if beats.length > 0 && !addingBeat && !isLocked}
      <button
        onclick={startAddingBeat}
        class="flex items-center gap-1 text-text-secondary hover:text-text-primary transition-colors text-sm"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>Add Beat</span>
      </button>
    {/if}
  </div>
  {#if beats.length > 0}
    <div class="space-y-4">
      {#each beats as beat, index}
        {@const isExpanded = ui.expandedBeatId === beat.id}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <article
          data-drag-beat={beat.id}
          data-testid="beat-item"
          class="bg-bg-panel rounded-lg overflow-hidden select-none relative"
          class:ring-2={dragOverBeatId === beat.id}
          class:ring-accent={dragOverBeatId === beat.id}
          use:registerBeatRef={beat.id}
          onmouseenter={() => (hoveredBeatId = beat.id)}
          onmouseleave={() => (hoveredBeatId = null)}
        >
          <!-- Beat Header -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="w-full bg-beat-header px-4 py-2 flex items-center gap-2 hover:bg-beat-header/80 transition-colors cursor-pointer"
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
                class="cursor-grab active:cursor-grabbing p-0.5 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
                class:opacity-0={hoveredBeatId !== beat.id}
                class:opacity-100={hoveredBeatId === beat.id}
                role="button"
                tabindex="-1"
                aria-label="Drag to reorder"
              >
                <GripVertical class="w-3.5 h-3.5" />
              </div>
            {/if}
            {#if editingBeatId === beat.id}
              <div class="flex-1 flex items-center gap-3 min-w-0">
                <span class="text-text-secondary shrink-0">
                  {#if isExpanded}
                    <ChevronDown class="w-4 h-4" />
                  {:else}
                    <ChevronRight class="w-4 h-4" />
                  {/if}
                </span>
                <span
                  class="w-6 h-6 rounded-full bg-accent text-white text-xs font-medium flex items-center justify-center shrink-0"
                >
                  {index + 1}
                </span>
                <input
                  data-rename-beat={beat.id}
                  type="text"
                  bind:value={editingBeatContent}
                  onkeydown={handleRenameKeydown}
                  onblur={saveRenameBeat}
                  class="flex-1 min-w-0 bg-bg-card rounded px-2 py-0.5 text-text-primary text-sm font-medium focus:outline-none focus:ring-1 focus:ring-accent"
                />
              </div>
            {:else}
              <button
                data-testid="beat-header"
                onclick={() => toggleBeat(beat.id)}
                aria-expanded={isExpanded}
                class="flex-1 flex items-center gap-3 text-left min-w-0"
              >
                <span class="text-text-secondary shrink-0">
                  {#if isExpanded}
                    <ChevronDown class="w-4 h-4" />
                  {:else}
                    <ChevronRight class="w-4 h-4" />
                  {/if}
                </span>
                <span
                  class="w-6 h-6 rounded-full bg-accent text-white text-xs font-medium flex items-center justify-center shrink-0"
                >
                  {index + 1}
                </span>
                <p class="text-text-primary text-sm font-medium flex-1 truncate">
                  {beat.content}
                </p>
                {#if beat.prose || draftProse.get(beat.id)}
                  <span class="text-xs text-text-secondary shrink-0" title="Word count">
                    {getBeatWordCount(draftProse.get(beat.id) ?? beat.prose)}w
                  </span>
                {/if}
              </button>
            {/if}
            {#if !isLocked}
              <button
                data-testid="beat-menu-button"
                onclick={(e) => {
                  e.stopPropagation();
                  beatContextMenu = { beat, x: e.clientX, y: e.clientY };
                }}
                class="p-1 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
                class:opacity-0={hoveredBeatId !== beat.id}
                class:opacity-100={hoveredBeatId === beat.id}
                aria-label="Beat menu"
              >
                <MoreVertical class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>

          <!-- Expanded Beat Content -->
          {#if isExpanded}
            <div
              class="border-t border-bg-card relative"
              style="min-height: 20rem; height: calc(100vh - 20rem); max-height: 50rem;"
            >
              <NovelEditor
                bind:this={novelEditorRef}
                content={beat.prose || ""}
                placeholder={isLocked ? "Scene is locked" : "Write your prose for this beat..."}
                readonly={isLocked}
                saveStatus={localSaveStatus}
                onUpdate={handleEditorUpdate(beat.id)}
              />
            </div>
          {:else if beat.prose}
            <div
              class="px-4 py-3 border-t border-bg-card cursor-pointer hover:bg-bg-card/50 transition-colors overflow-hidden"
              style="max-height: 6.5rem;"
              onclick={() => toggleBeat(beat.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  toggleBeat(beat.id);
                }
              }}
              role="button"
              tabindex="0"
            >
              <p class="text-text-primary font-prose leading-relaxed line-clamp-3">
                {stripHtml(beat.prose)}
              </p>
            </div>
          {/if}
        </article>
      {/each}
      {#if !addingBeat && !isLocked}
        <button
          onclick={startAddingBeat}
          class="w-full flex items-center justify-center gap-1.5 py-2 mt-2 text-text-secondary hover:text-text-primary text-sm transition-colors rounded-lg hover:bg-bg-card"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Add Beat</span>
        </button>
      {/if}
    </div>
  {:else if !addingBeat && !isLocked}
    <button
      onclick={startAddingBeat}
      class="w-full flex items-center justify-center gap-2 px-4 py-8 rounded-lg border border-dashed border-bg-card text-text-secondary hover:text-text-primary hover:border-accent transition-colors"
    >
      <Plus class="w-4 h-4" />
      <span class="text-sm">Add Your First Beat</span>
    </button>
  {:else if !addingBeat && isLocked}
    <div
      class="w-full flex items-center justify-center gap-2 px-4 py-8 rounded-lg border border-dashed border-bg-card text-text-secondary/50"
    >
      <Lock class="w-4 h-4" />
      <span class="text-sm">Scene is locked</span>
    </div>
  {/if}

  <!-- Add Beat Input -->
  {#if addingBeat && !isLocked}
    <div class="mt-4 bg-bg-panel rounded-lg p-4">
      <input
        type="text"
        class="w-full bg-bg-card rounded-lg px-4 py-3 text-text-primary text-sm border border-accent focus:outline-none"
        placeholder="Describe what happens in this beat..."
        bind:value={newBeatContent}
        onkeydown={handleNewBeatKeydown}
        disabled={creatingBeat}
      />
      <div class="flex items-center justify-between mt-3">
        <p class="text-text-secondary text-xs">Press Enter to create, Escape to cancel</p>
        <div class="flex gap-2">
          <button
            onclick={() => {
              addingBeat = false;
              newBeatContent = "";
            }}
            class="px-3 py-1.5 text-text-secondary hover:text-text-primary text-sm transition-colors"
            disabled={creatingBeat}
          >
            Cancel
          </button>
          <button
            onclick={createBeat}
            disabled={creatingBeat || !newBeatContent.trim()}
            class="px-3 py-1.5 bg-accent text-white text-sm rounded hover:bg-accent/80 transition-colors disabled:opacity-50"
          >
            {#if creatingBeat}
              <Loader2 class="w-4 h-4 animate-spin" />
            {:else}
              Create Beat
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
    title="Delete Beat"
    message="Are you sure you want to delete this beat? Any prose will be merged into the previous beat."
    onConfirm={executeDeleteBeat}
    onCancel={() => (deleteBeatDialog = null)}
  />
{/if}
