<script lang="ts">
  import {
    FileText,
    ChevronDown,
    Loader2,
    Plus,
    Pencil,
    Lock,
    CircleDot,
    CircleDashed,
    Lightbulb,
    Info,
    X,
    LayoutGrid,
    AlignLeft,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import { onDestroy, onMount } from "svelte";
  import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type {
    Beat,
    DiscoveryNote,
    EditorMode,
    PlanningStatus,
    ReferenceItem,
    ReferenceTypeId,
    Scene,
    SceneStatus,
    SceneType,
    Tag,
  } from "../types";
  import { proseSaves, type ProseSave } from "../utils/proseSaves";
  import type { ProseReplacement } from "../utils/proseSearch";
  import BeatView from "./BeatView.svelte";
  import PageView from "./PageView.svelte";
  import SluglineInput from "./SluglineInput.svelte";
  import TagSelector from "./TagSelector.svelte";
  import Tooltip from "./Tooltip.svelte";

  const isScreenplay = $derived(currentProject.value?.project_type === "screenplay");

  function countWordsInHtml(html: string | null | undefined): number {
    if (!html) return 0;
    const text = html
      .replace(/<[^>]*>/g, " ")
      .replace(/\s+/g, " ")
      .trim();
    return text ? text.split(/\s+/).length : 0;
  }

  const scenePageEstimate = $derived.by(() => {
    if (!isScreenplay || !currentProject.currentScene) return null;
    const scene = currentProject.currentScene;
    let words = countWordsInHtml(scene.prose);
    for (const beat of currentProject.beats) {
      words += countWordsInHtml(beat.prose);
    }
    return words / 250;
  });

  // Check if scene is locked (either directly or via parent chapter)
  const isLocked = $derived(
    currentProject.currentScene?.locked || currentProject.currentChapter?.locked || false
  );

  let beatViewRef: ReturnType<typeof BeatView> | undefined = $state();
  let scrollContainerRef: HTMLDivElement | undefined = $state();
  let synopsisSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Synopsis editing state
  let editingSynopsis = $state(false);
  let synopsisText = $state("");
  let synopsisSaving = $state(false);
  let metadataSaving = $state(false);
  let metadataError = $state<string | null>(null);
  let sceneTagIds = $state<string[]>([]);
  let allProjectTags = $state<Tag[]>([]);

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
    (option) => option.id !== "characters" && option.id !== "locations"
  );

  let sceneReferenceItems = $state<Record<ReferenceTypeId, ReferenceItem[]>>(
    {} as Record<ReferenceTypeId, ReferenceItem[]>
  );
  let sceneReferenceLoading = $state(false);
  let sceneReferenceError = $state<string | null>(null);
  let sceneReferenceRequestId = 0;
  let sceneTagsRequestId = 0;

  // Discovery notes state
  let discoveryNotesRequestId = 0;
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

  async function loadSceneTags(sceneId: string) {
    const requestId = ++sceneTagsRequestId;
    const projectId = currentProject.value?.id;
    if (!projectId) return;
    try {
      const [tags, entityTags] = await Promise.all([
        invoke<Tag[]>("get_tags", { projectId }),
        invoke<Tag[]>("get_entity_tags", { entityType: "scene", entityId: sceneId }),
      ]);
      if (requestId !== sceneTagsRequestId) return;
      allProjectTags = tags;
      sceneTagIds = entityTags.map((t) => t.id);
    } catch (e) {
      if (requestId !== sceneTagsRequestId) return;
      console.error("Failed to load scene tags:", e);
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

  let sceneTitleSaveTimeout: ReturnType<typeof setTimeout> | null = null;
  async function saveSceneTitle(scene: Scene, title: string) {
    if (sceneTitleSaveTimeout) clearTimeout(sceneTitleSaveTimeout);
    sceneTitleSaveTimeout = setTimeout(async () => {
      sceneTitleSaveTimeout = null;
      if (title.trim() === scene.title) return;
      try {
        await invoke("rename_scene", { sceneId: scene.id, title: title.trim() });
        currentProject.updateScene(scene.id, { title: title.trim() });
      } catch (e) {
        console.error("Failed to rename scene:", e);
      }
    }, 400);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "d" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      discoveryNotesVisible = !discoveryNotesVisible;
      return;
    }
    if (e.key === "Escape") {
      if (ui.expandedBeatId) {
        beatViewRef?.flushOnSceneChange();
      }
      beatViewRef?.handleEscape();
      if (editingSynopsis) {
        editingSynopsis = false;
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

  let lastSceneId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (lastSceneId && sceneId !== lastSceneId) {
      beatViewRef?.flushOnSceneChange();
      if (scrollContainerRef) scrollContainerRef.scrollTop = 0;
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
      sceneTagIds = [];
      return;
    }

    loadSceneReferenceItems(sceneId);
    loadSceneTags(sceneId);
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
    const requestId = ++discoveryNotesRequestId;
    discoveryNotesLoading = true;
    try {
      const notes = await invoke<DiscoveryNote[]>("get_discovery_notes", { sceneId });
      if (requestId !== discoveryNotesRequestId) return;
      discoveryNotes = notes;
    } catch (e) {
      if (requestId !== discoveryNotesRequestId) return;
      console.error("Failed to load discovery notes:", e);
      discoveryNotes = [];
    } finally {
      if (requestId === discoveryNotesRequestId) {
        discoveryNotesLoading = false;
      }
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

  // Page View state
  let switchingMode = $state(false);
  let pageProseContent = $state("");
  let pageEditorVersion = $state(0);
  let pageProseSaveTimeout: ReturnType<typeof setTimeout> | null = null;
  let pageProseSaveStatus = $state<"idle" | "saving" | "error">("idle");
  let showSwitchToBeatConfirm = $state(false);

  // Sync page prose content when scene changes
  let lastPageViewSceneId: string | null = null;
  let pageProseProjectId = "";
  $effect(() => {
    const scene = currentProject.currentScene;
    if ((scene?.id ?? null) === lastPageViewSceneId) return;
    if (pageProseSaveTimeout) {
      clearTimeout(pageProseSaveTimeout);
      savePageProse();
    }
    pageProseSaveStatus = "idle";
    lastPageViewSceneId = scene?.id ?? null;
    pageProseProjectId = currentProject.value?.id ?? "";
    if (!scene || scene.editor_mode !== "page") return;
    const unsaved = proseSaves
      .draftsForRecovery(pageProseProjectId)
      .find((draft) => draft.kind === "page" && draft.id === scene.id);
    pageProseContent = unsaved?.prose ?? scene.prose ?? "";
  });

  async function switchEditorMode(targetMode: EditorMode) {
    const scene = currentProject.currentScene;
    if (!scene || switchingMode || isLocked) return;

    if (targetMode === "beat" && scene.editor_mode === "page") {
      showSwitchToBeatConfirm = true;
      return;
    }

    await doSwitchMode(targetMode);
  }

  async function doSwitchMode(targetMode: EditorMode) {
    const scene = currentProject.currentScene;
    if (!scene || switchingMode || isLocked) return;

    switchingMode = true;
    try {
      await prepareForSearch();
      if (currentProject.currentScene?.id !== scene.id || isLocked) return;
      beatViewRef?.flushOnSceneChange();

      const updated = await invoke<Scene>("switch_scene_editor_mode", {
        sceneId: scene.id,
        mode: targetMode,
      });
      if (currentProject.currentScene?.id !== scene.id) return;
      currentProject.refreshCurrentScene(updated);

      if (targetMode === "page") {
        lastPageViewSceneId = updated.id;
        pageProseContent = updated.prose ?? "";
      } else if (targetMode === "beat") {
        const freshBeats = await invoke<Beat[]>("get_beats", {
          sceneId: scene.id,
        });
        if (currentProject.currentScene?.id !== scene.id) return;
        currentProject.setBeats(freshBeats);
      }
    } catch (e) {
      console.error("Failed to switch editor mode:", e);
      ui.showError(`Failed to switch editor mode: ${String(e)}`);
    } finally {
      switchingMode = false;
    }
  }

  function handlePageProseUpdate(html: string) {
    pageProseContent = html;
    // The store is the working copy used when navigating back, even while a save is in flight.
    // Failed writes remain recoverable in proseSaves; an older acknowledgement must not replace this text.
    if (currentProject.currentScene?.id === lastPageViewSceneId && lastPageViewSceneId) {
      currentProject.updateScene(lastPageViewSceneId, { prose: html });
    }
    if (pageProseSaveTimeout) clearTimeout(pageProseSaveTimeout);
    pageProseSaveStatus = "idle";
    pageProseSaveTimeout = setTimeout(() => savePageProse(), 500);
  }

  let pageSaveQueue: Promise<void> = Promise.resolve();

  function savePageProse() {
    const sceneId = lastPageViewSceneId;
    const prose = pageProseContent;
    pageProseSaveTimeout = null;
    if (!sceneId) return pageSaveQueue;
    const writing = proseSaves.save({
      projectId: pageProseProjectId,
      kind: "page",
      id: sceneId,
      prose,
    });
    pageSaveQueue = persistPageProse(sceneId, prose, pageProseProjectId, writing);
    return pageSaveQueue;
  }

  export async function prepareForSearch() {
    await beatViewRef?.flushForSearch();
    if (pageProseSaveTimeout) {
      clearTimeout(pageProseSaveTimeout);
      pageProseSaveTimeout = null;
      await savePageProse();
    }
    await pageSaveQueue;
    await proseSaves.flush(currentProject.value?.id ?? "", (save) => {
      if (currentProject.value?.id !== save.projectId) return;
      if (save.kind === "beat") currentProject.updateBeatProse(save.id, save.prose);
      else {
        currentProject.updateScene(save.id, { prose: save.prose });
        if (currentProject.currentScene?.id === save.id) pageProseContent = save.prose;
      }
    });
    pageProseSaveStatus = "idle";
  }

  export async function discardFailedSaves(drafts: ProseSave[]) {
    const projectId = currentProject.value?.id;
    const scene = currentProject.currentScene;
    const draftView = beatViewRef;
    // Reload before discarding: a read failure must leave the recoverable draft intact.
    const scenes = scene
      ? await invoke<Scene[]>("get_scenes", { chapterId: scene.chapter_id })
      : [];
    const freshScene = scenes.find((s) => s.id === scene?.id);
    const beats = freshScene ? await invoke<Beat[]>("get_beats", { sceneId: freshScene.id }) : [];
    if (currentProject.value?.id !== projectId || currentProject.currentScene?.id !== scene?.id) {
      throw new Error("The selected project or scene changed. Reopen Find and Replace.");
    }
    await proseSaves.discard(drafts, () => draftView?.discardFailedDrafts(drafts));
    if (currentProject.value?.id !== projectId || currentProject.currentScene?.id !== scene?.id)
      return;
    if (scene) {
      currentProject.setScenes(scenes);
      currentProject.setBeats(beats);
      currentProject.setCurrentScene(freshScene ?? null);
      if (drafts.some((draft) => draft.kind === "page" && draft.id === scene.id)) {
        if (pageProseSaveTimeout) clearTimeout(pageProseSaveTimeout);
        pageProseSaveTimeout = null;
        pageProseContent = freshScene?.prose ?? "";
        pageEditorVersion += 1;
      }
    }
    pageProseSaveStatus = "idle";
  }

  export function applySearchChanges(changes: Pick<ProseReplacement, "id" | "prose">[]) {
    for (const change of changes) {
      currentProject.updateBeatProse(change.id, change.prose);
      currentProject.updateScene(change.id, { prose: change.prose });
      if (currentProject.currentScene?.id === change.id) pageProseContent = change.prose;
    }
  }

  async function persistPageProse(
    sceneId: string,
    prose: string,
    projectId: string,
    writing: Promise<void>
  ) {
    pageProseSaveStatus = "saving";
    try {
      await writing;
      const newerQueued = proseSaves
        .draftsForRecovery(projectId)
        .some((draft) => draft.id === sceneId && draft.prose !== prose);
      const newerEditor = currentProject.currentScene?.id === sceneId && pageProseContent !== prose;
      if (currentProject.value?.id === projectId && !newerQueued && !newerEditor) {
        currentProject.updateScene(sceneId, { prose });
      }
      if (currentProject.currentScene?.id === sceneId) pageProseSaveStatus = "idle";
    } catch (e) {
      console.error("Failed to save page prose:", e);
      if (currentProject.currentScene?.id === sceneId) pageProseSaveStatus = "error";
    }
  }

  function stripHtmlTags(html: string): string {
    return html.replace(/<[^>]*>/g, "").trim();
  }

  function getPageWordCount(): number {
    if (!pageProseContent) return 0;
    return stripHtmlTags(pageProseContent)
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
  }

  onMount(() => {
    const dnHandler = () => (discoveryNotesVisible = !discoveryNotesVisible);
    window.addEventListener("kindling:toggleDiscoveryNotes", dnHandler);

    const emHandler = () => {
      const scene = currentProject.currentScene;
      if (!scene || scene.planning_status !== "fixed") return;
      switchEditorMode(scene.editor_mode === "page" ? "beat" : "page");
    };
    window.addEventListener("kindling:toggleEditorMode", emHandler);

    return () => {
      window.removeEventListener("kindling:toggleDiscoveryNotes", dnHandler);
      window.removeEventListener("kindling:toggleEditorMode", emHandler);
    };
  });

  onDestroy(() => {
    beatViewRef?.flushOnSceneChange();
    if (pageProseSaveTimeout) {
      clearTimeout(pageProseSaveTimeout);
      savePageProse();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div data-testid="scene-panel" class="flex-1 flex flex-col h-full overflow-hidden">
  {#if currentProject.currentScene}
    {@const scene = currentProject.currentScene}
    <div bind:this={scrollContainerRef} class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-8">
        <!-- Scene Title -->
        <header class="mb-8">
          <div class="flex items-center gap-3 flex-wrap">
            {#if isScreenplay && !isLocked}
              <SluglineInput
                value={scene.title}
                onSave={(slugline) => saveSceneTitle(scene, slugline)}
                locations={currentProject.locations}
                disabled={metadataSaving}
                class="flex-1 min-w-0"
              />
            {:else}
              <h1
                data-testid="scene-title"
                class="text-press-h1 font-heading font-semibold text-press-text"
              >
                {scene.title}
              </h1>
            {/if}
            {#if isLocked}
              <span
                class="flex items-center gap-1 px-2 py-1 bg-press-warning-wash text-press-warning rounded-lg text-press-ui"
              >
                <Lock class="w-4 h-4" />
                Locked
              </span>
            {/if}
          </div>
          {#if currentProject.currentChapter}
            <p class="text-press-muted text-press-ui mt-1">
              {currentProject.currentChapter.title}
            </p>
          {/if}
          {#if isScreenplay && scenePageEstimate !== null}
            <span class="text-press-eyebrow text-press-muted mt-1">
              ~{scenePageEstimate.toFixed(1)} pg
            </span>
          {/if}
          <div class="mt-4 flex flex-wrap gap-4">
            <div class="flex flex-col gap-1">
              <label for="scene-type" class="text-press-eyebrow text-press-muted">Scene type</label>
              <div class="relative">
                <select
                  id="scene-type"
                  value={scene.scene_type ?? "normal"}
                  onchange={(event) => handleSceneTypeChange(event, scene)}
                  class="appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                  disabled={isLocked || metadataSaving}
                >
                  {#each sceneTypeOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
            </div>
            <div class="flex flex-col gap-1">
              <label for="scene-status" class="text-press-eyebrow text-press-muted">Status</label>
              <div class="relative">
                <select
                  id="scene-status"
                  value={scene.scene_status ?? "draft"}
                  onchange={(event) => handleSceneStatusChange(event, scene)}
                  class="appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                  disabled={isLocked || metadataSaving}
                >
                  {#each sceneStatusOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
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
                  class="text-press-eyebrow text-press-muted cursor-help flex items-center gap-1"
                >
                  Planning
                  <Info class="w-3 h-3 text-press-muted" />
                </label>
              </Tooltip>
              <div class="relative">
                <select
                  id="planning-status"
                  value={scene.planning_status ?? "fixed"}
                  onchange={(event) => handleScenePlanningStatusChange(event, scene)}
                  class="appearance-none bg-press-sunken text-press-text text-press-ui border border-press-border rounded-lg pl-3 pr-8 py-2 focus:outline-none focus:border-press-accent focus:ring-1 focus:ring-press-focus cursor-pointer"
                  disabled={isLocked || metadataSaving}
                >
                  {#each planningStatusOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
                <ChevronDown
                  class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-press-muted pointer-events-none"
                />
              </div>
            </div>
          </div>
          {#if (scene.planning_status ?? "fixed") === "fixed"}
            <div class="flex flex-col gap-1">
              <span class="text-press-eyebrow text-press-muted">View</span>
              <div class="flex bg-press-sunken rounded-lg p-0.5">
                <Tooltip text="Beat cards" position="top">
                  <button
                    data-testid="view-beats"
                    onclick={() => switchEditorMode("beat")}
                    disabled={switchingMode || isLocked}
                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-press-ui transition-colors {scene.editor_mode ===
                    'beat'
                      ? 'bg-press-accent text-press-on-accent'
                      : 'text-press-muted hover:text-press-text'}"
                  >
                    <LayoutGrid class="w-3.5 h-3.5" />
                    Beats
                  </button>
                </Tooltip>
                <Tooltip text="Full page prose" position="top">
                  <button
                    data-testid="view-page"
                    onclick={() => switchEditorMode("page")}
                    disabled={switchingMode || isLocked}
                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-press-ui transition-colors {scene.editor_mode ===
                    'page'
                      ? 'bg-press-accent text-press-on-accent'
                      : 'text-press-muted hover:text-press-text'}"
                  >
                    <AlignLeft class="w-3.5 h-3.5" />
                    Page
                  </button>
                </Tooltip>
              </div>
            </div>
          {/if}
          {#if metadataError}
            <p class="text-press-eyebrow text-press-error mt-2">{metadataError}</p>
          {/if}
        </header>

        <!-- Scene tags -->
        {#if currentProject.value}
          <div class="mb-4 flex items-center gap-2">
            <span class="text-press-eyebrow text-press-muted shrink-0">Tags</span>
            <TagSelector
              projectId={currentProject.value.id}
              entityType="scene"
              entityId={scene.id}
              allTags={allProjectTags}
              entityTagIds={sceneTagIds}
              onTagsChanged={() => loadSceneTags(scene.id)}
            />
          </div>
        {/if}

        <!-- Planning status guidance (first-time, shown once on any scene) -->
        {#if !ui.hasSeenTooltip("planningStatus")}
          <div class="mb-6 px-4 py-3 bg-press-accent-wash border border-press-accent rounded-lg">
            <div class="flex items-start gap-2.5">
              <Lightbulb class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2">
                  <p class="text-press-ui font-medium text-press-text">Rolling outline</p>
                  <button
                    onclick={() => ui.markTooltipSeen("planningStatus")}
                    class="p-0.5 text-press-muted hover:text-press-text rounded transition-colors shrink-0"
                    aria-label="Dismiss"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>
                <p class="text-press-eyebrow text-press-muted leading-relaxed mt-1 mb-2.5">
                  The <strong class="text-press-text">Planning</strong> dropdown above controls how much
                  structure this scene has. Use it to work through your story gradually:
                </p>
                <div class="grid grid-cols-3 gap-3">
                  <div class="text-press-eyebrow">
                    <span class="font-medium text-press-muted flex items-center gap-1"
                      ><CircleDashed class="w-3 h-3" /> Undefined</span
                    >
                    <p class="text-press-muted mt-0.5">
                      A placeholder — you know it exists but haven't planned it.
                    </p>
                  </div>
                  <div class="text-press-eyebrow">
                    <span class="font-medium text-press-warning flex items-center gap-1"
                      ><CircleDot class="w-3 h-3" /> Flexible</span
                    >
                    <p class="text-press-muted mt-0.5">
                      You have the gist — a synopsis and rough direction.
                    </p>
                  </div>
                  <div class="text-press-eyebrow">
                    <span class="font-medium text-press-text flex items-center gap-1">Fixed</span>
                    <p class="text-press-muted mt-0.5">
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
          <div
            class="mb-8 p-6 bg-press-surface rounded-lg border border-dashed border-press-border"
          >
            <div class="flex items-start gap-3">
              <div
                class="w-8 h-8 rounded-full bg-press-border flex items-center justify-center shrink-0 mt-0.5"
              >
                <CircleDashed class="w-4 h-4 text-press-muted" />
              </div>
              <div>
                <h3 class="text-press-ui font-medium text-press-text mb-1">Undefined scene</h3>
                <p class="text-press-muted text-press-ui mb-1">
                  This is a placeholder — you know it exists but haven't planned it yet.
                </p>
                <p class="text-press-muted text-press-eyebrow mb-3">
                  Add a synopsis above to capture the gist, then promote it when you're ready to
                  flesh it out.
                </p>
                {#if !isLocked}
                  <div class="flex items-center gap-2">
                    <button
                      onclick={() => setScenePlanningStatus(scene, "flexible")}
                      class="px-3 py-1.5 rounded-md bg-press-accent-wash text-press-accent-text text-press-ui font-medium hover:text-press-text transition-colors"
                    >
                      Switch to Flexible
                    </button>
                    <button
                      onclick={() => setScenePlanningStatus(scene, "fixed")}
                      class="px-3 py-1.5 rounded-md text-press-muted text-press-ui hover:text-press-text hover:bg-press-sunken transition-colors"
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
          <div
            class="mb-8 p-6 bg-press-surface rounded-lg border border-dashed border-press-border"
          >
            <div class="flex items-start gap-3">
              <div
                class="w-8 h-8 rounded-full bg-press-warning-wash flex items-center justify-center shrink-0 mt-0.5"
              >
                <CircleDot class="w-4 h-4 text-press-warning" />
              </div>
              <div>
                <h3 class="text-press-ui font-medium text-press-text mb-1">Flexible scene</h3>
                <p class="text-press-muted text-press-ui mb-1">
                  You have an idea for this scene but haven't locked down the structure.
                </p>
                <p class="text-press-muted text-press-eyebrow mb-3">
                  Use the synopsis to capture your intent. When you're ready to break it into beats,
                  switch to Fixed.
                </p>
                {#if !isLocked}
                  <button
                    onclick={() => setScenePlanningStatus(scene, "fixed")}
                    class="px-3 py-1.5 rounded-md bg-press-accent-wash text-press-accent-text text-press-ui font-medium hover:text-press-text transition-colors"
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
          <div class="mb-8 px-4 py-3 bg-press-warning-wash border border-press-warning rounded-lg">
            <div class="flex items-center gap-2 text-press-warning">
              <Lock class="w-4 h-4" />
              <span class="font-medium">This scene is locked</span>
            </div>
            <p class="text-press-muted text-press-ui mt-1">
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
            <h2 class="text-press-ui font-semibold text-press-text uppercase tracking-wide">
              Synopsis
            </h2>
            {#if scene.synopsis && !editingSynopsis && !isLocked}
              <Tooltip text="Edit synopsis" position="left">
                <button
                  onclick={startEditingSynopsis}
                  class="text-press-muted hover:text-press-text transition-colors p-1"
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
                class="w-full min-h-[100px] bg-press-sunken rounded-lg p-4 text-press-text font-prose italic leading-relaxed resize-y border border-press-accent focus:outline-none"
                placeholder="Write a brief synopsis for this scene..."
                bind:value={synopsisText}
                oninput={(e) => handleSynopsisInput(e.currentTarget.value)}
              ></textarea>
              {#if synopsisSaving}
                <div class="absolute bottom-3 right-3 flex items-center gap-1.5 text-press-muted">
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  <span class="text-press-eyebrow">Saving...</span>
                </div>
              {/if}
            </div>
            <p class="text-press-muted text-press-eyebrow mt-2">
              Press Escape to close. Changes are saved automatically.
            </p>
          {:else if scene.synopsis}
            <div class="bg-press-surface rounded-lg p-4 border-l-2 border-press-accent">
              <p class="text-press-text font-prose italic">
                {scene.synopsis}
              </p>
            </div>
          {:else if !isLocked}
            <button
              onclick={startEditingSynopsis}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-press-border text-press-muted hover:text-press-text hover:border-press-accent transition-colors"
            >
              <Plus class="w-4 h-4" />
              <span class="text-press-ui">Add Synopsis</span>
            </button>
          {:else}
            <div
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-press-border text-press-muted"
            >
              <Lock class="w-4 h-4" />
              <span class="text-press-ui">Scene is locked</span>
            </div>
          {/if}
        </section>

        <!-- References (Fixed only) -->
        {#if (scene.planning_status ?? "fixed") === "fixed"}
          <section class="mb-8">
            <div class="flex items-center justify-between mb-2">
              <h2 class="text-press-ui font-semibold text-press-text uppercase tracking-wide">
                References
              </h2>
              {#if sceneReferenceLoading}
                <span class="text-press-eyebrow text-press-muted">Loading…</span>
              {/if}
            </div>
            {#if sceneReferenceError}
              <p class="text-press-eyebrow text-press-error">{sceneReferenceError}</p>
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
                        <div class="flex items-center gap-2 text-press-eyebrow text-press-muted">
                          <Icon class={`w-3.5 h-3.5 ${option.accentClass}`} />
                          <span class="font-medium">{option.label}</span>
                        </div>
                        <div class="mt-2 flex flex-wrap gap-2">
                          {#each items as item (item.id)}
                            <span
                              class="px-2 py-1 rounded-md bg-press-surface text-press-text text-press-eyebrow"
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
                <div class="text-press-ui text-press-muted">No linked reference notes.</div>
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
                class="text-press-ui font-semibold text-press-text uppercase tracking-wide group-hover:text-press-text transition-colors"
              >
                Discovery Notes
              </h2>
              <span class="text-press-eyebrow text-press-muted">
                {discoveryNotesVisible ? "Hide" : "Show"} (⌘D)
              </span>
            </button>
            {#if discoveryNotesVisible}
              {#if discoveryNotesLoading}
                <p class="text-press-ui text-press-muted">Loading…</p>
              {:else}
                <div class="space-y-3">
                  {#if !addingDiscoveryNote && !isLocked}
                    <button
                      type="button"
                      onclick={startAddingDiscoveryNote}
                      class="flex items-center gap-1 text-press-muted hover:text-press-text transition-colors text-press-ui"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Add note</span>
                    </button>
                  {/if}
                  {#if addingDiscoveryNote}
                    <div class="flex flex-col gap-2 p-3 rounded-lg bg-press-surface">
                      <textarea
                        bind:value={newDiscoveryNoteContent}
                        placeholder="What did you discover?"
                        rows="2"
                        class="w-full px-3 py-2 rounded-md bg-press-sunken text-press-text text-press-ui placeholder:text-press-muted resize-none focus:outline-none focus:ring-2 focus:ring-press-focus"
                      ></textarea>
                      <div class="flex gap-2">
                        <button
                          type="button"
                          onclick={createDiscoveryNote}
                          disabled={!newDiscoveryNoteContent.trim() || creatingDiscoveryNote}
                          class="px-3 py-1.5 rounded-md bg-press-accent text-press-on-accent text-press-ui font-medium"
                        >
                          {creatingDiscoveryNote ? "Adding…" : "Add"}
                        </button>
                        <button
                          type="button"
                          onclick={() => {
                            addingDiscoveryNote = false;
                            newDiscoveryNoteContent = "";
                          }}
                          class="px-3 py-1.5 rounded-md bg-press-sunken text-press-muted text-press-ui hover:text-press-text"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#each discoveryNotes as note}
                    {@const isEditing = editingDiscoveryNoteId === note.id}
                    <div class="p-3 rounded-lg bg-press-surface">
                      {#if isEditing}
                        <textarea
                          bind:value={editingDiscoveryNoteContent}
                          rows="2"
                          class="w-full px-3 py-2 rounded-md bg-press-sunken text-press-text text-press-ui resize-none focus:outline-none focus:ring-2 focus:ring-press-focus mb-2"
                        ></textarea>
                        <div class="flex gap-2">
                          <button
                            type="button"
                            onclick={() =>
                              updateDiscoveryNote(note.id, editingDiscoveryNoteContent)}
                            class="px-3 py-1.5 rounded-md bg-press-accent text-press-on-accent text-press-ui font-medium"
                          >
                            Save
                          </button>
                          <button
                            type="button"
                            onclick={() => {
                              editingDiscoveryNoteId = null;
                              editingDiscoveryNoteContent = "";
                            }}
                            class="px-3 py-1.5 rounded-md bg-press-sunken text-press-muted text-press-ui hover:text-press-text"
                          >
                            Cancel
                          </button>
                        </div>
                      {:else}
                        <p
                          class="font-prose text-press-body text-press-text whitespace-pre-wrap max-w-press-measure"
                        >
                          {note.content}
                        </p>
                        {#if note.tags && note.tags.length > 0}
                          <div class="flex flex-wrap gap-1 mt-2">
                            {#each note.tags as tag}
                              <span
                                class="px-1.5 py-0.5 rounded bg-press-sunken text-press-eyebrow text-press-muted"
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
                            class="text-press-eyebrow text-press-muted hover:text-press-text"
                          >
                            Edit
                          </button>
                          <button
                            type="button"
                            onclick={() => deleteDiscoveryNote(note.id)}
                            class="text-press-eyebrow text-press-muted hover:text-press-error"
                          >
                            Delete
                          </button>
                          <button
                            type="button"
                            onclick={() => promoteNoteToBeat(note)}
                            disabled={promotingNoteId === note.id}
                            class="text-press-eyebrow text-press-muted hover:text-press-accent-text"
                          >
                            {promotingNoteId === note.id ? "Promoting…" : "Promote to beat"}
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/each}
                  {#if discoveryNotes.length === 0 && !addingDiscoveryNote}
                    <p class="text-press-ui text-press-muted">No discovery notes yet.</p>
                  {/if}
                </div>
              {/if}
            {/if}
          </section>
        {/if}

        <!-- Page View (Fixed + Page mode) -->
        {#if (scene.planning_status ?? "fixed") === "fixed" && scene.editor_mode === "page"}
          {#key pageEditorVersion}
            <PageView
              content={pageProseContent}
              readonly={isLocked || switchingMode}
              saveStatus={pageProseSaveStatus}
              wordCount={getPageWordCount()}
              onUpdate={handlePageProseUpdate}
            />
          {/key}
        {/if}

        <!-- Beats (Fixed + Beat mode only) -->
        {#if (scene.planning_status ?? "fixed") === "fixed" && scene.editor_mode !== "page"}
          <BeatView
            bind:this={beatViewRef}
            beats={currentProject.beats}
            isLocked={isLocked || switchingMode}
          />
        {/if}

        <!-- Scene Prose fallback (Fixed + Beat mode only, if exists and no beats) -->
        {#if (scene.planning_status ?? "fixed") === "fixed" && scene.editor_mode !== "page" && scene.prose && currentProject.beats.length === 0}
          <section class="mt-8">
            <h2 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-4">
              Content
            </h2>
            <div class="bg-press-surface rounded-lg p-6">
              <p class="text-press-text font-prose leading-relaxed whitespace-pre-wrap">
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
      class="flex-1 flex flex-col items-center justify-center text-press-muted"
    >
      <FileText class="w-16 h-16 mb-4 opacity-50" strokeWidth={1.5} />
      <p class="text-press-body-lg">Select a scene to start writing</p>
      <p class="text-press-ui mt-1">Choose a scene from the sidebar to view its content</p>
    </div>
  {/if}
</div>

{#if showSwitchToBeatConfirm}
  <ConfirmDialog
    title="Switch to Beat View"
    message="Any changes made in Page View will be synced back to the corresponding beats. Continue?"
    confirmLabel="Switch"
    onConfirm={() => {
      showSwitchToBeatConfirm = false;
      doSwitchMode("beat");
    }}
    onCancel={() => (showSwitchToBeatConfirm = false)}
  />
{/if}
