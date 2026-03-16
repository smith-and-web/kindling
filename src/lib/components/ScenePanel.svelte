<script lang="ts">
  /* eslint-disable no-undef */
  import {
    FileText,
    ChevronRight,
    ChevronDown,
    Loader2,
    Plus,
    Pencil,
    Lock,
    GripVertical,
    Trash2,
    MoreVertical,
    CircleDot,
    CircleDashed,
    Lightbulb,
    Info,
    X,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ContextMenu from "./ContextMenu.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import { onDestroy, onMount, tick } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type {
    Beat,
    DiscoveryNote,
    PlanningStatus,
    ReferenceItem,
    ReferenceTypeId,
    Scene,
    SceneStatus,
    SceneType,
  } from "../types";
  import NovelEditor from "./NovelEditor.svelte";
  import Tooltip from "./Tooltip.svelte";

  // Check if scene is locked (either directly or via parent chapter)
  const isLocked = $derived(
    currentProject.currentScene?.locked || currentProject.currentChapter?.locked || false
  );

  // Refs for beat articles to scroll into view
  let beatRefs = new SvelteMap<string, HTMLElement>();

  // Action to register beat element refs
  function registerBeatRef(node: HTMLElement, beatId: string) {
    beatRefs.set(beatId, node);
    return {
      destroy() {
        beatRefs.delete(beatId);
      },
    };
  }

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let pendingSaveBeatId: string | null = null;
  let synopsisSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Synopsis editing state
  let editingSynopsis = $state(false);
  let synopsisText = $state("");
  let synopsisSaving = $state(false);
  let metadataSaving = $state(false);
  let metadataError = $state<string | null>(null);

  // New beat state
  let addingBeat = $state(false);
  let newBeatContent = $state("");
  let creatingBeat = $state(false);

  // Beat context menu and delete confirmation
  let beatContextMenu: { beat: Beat; x: number; y: number } | null = $state(null);
  let deleteBeatDialog: Beat | null = $state(null);
  let deletingBeat = $state(false);
  let novelEditorRef: { getSplitBeforeParagraph: () => number | null } | null = $state(null);

  // Beat drag-and-drop state
  let draggedBeatId: string | null = $state(null);
  let dragOverBeatId: string | null = $state(null);
  let isDraggingBeat = $state(false);
  let draggedBeatElement: HTMLElement | null = null;
  let currentDragOverBeatElement: HTMLElement | null = null;
  let hoveredBeatId: string | null = $state(null);

  const sceneTypeOptions: { value: SceneType; label: string }[] = [
    { value: "normal", label: "Normal" },
    { value: "notes", label: "Notes" },
    { value: "todo", label: "ToDo" },
    { value: "unused", label: "Unused" },
  ];

  const sceneStatusOptions: { value: SceneStatus; label: string }[] = [
    { value: "draft", label: "Draft" },
    { value: "revised", label: "Revised" },
    { value: "final", label: "Final" },
  ];

  const planningStatusOptions: { value: PlanningStatus; label: string }[] = [
    { value: "fixed", label: "Fixed" },
    { value: "flexible", label: "Flexible" },
    { value: "undefined", label: "Undefined" },
  ];

  const sceneReferenceOptions = REFERENCE_TYPE_OPTIONS.filter(
    (option) => option.id === "items" || option.id === "objectives" || option.id === "organizations"
  );

  let sceneReferenceItems = $state<Record<ReferenceTypeId, ReferenceItem[]>>(
    {} as Record<ReferenceTypeId, ReferenceItem[]>
  );
  let sceneReferenceLoading = $state(false);
  let sceneReferenceError = $state<string | null>(null);
  let sceneReferenceRequestId = 0;

  // Discovery notes state
  let discoveryNotesVisible = $state(false);
  let discoveryNotes = $state<DiscoveryNote[]>([]);
  let discoveryNotesLoading = $state(false);
  let addingDiscoveryNote = $state(false);
  let newDiscoveryNoteContent = $state("");
  let creatingDiscoveryNote = $state(false);
  let editingDiscoveryNoteId: string | null = $state(null);
  let editingDiscoveryNoteContent = $state("");
  let promotingNoteId: string | null = $state(null);

  async function loadSceneReferenceItems(sceneId: string) {
    const requestId = ++sceneReferenceRequestId;
    sceneReferenceLoading = true;
    sceneReferenceError = null;
    try {
      const results = await Promise.all(
        sceneReferenceOptions.map(async (option) => {
          const items = await invoke<ReferenceItem[]>("get_scene_reference_items", {
            sceneId,
            referenceType: option.id,
          });
          return [option.id, items] as const;
        })
      );

      if (requestId !== sceneReferenceRequestId) return;
      const next = {} as Record<ReferenceTypeId, ReferenceItem[]>;
      for (const [referenceType, items] of results) {
        next[referenceType] = items;
      }
      sceneReferenceItems = next;
    } catch (e) {
      if (requestId !== sceneReferenceRequestId) return;
      console.error("Failed to load scene reference items:", e);
      sceneReferenceError = e instanceof Error ? e.message : "Failed to load scene reference items";
      sceneReferenceItems = {} as Record<ReferenceTypeId, ReferenceItem[]>;
    } finally {
      if (requestId === sceneReferenceRequestId) {
        sceneReferenceLoading = false;
      }
    }
  }

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

  async function toggleBeat(beatId: string) {
    if (ui.expandedBeatId === beatId) {
      flushPendingSave(beatId);
      // Collapsing - sync any pending prose updates to the store
      syncPendingProse(beatId);
      ui.setExpandedBeat(null);
    } else {
      // If we're switching from another beat, sync its pending updates first
      if (ui.expandedBeatId) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
      }
      ui.setExpandedBeat(beatId);
      // Wait for DOM to update, then scroll the beat into view
      await tick();
      const beatElement = beatRefs.get(beatId);
      if (beatElement) {
        beatElement.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
  }

  // Track pending prose updates to sync to store when beat is collapsed
  let pendingProseUpdates = new SvelteMap<string, string>();
  let draftProse = new SvelteMap<string, string>();

  // Local save status to avoid global store updates causing re-renders
  let localSaveStatus = $state<"idle" | "saving" | "error">("idle");

  async function saveBeatProse(beatId: string, prose: string) {
    localSaveStatus = "saving";
    try {
      await invoke("save_beat_prose", { beatId, prose });
      if (!currentProject.beats.some((beat) => beat.id === beatId)) {
        draftProse.delete(beatId);
        localSaveStatus = "idle";
        return;
      }
      // Don't update the store while editing - it causes re-renders and flashing
      // Instead, track the update and sync when the beat is collapsed
      pendingProseUpdates.set(beatId, prose);
      draftProse.delete(beatId);

      if (ui.expandedBeatId !== beatId) {
        currentProject.updateBeatProse(beatId, prose);
        pendingProseUpdates.delete(beatId);
      }
      // Keep showing "saving" for 1 more second so user sees the indicator
      setTimeout(() => {
        localSaveStatus = "idle";
      }, 1000);
    } catch (e) {
      console.error("Failed to save beat prose:", e);
      localSaveStatus = "error";
    }
  }

  async function saveSceneMetadata(scene: Scene, nextType: SceneType, nextStatus: SceneStatus) {
    if (metadataSaving) return;
    metadataSaving = true;
    metadataError = null;

    try {
      await invoke("update_scene_metadata", {
        sceneId: scene.id,
        metadata: {
          scene_type: nextType,
          scene_status: nextStatus,
        },
      });
      currentProject.updateScene(scene.id, {
        scene_type: nextType,
        scene_status: nextStatus,
      });
    } catch (e) {
      metadataError = e instanceof Error ? e.message : "Failed to update scene metadata";
    } finally {
      metadataSaving = false;
    }
  }

  function handleSceneTypeChange(event: Event, scene: Scene) {
    const nextType = (event.currentTarget as HTMLSelectElement).value as SceneType;
    const nextStatus = scene.scene_status ?? "draft";
    saveSceneMetadata(scene, nextType, nextStatus);
  }

  function handleSceneStatusChange(event: Event, scene: Scene) {
    const nextStatus = (event.currentTarget as HTMLSelectElement).value as SceneStatus;
    const nextType = scene.scene_type ?? "normal";
    saveSceneMetadata(scene, nextType, nextStatus);
  }

  async function setScenePlanningStatus(scene: Scene, nextStatus: PlanningStatus) {
    if (metadataSaving) return;
    metadataSaving = true;
    metadataError = null;
    try {
      await invoke("update_scene_planning_status", {
        sceneId: scene.id,
        planningStatus: nextStatus,
      });
      currentProject.updateScene(scene.id, { planning_status: nextStatus });
    } catch (e) {
      metadataError = e instanceof Error ? e.message : "Failed to update planning status";
    } finally {
      metadataSaving = false;
    }
  }

  function handleScenePlanningStatusChange(event: Event, scene: Scene) {
    const nextStatus = (event.currentTarget as HTMLSelectElement).value as PlanningStatus;
    setScenePlanningStatus(scene, nextStatus);
  }

  function handleProseInput(beatId: string, value: string) {
    // Debounce save by 500ms
    draftProse.set(beatId, value);
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "d" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      discoveryNotesVisible = !discoveryNotesVisible;
      return;
    }
    if (e.key === "Escape") {
      if (ui.expandedBeatId) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
        ui.setExpandedBeat(null);
      }
      if (editingSynopsis) {
        editingSynopsis = false;
      }
      if (addingBeat) {
        addingBeat = false;
        newBeatContent = "";
      }
      if (addingDiscoveryNote) {
        addingDiscoveryNote = false;
        newDiscoveryNoteContent = "";
      }
      if (editingDiscoveryNoteId) {
        editingDiscoveryNoteId = null;
        editingDiscoveryNoteContent = "";
      }
    }
  }

  // Synopsis functions
  function startEditingSynopsis() {
    synopsisText = currentProject.currentScene?.synopsis || "";
    editingSynopsis = true;
  }

  async function saveSynopsis() {
    if (!currentProject.currentScene) return;
    synopsisSaving = true;
    try {
      const synopsis = synopsisText.trim() || null;
      await invoke("save_scene_synopsis", {
        sceneId: currentProject.currentScene.id,
        synopsis,
      });
      currentProject.updateSceneSynopsis(currentProject.currentScene.id, synopsis);
      editingSynopsis = false;
    } catch (e) {
      console.error("Failed to save synopsis:", e);
    } finally {
      synopsisSaving = false;
    }
  }

  function handleSynopsisInput(value: string) {
    synopsisText = value;
    // Debounce auto-save
    if (synopsisSaveTimeout) {
      clearTimeout(synopsisSaveTimeout);
    }
    synopsisSaveTimeout = setTimeout(() => {
      saveSynopsis();
    }, 1000);
  }

  // Beat functions
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
      // Auto-expand the new beat for immediate editing
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

  function openBeatContextMenu(e: MouseEvent, beat: Beat) {
    e.preventDefault();
    e.stopPropagation();
    beatContextMenu = { beat, x: e.clientX, y: e.clientY };
  }

  function closeBeatContextMenu() {
    beatContextMenu = null;
  }

  function getBeatContextMenuItems(beat: Beat) {
    const beats = currentProject.beats;
    const beatIndex = beats.findIndex((b) => b.id === beat.id);
    const nextBeat = beatIndex >= 0 && beatIndex < beats.length - 1 ? beats[beatIndex + 1] : null;
    const canSplit =
      beat.prose?.trim() &&
      ui.expandedBeatId === beat.id &&
      novelEditorRef &&
      (novelEditorRef.getSplitBeforeParagraph() ?? 0) >= 1;

    return [
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
        action: () => confirmDeleteBeat(beat),
        danger: true,
      },
    ];
  }

  function confirmDeleteBeat(beat: Beat) {
    beatContextMenu = null;
    deleteBeatDialog = beat;
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
    beatContextMenu = null;
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
      const beats = await invoke<Beat[]>("get_beats", {
        sceneId: currentProject.currentScene.id,
      });
      currentProject.setBeats(beats);
      ui.setExpandedBeat(newBeat.id);
    } catch (e) {
      console.error("Failed to split beat:", e);
    }
  }

  async function executeMergeBeats(first: Beat, second: Beat) {
    beatContextMenu = null;
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
      const beats = await invoke<Beat[]>("get_beats", {
        sceneId: currentProject.currentScene.id,
      });
      currentProject.setBeats(beats);
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
      const beats = currentProject.beats;
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

  // Strip HTML tags for plain text preview
  function stripHtml(html: string): string {
    return html.replace(/<[^>]*>/g, "").trim();
  }

  function getBeatWordCount(prose: string | null): number {
    if (!prose) return 0;
    return stripHtml(prose)
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
  }

  let lastSceneId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (lastSceneId && sceneId !== lastSceneId) {
      flushPendingSave(ui.expandedBeatId ?? undefined);
      if (ui.expandedBeatId) {
        syncPendingProse(ui.expandedBeatId);
      }
      pendingProseUpdates.clear();
      draftProse.clear();
      ui.setExpandedBeat(null);
    }
    lastSceneId = sceneId;
  });

  let lastSceneReferenceId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (sceneId === lastSceneReferenceId) return;
    lastSceneReferenceId = sceneId;

    if (!sceneId) {
      sceneReferenceItems = {} as Record<ReferenceTypeId, ReferenceItem[]>;
      sceneReferenceError = null;
      sceneReferenceLoading = false;
      return;
    }

    loadSceneReferenceItems(sceneId);
  });

  let lastSceneReferenceRefreshId = -1;
  $effect(() => {
    const refreshId = ui.sceneReferenceRefreshId;
    const sceneId = currentProject.currentScene?.id ?? null;
    if (!sceneId) return;
    if (refreshId === lastSceneReferenceRefreshId) return;
    lastSceneReferenceRefreshId = refreshId;
    loadSceneReferenceItems(sceneId);
  });

  async function loadDiscoveryNotes(sceneId: string) {
    discoveryNotesLoading = true;
    try {
      discoveryNotes = await invoke<DiscoveryNote[]>("get_discovery_notes", { sceneId });
    } catch (e) {
      console.error("Failed to load discovery notes:", e);
      discoveryNotes = [];
    } finally {
      discoveryNotesLoading = false;
    }
  }

  let lastDiscoveryNotesSceneId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (sceneId === lastDiscoveryNotesSceneId) return;
    lastDiscoveryNotesSceneId = sceneId;
    if (!sceneId) {
      discoveryNotes = [];
      return;
    }
    loadDiscoveryNotes(sceneId);
  });

  async function createDiscoveryNote() {
    if (!currentProject.currentScene || !newDiscoveryNoteContent.trim()) return;
    creatingDiscoveryNote = true;
    try {
      const note = await invoke<DiscoveryNote>("create_discovery_note", {
        sceneId: currentProject.currentScene.id,
        content: newDiscoveryNoteContent.trim(),
        tags: [],
      });
      discoveryNotes = [...discoveryNotes, note];
      addingDiscoveryNote = false;
      newDiscoveryNoteContent = "";
    } catch (e) {
      console.error("Failed to create discovery note:", e);
    } finally {
      creatingDiscoveryNote = false;
    }
  }

  async function updateDiscoveryNote(noteId: string, content: string) {
    try {
      const updated = await invoke<DiscoveryNote>("update_discovery_note", {
        noteId,
        content,
        tags: null,
      });
      discoveryNotes = discoveryNotes.map((n) => (n.id === noteId ? updated : n));
      editingDiscoveryNoteId = null;
      editingDiscoveryNoteContent = "";
    } catch (e) {
      console.error("Failed to update discovery note:", e);
    }
  }

  async function deleteDiscoveryNote(noteId: string) {
    try {
      await invoke("delete_discovery_note", { noteId });
      discoveryNotes = discoveryNotes.filter((n) => n.id !== noteId);
    } catch (e) {
      console.error("Failed to delete discovery note:", e);
    }
  }

  async function promoteNoteToBeat(note: DiscoveryNote) {
    promotingNoteId = note.id;
    try {
      const beat = await invoke<Beat>("promote_discovery_note_to_beat", { noteId: note.id });
      currentProject.addBeat(beat);
      discoveryNotes = discoveryNotes.filter((n) => n.id !== note.id);
      ui.setExpandedBeat(beat.id);
    } catch (e) {
      console.error("Failed to promote note to beat:", e);
    } finally {
      promotingNoteId = null;
    }
  }

  function startAddingDiscoveryNote() {
    addingDiscoveryNote = true;
    newDiscoveryNoteContent = "";
  }

  onMount(() => {
    const handler = () => (discoveryNotesVisible = !discoveryNotesVisible);
    window.addEventListener("kindling:toggleDiscoveryNotes", handler);
    return () => window.removeEventListener("kindling:toggleDiscoveryNotes", handler);
  });

  onDestroy(() => {
    flushPendingSave(ui.expandedBeatId ?? undefined);
    if (ui.expandedBeatId) {
      syncPendingProse(ui.expandedBeatId);
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div data-testid="scene-panel" class="flex-1 flex flex-col h-full overflow-hidden">
  {#if currentProject.currentScene}
    {@const scene = currentProject.currentScene}
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-8">
        <!-- Scene Title -->
        <header class="mb-8">
          <div class="flex items-center gap-3">
            <h1
              data-testid="scene-title"
              class="text-3xl font-heading font-semibold text-text-primary"
            >
              {scene.title}
            </h1>
            {#if isLocked}
              <span
                class="flex items-center gap-1 px-2 py-1 bg-amber-500/10 text-amber-500 rounded-lg text-sm"
              >
                <Lock class="w-4 h-4" />
                Locked
              </span>
            {/if}
          </div>
          {#if currentProject.currentChapter}
            <p class="text-text-secondary text-sm mt-1">
              {currentProject.currentChapter.title}
            </p>
          {/if}
          <div class="mt-4 flex flex-wrap gap-4">
            <div class="flex flex-col gap-1">
              <label for="scene-type" class="text-xs text-text-secondary">Scene type</label>
              <div class="relative">
                <select
                  id="scene-type"
                  value={scene.scene_type ?? "normal"}
                  onchange={(event) => handleSceneTypeChange(event, scene)}
                  class="appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer disabled:opacity-60"
                  disabled={isLocked || metadataSaving}
                >
                  {#each sceneTypeOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <label for="scene-status" class="text-xs text-text-secondary">Status</label>
              <div class="relative">
                <select
                  id="scene-status"
                  value={scene.scene_status ?? "draft"}
                  onchange={(event) => handleSceneStatusChange(event, scene)}
                  class="appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer disabled:opacity-60"
                  disabled={isLocked || metadataSaving}
                >
                  {#each sceneStatusOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <Tooltip
                text="Controls how much structure this scene has: Undefined → Flexible → Fixed"
                position="top"
              >
                <label
                  for="planning-status"
                  class="text-xs text-text-secondary cursor-help flex items-center gap-1"
                >
                  Planning
                  <Info class="w-3 h-3 text-text-secondary/50" />
                </label>
              </Tooltip>
              <div class="relative">
                <select
                  id="planning-status"
                  value={scene.planning_status ?? "fixed"}
                  onchange={(event) => handleScenePlanningStatusChange(event, scene)}
                  class="appearance-none bg-bg-card text-text-primary text-sm border border-bg-card rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 cursor-pointer disabled:opacity-60"
                  disabled={isLocked || metadataSaving}
                >
                  {#each planningStatusOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary pointer-events-none"
                />
              </div>
            </div>
          </div>
          {#if metadataError}
            <p class="text-xs text-red-400 mt-2">{metadataError}</p>
          {/if}
        </header>

        <!-- Planning status guidance (first-time, shown once on any scene) -->
        {#if !ui.hasSeenTooltip("planningStatus")}
          <div class="mb-6 px-4 py-3 bg-accent/5 border border-accent/15 rounded-lg">
            <div class="flex items-start gap-2.5">
              <Lightbulb class="w-4 h-4 text-accent shrink-0 mt-0.5" />
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2">
                  <p class="text-sm font-medium text-text-primary">Rolling outline</p>
                  <button
                    onclick={() => ui.markTooltipSeen("planningStatus")}
                    class="p-0.5 text-text-secondary hover:text-text-primary rounded transition-colors shrink-0"
                    aria-label="Dismiss"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>
                <p class="text-xs text-text-secondary leading-relaxed mt-1 mb-2.5">
                  The <strong class="text-text-primary">Planning</strong> dropdown above controls how
                  much structure this scene has. Use it to work through your story gradually:
                </p>
                <div class="grid grid-cols-3 gap-3">
                  <div class="text-xs">
                    <span class="font-medium text-text-secondary/60 flex items-center gap-1"
                      ><CircleDashed class="w-3 h-3" /> Undefined</span
                    >
                    <p class="text-text-secondary/70 mt-0.5">
                      A placeholder — you know it exists but haven't planned it.
                    </p>
                  </div>
                  <div class="text-xs">
                    <span class="font-medium text-amber-500/70 flex items-center gap-1"
                      ><CircleDot class="w-3 h-3" /> Flexible</span
                    >
                    <p class="text-text-secondary/70 mt-0.5">
                      You have the gist — a synopsis and rough direction.
                    </p>
                  </div>
                  <div class="text-xs">
                    <span class="font-medium text-text-primary flex items-center gap-1">Fixed</span>
                    <p class="text-text-secondary/70 mt-0.5">
                      Full structure with beats, references, and notes.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Undefined: Placeholder view -->
        {#if (scene.planning_status ?? "fixed") === "undefined"}
          <div class="mb-8 p-6 bg-bg-panel rounded-lg border border-dashed border-bg-card">
            <div class="flex items-start gap-3">
              <div
                class="w-8 h-8 rounded-full bg-text-secondary/10 flex items-center justify-center shrink-0 mt-0.5"
              >
                <CircleDashed class="w-4 h-4 text-text-secondary/60" />
              </div>
              <div>
                <h3 class="text-sm font-medium text-text-primary mb-1">Undefined scene</h3>
                <p class="text-text-secondary text-sm mb-1">
                  This is a placeholder — you know it exists but haven't planned it yet.
                </p>
                <p class="text-text-secondary/70 text-xs mb-3">
                  Add a synopsis above to capture the gist, then promote it when you're ready to
                  flesh it out.
                </p>
                {#if !isLocked}
                  <div class="flex items-center gap-2">
                    <button
                      onclick={() => setScenePlanningStatus(scene, "flexible")}
                      class="px-3 py-1.5 rounded-md bg-accent/10 text-accent text-sm font-medium hover:bg-accent/20 transition-colors"
                    >
                      Switch to Flexible
                    </button>
                    <button
                      onclick={() => setScenePlanningStatus(scene, "fixed")}
                      class="px-3 py-1.5 rounded-md text-text-secondary text-sm hover:text-text-primary hover:bg-bg-card transition-colors"
                    >
                      Go straight to Fixed
                    </button>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        {:else if (scene.planning_status ?? "fixed") === "flexible"}
          <!-- Flexible: Synopsis + prompt to add beats -->
          <div class="mb-8 p-6 bg-bg-panel rounded-lg border border-dashed border-bg-card">
            <div class="flex items-start gap-3">
              <div
                class="w-8 h-8 rounded-full bg-amber-500/10 flex items-center justify-center shrink-0 mt-0.5"
              >
                <CircleDot class="w-4 h-4 text-amber-500/70" />
              </div>
              <div>
                <h3 class="text-sm font-medium text-text-primary mb-1">Flexible scene</h3>
                <p class="text-text-secondary text-sm mb-1">
                  You have an idea for this scene but haven't locked down the structure.
                </p>
                <p class="text-text-secondary/70 text-xs mb-3">
                  Use the synopsis to capture your intent. When you're ready to break it into beats,
                  switch to Fixed.
                </p>
                {#if !isLocked}
                  <button
                    onclick={() => setScenePlanningStatus(scene, "fixed")}
                    class="px-3 py-1.5 rounded-md bg-accent/10 text-accent text-sm font-medium hover:bg-accent/20 transition-colors"
                  >
                    Define beats
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/if}

        <!-- Locked Banner -->
        {#if isLocked}
          <div class="mb-8 px-4 py-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
            <div class="flex items-center gap-2 text-amber-500">
              <Lock class="w-4 h-4" />
              <span class="font-medium">This scene is locked</span>
            </div>
            <p class="text-text-secondary text-sm mt-1">
              {#if currentProject.currentChapter?.locked}
                The parent chapter is locked. Unlock the chapter to edit this scene.
              {:else}
                Unlock this scene from the sidebar to make changes.
              {/if}
            </p>
          </div>
        {/if}

        <!-- Synopsis (shown for all planning statuses) -->
        <section class="mb-8">
          <div class="flex items-center justify-between mb-2">
            <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide">
              Synopsis
            </h2>
            {#if scene.synopsis && !editingSynopsis && !isLocked}
              <Tooltip text="Edit synopsis" position="left">
                <button
                  onclick={startEditingSynopsis}
                  class="text-text-secondary hover:text-text-primary transition-colors p-1"
                  aria-label="Edit synopsis"
                >
                  <Pencil class="w-3.5 h-3.5" />
                </button>
              </Tooltip>
            {/if}
          </div>
          {#if editingSynopsis && !isLocked}
            <div class="relative">
              <textarea
                class="w-full min-h-[100px] bg-bg-card rounded-lg p-4 text-text-primary font-prose italic leading-relaxed resize-y border border-accent focus:outline-none"
                placeholder="Write a brief synopsis for this scene..."
                bind:value={synopsisText}
                oninput={(e) => handleSynopsisInput(e.currentTarget.value)}
              ></textarea>
              {#if synopsisSaving}
                <div
                  class="absolute bottom-3 right-3 flex items-center gap-1.5 text-text-secondary/50"
                >
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  <span class="text-xs">Saving...</span>
                </div>
              {/if}
            </div>
            <p class="text-text-secondary text-xs mt-2">
              Press Escape to close. Changes are saved automatically.
            </p>
          {:else if scene.synopsis}
            <div class="bg-bg-panel rounded-lg p-4 border-l-2 border-accent">
              <p class="text-text-primary font-prose italic">
                {scene.synopsis}
              </p>
            </div>
          {:else if !isLocked}
            <button
              onclick={startEditingSynopsis}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-bg-card text-text-secondary hover:text-text-primary hover:border-accent transition-colors"
            >
              <Plus class="w-4 h-4" />
              <span class="text-sm">Add Synopsis</span>
            </button>
          {:else}
            <div
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-bg-card text-text-secondary/50"
            >
              <Lock class="w-4 h-4" />
              <span class="text-sm">Scene is locked</span>
            </div>
          {/if}
        </section>

        <!-- References (Fixed only) -->
        {#if (scene.planning_status ?? "fixed") === "fixed"}
          <section class="mb-8">
            <div class="flex items-center justify-between mb-2">
              <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide">
                References
              </h2>
              {#if sceneReferenceLoading}
                <span class="text-xs text-text-secondary">Loading…</span>
              {/if}
            </div>
            {#if sceneReferenceError}
              <p class="text-xs text-red-400">{sceneReferenceError}</p>
            {:else}
              {@const hasSceneReferences = sceneReferenceOptions.some(
                (option) => (sceneReferenceItems[option.id]?.length ?? 0) > 0
              )}
              {#if hasSceneReferences}
                <div class="space-y-4">
                  {#each sceneReferenceOptions as option (option.id)}
                    {@const items = sceneReferenceItems[option.id] ?? []}
                    {#if items.length > 0}
                      {@const Icon = option.icon}
                      <div>
                        <div class="flex items-center gap-2 text-xs text-text-secondary">
                          <Icon class={`w-3.5 h-3.5 ${option.accentClass}`} />
                          <span class="font-medium">{option.label}</span>
                        </div>
                        <div class="mt-2 flex flex-wrap gap-2">
                          {#each items as item (item.id)}
                            <span
                              class="px-2 py-1 rounded-md bg-bg-panel text-text-primary text-xs"
                            >
                              {item.name}
                            </span>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  {/each}
                </div>
              {:else if !sceneReferenceLoading}
                <div class="text-sm text-text-secondary">
                  No linked items, objectives, or organizations.
                </div>
              {/if}
            {/if}
          </section>
        {/if}

        <!-- Discovery Notes (Fixed only, Cmd/Ctrl+D) -->
        {#if (scene.planning_status ?? "fixed") === "fixed"}
          <section class="mb-8">
            <button
              type="button"
              onclick={() => (discoveryNotesVisible = !discoveryNotesVisible)}
              class="flex items-center justify-between w-full mb-2 text-left group"
            >
              <h2
                class="text-sm font-semibold text-text-primary uppercase tracking-wide group-hover:text-text-primary transition-colors"
              >
                Discovery Notes
              </h2>
              <span class="text-xs text-text-secondary">
                {discoveryNotesVisible ? "Hide" : "Show"} (⌘D)
              </span>
            </button>
            {#if discoveryNotesVisible}
              {#if discoveryNotesLoading}
                <p class="text-sm text-text-secondary">Loading…</p>
              {:else}
                <div class="space-y-3">
                  {#if !addingDiscoveryNote && !isLocked}
                    <button
                      type="button"
                      onclick={startAddingDiscoveryNote}
                      class="flex items-center gap-1 text-text-secondary hover:text-text-primary transition-colors text-sm"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Add note</span>
                    </button>
                  {/if}
                  {#if addingDiscoveryNote}
                    <div class="flex flex-col gap-2 p-3 rounded-lg bg-bg-panel">
                      <textarea
                        bind:value={newDiscoveryNoteContent}
                        placeholder="What did you discover?"
                        rows="2"
                        class="w-full px-3 py-2 rounded-md bg-bg-base text-text-primary text-sm placeholder:text-text-secondary/50 resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                      ></textarea>
                      <div class="flex gap-2">
                        <button
                          type="button"
                          onclick={createDiscoveryNote}
                          disabled={!newDiscoveryNoteContent.trim() || creatingDiscoveryNote}
                          class="px-3 py-1.5 rounded-md bg-accent text-accent-foreground text-sm font-medium disabled:opacity-50"
                        >
                          {creatingDiscoveryNote ? "Adding…" : "Add"}
                        </button>
                        <button
                          type="button"
                          onclick={() => {
                            addingDiscoveryNote = false;
                            newDiscoveryNoteContent = "";
                          }}
                          class="px-3 py-1.5 rounded-md bg-bg-card text-text-secondary text-sm hover:text-text-primary"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#each discoveryNotes as note}
                    {@const isEditing = editingDiscoveryNoteId === note.id}
                    <div class="p-3 rounded-lg bg-bg-panel">
                      {#if isEditing}
                        <textarea
                          bind:value={editingDiscoveryNoteContent}
                          rows="2"
                          class="w-full px-3 py-2 rounded-md bg-bg-base text-text-primary text-sm resize-none focus:outline-none focus:ring-2 focus:ring-accent mb-2"
                        ></textarea>
                        <div class="flex gap-2">
                          <button
                            type="button"
                            onclick={() =>
                              updateDiscoveryNote(note.id, editingDiscoveryNoteContent)}
                            class="px-3 py-1.5 rounded-md bg-accent text-accent-foreground text-sm font-medium"
                          >
                            Save
                          </button>
                          <button
                            type="button"
                            onclick={() => {
                              editingDiscoveryNoteId = null;
                              editingDiscoveryNoteContent = "";
                            }}
                            class="px-3 py-1.5 rounded-md bg-bg-card text-text-secondary text-sm hover:text-text-primary"
                          >
                            Cancel
                          </button>
                        </div>
                      {:else}
                        <p class="text-sm text-text-primary whitespace-pre-wrap">{note.content}</p>
                        {#if note.tags && note.tags.length > 0}
                          <div class="flex flex-wrap gap-1 mt-2">
                            {#each note.tags as tag}
                              <span
                                class="px-1.5 py-0.5 rounded bg-bg-base text-xs text-text-secondary"
                              >
                                {tag}
                              </span>
                            {/each}
                          </div>
                        {/if}
                        <div class="flex gap-2 mt-2">
                          <button
                            type="button"
                            onclick={() => {
                              editingDiscoveryNoteId = note.id;
                              editingDiscoveryNoteContent = note.content;
                            }}
                            class="text-xs text-text-secondary hover:text-text-primary"
                          >
                            Edit
                          </button>
                          <button
                            type="button"
                            onclick={() => deleteDiscoveryNote(note.id)}
                            class="text-xs text-text-secondary hover:text-red-400"
                          >
                            Delete
                          </button>
                          <button
                            type="button"
                            onclick={() => promoteNoteToBeat(note)}
                            disabled={promotingNoteId === note.id}
                            class="text-xs text-text-secondary hover:text-accent disabled:opacity-50"
                          >
                            {promotingNoteId === note.id ? "Promoting…" : "Promote to beat"}
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/each}
                  {#if discoveryNotes.length === 0 && !addingDiscoveryNote}
                    <p class="text-sm text-text-secondary">No discovery notes yet.</p>
                  {/if}
                </div>
              {/if}
            {/if}
          </section>
        {/if}

        <!-- Beats (Fixed only) -->
        {#if (scene.planning_status ?? "fixed") === "fixed"}
          <section>
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide">Beats</h2>
              {#if currentProject.beats.length > 0 && !addingBeat && !isLocked}
                <button
                  onclick={startAddingBeat}
                  class="flex items-center gap-1 text-text-secondary hover:text-text-primary transition-colors text-sm"
                >
                  <Plus class="w-3.5 h-3.5" />
                  <span>Add Beat</span>
                </button>
              {/if}
            </div>
            {#if currentProject.beats.length > 0}
              <div class="space-y-4">
                {#each currentProject.beats as beat, index}
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
                    <!-- Beat Header (clickable to expand) -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                      class="w-full bg-beat-header px-4 py-2 flex items-center gap-2 hover:bg-beat-header/80 transition-colors cursor-pointer"
                      oncontextmenu={(e) => !isLocked && openBeatContextMenu(e, beat)}
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
                      <button
                        data-testid="beat-header"
                        onclick={() => toggleBeat(beat.id)}
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
                        {#if beat.prose}
                          <span class="text-xs text-text-secondary shrink-0" title="Word count">
                            {getBeatWordCount(beat.prose)}w
                          </span>
                        {/if}
                      </button>
                      {#if !isLocked}
                        <button
                          data-testid="beat-menu-button"
                          onclick={(e) => openBeatContextMenu(e, beat)}
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
                      <div class="border-t border-bg-card relative" style="height: 50rem;">
                        <NovelEditor
                          bind:this={novelEditorRef}
                          content={beat.prose || ""}
                          placeholder={isLocked
                            ? "Scene is locked"
                            : "Write your prose for this beat..."}
                          readonly={isLocked}
                          saveStatus={localSaveStatus}
                          onUpdate={handleEditorUpdate(beat.id)}
                        />
                      </div>
                    {:else if beat.prose}
                      <!-- Show preview of prose when collapsed -->
                      <div class="px-4 py-3 border-t border-bg-card">
                        <p
                          class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap line-clamp-3"
                        >
                          {stripHtml(beat.prose)}
                        </p>
                      </div>
                    {/if}
                  </article>
                {/each}
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
        {/if}

        <!-- Scene Prose (Fixed only, if exists and no beats) -->
        {#if (scene.planning_status ?? "fixed") === "fixed" && scene.prose && currentProject.beats.length === 0}
          <section class="mt-8">
            <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide mb-4">
              Content
            </h2>
            <div class="bg-bg-panel rounded-lg p-6">
              <p class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap">
                {scene.prose}
              </p>
            </div>
          </section>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Empty State -->
    <div
      data-testid="empty-state"
      class="flex-1 flex flex-col items-center justify-center text-text-secondary"
    >
      <FileText class="w-16 h-16 mb-4 opacity-50" strokeWidth={1.5} />
      <p class="text-lg">Select a scene to start writing</p>
      <p class="text-sm mt-1">Choose a scene from the sidebar to view its content</p>
    </div>
  {/if}
</div>

{#if beatContextMenu}
  <ContextMenu
    items={getBeatContextMenuItems(beatContextMenu.beat)}
    x={beatContextMenu.x}
    y={beatContextMenu.y}
    onClose={closeBeatContextMenu}
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
