<!--
  Sidebar.svelte - Main navigation sidebar

  Displays the project chapter/scene tree with:
  - Drag-and-drop reordering
  - Context menus for actions
  - Create/delete functionality
  - Sync button for reimporting
-->
<script lang="ts">
  import WritingProgress from "./WritingProgress.svelte";
  import { writing } from "../stores/writing.svelte";
  import { supportsSync } from "../importFormats";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick, untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ChevronDown,
    ChevronRight,
    PanelLeftClose,
    PanelLeftOpen,
    Clock,
    Folder,
    Home,
    Plus,
    Trash2,
    GripVertical,
    RefreshCw,
    Pencil,
    MoreVertical,
    Copy,
    Archive,
    Lock,
    Unlock,
    Download,
    BookOpen,
    StickyNote,
    CheckSquare,
    EyeOff,
    CircleDot,
    CircleDashed,
    Filter,
    Settings,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { session } from "../stores/session.svelte";
  import { ui } from "../stores/ui.svelte";
  import type {
    Beat,
    Chapter,
    PlanningStatus,
    SavedFilter,
    Scene,
    SceneStatus,
    SceneType,
    SyncPreview,
    ReimportSummary,
    ExportResult,
  } from "../types";
  import ArchivePanel from "./ArchivePanel.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import PartDeleteDialog from "./PartDeleteDialog.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import RenameDialog from "./RenameDialog.svelte";
  import SyncDialog from "./SyncDialog.svelte";
  import SyncSummaryDialog from "./SyncSummaryDialog.svelte";
  import ExportDialog from "./ExportDialog.svelte";
  import ExportSuccessDialog from "./ExportSuccessDialog.svelte";
  import SnapshotsPanel from "./SnapshotsPanel.svelte";
  import BrandWordmark from "./BrandWordmark.svelte";

  import type { ComponentType } from "svelte";

  interface MenuItem {
    label: string;
    icon?: ComponentType;
    action: () => void | Promise<void>;
    disabled?: boolean;
    danger?: boolean;
    divider?: boolean;
    children?: MenuItem[];
  }

  let {
    prepareWritingReset,
    beforeCloseProject,
    onOpenSettings,
  }: {
    prepareWritingReset?: () => Promise<void>;
    beforeCloseProject?: () => Promise<void>;
    onOpenSettings?: () => void;
  } = $props();

  let loading = $state(false);
  let chaptersRequestId = 0;
  let scenesRequestId = 0;
  let beatsRequestId = 0;
  let expandedChapters = new SvelteSet<string>();
  // Navigation outside the sidebar (for example search results) must reveal its chapter.
  $effect(() => {
    const chapterId = currentProject.currentChapter?.id;
    untrack(() => {
      if (chapterId) {
        expandedChapters.add(chapterId);
        const group = partGroups.find((group) =>
          group.chapters.some((chapter) => chapter.id === chapterId)
        );
        if (group?.part) expandedParts.add(group.part.id);
      }
    });
  });
  let expandedParts = new SvelteSet<string>();

  // Group chapters under their preceding Parts
  interface PartGroup {
    part: Chapter | null; // null for chapters before first Part
    chapters: Chapter[];
  }

  const partGroups = $derived.by(() => {
    const chapters = currentProject.chapters.filter((c) => !c.archived);
    const groups: PartGroup[] = [];
    let currentGroup: PartGroup = { part: null, chapters: [] };

    for (const chapter of chapters) {
      if (chapter.is_part) {
        // Save previous group if it has content
        if (currentGroup.part !== null || currentGroup.chapters.length > 0) {
          groups.push(currentGroup);
        }
        // Start new group with this Part
        currentGroup = { part: chapter, chapters: [] };
      } else {
        currentGroup.chapters.push(chapter);
      }
    }

    // Push final group
    if (currentGroup.part !== null || currentGroup.chapters.length > 0) {
      groups.push(currentGroup);
    }

    return groups;
  });

  const sceneStatusOptions: { value: SceneStatus | "all"; label: string }[] = [
    { value: "all", label: "All" },
    { value: "draft", label: "Draft" },
    { value: "revised", label: "Revised" },
    { value: "final", label: "Final" },
  ];

  const sceneTypeLabels: Record<SceneType, string> = {
    normal: "Normal",
    notes: "Notes",
    todo: "ToDo",
    unused: "Unused",
  };

  const sceneTypeFilterOptions: {
    type: SceneType;
    label: string;
    icon: ComponentType;
  }[] = [
    { type: "notes", label: "Notes", icon: StickyNote },
    { type: "todo", label: "ToDo", icon: CheckSquare },
    { type: "unused", label: "Unused", icon: EyeOff },
  ];

  const sceneStatusLabels: Record<SceneStatus, string> = {
    draft: "Draft",
    revised: "Revised",
    final: "Final",
  };

  let showNotesScenes = $state(true);
  let showTodoScenes = $state(true);
  let showUnusedScenes = $state(true);
  let sceneStatusFilter = $state<SceneStatus | "all">("all");
  let outlineViewFilter = $state<"all" | "planned_only" | "next_5">("all");

  function isSceneTypeVisible(type: SceneType) {
    if (type === "notes") return showNotesScenes;
    if (type === "todo") return showTodoScenes;
    if (type === "unused") return showUnusedScenes;
    return true;
  }

  function toggleSceneTypeVisible(type: SceneType) {
    if (type === "notes") {
      showNotesScenes = !showNotesScenes;
      return;
    }
    if (type === "todo") {
      showTodoScenes = !showTodoScenes;
      return;
    }
    if (type === "unused") {
      showUnusedScenes = !showUnusedScenes;
    }
  }

  const filteredScenes = $derived.by((): Scene[] => {
    let scenes = currentProject.scenes.filter((scene) => {
      const type = scene.scene_type ?? "normal";
      const status = scene.scene_status ?? "draft";
      const typeAllowed =
        type === "normal" ||
        (type === "notes" && showNotesScenes) ||
        (type === "todo" && showTodoScenes) ||
        (type === "unused" && showUnusedScenes);
      const statusAllowed = sceneStatusFilter === "all" || status === sceneStatusFilter;
      return typeAllowed && statusAllowed;
    });
    // Rolling outline filters
    if (outlineViewFilter === "planned_only") {
      scenes = scenes.filter((s) => (s.planning_status ?? "fixed") !== "undefined");
    } else if (outlineViewFilter === "next_5") {
      const currentIndex = currentProject.currentScene
        ? scenes.findIndex((s) => s.id === currentProject.currentScene!.id)
        : -1;
      const start = currentIndex >= 0 ? currentIndex : 0;
      scenes = scenes.slice(start, start + 5);
    }
    return scenes;
  });

  // Create new content state
  let creatingChapter = $state(false);
  let creatingPart = $state(false);
  let creatingScene = $state(false);
  let newTitle = $state("");

  // Split button dropdown state
  let showNewDropdown = $state(false);
  let newButtonRef: HTMLElement | null = $state(null);

  // Delete confirmation state
  let deleteDialog: {
    type: "chapter" | "scene";
    id: string;
    title: string;
    message: string;
  } | null = $state(null);

  // Part delete dialog state (separate because it has options)
  let partDeleteDialog: {
    partId: string;
    partTitle: string;
    childChapterIds: string[];
  } | null = $state(null);

  // Drag-and-drop state (using pointer events, more reliable than HTML5 drag API in webviews)
  let draggedItem: { type: "chapter" | "scene"; id: string } | null = $state(null);
  let dragOverId: string | null = $state(null);
  let isDragging = $state(false);
  let draggedElement: globalThis.HTMLElement | null = null;
  let currentDragOverElement: globalThis.HTMLElement | null = null;

  // Hover state for showing action buttons

  // Sync state (dialogs are now separate components)
  let loadingSyncPreview = $state(false);
  let showSyncDialog = $state(false);
  let syncPreview: SyncPreview | null = $state(null);
  let syncSummary: ReimportSummary | null = $state(null);

  // Context menu state
  let contextMenu: {
    type: "chapter" | "scene";
    id: string;
    x: number;
    y: number;
    item: Chapter | Scene;
  } | null = $state(null);

  // Rename dialog state
  let renameDialog: {
    type: "chapter" | "scene";
    id: string;
    title: string;
  } | null = $state(null);

  // Archive panel state
  let showArchivePanel = $state(false);

  // Snapshots panel state
  let showSnapshotsPanel = $state(false);

  // Header "more" menu
  let showMoreMenu = $state(false);
  let moreMenuRef: HTMLElement | null = $state(null);

  // Filter popover
  let showFilterPopover = $state(false);
  let filterPopoverRef: HTMLElement | null = $state(null);

  const hasActiveFilters = $derived(
    sceneStatusFilter !== "all" || !showNotesScenes || !showTodoScenes || !showUnusedScenes
  );

  let savedFilters = $state<SavedFilter[]>([]);
  let savedFilterName = $state("");
  let showSaveFilterInput = $state(false);

  async function loadSavedFilters() {
    const projectId = currentProject.value?.id;
    if (!projectId) return;
    try {
      savedFilters = await invoke<SavedFilter[]>("get_saved_filters", { projectId });
    } catch (e) {
      console.error("Failed to load saved filters:", e);
    }
  }

  async function saveCurrentFilter() {
    const projectId = currentProject.value?.id;
    const name = savedFilterName.trim();
    if (!projectId || !name) return;

    const config = {
      sceneStatusFilter,
      showNotesScenes,
      showTodoScenes,
      showUnusedScenes,
    };

    try {
      await invoke("save_filter", {
        projectId,
        name,
        entityType: "scene",
        filterJson: JSON.stringify(config),
      });
      savedFilterName = "";
      showSaveFilterInput = false;
      await loadSavedFilters();
    } catch (e) {
      console.error("Failed to save filter:", e);
    }
  }

  function applySavedFilter(filter: SavedFilter) {
    try {
      const config = JSON.parse(filter.filter_json);
      if (config.sceneStatusFilter) sceneStatusFilter = config.sceneStatusFilter;
      if (config.showNotesScenes !== undefined) showNotesScenes = config.showNotesScenes;
      if (config.showTodoScenes !== undefined) showTodoScenes = config.showTodoScenes;
      if (config.showUnusedScenes !== undefined) showUnusedScenes = config.showUnusedScenes;
    } catch {
      console.error("Failed to parse saved filter:", filter.name);
    }
    showFilterPopover = false;
  }

  async function deleteSavedFilter(filterId: string) {
    try {
      await invoke("delete_saved_filter", { filterId });
      await loadSavedFilters();
    } catch (e) {
      console.error("Failed to delete saved filter:", e);
    }
  }

  // Labels for Part/Chapter vs Act/Sequence (screenplay projects)
  const partLabel = $derived(currentProject.value?.project_type === "screenplay" ? "Act" : "Part");
  const chapterLabel = $derived(
    currentProject.value?.project_type === "screenplay" ? "Sequence" : "Chapter"
  );

  // Chapter synopsis editing
  let editingChapterSynopsisId: string | null = $state(null);
  let chapterSynopsisText = $state("");
  let chapterSynopsisSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  function startEditingChapterSynopsis(chapter: Chapter) {
    editingChapterSynopsisId = chapter.id;
    chapterSynopsisText = chapter.synopsis ?? "";
  }

  function handleChapterSynopsisInput(chapterId: string) {
    if (chapterSynopsisSaveTimeout) clearTimeout(chapterSynopsisSaveTimeout);
    chapterSynopsisSaveTimeout = setTimeout(() => {
      saveChapterSynopsis(chapterId);
    }, 600);
  }

  async function saveChapterSynopsis(chapterId: string) {
    const text = chapterSynopsisText.trim() || null;
    try {
      await invoke("update_chapter_synopsis", {
        chapterId,
        synopsis: text,
      });
      currentProject.updateChapter(chapterId, { synopsis: text });
    } catch (e) {
      console.error("Failed to save chapter synopsis:", e);
    }
  }

  function finishEditingChapterSynopsis(chapterId: string) {
    if (chapterSynopsisSaveTimeout) {
      clearTimeout(chapterSynopsisSaveTimeout);
      chapterSynopsisSaveTimeout = null;
    }
    saveChapterSynopsis(chapterId);
    editingChapterSynopsisId = null;
  }

  // Export dialog state
  let exportDialog: {
    scope: "project" | "chapter" | "scene";
    scopeId: string | null;
    scopeTitle: string;
  } | null = $state(null);

  let exportResult: ExportResult | null = $state(null);

  // Page count estimate for screenplay projects
  let pageCountEstimate: { pages: number; words: number; target: string } | null = $state(null);

  $effect(() => {
    if (!currentProject.value) pageCountEstimate = null;
  });

  async function loadPageCountEstimate() {
    if (!currentProject.value || currentProject.value.project_type !== "screenplay") {
      pageCountEstimate = null;
      return;
    }
    try {
      const result = await invoke<{ pages: number; words: number; target: string }>(
        "get_page_count_estimate",
        { projectId: currentProject.value.id }
      );
      if (currentProject.value?.project_type === "screenplay") {
        pageCountEstimate = result;
      }
    } catch {
      pageCountEstimate = null;
    }
  }

  async function loadChapters(resume = false) {
    if (!currentProject.value) return;
    const projectId = currentProject.value.id;
    const requestId = ++chaptersRequestId;

    loading = true;
    try {
      const [chapters, saved] = await Promise.all([
        invoke<Chapter[]>("get_chapters", { projectId }),
        resume ? session.load(projectId) : Promise.resolve(null),
      ]);
      if (requestId !== chaptersRequestId || currentProject.value?.id !== projectId) return;

      currentProject.setChapters(chapters);

      // Auto-expand all Parts
      expandedParts.clear();
      for (const chapter of chapters) {
        if (chapter.is_part) {
          expandedParts.add(chapter.id);
        }
      }

      const savedChapter = chapters.find((c) => c.id === saved?.current_chapter_id && !c.is_part);
      if (saved && !savedChapter) session.open(projectId);
      const firstChapter = savedChapter ?? chapters.find((c) => !c.is_part);
      if (firstChapter) {
        expandedChapters.clear();
        expandedChapters.add(firstChapter.id);
        await loadScenes(firstChapter, !savedChapter);
        if (requestId !== chaptersRequestId || currentProject.value?.id !== projectId) return;
        if (savedChapter && saved && session.matches(projectId, saved.current_scene_id!)) {
          const scene = currentProject.scenes.find((s) => s.id === saved?.current_scene_id);
          if (scene) {
            await selectScene(scene);
            if (requestId !== chaptersRequestId || currentProject.currentScene?.id !== scene.id)
              return;
            // ScenePanel clears the old expanded beat when the selection changes.
            await tick();
            if (
              requestId !== chaptersRequestId ||
              currentProject.value?.id !== projectId ||
              currentProject.currentScene?.id !== scene.id
            )
              return;
            ui.setExpandedBeat(
              scene.editor_mode !== "page" &&
                currentProject.beats.some((b) => b.id === saved?.current_beat_id)
                ? saved!.current_beat_id
                : null
            );
            session.restoreViewport(projectId, scene.id, saved.scroll_position ?? 0);
          } else {
            session.open(projectId);
            if (
              currentProject.scenes.length === 1 &&
              currentProject.value?.project_type === "screenplay"
            ) {
              await selectScene(currentProject.scenes[0]);
            }
          }
        }
      }
      if (currentProject.value?.project_type === "screenplay") {
        loadPageCountEstimate();
      }
    } catch (e) {
      console.error("Failed to load chapters:", e);
      ui.showError(
        `Failed to load chapters: ${typeof e === "string" ? e : ((e as Error)?.message ?? String(e))}`
      );
    } finally {
      if (requestId === chaptersRequestId) {
        loading = false;
      }
    }
  }

  async function toggleChapter(chapter: Chapter) {
    if (expandedChapters.has(chapter.id)) {
      expandedChapters.delete(chapter.id);
      // If collapsing the current chapter, clear selection
      if (currentProject.currentChapter?.id === chapter.id) {
        currentProject.setCurrentChapter(null);
        currentProject.setScenes([]);
        currentProject.setCurrentScene(null);
        currentProject.setBeats([]);
      }
    } else {
      // Collapse all other chapters and expand only this one
      expandedChapters.clear();
      expandedChapters.add(chapter.id);
      await loadScenes(chapter);
    }
  }

  async function loadScenes(chapter: Chapter, autoSelect = true) {
    const requestId = ++scenesRequestId;
    const chapterId = chapter.id;
    currentProject.setCurrentChapter(chapter);
    try {
      const scenes = await invoke<Scene[]>("get_scenes", {
        chapterId: chapter.id,
      });
      if (requestId !== scenesRequestId || currentProject.currentChapter?.id !== chapterId) return;
      currentProject.setScenes(scenes);
      if (
        scenes.length === 1 &&
        currentProject.value?.project_type === "screenplay" &&
        autoSelect
      ) {
        selectScene(scenes[0]);
      }
    } catch (e) {
      console.error("Failed to load scenes:", e);
    }
  }

  async function selectScene(scene: Scene) {
    const requestId = ++beatsRequestId;
    const sceneId = scene.id;
    currentProject.setCurrentScene(scene);
    try {
      const beats = await invoke<Beat[]>("get_beats", { sceneId: scene.id });
      if (requestId !== beatsRequestId || currentProject.currentScene?.id !== sceneId) return;
      currentProject.setBeats(beats);
    } catch (e) {
      console.error("Failed to load beats:", e);
    }
  }

  async function goHome() {
    try {
      await beforeCloseProject?.();
      currentProject.setProject(null);
      ui.setView("start");
    } catch (error) {
      ui.showError(String(error));
    }
  }

  function toggleSidebar() {
    ui.toggleSidebar();
  }

  function isChapterExpanded(chapterId: string): boolean {
    return expandedChapters.has(chapterId);
  }

  function checkPartExpanded(partId: string): boolean {
    return expandedParts.has(partId);
  }

  function togglePartExpanded(partId: string) {
    if (expandedParts.has(partId)) {
      expandedParts.delete(partId);
    } else {
      // Multiple Parts can be expanded simultaneously
      expandedParts.add(partId);
    }
  }

  // === Create Chapter/Part/Scene ===
  function startCreatingChapter() {
    creatingChapter = true;
    creatingPart = false;
    creatingScene = false;
    showNewDropdown = false;
    newTitle = "";
  }

  function startCreatingPart() {
    creatingPart = true;
    creatingChapter = false;
    creatingScene = false;
    showNewDropdown = false;
    newTitle = "";
  }

  function startCreatingScene() {
    creatingScene = true;
    creatingChapter = false;
    creatingPart = false;
    newTitle = "";
  }

  function cancelCreate() {
    creatingChapter = false;
    creatingPart = false;
    creatingScene = false;
    newTitle = "";
  }

  // Get the insertion point for new chapters/parts (after current selection, or null for end)
  function getInsertionPoint(): string | null {
    if (currentProject.currentChapter) {
      return currentProject.currentChapter.id;
    }
    return null;
  }

  async function createChapter() {
    if (!newTitle.trim() || !currentProject.value) return;
    try {
      const afterId = getInsertionPoint();
      const chapter = await invoke<Chapter>("create_chapter", {
        projectId: currentProject.value.id,
        title: newTitle.trim(),
        isPart: false,
        afterId,
      });
      currentProject.addChapter(chapter, afterId);
      expandedChapters.clear();
      expandedChapters.add(chapter.id);
      await loadScenes(chapter);
      cancelCreate();
    } catch (e) {
      console.error("Failed to create chapter:", e);
    }
  }

  async function createPart() {
    if (!newTitle.trim() || !currentProject.value) return;
    try {
      const afterId = getInsertionPoint();
      const part = await invoke<Chapter>("create_chapter", {
        projectId: currentProject.value.id,
        title: newTitle.trim(),
        isPart: true,
        afterId,
      });
      currentProject.addChapter(part, afterId);
      cancelCreate();
    } catch (e) {
      console.error("Failed to create part:", e);
    }
  }

  async function createScene() {
    if (!newTitle.trim() || !currentProject.currentChapter) return;
    try {
      const scene = await invoke<Scene>("create_scene", {
        chapterId: currentProject.currentChapter.id,
        title: newTitle.trim(),
      });
      currentProject.addScene(scene);
      await selectScene(scene);
      cancelCreate();
    } catch (e) {
      console.error("Failed to create scene:", e);
    }
  }

  function handleCreateKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      if (creatingChapter) createChapter();
      else if (creatingPart) createPart();
      else if (creatingScene) createScene();
    } else if (e.key === "Escape") {
      cancelCreate();
    }
  }

  function handleClickOutsideDropdown(event: MouseEvent) {
    if (
      showNewDropdown &&
      newButtonRef &&
      !newButtonRef.contains(event.target as globalThis.Node)
    ) {
      showNewDropdown = false;
    }
    if (showMoreMenu && moreMenuRef && !moreMenuRef.contains(event.target as globalThis.Node)) {
      showMoreMenu = false;
    }
    if (
      showFilterPopover &&
      filterPopoverRef &&
      !filterPopoverRef.contains(event.target as globalThis.Node)
    ) {
      showFilterPopover = false;
    }
  }

  // === Delete Chapter/Scene ===
  async function confirmDeleteChapter(chapter: Chapter) {
    try {
      // Check if this is a Part with child chapters
      if (chapter.is_part) {
        // Find child chapters (chapters between this Part and the next Part)
        const childChapterIds = getChildChaptersForPart(chapter.id);
        if (childChapterIds.length > 0) {
          // Show Part delete dialog with options
          partDeleteDialog = {
            partId: chapter.id,
            partTitle: chapter.title,
            childChapterIds,
          };
          return;
        }
      }

      // Regular chapter or Part with no children - show standard delete dialog
      const counts = await invoke<{ scene_count: number; beat_count: number }>(
        "get_chapter_content_counts",
        { chapterId: chapter.id }
      );
      deleteDialog = {
        type: "chapter",
        id: chapter.id,
        title: chapter.title,
        message: `This will delete “${chapter.title}” with ${counts.scene_count} scene${counts.scene_count !== 1 ? "s" : ""} and ${counts.beat_count} beat${counts.beat_count !== 1 ? "s" : ""}.`,
      };
    } catch (e) {
      console.error("Failed to get content counts:", e);
    }
  }

  // Get IDs of chapters that belong to a Part (chapters between this Part and the next Part)
  function getChildChaptersForPart(partId: string): string[] {
    const chapters = currentProject.chapters;
    const partIndex = chapters.findIndex((c) => c.id === partId);
    if (partIndex === -1) return [];

    const childIds: string[] = [];
    for (let i = partIndex + 1; i < chapters.length; i++) {
      if (chapters[i].is_part) break; // Stop at next Part
      childIds.push(chapters[i].id);
    }
    return childIds;
  }

  async function confirmDeleteScene(scene: Scene) {
    try {
      const beatCount = await invoke<number>("get_scene_beat_count", { sceneId: scene.id });
      deleteDialog = {
        type: "scene",
        id: scene.id,
        title: scene.title,
        message: `This will delete “${scene.title}” with ${beatCount} beat${beatCount !== 1 ? "s" : ""}.`,
      };
    } catch (e) {
      console.error("Failed to get beat count:", e);
    }
  }

  async function executeDelete() {
    if (!deleteDialog) return;
    try {
      if (deleteDialog.type === "chapter") {
        await invoke("delete_chapter", { chapterId: deleteDialog.id });
        currentProject.removeChapter(deleteDialog.id);
        if (currentProject.currentChapter?.id === deleteDialog.id) {
          currentProject.setCurrentChapter(null);
          currentProject.setScenes([]);
          currentProject.setCurrentScene(null);
          currentProject.setBeats([]);
        }
      } else {
        await invoke("delete_scene", {
          chapterId: currentProject.currentChapter!.id,
          sceneId: deleteDialog.id,
        });
        currentProject.removeScene(deleteDialog.id);
        if (currentProject.currentScene?.id === deleteDialog.id) {
          currentProject.setCurrentScene(null);
          currentProject.setBeats([]);
        }
      }
    } catch (e) {
      console.error("Failed to delete:", e);
    } finally {
      deleteDialog = null;
    }
  }

  // Delete Part only, keeping child chapters
  async function executeDeletePartOnly() {
    if (!partDeleteDialog) return;
    try {
      await invoke("delete_chapter", { chapterId: partDeleteDialog.partId });
      currentProject.removeChapter(partDeleteDialog.partId);
    } catch (e) {
      console.error("Failed to delete part:", e);
    } finally {
      partDeleteDialog = null;
    }
  }

  // Delete Part and all its child chapters
  async function executeDeletePartAndChapters() {
    if (!partDeleteDialog) return;
    try {
      const deletedIds = await invoke<string[]>("delete_part_and_chapters", {
        partId: partDeleteDialog.partId,
        expectedChildIds: partDeleteDialog.childChapterIds,
      });
      for (const chapterId of deletedIds) {
        currentProject.removeChapter(chapterId);
        if (currentProject.currentChapter?.id === chapterId) {
          currentProject.setCurrentChapter(null);
          currentProject.setScenes([]);
          currentProject.setCurrentScene(null);
          currentProject.setBeats([]);
        }
      }
    } catch (e) {
      console.error("Failed to delete part and chapters:", e);
      ui.showError(`Failed to delete Part: ${String(e)}`);
      await loadChapters();
    } finally {
      partDeleteDialog = null;
    }
  }

  // === Drag and Drop (pointer-based, more reliable than HTML5 drag API in webviews) ===
  function onDragHandleMouseDown(e: globalThis.MouseEvent, type: "chapter" | "scene", id: string) {
    e.preventDefault();
    e.stopPropagation();
    draggedItem = { type, id };
    isDragging = true;

    // Find the dragged element by traversing up from the handle
    const target = e.currentTarget as globalThis.HTMLElement;
    const dataAttr = type === "chapter" ? "[data-drag-chapter]" : "[data-drag-scene]";
    draggedElement = target.closest(dataAttr) as globalThis.HTMLElement;
    if (draggedElement) {
      draggedElement.style.opacity = "0.5";
    }

    document.addEventListener("mousemove", onDragMouseMove);
    document.addEventListener("mouseup", onDragMouseUp);
    document.body.style.cursor = "grabbing";
    document.body.style.userSelect = "none";
  }

  function onDragMouseMove(e: globalThis.MouseEvent) {
    if (!isDragging || !draggedItem) return;

    // Clear previous hover styling
    if (currentDragOverElement) {
      currentDragOverElement.style.outline = "";
    }

    // Find which item we're hovering over (same type only)
    const dataAttr = draggedItem.type === "chapter" ? "[data-drag-chapter]" : "[data-drag-scene]";
    const itemElements = document.querySelectorAll(dataAttr);
    let foundElement: globalThis.HTMLElement | null = null;
    let foundId: string | null = null;

    for (const el of itemElements) {
      const rect = el.getBoundingClientRect();
      const itemId = el.getAttribute(
        draggedItem.type === "chapter" ? "data-drag-chapter" : "data-drag-scene"
      );
      if (
        itemId &&
        itemId !== draggedItem.id &&
        e.clientY >= rect.top &&
        e.clientY <= rect.bottom
      ) {
        foundId = itemId;
        foundElement = el as globalThis.HTMLElement;
        break;
      }
    }

    dragOverId = foundId;
    currentDragOverElement = foundElement;

    // Style the hover target
    if (foundElement) {
      foundElement.style.outline = "2px solid var(--color-accent)";
    }
  }

  async function onDragMouseUp() {
    document.removeEventListener("mousemove", onDragMouseMove);
    document.removeEventListener("mouseup", onDragMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";

    // Clear visual styling
    if (draggedElement) {
      draggedElement.style.opacity = "";
    }
    if (currentDragOverElement) {
      currentDragOverElement.style.outline = "";
    }

    if (draggedItem && dragOverId && draggedItem.id !== dragOverId) {
      // Perform the reorder
      const items =
        draggedItem.type === "chapter" ? currentProject.chapters : currentProject.scenes;
      const fromIndex = items.findIndex((item) => item.id === draggedItem!.id);
      const toIndex = items.findIndex((item) => item.id === dragOverId);

      if (fromIndex !== -1 && toIndex !== -1) {
        const newOrder = [...items];
        const [moved] = newOrder.splice(fromIndex, 1);
        newOrder.splice(toIndex, 0, moved);
        const newIds = newOrder.map((item) => item.id);

        try {
          if (draggedItem.type === "chapter" && currentProject.value) {
            await invoke("reorder_chapters", {
              projectId: currentProject.value.id,
              chapterIds: newIds,
            });
            currentProject.reorderChapters(newIds);
          } else if (draggedItem.type === "scene" && currentProject.currentChapter) {
            await invoke("reorder_scenes", {
              chapterId: currentProject.currentChapter.id,
              sceneIds: newIds,
            });
            currentProject.reorderScenes(newIds);
          }
        } catch (e) {
          console.error("Failed to reorder:", e);
        }
      }
    }

    isDragging = false;
    draggedItem = null;
    dragOverId = null;
    draggedElement = null;
    currentDragOverElement = null;
  }

  // === Sync ===
  async function handleSyncClick() {
    if (!currentProject.value) return;
    loadingSyncPreview = true;
    try {
      const preview = await invoke<SyncPreview>("get_sync_preview", {
        projectId: currentProject.value.id,
      });
      syncPreview = preview;
      showSyncDialog = true;
    } catch (e) {
      console.error("Failed to get sync preview:", e);
    } finally {
      loadingSyncPreview = false;
    }
  }

  function closeSyncDialog() {
    showSyncDialog = false;
    syncPreview = null;
  }

  onMount(() => {
    const handler = () => handleSyncClick();
    window.addEventListener("kindling:sync", handler);
    return () => {
      chaptersRequestId++;
      scenesRequestId++;
      beatsRequestId++;
      window.removeEventListener("kindling:sync", handler);
    };
  });

  async function handleSyncComplete(summary: ReimportSummary) {
    syncSummary = summary;
    showSyncDialog = false;
    syncPreview = null;

    // Remember current selection to restore after reload
    const currentChapterId = currentProject.currentChapter?.id;
    const currentSceneId = currentProject.currentScene?.id;

    // Reload chapters
    await loadChapters();

    // Restore chapter and scene selection, reloading their data from DB
    if (currentChapterId) {
      const chapter = currentProject.chapters.find((c) => c.id === currentChapterId);
      if (chapter) {
        await loadScenes(chapter);
        expandedChapters.add(chapter.id);

        if (currentSceneId) {
          // Re-fetch the scene from the updated scenes list
          const scene = currentProject.scenes.find((s) => s.id === currentSceneId);
          if (scene) {
            await selectScene(scene);
          }
        }
      }
    }
  }

  function closeSyncSummary() {
    syncSummary = null;
  }

  // === Context Menu ===
  function openContextMenu(e: MouseEvent, type: "chapter" | "scene", item: Chapter | Scene) {
    e.preventDefault();
    e.stopPropagation();
    // Keyboard activation (Enter/Space on the menu button) and synthetic clicks
    // carry no pointer position, so anchor the menu to the button instead of (0, 0).
    let x = e.clientX;
    let y = e.clientY;
    if (x === 0 && y === 0 && e.currentTarget instanceof globalThis.HTMLElement) {
      const rect = e.currentTarget.getBoundingClientRect();
      x = rect.left;
      y = rect.bottom;
    }
    contextMenu = {
      type,
      id: item.id,
      x,
      y,
      item,
    };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function getContextMenuItems(type: "chapter" | "scene", item: Chapter | Scene): MenuItem[] {
    const isLocked = "locked" in item && item.locked;
    const isPart = type === "chapter" && "is_part" in item && (item as Chapter).is_part;

    return [
      {
        label: "Rename",
        icon: Pencil,
        action: () => {
          renameDialog = {
            type,
            id: item.id,
            title: item.title,
          };
        },
        disabled: isLocked,
      },
      {
        label: "Planning",
        action: () => {},
        disabled: isLocked,
        children: [
          {
            label: "Fixed",
            action: () => setPlanningStatus(type, item, "fixed"),
            disabled: isLocked || (item as Chapter & Scene).planning_status === "fixed",
          },
          {
            label: "Flexible",
            action: () => setPlanningStatus(type, item, "flexible"),
            disabled: isLocked || (item as Chapter & Scene).planning_status === "flexible",
          },
          {
            label: "Undefined",
            action: () => setPlanningStatus(type, item, "undefined"),
            disabled: isLocked || (item as Chapter & Scene).planning_status === "undefined",
          },
        ],
      },
      {
        label: "Duplicate",
        icon: Copy,
        action: () => handleDuplicate(type, item.id),
      },
      // Convert to Part/Chapter option (only for chapters)
      ...(type === "chapter"
        ? [
            {
              label: isPart ? `Convert to ${chapterLabel}` : `Convert to ${partLabel}`,
              icon: BookOpen,
              action: () => handleTogglePart(item.id, !isPart),
              disabled: isLocked,
            },
          ]
        : []),
      { divider: true, label: "", action: () => {} },
      {
        label: isLocked ? "Unlock" : "Lock",
        icon: isLocked ? Unlock : Lock,
        action: () => handleToggleLock(type, item.id, isLocked),
      },
      {
        label: "Archive",
        icon: Archive,
        action: () => handleArchive(type, item.id),
        disabled: isLocked,
      },
      {
        label: "Export",
        icon: Download,
        action: () => {
          exportDialog = {
            scope: type,
            scopeId: item.id,
            scopeTitle: item.title,
          };
        },
      },
      { divider: true, label: "", action: () => {} },
      {
        label: "Delete",
        icon: Trash2,
        action: () => {
          if (type === "chapter") {
            confirmDeleteChapter(item as Chapter);
          } else {
            confirmDeleteScene(item as Scene);
          }
        },
        danger: true,
        disabled: isLocked,
      },
    ];
  }

  // === Context Menu Actions ===
  async function handleRename(type: "chapter" | "scene", id: string, newTitle: string) {
    try {
      if (type === "chapter") {
        await invoke("rename_chapter", { chapterId: id, title: newTitle });
        currentProject.updateChapter(id, { title: newTitle });
      } else {
        await invoke("rename_scene", { sceneId: id, title: newTitle });
        currentProject.updateScene(id, { title: newTitle });
      }
    } catch (e) {
      console.error("Failed to rename:", e);
      throw e;
    }
  }

  async function handleDuplicate(type: "chapter" | "scene", id: string) {
    try {
      if (type === "chapter") {
        const newChapter = await invoke<Chapter>("duplicate_chapter", { chapterId: id });
        currentProject.addChapter(newChapter);
      } else {
        const newScene = await invoke<Scene>("duplicate_scene", { sceneId: id });
        currentProject.addScene(newScene);
      }
    } catch (e) {
      console.error("Failed to duplicate:", e);
    }
  }

  async function handleTogglePart(chapterId: string, isPart: boolean) {
    try {
      await invoke("set_chapter_is_part", { chapterId, isPart });
      currentProject.updateChapter(chapterId, { is_part: isPart });
    } catch (e) {
      console.error("Failed to toggle part status:", e);
    }
  }

  async function setPlanningStatus(
    type: "chapter" | "scene",
    item: Chapter | Scene,
    status: PlanningStatus
  ) {
    try {
      if (type === "chapter") {
        await invoke("update_chapter_planning_status", {
          chapterId: item.id,
          planningStatus: status,
        });
        currentProject.updateChapter(item.id, { planning_status: status });
      } else {
        await invoke("update_scene_planning_status", {
          sceneId: item.id,
          planningStatus: status,
        });
        currentProject.updateScene(item.id, { planning_status: status });
      }
    } catch (e) {
      console.error("Failed to update planning status:", e);
    }
  }

  async function handleArchive(type: "chapter" | "scene", id: string) {
    try {
      if (type === "chapter") {
        await invoke("archive_chapter", { chapterId: id });
        currentProject.removeChapter(id);
      } else {
        await invoke("archive_scene", { sceneId: id });
        currentProject.removeScene(id);
      }
    } catch (e) {
      console.error("Failed to archive:", e);
    }
  }

  async function handleToggleLock(type: "chapter" | "scene", id: string, currentlyLocked: boolean) {
    try {
      if (type === "chapter") {
        if (currentlyLocked) {
          await invoke("unlock_chapter", { chapterId: id });
          currentProject.updateChapter(id, { locked: false });
        } else {
          await invoke("lock_chapter", { chapterId: id });
          currentProject.updateChapter(id, { locked: true });
        }
      } else {
        if (currentlyLocked) {
          await invoke("unlock_scene", { sceneId: id });
          currentProject.updateScene(id, { locked: false });
        } else {
          await invoke("lock_scene", { sceneId: id });
          currentProject.updateScene(id, { locked: true });
        }
      }
    } catch (e) {
      console.error("Failed to toggle lock:", e);
    }
  }

  // Track isImporting state to properly handle chapter loading
  const isImporting = $derived(ui.isImporting);

  let requestedChapterProject: string | null = null;
  $effect(() => {
    const projectId = currentProject.value?.id ?? null;
    const importing = isImporting;
    if (!projectId) {
      requestedChapterProject = null;
    } else if (!importing && requestedChapterProject !== projectId) {
      requestedChapterProject = projectId;
      // An empty result is loaded data. Loading reads and writes outline state,
      // which must not become dependencies of this project-selection effect.
      untrack(() => {
        void loadChapters(true);
        void loadSavedFilters();
      });
    }
  });

  // Close dropdowns when clicking outside
  $effect(() => {
    if (showNewDropdown || showMoreMenu || showFilterPopover) {
      document.addEventListener("click", handleClickOutsideDropdown);
      return () => {
        document.removeEventListener("click", handleClickOutsideDropdown);
      };
    }
  });
</script>

<aside
  data-testid="sidebar"
  class="sidebar"
  class:is-collapsed={ui.sidebarCollapsed}
  aria-label="Project outline sidebar"
>
  {#if ui.sidebarCollapsed}
    <div class="sb-rail">
      <button
        type="button"
        onclick={toggleSidebar}
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label="Expand sidebar"
        title="Expand sidebar"
      >
        <PanelLeftOpen class="w-5 h-5" aria-hidden="true" />
      </button>
      <img
        class="sb-rail-flame on-light"
        src="/brand/kindling-flame.svg"
        alt=""
        width="24"
        height="24"
      />
      <img
        class="sb-rail-flame on-dark"
        src="/brand/kindling-flame-reversed.svg"
        alt=""
        width="24"
        height="24"
      />
    </div>
  {/if}
  <!-- Header -->
  <div class="sb-head" inert={ui.sidebarCollapsed}>
    <div class="sb-brand">
      <BrandWordmark />
      <button
        type="button"
        onclick={toggleSidebar}
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label="Collapse sidebar"
        title="Collapse sidebar"
      >
        <PanelLeftClose class="w-5 h-5" aria-hidden="true" />
      </button>
    </div>
    {#if currentProject.value}
      <button
        type="button"
        data-testid="sidebar-home"
        onclick={goHome}
        class="ka-button ka-button--ghost sb-home"
        aria-label="Home — all projects"
        title="Return home to all projects"
      >
        <Home class="w-5 h-5" aria-hidden="true" />
        <span>Home</span>
        <small>All projects</small>
      </button>
      <!-- Project name with action icons -->
      <div class="sb-project">
        <div class="sb-project-title">
          <h2 class="sb-project-name" title={currentProject.value.name}>
            {currentProject.value.name}
          </h2>
          {#if currentProject.value.project_type === "screenplay" && pageCountEstimate}
            <span
              class="ka-badge"
              title="{pageCountEstimate.words} words · target: {pageCountEstimate.target}"
            >
              {pageCountEstimate.pages.toFixed(1)} / {pageCountEstimate.target}
            </span>
          {/if}
        </div>
        <!-- Action icons (primary only; secondary behind more menu) -->
        <div class="sb-project-actions">
          {#if currentProject.value.source_path && supportsSync(currentProject.value.source_type)}
            <button
              type="button"
              data-testid="sync-button"
              onclick={handleSyncClick}
              disabled={loadingSyncPreview}
              aria-busy={loadingSyncPreview || undefined}
              class="ka-button ka-button--ghost ka-icon-button"
              aria-label="Sync from source"
              title="Sync from source"
            >
              <RefreshCw
                class="w-5 h-5 {loadingSyncPreview ? 'animate-spin' : ''}"
                aria-hidden="true"
              />
            </button>
          {/if}
          <div class="relative" bind:this={moreMenuRef}>
            <button
              type="button"
              onclick={() => (showMoreMenu = !showMoreMenu)}
              class="ka-button ka-button--ghost ka-icon-button"
              aria-label="More actions"
              title="More actions"
              aria-haspopup="menu"
              aria-expanded={showMoreMenu}
              data-testid="more-actions-button"
            >
              <MoreVertical class="w-5 h-5" aria-hidden="true" />
            </button>
            {#if showMoreMenu}
              <div
                class="ka-menu-list app-popover sb-popover"
                role="menu"
                aria-label="Project actions"
              >
                <button
                  data-testid="export-button"
                  onclick={() => {
                    showMoreMenu = false;
                    if (currentProject.value) {
                      exportDialog = {
                        scope: "project",
                        scopeId: null,
                        scopeTitle: currentProject.value.name,
                      };
                    }
                  }}
                  role="menuitem"
                  class="sb-menuitem"
                >
                  <Download class="w-5 h-5" aria-hidden="true" />
                  Export
                </button>
                <button
                  data-testid="snapshots-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showSnapshotsPanel = true;
                  }}
                  role="menuitem"
                  class="sb-menuitem"
                >
                  <Clock class="w-5 h-5" aria-hidden="true" />
                  Snapshots
                </button>
                <button
                  data-testid="archive-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showArchivePanel = true;
                  }}
                  role="menuitem"
                  class="sb-menuitem"
                >
                  <Archive class="w-5 h-5" aria-hidden="true" />
                  Archive
                </button>
              </div>
            {/if}
          </div>
        </div>
      </div>
      <WritingProgress prepareReset={prepareWritingReset} />
    {/if}
  </div>

  <!-- Chapter/Scene Tree -->
  <div class="sb-scroll" inert={ui.sidebarCollapsed}>
    {#if loading}
      <div class="ka-progress od-field sb-status" role="status">
        <span>Opening project…</span>
        <progress aria-label="Opening project"></progress>
      </div>
    {:else if currentProject.chapters.length === 0}
      <p class="sb-status ka-help">No chapters yet. Add one below to start outlining.</p>
    {:else}
      <nav class="sb-tree" aria-label="Project outline">
        {#each partGroups as group}
          <!-- Part header (if this group has a Part) -->
          {#if group.part}
            {@const part = group.part}
            {@const isPartExpanded = checkPartExpanded(part.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              data-testid="part-item"
              data-drag-chapter={part.id}
              class="sb-group sb-part"
              class:is-drop-target={dragOverId === part.id}
            >
              <!-- Part row -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="sb-row" oncontextmenu={(e) => openContextMenu(e, "chapter", part)}>
                <!-- Drag handle -->
                <div
                  data-testid="drag-handle"
                  onmousedown={(e) => onDragHandleMouseDown(e, "chapter", part.id)}
                  class="sb-grip"
                  aria-hidden="true"
                  title="Drag to reorder"
                >
                  <GripVertical class="w-4 h-4" aria-hidden="true" />
                </div>

                <button
                  onclick={() => togglePartExpanded(part.id)}
                  class="sb-row-main"
                  aria-expanded={isPartExpanded}
                >
                  <ChevronRight class="sb-chev" aria-hidden="true" />
                  {#if part.locked}
                    <Lock class="sb-glyph is-warning" aria-label="Locked" />
                  {:else if (part.planning_status ?? "fixed") === "flexible"}
                    <CircleDot class="sb-glyph is-warning" aria-label="Flexible" />
                  {:else if (part.planning_status ?? "fixed") === "undefined"}
                    <CircleDashed class="sb-glyph" aria-label="Undefined" />
                  {/if}
                  <span
                    data-testid="part-title"
                    class="sb-row-title sb-part-title"
                    class:is-locked={part.locked}>{part.title}</span
                  >
                </button>

                <!-- Three-dot menu button -->
                <button
                  data-testid="menu-button"
                  onclick={(e) => openContextMenu(e, "chapter", part)}
                  class="ka-button ka-button--ghost ka-icon-button sb-row-menu"
                  aria-label="{partLabel} menu"
                  aria-haspopup="menu"
                >
                  <MoreVertical class="w-5 h-5" aria-hidden="true" />
                </button>
              </div>
            </div>
          {/if}

          <!-- Chapters in this group (collapsible under Part) -->
          {#if !group.part || checkPartExpanded(group.part.id)}
            <div class:sb-part-children={group.part}>
              {#each group.chapters as chapter}
                {@const isExpanded = isChapterExpanded(chapter.id)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  data-testid="chapter-item"
                  data-drag-chapter={chapter.id}
                  class="sb-group"
                  class:is-drop-target={dragOverId === chapter.id}
                >
                  <!-- Chapter row -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="sb-chapter"
                    oncontextmenu={(e) => openContextMenu(e, "chapter", chapter)}
                  >
                    <div class="sb-row">
                      <!-- Drag handle -->
                      <div
                        data-testid="drag-handle"
                        onmousedown={(e) => onDragHandleMouseDown(e, "chapter", chapter.id)}
                        class="sb-grip"
                        aria-hidden="true"
                        title="Drag to reorder"
                      >
                        <GripVertical class="w-4 h-4" aria-hidden="true" />
                      </div>

                      <button
                        onclick={() => toggleChapter(chapter)}
                        class="sb-row-main"
                        aria-expanded={isExpanded}
                      >
                        <ChevronRight class="sb-chev" aria-hidden="true" />
                        {#if chapter.locked}
                          <Lock class="sb-glyph is-warning" aria-label="Locked" />
                        {:else if (chapter.planning_status ?? "fixed") === "flexible"}
                          <CircleDot class="sb-glyph is-warning" aria-label="Flexible" />
                        {:else if (chapter.planning_status ?? "fixed") === "undefined"}
                          <CircleDashed class="sb-glyph" aria-label="Undefined" />
                        {/if}
                        <span
                          data-testid="chapter-title"
                          class="sb-row-title sb-chapter-title"
                          class:is-locked={chapter.locked}
                          title={chapter.title}>{chapter.title}</span
                        >
                        {#if writing.value?.chapter_words?.[chapter.id] !== undefined}
                          <small class="sb-row-trail"
                            >{writing.value.chapter_words[chapter.id].toLocaleString()} words</small
                          >
                        {/if}
                      </button>

                      <!-- Three-dot menu button -->
                      <button
                        data-testid="menu-button"
                        onclick={(e) => openContextMenu(e, "chapter", chapter)}
                        class="ka-button ka-button--ghost ka-icon-button sb-row-menu"
                        aria-label="{chapterLabel} menu"
                        aria-haspopup="menu"
                      >
                        <MoreVertical class="w-5 h-5" aria-hidden="true" />
                      </button>
                    </div>

                    {#if isExpanded && currentProject.currentChapter?.id === chapter.id}
                      {@const chapterPlanning = chapter.planning_status ?? "fixed"}

                      <!-- Chapter synopsis (Flexible/Undefined only) -->
                      {#if chapterPlanning !== "fixed"}
                        <div class="sb-synopsis">
                          {#if editingChapterSynopsisId === chapter.id}
                            <!-- svelte-ignore a11y_autofocus -->
                            <textarea
                              bind:value={chapterSynopsisText}
                              oninput={() => handleChapterSynopsisInput(chapter.id)}
                              onblur={() => finishEditingChapterSynopsis(chapter.id)}
                              placeholder="{chapterLabel} synopsis…"
                              aria-label="{chapterLabel} synopsis"
                              rows="3"
                              autofocus
                            ></textarea>
                          {:else}
                            <button
                              onclick={() => startEditingChapterSynopsis(chapter)}
                              class="sb-synopsis-text"
                              class:is-empty={!chapter.synopsis}
                            >
                              {chapter.synopsis || "Add synopsis…"}
                            </button>
                          {/if}
                        </div>
                      {/if}

                      <!-- Undefined chapter: placeholder, no scene list -->
                      {#if chapterPlanning === "undefined"}
                        <div class="sb-children">
                          <div class="sb-undefined">
                            <p class="ka-help">
                              This chapter is undefined. Add a synopsis and scenes will appear when
                              you promote it to Flexible or Fixed.
                            </p>
                            {#if !chapter.locked}
                              <button
                                type="button"
                                onclick={() => setPlanningStatus("chapter", chapter, "flexible")}
                                class="ka-button ka-button--secondary sb-small-button"
                              >
                                Switch to Flexible
                              </button>
                            {/if}
                          </div>
                        </div>

                        <!-- Flexible chapter: simplified scene titles, no filters/drag -->
                      {:else if chapterPlanning === "flexible"}
                        <div class="sb-children">
                          {#each filteredScenes as scene}
                            {@const isSelected = currentProject.currentScene?.id === scene.id}
                            <button
                              onclick={() => selectScene(scene)}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                              class="sb-row-main sb-scene-main"
                              class:is-selected={isSelected}
                              aria-current={isSelected ? "page" : undefined}
                            >
                              {#if (scene.planning_status ?? "fixed") === "flexible"}
                                <CircleDot class="sb-glyph is-warning" aria-label="Flexible" />
                              {:else if (scene.planning_status ?? "fixed") === "undefined"}
                                <CircleDashed class="sb-glyph" aria-label="Undefined" />
                              {/if}
                              <span class="sb-row-title" title={scene.title}>{scene.title}</span>
                              {#if writing.value?.scene_words?.[scene.id] !== undefined}
                                <small class="sb-row-trail"
                                  >{writing.value.scene_words[scene.id].toLocaleString()} words</small
                                >
                              {/if}
                            </button>
                          {/each}

                          {#if creatingScene}
                            <div class="sb-new-input">
                              <!-- svelte-ignore a11y_autofocus -->
                              <input
                                data-testid="title-input"
                                type="text"
                                bind:value={newTitle}
                                onkeydown={handleCreateKeydown}
                                onblur={cancelCreate}
                                placeholder="Scene title…"
                                aria-label="New scene title"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="sb-row-main sb-new"
                            >
                              <Plus class="w-4 h-4" aria-hidden="true" />
                              New scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="sb-empty">No scenes yet</span>
                          {/if}

                          {#if !chapter.locked}
                            <div class="sb-define">
                              <button
                                type="button"
                                onclick={() => setPlanningStatus("chapter", chapter, "fixed")}
                                class="ka-button ka-button--ghost sb-small-button"
                              >
                                Define full structure
                              </button>
                            </div>
                          {/if}
                        </div>

                        <!-- Fixed chapter: full scene list with filters, drag, icons -->
                      {:else}
                        <div class="sb-filter">
                          <fieldset class="ka-segments sb-filter-segments">
                            <legend class="ka-sr">Show scenes</legend>
                            <div class="ka-segment-track">
                              {#each [{ value: "all", label: "All", title: "Show all scenes" }, { value: "planned_only", label: "Planned", title: "Show only planned scenes" }, { value: "next_5", label: "Next 5", title: "Show next 5 scenes" }] as option (option.value)}
                                <label
                                  class="ka-segment"
                                  class:ka-selected={outlineViewFilter === option.value}
                                  title={option.title}
                                >
                                  <input
                                    type="radio"
                                    name={`outline-filter-${chapter.id}`}
                                    value={option.value}
                                    bind:group={outlineViewFilter}
                                  />
                                  <span>{option.label}</span>
                                </label>
                              {/each}
                            </div>
                          </fieldset>

                          <!-- Filter popover trigger -->
                          <div class="relative" bind:this={filterPopoverRef}>
                            <button
                              type="button"
                              onclick={() => (showFilterPopover = !showFilterPopover)}
                              class="ka-button ka-button--ghost ka-icon-button sb-filter-button"
                              class:is-active={hasActiveFilters}
                              aria-label={hasActiveFilters
                                ? "Filter by type & status (filters on)"
                                : "Filter by type & status"}
                              title="Filter by type & status"
                              aria-expanded={showFilterPopover}
                            >
                              <Filter class="w-5 h-5" aria-hidden="true" />
                            </button>
                            {#if showFilterPopover}
                              <div class="app-popover sb-filter-popover">
                                <div class="sb-filter-head">
                                  <span class="sb-filter-heading">Filters</span>
                                  {#if hasActiveFilters}
                                    <button
                                      type="button"
                                      class="ka-button ka-button--ghost sb-small-button"
                                      onclick={() => {
                                        sceneStatusFilter = "all";
                                        showNotesScenes = true;
                                        showTodoScenes = true;
                                        showUnusedScenes = true;
                                      }}
                                    >
                                      Reset
                                    </button>
                                  {/if}
                                </div>

                                <!-- Type filter -->
                                <div class="sb-filter-field">
                                  <span class="sb-filter-label">Scene type</span>
                                  <div class="sb-filter-chips">
                                    {#each sceneTypeFilterOptions as option}
                                      {@const TypeIcon = option.icon}
                                      <button
                                        type="button"
                                        class="ka-tag sb-type-chip"
                                        class:is-on={isSceneTypeVisible(option.type)}
                                        onclick={() => toggleSceneTypeVisible(option.type)}
                                        aria-pressed={isSceneTypeVisible(option.type)}
                                      >
                                        <TypeIcon class="w-4 h-4" aria-hidden="true" />
                                        {option.label}
                                      </button>
                                    {/each}
                                  </div>
                                </div>

                                <!-- Status filter -->
                                <div class="ka-field od-field sb-filter-field">
                                  <label for={`scene-status-filter-${chapter.id}`}>Status</label>
                                  <select
                                    id={`scene-status-filter-${chapter.id}`}
                                    bind:value={sceneStatusFilter}
                                    aria-label="Scene status filter"
                                  >
                                    {#each sceneStatusOptions as option}
                                      <option value={option.value}>{option.label}</option>
                                    {/each}
                                  </select>
                                </div>

                                <!-- Saved filters -->
                                {#if savedFilters.length > 0}
                                  <div class="sb-filter-section">
                                    <span class="sb-filter-label">Saved filters</span>
                                    {#each savedFilters as filter}
                                      <div class="sb-saved-filter">
                                        <button
                                          type="button"
                                          onclick={() => applySavedFilter(filter)}
                                          class="ka-button ka-button--ghost sb-saved-name"
                                        >
                                          {filter.name}
                                        </button>
                                        <button
                                          type="button"
                                          onclick={() => deleteSavedFilter(filter.id)}
                                          class="ka-button ka-button--ghost ka-icon-button sb-danger-icon"
                                          aria-label="Delete saved filter {filter.name}"
                                        >
                                          <Trash2 class="w-5 h-5" aria-hidden="true" />
                                        </button>
                                      </div>
                                    {/each}
                                  </div>
                                {/if}

                                <!-- Save current filter -->
                                {#if hasActiveFilters}
                                  <div class="sb-filter-section">
                                    {#if showSaveFilterInput}
                                      <div class="sb-save-filter">
                                        <input
                                          type="text"
                                          bind:value={savedFilterName}
                                          placeholder="Filter name…"
                                          aria-label="Filter name"
                                          onkeydown={(e) =>
                                            e.key === "Enter" && saveCurrentFilter()}
                                        />
                                        <button
                                          type="button"
                                          onclick={saveCurrentFilter}
                                          disabled={!savedFilterName.trim()}
                                          class="ka-button ka-button--secondary"
                                        >
                                          Save
                                        </button>
                                      </div>
                                    {:else}
                                      <button
                                        type="button"
                                        onclick={() => (showSaveFilterInput = true)}
                                        class="ka-button ka-button--ghost sb-small-button"
                                      >
                                        Save current filter…
                                      </button>
                                    {/if}
                                  </div>
                                {/if}
                              </div>
                            {/if}
                          </div>
                        </div>

                        <div class="sb-children">
                          {#each filteredScenes as scene}
                            {@const isSelected = currentProject.currentScene?.id === scene.id}
                            {@const isLocked = scene.locked || chapter.locked}
                            {@const sceneType = scene.scene_type ?? "normal"}
                            {@const sceneStatus = scene.scene_status ?? "draft"}
                            {@const planningStatus = scene.planning_status ?? "fixed"}
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                              data-drag-scene={scene.id}
                              data-testid="scene-item"
                              class="sb-row sb-scene"
                              class:is-selected={isSelected}
                              class:is-drop-target={dragOverId === scene.id}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                            >
                              <!-- Scene drag handle -->
                              <div
                                data-testid="drag-handle"
                                onmousedown={(e) => onDragHandleMouseDown(e, "scene", scene.id)}
                                class="sb-grip"
                                aria-hidden="true"
                                title="Drag to reorder"
                              >
                                <GripVertical class="w-4 h-4" aria-hidden="true" />
                              </div>

                              <button
                                onclick={() => selectScene(scene)}
                                class="sb-row-main sb-scene-main"
                                class:is-selected={isSelected}
                                aria-current={isSelected ? "page" : undefined}
                              >
                                <!-- Planning status / lock indicator (single leading icon) -->
                                {#if isLocked}
                                  <Lock class="sb-glyph is-warning" aria-label="Locked" />
                                {:else if planningStatus === "flexible"}
                                  <CircleDot class="sb-glyph is-warning" aria-label="Flexible" />
                                {:else if planningStatus === "undefined"}
                                  <CircleDashed class="sb-glyph" aria-label="Undefined" />
                                {/if}
                                <span
                                  data-testid="scene-title"
                                  class="sb-row-title"
                                  class:is-locked={isLocked}
                                  title={scene.title}>{scene.title}</span
                                >
                                <!-- Trailing metadata: count, scene type, status (shape + text) -->
                                <span class="sb-row-trail">
                                  {#if writing.value?.scene_words?.[scene.id] !== undefined}
                                    <small
                                      >{writing.value.scene_words[scene.id].toLocaleString()} words</small
                                    >
                                  {/if}
                                  {#if sceneType !== "normal"}
                                    {@const SceneTypeIcon = sceneTypeFilterOptions.find(
                                      (option) => option.type === sceneType
                                    )?.icon}
                                    {#if SceneTypeIcon}
                                      <SceneTypeIcon
                                        class="sb-glyph"
                                        aria-label={sceneTypeLabels[sceneType as SceneType]}
                                      />
                                    {/if}
                                  {/if}
                                  <span
                                    class="sb-status-dot is-{sceneStatus}"
                                    title={sceneStatusLabels[sceneStatus]}
                                  ></span>
                                  <span class="ka-sr">Status: {sceneStatusLabels[sceneStatus]}</span
                                  >
                                </span>
                              </button>

                              <!-- Scene menu button -->
                              <button
                                data-testid="menu-button"
                                onclick={(e) => openContextMenu(e, "scene", scene)}
                                class="ka-button ka-button--ghost ka-icon-button sb-row-menu"
                                aria-label="Scene menu"
                                aria-haspopup="menu"
                              >
                                <MoreVertical class="w-5 h-5" aria-hidden="true" />
                              </button>
                            </div>
                          {/each}

                          <!-- New Scene Button or Input -->
                          {#if creatingScene}
                            <div class="sb-new-input">
                              <!-- svelte-ignore a11y_autofocus -->
                              <input
                                data-testid="title-input"
                                type="text"
                                bind:value={newTitle}
                                onkeydown={handleCreateKeydown}
                                onblur={cancelCreate}
                                placeholder="Scene title…"
                                aria-label="New scene title"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="sb-row-main sb-new"
                            >
                              <Plus class="w-4 h-4" aria-hidden="true" />
                              New scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="sb-empty">No scenes yet</span>
                          {:else if filteredScenes.length === 0 && !creatingScene}
                            <span class="sb-empty">No scenes match these filters</span>
                          {/if}
                        </div>
                      {/if}
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {/each}

        {#if creatingChapter || creatingPart}
          <div class="sb-new-input">
            <!-- svelte-ignore a11y_autofocus -->
            <input
              data-testid="title-input"
              type="text"
              bind:value={newTitle}
              onkeydown={handleCreateKeydown}
              onblur={cancelCreate}
              placeholder={creatingPart ? `${partLabel} title…` : `${chapterLabel} title…`}
              aria-label={creatingPart ? `New ${partLabel} title` : `New ${chapterLabel} title`}
              autofocus
            />
          </div>
        {/if}
      </nav>
    {/if}
  </div>
  {#if currentProject.value}
    <footer class="sb-foot" inert={ui.sidebarCollapsed}>
      {#if !creatingChapter && !creatingPart}
        <!-- Split button: New Chapter (default) with dropdown for New Part -->
        <div class="relative" bind:this={newButtonRef}>
          <div class="sb-split">
            <!-- Main action: New Chapter/Sequence -->
            <button
              type="button"
              data-testid="new-chapter-button"
              onclick={startCreatingChapter}
              class="ka-button ka-button--secondary"
            >
              <Plus class="w-5 h-5" aria-hidden="true" />
              New {chapterLabel.toLowerCase()}
            </button>
            <!-- Dropdown trigger -->
            <button
              type="button"
              data-testid="new-dropdown-button"
              onclick={() => (showNewDropdown = !showNewDropdown)}
              class="ka-button ka-button--secondary ka-icon-button"
              aria-label="More options"
              aria-haspopup="menu"
              aria-expanded={showNewDropdown}
            >
              <ChevronDown class="w-5 h-5" aria-hidden="true" />
            </button>
          </div>

          <!-- Dropdown menu -->
          {#if showNewDropdown}
            <div
              class="ka-menu-list app-popover sb-popover sb-popover-up"
              role="menu"
              aria-label="Create"
            >
              <button
                data-testid="dropdown-new-chapter"
                onclick={startCreatingChapter}
                role="menuitem"
                class="sb-menuitem"
              >
                <Folder class="w-5 h-5" aria-hidden="true" />
                New {chapterLabel.toLowerCase()}
              </button>
              <button
                data-testid="dropdown-new-part"
                onclick={startCreatingPart}
                role="menuitem"
                class="sb-menuitem"
              >
                <BookOpen class="w-5 h-5" aria-hidden="true" />
                New {partLabel.toLowerCase()}
              </button>
            </div>
          {/if}
        </div>
      {/if}
      {#if onOpenSettings}
        <button
          type="button"
          data-testid="sidebar-settings-button"
          onclick={onOpenSettings}
          class="ka-button ka-button--ghost sb-settings"
        >
          <Settings class="w-5 h-5" aria-hidden="true" />
          Settings
        </button>
      {/if}
    </footer>
  {:else if onOpenSettings}
    <footer class="sb-foot" inert={ui.sidebarCollapsed}>
      <button
        type="button"
        data-testid="sidebar-settings-button"
        onclick={onOpenSettings}
        class="ka-button ka-button--ghost sb-settings"
      >
        <Settings class="w-5 h-5" aria-hidden="true" />
        Settings
      </button>
    </footer>
  {/if}
</aside>

<!-- Sync Preview Dialog -->
{#if showSyncDialog && syncPreview && currentProject.value}
  <SyncDialog
    projectId={currentProject.value.id}
    {syncPreview}
    onClose={closeSyncDialog}
    onSyncComplete={handleSyncComplete}
  />
{/if}

<!-- Delete Confirmation Dialog -->
{#if deleteDialog}
  <ConfirmDialog
    title="Delete {deleteDialog.type === 'chapter' ? 'chapter' : 'scene'}"
    message={deleteDialog.message}
    onConfirm={executeDelete}
    onCancel={() => (deleteDialog = null)}
  />
{/if}

<!-- Part Delete Dialog (with options) -->
{#if partDeleteDialog}
  <PartDeleteDialog
    partTitle={partDeleteDialog.partTitle}
    childChapterCount={partDeleteDialog.childChapterIds.length}
    {partLabel}
    {chapterLabel}
    onDeletePartOnly={executeDeletePartOnly}
    onDeletePartAndChapters={executeDeletePartAndChapters}
    onCancel={() => (partDeleteDialog = null)}
  />
{/if}

<!-- Sync Summary Dialog -->
{#if syncSummary}
  <SyncSummaryDialog summary={syncSummary} onClose={closeSyncSummary} />
{/if}

<!-- Context Menu -->
{#if contextMenu}
  <ContextMenu
    items={getContextMenuItems(contextMenu.type, contextMenu.item)}
    x={contextMenu.x}
    y={contextMenu.y}
    onClose={closeContextMenu}
  />
{/if}

<!-- Rename Dialog -->
{#if renameDialog}
  <RenameDialog
    title="Rename {renameDialog.type === 'chapter' ? chapterLabel : 'Scene'}"
    currentName={renameDialog.title}
    onSave={(newName) => handleRename(renameDialog!.type, renameDialog!.id, newName)}
    onClose={() => (renameDialog = null)}
  />
{/if}

<!-- Archive Panel -->
{#if showArchivePanel}
  <ArchivePanel onClose={() => (showArchivePanel = false)} />
{/if}

<!-- Snapshots Panel -->
{#if showSnapshotsPanel}
  <SnapshotsPanel
    prepareRestore={async () => {
      await prepareWritingReset?.();
    }}
    onClose={() => (showSnapshotsPanel = false)}
  />
{/if}

<!-- Export Dialog -->
{#if exportDialog}
  <ExportDialog
    scope={exportDialog.scope}
    scopeId={exportDialog.scopeId}
    scopeTitle={exportDialog.scopeTitle}
    onClose={() => (exportDialog = null)}
    onSuccess={(result) => {
      exportDialog = null;
      exportResult = result;
    }}
  />
{/if}

<!-- Export Success Dialog -->
{#if exportResult}
  <ExportSuccessDialog result={exportResult} onClose={() => (exportResult = null)} />
{/if}

<style>
  /* Press workspace sidebar: surface column, hairline groups, 44px rows. */
  .sidebar {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    flex: none;
    width: 304px;
    min-height: 0;
    height: 100%;
    background: var(--color-surface);
    border-right: var(--border-hair);
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .sidebar > * {
    min-width: 0;
  }
  .sidebar.is-collapsed {
    display: block;
    width: 56px;
  }
  /* Compact windows (about 1100 wide): the writing column keeps the room. */
  @media (max-width: 1280px) {
    .sidebar:not(.is-collapsed) {
      width: 256px;
    }
  }
  .sidebar.is-collapsed > :not(.sb-rail) {
    display: none;
  }
  .sb-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2xs);
    padding: var(--space-2xs) 6px;
  }
  .sb-rail-flame {
    width: 24px;
    height: 24px;
    margin: 10px 0;
  }
  .on-dark {
    display: none;
  }
  :global([data-theme="dark"]) .sidebar .on-light {
    display: none;
  }
  :global([data-theme="dark"]) .sidebar .on-dark {
    display: block;
  }

  .sb-head {
    padding: 0 var(--space-xs);
  }
  .sb-brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2xs) 0 var(--space-2xs) var(--space-3xs);
  }
  .sb-home {
    width: 100%;
    justify-content: flex-start;
    padding: var(--space-2xs) var(--space-xs);
  }
  .sb-home small {
    margin-left: auto;
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .sb-project {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2xs);
    margin-top: var(--space-2xs);
    padding: var(--space-2xs) 0 var(--space-2xs) var(--space-xs);
    border-top: var(--border-hair);
  }
  /* The name gets the full row (two lines before it clamps); a screenplay's
     page estimate sits under it rather than squeezing it. */
  .sb-project-title {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3xs);
    min-width: 0;
    flex: 1;
  }
  .sb-project-name {
    margin: 0;
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow-wrap: anywhere;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .sb-project-actions {
    display: flex;
    align-items: center;
    flex: none;
  }

  .sb-scroll {
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    padding: 0 var(--space-xs) var(--space-s);
  }
  .sb-status {
    margin: var(--space-s) var(--space-xs);
  }
  .sb-tree {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3xs);
    margin-top: var(--space-xs);
  }
  .sb-part {
    margin-top: var(--space-s);
  }
  .sb-part-children {
    margin-left: var(--space-xs);
    padding-left: var(--space-2xs);
    border-left: var(--border-hair);
  }

  /* A row: optional grip cue, the full-width main target, a trailing menu
     that replaces the trailing metadata on hover/focus without shifting. */
  .sb-row {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
    border-radius: var(--radius-xs);
  }
  .sb-row-main {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex: 1;
    min-width: 0;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs) var(--space-2xs) var(--space-2xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  @media (hover: hover) {
    .sb-row-main:not(.is-selected):hover {
      background: var(--color-surface-sunken);
    }
  }
  .sb-row-main.is-selected {
    background: var(--color-accent-text);
    color: var(--color-on-accent);
  }
  .sb-row-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sb-row-title.is-locked {
    color: var(--color-text-muted);
  }
  .is-selected .sb-row-title.is-locked {
    color: inherit;
  }
  .sb-chapter-title {
    font-weight: 500;
  }
  .sb-part-title {
    font: 550 var(--text-ui) / 1.3 var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .sb-row-trail {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex: none;
    font-size: var(--text-small);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .sb-row-trail small {
    font-size: inherit;
  }
  .sb-chapter-title + .sb-row-trail::after {
    content: "";
    width: 8px;
  }
  .is-selected .sb-row-trail {
    color: inherit;
  }
  .sb-row:hover .sb-row-trail,
  .sb-row:focus-within .sb-row-trail {
    visibility: hidden;
  }
  .sb-row-menu {
    position: absolute;
    right: 0;
    top: 0;
    width: var(--control-target);
    min-height: var(--control-target);
    padding: 0;
    color: var(--color-text-muted);
    opacity: 0;
  }
  .sb-row:hover > .sb-row-menu,
  .sb-row:focus-within > .sb-row-menu {
    opacity: 1;
  }
  .sb-row.is-selected > .sb-row-menu {
    color: var(--color-on-accent);
  }
  @media (hover: hover) {
    .sb-row.is-selected > .sb-row-menu:hover {
      background: transparent;
      box-shadow: inset 0 0 0 1px currentColor;
    }
  }
  .sb-grip {
    position: absolute;
    left: -14px;
    top: 50%;
    display: flex;
    width: 14px;
    margin-top: -8px;
    color: var(--color-text-muted);
    cursor: grab;
    opacity: 0;
  }
  .sb-grip:active {
    cursor: grabbing;
  }
  .sb-row:hover > .sb-grip,
  .sb-row:focus-within > .sb-grip {
    opacity: 1;
  }
  .is-drop-target > .sb-row::before,
  .sb-row.is-drop-target::before,
  .sb-group.is-drop-target > .sb-chapter > .sb-row::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: -3px;
    height: 2px;
    border-radius: 2px;
    background: var(--color-accent-text);
  }
  .sidebar :global(.sb-chev) {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--color-text-muted);
    transition: transform var(--ka-motion) ease-out;
  }
  .sidebar [aria-expanded="true"] > :global(.sb-chev) {
    transform: rotate(90deg);
  }
  .sidebar :global(.sb-glyph) {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--color-text-muted);
  }
  .sidebar :global(.sb-glyph.is-warning) {
    color: var(--color-warning);
  }
  .sidebar .is-selected :global(.sb-glyph) {
    color: inherit;
  }

  /* Status: colour plus shape. Draft is a hollow ring; revised and final fill. */
  .sb-status-dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
  }
  .sb-status-dot.is-draft {
    box-shadow: inset 0 0 0 1.5px var(--color-text-muted);
  }
  .sb-status-dot.is-revised {
    background: var(--color-warning);
  }
  .sb-status-dot.is-final {
    background: var(--color-success);
  }
  .is-selected .sb-status-dot.is-draft {
    box-shadow: inset 0 0 0 1.5px currentColor;
  }
  .is-selected .sb-status-dot:is(.is-revised, .is-final) {
    background: currentColor;
  }

  .sb-children {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    min-width: 0;
    gap: 2px;
    margin: var(--space-3xs) 0 var(--space-2xs) var(--space-xs);
    padding-left: var(--space-2xs);
    border-left: var(--border-hair);
  }
  .sb-new {
    color: var(--color-text-muted);
  }
  .sb-new:hover {
    color: var(--color-text);
  }
  .sb-empty {
    padding: var(--space-3xs) var(--space-2xs);
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .sb-new-input {
    padding: var(--space-3xs) 0;
  }
  .sb-new-input input,
  .sb-save-filter input {
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-ui);
  }
  .sb-synopsis {
    margin: 0 0 var(--space-3xs) calc(var(--space-l) - 4px);
  }
  .sb-synopsis textarea {
    width: 100%;
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
    resize: vertical;
  }
  .sb-synopsis-text {
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text-muted);
    font: italic var(--text-small) / 1.5 var(--font-body);
    text-align: left;
    cursor: text;
  }
  .sb-synopsis-text:hover {
    background: var(--color-surface-sunken);
  }
  .sb-undefined {
    display: grid;
    gap: var(--space-2xs);
    padding: var(--space-2xs);
  }
  .sb-define {
    padding-top: var(--space-3xs);
  }
  .sb-small-button {
    justify-self: start;
    font-size: var(--text-small);
    padding-inline: var(--space-xs);
  }

  /* Scene filter: the shared segmented control at label scale. */
  /* Aligned with the scene rows below it (indent + rule + row padding). */
  .sb-filter {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin: var(--space-3xs) 0 var(--space-3xs) calc(var(--space-xs) + var(--space-2xs) + 1px);
  }
  .sb-filter-segments {
    flex: 1;
    min-width: 0;
  }
  .sb-filter-segments .ka-segment-track {
    flex-wrap: nowrap;
  }
  /* Sized to their labels (Press segments share width equally, which
     crowds "Planned" in the compact sidebar). */
  .sb-filter-segments .ka-segment {
    flex: 1 1 auto;
    min-width: 0;
    padding: var(--space-2xs) 6px;
    font-size: var(--text-small);
    white-space: nowrap;
  }
  /* In the compact sidebar the filter gives up its indent so all three
     segments keep their padding. */
  @media (max-width: 1280px) {
    .sb-filter {
      margin-left: var(--space-3xs);
    }
  }
  .sb-filter-button.is-active {
    color: var(--color-accent-text);
    box-shadow: inset 0 0 0 1px var(--color-accent-text);
  }
  .sb-filter-popover {
    position: absolute;
    right: 0;
    top: calc(100% + var(--space-3xs));
    z-index: var(--z-dropdown);
    display: grid;
    gap: var(--space-s);
    width: 260px;
    padding: var(--space-s);
  }
  .sb-filter-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .sb-filter-heading {
    font: 550 var(--text-ui) / 1.3 var(--font-display);
  }
  .sb-filter-field {
    display: grid;
    gap: var(--space-3xs);
  }
  .sb-filter-label,
  .sb-filter-field label {
    font: 500 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .sb-filter-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3xs);
  }
  .sb-type-chip {
    font-size: var(--text-small);
    color: var(--color-text-muted);
    background: transparent;
  }
  .sb-type-chip.is-on {
    color: var(--color-accent-text);
    border-color: var(--color-accent-text);
    background: var(--color-accent-wash);
  }
  .sb-filter-section {
    display: grid;
    gap: var(--space-3xs);
    padding-top: var(--space-2xs);
    border-top: var(--border-hair);
  }
  .sb-saved-filter {
    display: flex;
    align-items: center;
  }
  .sb-saved-name {
    flex: 1;
    min-width: 0;
    justify-content: flex-start;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sb-danger-icon:hover {
    color: var(--color-error);
    background: var(--color-error-wash) !important;
  }
  .sb-save-filter {
    display: flex;
    gap: var(--space-3xs);
  }

  /* Menus anchored to their trigger. */
  .sb-popover {
    position: absolute;
    right: 0;
    top: calc(100% + var(--space-3xs));
    z-index: var(--z-dropdown);
    margin: 0;
    width: 232px;
  }
  .sb-popover-up {
    top: auto;
    bottom: calc(100% + var(--space-3xs));
    left: 0;
    right: 0;
    width: auto;
  }
  /* Icon then label, left-aligned (Press spaces menu buttons apart for a
     trailing value; these items have none). */
  .sb-menuitem {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: var(--space-xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  .sb-menuitem :global(svg) {
    color: var(--color-text-muted);
  }
  @media (hover: hover) {
    .sb-menuitem:hover {
      background: var(--color-surface-sunken);
    }
  }

  .sb-foot {
    display: grid;
    gap: var(--space-3xs);
    padding: var(--space-xs);
    border-top: var(--border-hair);
  }
  .sb-split {
    display: flex;
  }
  .sb-split .ka-button:first-child {
    flex: 1;
    border-radius: var(--radius-xs) 0 0 var(--radius-xs);
  }
  .sb-split .ka-button:last-child {
    border-left: 0;
    border-radius: 0 var(--radius-xs) var(--radius-xs) 0;
  }
  .sb-settings {
    justify-content: flex-start;
  }
</style>
