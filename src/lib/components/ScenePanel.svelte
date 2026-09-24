<script lang="ts">
  import { shortcuts } from "../stores/shortcuts.svelte";
  import { countWordsInHtml } from "../utils/wordCount";
  import { writing } from "../stores/writing.svelte";
  import type { SceneReview } from "../utils/revisions";
  import WritingStatusBar from "./WritingStatusBar.svelte";
  import Previously from "./Previously.svelte";
  import {
    FileText,
    History,
    ChevronRight,
    Loader2,
    Plus,
    Pencil,
    Lock,
    CircleDot,
    CircleDashed,
    Lightbulb,
    Info,
    X,
    List,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import { onDestroy, onMount, untrack } from "svelte";
  import { trackScrollPosition } from "../utils/scrollPosition";
  import { session } from "../stores/session.svelte";
  import { synopsisSaves } from "../stores/synopsisSaves.svelte";
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

  let {
    onOpenEditorial,
  }: {
    onOpenEditorial?: (
      projectId: string,
      sceneId: string,
      cursor?: { sourceId: string; from: number; to: number }
    ) => Promise<void>;
  } = $props();
  let openingRevisions = $state(false);
  let previousSceneVersion = $state(0);
  let previousSceneLoading = $state(true);
  let revisionEditorVersion = $state(0);
  async function openRevisions() {
    const scene = currentProject.currentScene;
    const project = currentProject.value;
    if (!scene || !project) return;
    openingRevisions = true;
    try {
      await prepareForSearch();
      if (proseSaves.draftsForRecovery(project.id).length)
        throw new Error("Save or recover unsaved prose before opening Revisions.");
      if (currentProject.currentScene?.id !== scene.id) return;
      const anchor = window
        .getSelection()
        ?.anchorNode?.parentElement?.closest<HTMLElement>("[data-source-id]");
      const wrapper = anchor ?? document.querySelector<HTMLElement>("[data-source-id]");
      const editorElement = wrapper?.querySelector<HTMLElement>(".tiptap") as
        | (HTMLElement & { editor?: import("@tiptap/core").Editor })
        | null;
      const selection = editorElement?.editor?.state.selection;
      await onOpenEditorial?.(
        project.id,
        scene.id,
        selection && wrapper?.dataset.sourceId
          ? { sourceId: wrapper.dataset.sourceId, from: selection.from, to: selection.to }
          : undefined
      );
    } catch (e) {
      ui.showError(String(e));
    } finally {
      openingRevisions = false;
    }
  }
  export function applyRevision(review: SceneReview) {
    previousSceneVersion++;
    if (currentProject.currentScene?.id !== review.scene_id) return;
    const prose = review.documents.find((d) => d.id === review.scene_id)?.html ?? "";
    currentProject.updateScene(review.scene_id, { prose, editor_mode: review.mode });
    pageProseContent = prose;
    pageViewSceneId = review.mode === "page" ? review.scene_id : null;
    for (const doc of review.documents) {
      if (doc.id !== review.scene_id) currentProject.updateBeatProse(doc.id, doc.html);
    }
    pageEditorVersion++;
    revisionEditorVersion++;
    writing.scheduleRefresh(currentProject.value!.id);
  }

  const isScreenplay = $derived(currentProject.value?.project_type === "screenplay");
  // The breadcrumb names the scene's own chapter: selecting or creating
  // another chapter changes currentChapter before the scene changes.
  const sceneChapter = $derived(
    currentProject.chapters.find((c) => c.id === currentProject.currentScene?.chapter_id) ??
      currentProject.currentChapter
  );

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
  let scrollTracker = $state.raw<ReturnType<typeof trackScrollPosition> | null>(null);

  function trackSceneScroll(
    node: HTMLElement,
    identifiers: { projectId: string; sceneId: string }
  ) {
    let ids = identifiers;
    function create() {
      const { projectId, sceneId } = ids;
      const tracker = trackScrollPosition(node, (position) => {
        session.update(projectId, sceneId, { scroll_position: position });
      });
      tracker.set(0);
      scrollTracker = tracker;
      return tracker;
    }
    let tracker = create();
    return {
      update(next: typeof identifiers) {
        if (next.projectId === ids.projectId && next.sceneId === ids.sceneId) return;
        tracker.destroy();
        ids = next;
        tracker = create();
      },
      destroy() {
        tracker.destroy();
        if (scrollTracker === tracker) scrollTracker = null;
      },
    };
  }

  // Synopsis editing state
  let editingSynopsis = $state(false);
  let synopsisText = $state("");
  const synopsisSave = $derived(
    synopsisSaves.getState(currentProject.value?.id, currentProject.currentScene?.id)
  );
  const synopsis = $derived(
    synopsisSave.draft ? synopsisSave.draft.synopsis : currentProject.currentScene?.synopsis
  );
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
    if (e.key === "Escape") {
      if (ui.expandedBeatId) {
        beatViewRef?.flushOnSceneChange();
      }
      beatViewRef?.handleEscape();
      if (editingSynopsis) {
        flushSynopsisSave();
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
    synopsisText = synopsis || "";
    editingSynopsis = true;
  }

  function flushSynopsisSave() {
    if (synopsisProjectId && synopsisSceneId) {
      void synopsisSaves.flush(synopsisProjectId, synopsisSceneId).catch(() => {});
    }
  }

  function handleSynopsisInput(value: string) {
    synopsisText = value;
    const sceneId = currentProject.currentScene?.id;
    const projectId = currentProject.value?.id;
    if (!sceneId || !projectId) return;
    synopsisSaves.stage({ projectId, sceneId, synopsis: value.trim() || null });
  }

  let synopsisProjectId: string | undefined;
  let synopsisSceneId: string | undefined;
  $effect(() => {
    // Only navigation should close the editor; autosave updates the same scene object.
    const projectId = currentProject.value?.id;
    const sceneId = currentProject.currentScene?.id;
    if (projectId === synopsisProjectId && sceneId === synopsisSceneId) return;
    untrack(() => {
      flushSynopsisSave();
      editingSynopsis = false;
      synopsisText = "";
    });
    synopsisProjectId = projectId;
    synopsisSceneId = sceneId;
  });

  let lastSceneId: string | null = null;
  let completedViewport: typeof session.viewport = null;
  // A cancelled navigation publishes no request. This effect owns only the viewport it applies.
  $effect(() => {
    const request = session.viewport;
    const tracker = scrollTracker;
    if (
      !request ||
      !tracker ||
      request === completedViewport ||
      request.projectId !== currentProject.value?.id ||
      request.sceneId !== currentProject.currentScene?.id ||
      discoveryNotesLoading ||
      sceneReferenceLoading ||
      previousSceneLoading
    )
      return;
    return untrack(() =>
      tracker.restore(request.position, () => {
        completedViewport = request;
      })
    );
  });

  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (lastSceneId && sceneId !== lastSceneId) {
      beatViewRef?.flushOnSceneChange();
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
  // The page-mode scene whose prose pageProseContent currently holds.
  let pageViewSceneId = $state<string | null>(null);
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
    if (!scene || scene.editor_mode !== "page") {
      pageViewSceneId = null;
      return;
    }
    pageProseContent = initialPageProse(scene);
    pageViewSceneId = scene.id;
  });

  function initialPageProse(scene: Scene) {
    const unsaved = proseSaves
      .draftsForRecovery(currentProject.value?.id ?? "")
      .find((draft) => draft.kind === "page" && draft.id === scene.id);
    return unsaved?.prose ?? scene.prose ?? "";
  }

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
    if (!scene || switchingMode || isLocked || scene.editor_mode === targetMode) return;

    switchingMode = true;
    try {
      await prepareForSearch();
      if (
        proseSaves
          .draftsForRecovery(currentProject.value?.id ?? "")
          .some((draft) =>
            draft.kind === "page"
              ? draft.id === scene.id
              : currentProject.beats.some(
                  (beat) => beat.scene_id === scene.id && beat.id === draft.id
                )
          )
      ) {
        throw new Error(
          "Save, recover, or discard unsaved prose in Find and Replace before switching editor modes."
        );
      }
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
        pageViewSceneId = updated.id;
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

  // Called atomically with queue discard when the user approves quitting without saving.
  export function discardProseDraftsForClose(drafts: ProseSave[]) {
    beatViewRef?.discardFailedDrafts(drafts);
    if (
      drafts.some(
        (draft) =>
          draft.kind === "page" &&
          draft.projectId === pageProseProjectId &&
          draft.id === lastPageViewSceneId &&
          draft.prose === pageProseContent
      )
    ) {
      if (pageProseSaveTimeout) clearTimeout(pageProseSaveTimeout);
      pageProseSaveTimeout = null;
    }
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
    previousSceneVersion++;
  }

  export function applySearchChanges(changes: Pick<ProseReplacement, "id" | "prose">[]) {
    previousSceneVersion++;
    void writing.refresh(currentProject.value?.id);
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

  function getPageWordCount(): number {
    return countWordsInHtml(pageProseContent);
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
    flushSynopsisSave();
    beatViewRef?.flushOnSceneChange();
    if (pageProseSaveTimeout) {
      clearTimeout(pageProseSaveTimeout);
      savePageProse();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div data-testid="scene-panel" class="scene">
  {#if currentProject.currentScene && currentProject.value}
    {@const scene = currentProject.currentScene}
    {@const projectId = currentProject.value.id}
    {@const planning = scene.planning_status ?? "fixed"}
    <div class="scene-top">
      <button
        type="button"
        disabled={openingRevisions}
        aria-busy={openingRevisions || undefined}
        onclick={openRevisions}
        class="ka-button ka-button--ghost"
      >
        <History class="w-5 h-5" aria-hidden="true" />
        {openingRevisions ? "Opening revisions…" : "Revisions"}
      </button>
      {#if planning === "fixed"}
        <div class="ka-segment-track scene-view" role="group" aria-label="View">
          <button
            type="button"
            data-testid="view-beats"
            onclick={() => switchEditorMode("beat")}
            disabled={switchingMode || isLocked}
            class="ka-segment"
            class:ka-selected={scene.editor_mode === "beat"}
            aria-pressed={scene.editor_mode === "beat"}
            title="Beat cards"
          >
            <List class="w-5 h-5" aria-hidden="true" />
            Beats
          </button>
          <button
            type="button"
            data-testid="view-page"
            onclick={() => switchEditorMode("page")}
            disabled={switchingMode || isLocked}
            class="ka-segment"
            class:ka-selected={scene.editor_mode === "page"}
            aria-pressed={scene.editor_mode === "page"}
            title="Full page prose"
          >
            <FileText class="w-5 h-5" aria-hidden="true" />
            Page
          </button>
        </div>
      {/if}
    </div>
    <div use:trackSceneScroll={{ projectId, sceneId: scene.id }} class="scene-scroll">
      <div class="scene-col">
        <Previously refreshVersion={previousSceneVersion} bind:loading={previousSceneLoading} />
        <!-- Scene Title -->
        <header class="scene-header">
          <p class="scene-eyebrow">
            {#if sceneChapter}
              <span>{isScreenplay ? "Sequence" : "Chapter"} · {sceneChapter.title}</span>
            {/if}
            {#if isScreenplay && scenePageEstimate !== null}
              <span>~{scenePageEstimate.toFixed(1)} pages</span>
            {/if}
            {#if isLocked}
              <span class="ka-badge ka-badge--warning scene-locked">
                <Lock class="w-4 h-4" aria-hidden="true" />
                Locked
              </span>
            {/if}
          </p>
          {#if isScreenplay && !isLocked}
            <SluglineInput
              value={scene.title}
              onSave={(slugline) => saveSceneTitle(scene, slugline)}
              locations={currentProject.locations}
              disabled={metadataSaving}
              class="flex-1 min-w-0"
            />
          {:else}
            <h1 data-testid="scene-title" class="scene-title">{scene.title}</h1>
          {/if}
          <div class="scene-meta">
            <div class="ka-field od-field">
              <label for="scene-type">Scene type</label>
              <select
                id="scene-type"
                value={scene.scene_type ?? "normal"}
                onchange={(event) => handleSceneTypeChange(event, scene)}
                disabled={isLocked || metadataSaving}
              >
                {#each sceneTypeOptions as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
            <div class="ka-field od-field">
              <label for="scene-status">Status</label>
              <select
                id="scene-status"
                value={scene.scene_status ?? "draft"}
                onchange={(event) => handleSceneStatusChange(event, scene)}
                disabled={isLocked || metadataSaving}
              >
                {#each sceneStatusOptions as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
            <div class="ka-field od-field">
              <span class="scene-label-tip">
                <label for="planning-status">Planning</label>
                <Tooltip
                  text="Controls how much structure this scene has: Undefined → Flexible → Fixed"
                  position="top"
                >
                  <button
                    type="button"
                    class="scene-tip"
                    aria-label="About planning"
                    aria-describedby="planning-status-help"
                  >
                    <Info class="w-4 h-4" aria-hidden="true" />
                  </button>
                </Tooltip>
              </span>
              <select
                id="planning-status"
                value={planning}
                onchange={(event) => handleScenePlanningStatusChange(event, scene)}
                disabled={isLocked || metadataSaving}
                aria-describedby="planning-status-help"
              >
                {#each planningStatusOptions as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
              <span id="planning-status-help" class="ka-sr"
                >Controls how much structure this scene has: Undefined, Flexible or Fixed.</span
              >
            </div>
          </div>
          {#if metadataError}
            <p class="ka-error scene-meta-error" role="alert">{metadataError}</p>
          {/if}

          <!-- Scene tags -->
          {#if currentProject.value}
            <div class="scene-tags">
              <span class="scene-tags-label">Tags</span>
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
        </header>

        <!-- Planning status guidance (first-time, shown once on any scene) -->
        {#if !ui.hasSeenTooltip("planningStatus")}
          <aside class="scene-guide" aria-labelledby="planning-guide-title">
            <div class="scene-guide-head">
              <Lightbulb class="w-5 h-5" aria-hidden="true" />
              <h3 id="planning-guide-title">Rolling outline</h3>
              <button
                type="button"
                onclick={() => ui.markTooltipSeen("planningStatus")}
                class="ka-button ka-button--ghost ka-icon-button"
                aria-label="Dismiss"
                title="Dismiss"
              >
                <X class="w-5 h-5" aria-hidden="true" />
              </button>
            </div>
            <p class="scene-guide-lede">
              The <strong>Planning</strong> menu above controls how much structure this scene has. Use
              it to work through your story gradually:
            </p>
            <dl class="scene-guide-steps">
              <div>
                <dt><CircleDashed class="w-4 h-4" aria-hidden="true" /> Undefined</dt>
                <dd>A placeholder — you know it exists but haven’t planned it.</dd>
              </div>
              <div>
                <dt><CircleDot class="w-4 h-4 is-warning" aria-hidden="true" /> Flexible</dt>
                <dd>You have the gist — a synopsis and rough direction.</dd>
              </div>
              <div>
                <dt>Fixed</dt>
                <dd>Full structure with beats, references and notes.</dd>
              </div>
            </dl>
          </aside>
        {/if}

        <!-- Undefined / Flexible: what this planning level means and how to move on -->
        {#if planning === "undefined"}
          <section class="scene-planning" aria-labelledby="planning-undefined-title">
            <CircleDashed class="w-5 h-5 scene-planning-icon" aria-hidden="true" />
            <div class="scene-planning-body">
              <h3 id="planning-undefined-title">Undefined scene</h3>
              <p>This is a placeholder — you know it exists but haven’t planned it yet.</p>
              <p class="ka-help">
                Add a synopsis below to capture the gist, then promote it when you’re ready to flesh
                it out.
              </p>
              {#if !isLocked}
                <div class="scene-planning-actions">
                  <button
                    type="button"
                    onclick={() => setScenePlanningStatus(scene, "flexible")}
                    class="ka-button ka-button--secondary"
                  >
                    Switch to Flexible
                  </button>
                  <button
                    type="button"
                    onclick={() => setScenePlanningStatus(scene, "fixed")}
                    class="ka-button ka-button--ghost"
                  >
                    Go straight to Fixed
                  </button>
                </div>
              {/if}
            </div>
          </section>
        {:else if planning === "flexible"}
          <section class="scene-planning" aria-labelledby="planning-flexible-title">
            <CircleDot class="w-5 h-5 scene-planning-icon is-warning" aria-hidden="true" />
            <div class="scene-planning-body">
              <h3 id="planning-flexible-title">Flexible scene</h3>
              <p>You have an idea for this scene but haven’t locked down the structure.</p>
              <p class="ka-help">
                Use the synopsis to capture your intent. When you’re ready to break it into beats,
                switch to Fixed.
              </p>
              {#if !isLocked}
                <div class="scene-planning-actions">
                  <button
                    type="button"
                    onclick={() => setScenePlanningStatus(scene, "fixed")}
                    class="ka-button ka-button--secondary"
                  >
                    Define beats
                  </button>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Locked Banner -->
        {#if isLocked}
          <div class="ka-notice ka-notice--warning od-row-top scene-locked-notice" role="status">
            <Lock class="w-5 h-5 shrink-0" aria-hidden="true" />
            <div class="od-field od-fill">
              <strong>This scene is locked</strong>
              <p>
                {#if currentProject.currentChapter?.locked}
                  The parent chapter is locked. Unlock the chapter to edit this scene.
                {:else}
                  Unlock this scene from the sidebar to make changes.
                {/if}
              </p>
            </div>
          </div>
        {/if}

        <!-- Synopsis (shown for all planning statuses) -->
        <section class="scene-section" aria-labelledby="synopsis-title">
          <div class="scene-section-head">
            <h3 id="synopsis-title">Synopsis</h3>
            {#if synopsis && !editingSynopsis && !isLocked}
              <button
                type="button"
                onclick={startEditingSynopsis}
                class="ka-button ka-button--ghost ka-icon-button"
                aria-label="Edit synopsis"
                title="Edit synopsis"
              >
                <Pencil class="w-5 h-5" aria-hidden="true" />
              </button>
            {/if}
          </div>
          {#if editingSynopsis && !isLocked}
            <div class="ka-field od-field scene-synopsis-field">
              <label for="scene-synopsis" class="ka-sr">Synopsis</label>
              <textarea
                id="scene-synopsis"
                placeholder="Write a brief synopsis for this scene…"
                bind:value={synopsisText}
                oninput={(e) => handleSynopsisInput(e.currentTarget.value)}
                aria-describedby="scene-synopsis-help"
              ></textarea>
              <p id="scene-synopsis-help" class="ka-help scene-synopsis-help">
                {#if synopsisSave.saving}
                  <Loader2 class="w-4 h-4 animate-spin" aria-hidden="true" />
                  Saving…
                {:else}
                  Press Escape to close. Changes are saved automatically.
                {/if}
              </p>
            </div>
          {:else if synopsis}
            <p class="scene-synopsis">{synopsis}</p>
          {:else if !isLocked}
            <button
              type="button"
              onclick={startEditingSynopsis}
              class="ka-button ka-button--secondary"
            >
              <Plus class="w-5 h-5" aria-hidden="true" />
              Add synopsis
            </button>
          {:else}
            <p class="ka-help scene-quiet">
              <Lock class="w-4 h-4" aria-hidden="true" /> Scene is locked
            </p>
          {/if}
          {#if synopsisSave.error}
            <div role="alert" class="ka-notice ka-notice--error scene-inline-notice">
              <p>Synopsis not saved: {synopsisSave.error}. Your draft is kept for retry.</p>
              <button
                type="button"
                onclick={flushSynopsisSave}
                disabled={synopsisSave.saving}
                class="ka-button ka-button--secondary"
                aria-label="Retry synopsis save">Retry saving</button
              >
            </div>
          {:else if synopsisSave.draft && !synopsisSave.saving}
            <p role="status" class="ka-help">Unsaved changes</p>
          {/if}
        </section>

        <!-- References (Fixed only) -->
        {#if planning === "fixed"}
          <section class="scene-section" aria-labelledby="scene-references-title">
            <div class="scene-section-head">
              <h3 id="scene-references-title">References</h3>
              {#if sceneReferenceLoading}
                <span class="ka-help" role="status">Loading…</span>
              {/if}
            </div>
            {#if sceneReferenceError}
              <p class="ka-error" role="alert">{sceneReferenceError}</p>
            {:else}
              {@const hasSceneReferences = sceneReferenceOptions.some(
                (option) => (sceneReferenceItems[option.id]?.length ?? 0) > 0
              )}
              {#if hasSceneReferences}
                <dl class="scene-refs">
                  {#each sceneReferenceOptions as option (option.id)}
                    {@const items = sceneReferenceItems[option.id] ?? []}
                    {#if items.length > 0}
                      {@const Icon = option.icon}
                      <div>
                        <dt><Icon class="w-4 h-4" aria-hidden="true" /> {option.label}</dt>
                        <dd>
                          {#each items as item (item.id)}
                            <span class="ka-badge">{item.name}</span>
                          {/each}
                        </dd>
                      </div>
                    {/if}
                  {/each}
                </dl>
              {:else if !sceneReferenceLoading}
                <p class="ka-help">
                  No items or other references linked yet. Characters and locations are in the
                  references panel.
                </p>
              {/if}
            {/if}
          </section>
        {/if}

        <!-- Discovery Notes (Fixed only) -->
        {#if planning === "fixed"}
          <section class="scene-section" aria-labelledby="discovery-notes-title">
            <button
              type="button"
              onclick={() => (discoveryNotesVisible = !discoveryNotesVisible)}
              class="scene-disclose"
              aria-expanded={discoveryNotesVisible}
              aria-controls="discovery-notes-body"
            >
              <ChevronRight class="w-5 h-5 scene-chev" aria-hidden="true" />
              <h3 id="discovery-notes-title">Discovery notes</h3>
              <span class="scene-disclose-hint">
                {discoveryNotesVisible ? "Hide" : "Show"}
                <kbd>{shortcuts.label("toggle_discovery_notes")}</kbd>
              </span>
            </button>
            {#if discoveryNotesVisible}
              <div id="discovery-notes-body" class="scene-notes">
                {#if discoveryNotesLoading}
                  <p class="ka-help" role="status">Loading…</p>
                {:else}
                  {#if addingDiscoveryNote}
                    <div class="ka-field od-field scene-note-form">
                      <label for="new-discovery-note" class="ka-sr">New discovery note</label>
                      <textarea
                        id="new-discovery-note"
                        bind:value={newDiscoveryNoteContent}
                        placeholder="What did you discover?"
                        rows="3"
                      ></textarea>
                      <div class="ka-row">
                        <button
                          type="button"
                          onclick={createDiscoveryNote}
                          disabled={!newDiscoveryNoteContent.trim() || creatingDiscoveryNote}
                          aria-busy={creatingDiscoveryNote || undefined}
                          class="ka-button"
                        >
                          {creatingDiscoveryNote ? "Adding…" : "Add note"}
                        </button>
                        <button
                          type="button"
                          onclick={() => {
                            addingDiscoveryNote = false;
                            newDiscoveryNoteContent = "";
                          }}
                          class="ka-button ka-button--ghost"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#if discoveryNotes.length > 0}
                    <ul class="scene-note-list">
                      {#each discoveryNotes as note}
                        {@const isEditing = editingDiscoveryNoteId === note.id}
                        <li class="scene-note">
                          {#if isEditing}
                            <div class="ka-field od-field">
                              <label for={`note-${note.id}`} class="ka-sr"
                                >Edit discovery note</label
                              >
                              <textarea
                                id={`note-${note.id}`}
                                bind:value={editingDiscoveryNoteContent}
                                rows="3"
                              ></textarea>
                            </div>
                            <div class="ka-row">
                              <button
                                type="button"
                                onclick={() =>
                                  updateDiscoveryNote(note.id, editingDiscoveryNoteContent)}
                                class="ka-button"
                              >
                                Save
                              </button>
                              <button
                                type="button"
                                onclick={() => {
                                  editingDiscoveryNoteId = null;
                                  editingDiscoveryNoteContent = "";
                                }}
                                class="ka-button ka-button--ghost"
                              >
                                Cancel
                              </button>
                            </div>
                          {:else}
                            <p class="scene-note-text">{note.content}</p>
                            {#if note.tags && note.tags.length > 0}
                              <div class="scene-note-tags">
                                {#each note.tags as tag}
                                  <span class="ka-badge">{tag}</span>
                                {/each}
                              </div>
                            {/if}
                            <div class="scene-note-actions">
                              <button
                                type="button"
                                onclick={() => {
                                  editingDiscoveryNoteId = note.id;
                                  editingDiscoveryNoteContent = note.content;
                                }}
                                class="ka-button ka-button--ghost"
                              >
                                Edit
                              </button>
                              <button
                                type="button"
                                onclick={() => promoteNoteToBeat(note)}
                                disabled={promotingNoteId === note.id}
                                aria-busy={promotingNoteId === note.id || undefined}
                                class="ka-button ka-button--ghost"
                              >
                                {promotingNoteId === note.id ? "Promoting…" : "Promote to beat"}
                              </button>
                              <button
                                type="button"
                                onclick={() => deleteDiscoveryNote(note.id)}
                                class="ka-button ka-button--ghost scene-danger"
                              >
                                Delete
                              </button>
                            </div>
                          {/if}
                        </li>
                      {/each}
                    </ul>
                  {:else if !addingDiscoveryNote}
                    <p class="ka-help">
                      No discovery notes yet. Capture what you find while drafting — notes are not
                      part of the manuscript.
                    </p>
                  {/if}
                  {#if !addingDiscoveryNote && !isLocked}
                    <button
                      type="button"
                      onclick={startAddingDiscoveryNote}
                      class="ka-button ka-button--secondary scene-add-note"
                    >
                      <Plus class="w-5 h-5" aria-hidden="true" />
                      Add note
                    </button>
                  {/if}
                {/if}
              </div>
            {/if}
          </section>
        {/if}

        <!-- Page View (Fixed + Page mode) -->
        {#if planning === "fixed" && scene.editor_mode === "page"}
          <!-- Keyed per scene so undo history never reaches into the previous scene. Until the
               effect loads this scene, mount with its own prose rather than the last scene's. -->
          {#key `${scene.id}:${pageEditorVersion}`}
            <PageView
              projectId={currentProject.value?.id}
              sceneId={scene.id}
              content={pageViewSceneId === scene.id ? pageProseContent : initialPageProse(scene)}
              readonly={isLocked || switchingMode || openingRevisions}
              saveStatus={pageProseSaveStatus}
              wordCount={getPageWordCount()}
              onUpdate={handlePageProseUpdate}
            />
          {/key}
        {/if}

        <!-- Beats (Fixed + Beat mode only) -->
        {#if planning === "fixed" && scene.editor_mode !== "page"}
          {#key revisionEditorVersion}
            <BeatView
              bind:this={beatViewRef}
              beats={currentProject.beats}
              isLocked={isLocked || switchingMode || openingRevisions}
            />
          {/key}
        {/if}

        <!-- Scene Prose fallback (Fixed + Beat mode only, if exists and no beats) -->
        {#if planning === "fixed" && scene.editor_mode !== "page" && scene.prose && currentProject.beats.length === 0}
          <section class="scene-section" aria-labelledby="scene-content-title">
            <div class="scene-section-head">
              <h3 id="scene-content-title">Content</h3>
            </div>
            <div class="app-prose-sheet scene-content-sheet">
              <p>{scene.prose}</p>
            </div>
          </section>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Empty State -->
    <div data-testid="empty-state" class="scene-empty">
      <div class="ka-empty od-stack">
        <FileText class="w-7 h-7" aria-hidden="true" />
        <h2>Select a scene to start writing</h2>
        <p>Choose a scene from the sidebar to view its content.</p>
      </div>
    </div>
  {/if}
  {#key currentProject.value?.id}
    <WritingStatusBar />
  {/key}
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

<style>
  .scene {
    flex: 1;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--color-bg);
    color: var(--color-text);
  }
  .scene:has(> .scene-empty) {
    grid-template-rows: minmax(0, 1fr) auto;
  }
  .scene-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    padding: var(--space-xs) var(--space-m);
    border-bottom: var(--border-hair);
  }
  .scene-view {
    flex-wrap: nowrap;
  }
  .scene-view .ka-segment {
    flex: none;
    padding: var(--space-2xs) var(--space-s);
  }
  .scene-scroll {
    min-height: 0;
    overflow-y: auto;
  }
  .scene-col {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-l) var(--space-m) var(--space-3xl);
  }

  .scene-header {
    margin-top: var(--space-2xs);
  }
  .scene-eyebrow {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-xs);
    min-height: 24px;
    margin: 0 0 var(--space-2xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .scene-locked {
    gap: var(--space-3xs);
  }
  .scene-title {
    margin: 0;
    font: 550 var(--ka-heading) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .scene-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-s);
    margin-top: var(--space-m);
  }
  .scene-meta .ka-field {
    flex: 1 1 136px;
    width: auto;
    min-width: 0;
    max-width: 200px;
    gap: var(--space-3xs);
  }
  .scene-meta .ka-field label {
    font: 500 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .scene-meta .ka-field select {
    padding: var(--space-2xs) var(--space-xs);
  }
  .scene-label-tip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-3xs);
    font: 500 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  /* A 16px glyph with a 40px hit area: the one inline exception to 44px,
     sitting inside a label row. */
  .scene-tip {
    position: relative;
    display: inline-flex;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: none;
    color: var(--color-text-muted);
    cursor: help;
  }
  .scene-tip::after {
    content: "";
    position: absolute;
    inset: -12px;
  }
  .scene-meta-error {
    margin-top: var(--space-2xs);
  }
  .scene-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2xs);
    margin-top: var(--space-s);
  }
  .scene-tags-label {
    margin-right: var(--space-3xs);
    font: 500 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }

  .scene-guide,
  .scene-planning {
    margin-top: var(--space-m);
    padding: var(--space-s);
    border: var(--border-hair);
    border-radius: var(--radius-m);
    background: var(--color-surface);
  }
  .scene-guide-head {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin: calc(-1 * var(--space-2xs)) calc(-1 * var(--space-2xs)) 0 0;
    color: var(--color-text-muted);
  }
  .scene-guide-head h3,
  .scene-planning-body h3 {
    flex: 1;
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .scene-guide-lede {
    margin: var(--space-2xs) 0 var(--space-s);
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .scene-guide-steps {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-s);
    margin: 0;
    font: var(--text-small) / 1.5 var(--font-ui);
  }
  .scene-guide-steps dt {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    font-weight: 500;
    color: var(--color-text);
  }
  .scene-guide-steps dd {
    margin: var(--space-3xs) 0 0;
    color: var(--color-text-muted);
  }
  .scene :global(.is-warning) {
    color: var(--color-warning);
  }
  .scene-planning {
    display: flex;
    gap: var(--space-xs);
  }
  .scene-planning :global(.scene-planning-icon) {
    flex: none;
    margin-top: 2px;
    color: var(--color-text-muted);
  }
  .scene-planning-body {
    display: grid;
    gap: var(--space-2xs);
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .scene-planning-body p {
    margin: 0;
  }
  .scene-planning-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    margin-top: var(--space-3xs);
  }
  .scene-locked-notice {
    margin-top: var(--space-m);
  }

  .scene-section {
    margin-top: var(--space-m);
    padding-top: var(--space-s);
    border-top: var(--border-hair);
  }
  .scene-section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
    min-height: var(--control-target);
    margin-bottom: var(--space-2xs);
  }
  .scene-section h3 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .scene-synopsis {
    margin: 0;
    max-width: var(--measure);
    font: italic var(--text-body-lg) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
    white-space: pre-wrap;
  }
  .scene-synopsis-field textarea {
    min-height: 120px;
    font: italic var(--text-body-lg) / var(--leading-relaxed) var(--font-body);
  }
  .scene-synopsis-help,
  .scene-quiet {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .scene-inline-notice {
    margin-top: var(--space-2xs);
    justify-items: start;
  }

  .scene-refs {
    display: grid;
    gap: var(--space-xs);
    margin: 0;
  }
  .scene-refs > div {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr);
    gap: var(--space-m);
    align-items: center;
  }
  .scene-refs dt {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .scene-refs dd {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    margin: 0;
  }

  .scene-disclose {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    width: 100%;
    min-height: var(--control-target);
    padding: 0;
    border: 0;
    border-radius: var(--radius-xs);
    background: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }
  .scene-disclose h3 {
    flex: 1;
  }
  .scene-disclose :global(.scene-chev) {
    color: var(--color-text-muted);
    transition: transform var(--ka-motion) ease-out;
  }
  .scene-disclose[aria-expanded="true"] :global(.scene-chev) {
    transform: rotate(90deg);
  }
  .scene-disclose-hint {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .scene-disclose-hint kbd {
    font: var(--text-small) / 1.4 var(--font-mono);
  }
  .scene-notes {
    display: grid;
    gap: var(--space-s);
    margin-top: var(--space-2xs);
  }
  .scene-note-form {
    gap: var(--space-2xs);
  }
  .scene-note-list {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .scene-note {
    display: grid;
    gap: var(--space-2xs);
    padding: var(--space-s) 0 var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .scene-note-text {
    margin: 0;
    max-width: var(--measure);
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
    white-space: pre-wrap;
  }
  .scene-note-tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3xs);
  }
  .scene-note-actions {
    display: flex;
    flex-wrap: wrap;
    margin-left: calc(-1 * var(--space-s));
  }
  .scene-danger:hover {
    color: var(--color-error);
  }
  .scene-add-note {
    justify-self: start;
  }
  .scene-content-sheet {
    padding: var(--space-xl);
  }
  .scene-content-sheet p {
    max-width: var(--measure);
    margin: 0 auto;
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    white-space: pre-wrap;
  }

  .scene-empty {
    display: grid;
    place-items: center;
    min-height: 0;
    padding: var(--space-xl);
  }
  .scene-empty .ka-empty {
    align-items: center;
    text-align: center;
  }
  .scene-empty h2 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    color: var(--color-text);
  }
</style>
