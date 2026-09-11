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
    ChevronsLeft,
    ChevronsRight,
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
  import Tooltip from "./Tooltip.svelte";
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

  const sceneStatusClasses: Record<SceneStatus, string> = {
    draft: "bg-press-border",
    revised: "bg-press-warning",
    final: "bg-press-success",
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
  let hoveredChapterId: string | null = $state(null);
  let hoveredSceneId: string | null = $state(null);

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
        message: `This will delete "${chapter.title}" with ${counts.scene_count} scene${counts.scene_count !== 1 ? "s" : ""} and ${counts.beat_count} beat${counts.beat_count !== 1 ? "s" : ""}.`,
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
        message: `This will delete "${scene.title}" with ${beatCount} beat${beatCount !== 1 ? "s" : ""}.`,
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
      // Delete child chapters first (in reverse order to avoid index issues)
      for (const chapterId of [...partDeleteDialog.childChapterIds].reverse()) {
        await invoke("delete_chapter", { chapterId });
        currentProject.removeChapter(chapterId);
        // Clear selection if we deleted the current chapter
        if (currentProject.currentChapter?.id === chapterId) {
          currentProject.setCurrentChapter(null);
          currentProject.setScenes([]);
          currentProject.setCurrentScene(null);
          currentProject.setBeats([]);
        }
      }
      // Then delete the Part itself
      await invoke("delete_chapter", { chapterId: partDeleteDialog.partId });
      currentProject.removeChapter(partDeleteDialog.partId);
    } catch (e) {
      console.error("Failed to delete part and chapters:", e);
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

  $effect(() => {
    // These reads establish dependencies
    const project = currentProject.value;
    const chaptersLoaded = currentProject.chapters.length > 0;
    const importing = isImporting;

    if (project && !importing && !chaptersLoaded) {
      loadChapters(true);
      loadSavedFilters();
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
  class="bg-press-surface border-r border-press-border flex flex-col h-full transition-all duration-200"
  class:w-80={!ui.sidebarCollapsed}
  class:w-0={ui.sidebarCollapsed}
  class:overflow-hidden={ui.sidebarCollapsed}
  class:opacity-0={ui.sidebarCollapsed}
  class:border-r-0={ui.sidebarCollapsed}
  class:p-0={ui.sidebarCollapsed}
>
  <!-- Header -->
  <div class="p-4 border-b border-press-border">
    <div class="flex items-center justify-between">
      <BrandWordmark />
      <Tooltip text="Collapse sidebar" position="bottom">
        <button
          onclick={toggleSidebar}
          class="text-press-muted hover:text-press-text p-1"
          aria-label="Collapse sidebar"
        >
          <ChevronsLeft class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>
    {#if currentProject.value}
      <button
        type="button"
        data-testid="sidebar-home"
        onclick={goHome}
        class="mt-3 mb-3 w-full flex items-center gap-2 px-3 py-2 text-press-ui text-press-text bg-press-sunken hover:bg-press-accent-wash rounded-md transition-colors"
        aria-label="Home — all projects"
        title="Return home to all projects"
      >
        <Home class="w-4 h-4 text-press-accent-text" />
        <span class="font-medium">Home</span>
        <span class="ml-auto text-press-eyebrow text-press-muted">All projects</span>
      </button>
      <!-- Project name with action icons -->
      <div class="flex items-center justify-between mt-2 gap-2">
        <div class="flex items-center gap-2 min-w-0 flex-1">
          <p class="text-press-text text-press-base font-semibold truncate">
            {currentProject.value.name}
          </p>
          {#if currentProject.value.project_type === "screenplay" && pageCountEstimate}
            <span
              class="shrink-0 text-press-eyebrow text-press-muted bg-press-sunken px-1.5 py-0.5 rounded"
              title="{pageCountEstimate.words} words · target: {pageCountEstimate.target}"
            >
              {pageCountEstimate.pages.toFixed(1)} / {pageCountEstimate.target}
            </span>
          {/if}
        </div>
        <!-- Action icons (primary only; secondary behind more menu) -->
        <div class="flex items-center gap-0.5 shrink-0">
          {#if currentProject.value.source_path && supportsSync(currentProject.value.source_type)}
            <Tooltip text="Sync from source" position="bottom">
              <button
                data-testid="sync-button"
                onclick={handleSyncClick}
                disabled={loadingSyncPreview}
                class="p-1.5 text-press-muted hover:text-press-text hover:bg-press-sunken rounded transition-colors"
                aria-label="Sync from source"
              >
                <RefreshCw class="w-4 h-4 {loadingSyncPreview ? 'animate-spin' : ''}" />
              </button>
            </Tooltip>
          {/if}
          <div class="relative" bind:this={moreMenuRef}>
            <Tooltip text="More actions" position="bottom">
              <button
                onclick={() => (showMoreMenu = !showMoreMenu)}
                class="p-1.5 text-press-muted hover:text-press-text hover:bg-press-sunken rounded transition-colors"
                aria-label="More actions"
                data-testid="more-actions-button"
              >
                <MoreVertical class="w-4 h-4" />
              </button>
            </Tooltip>
            {#if showMoreMenu}
              <div
                class="absolute right-0 mt-1 w-48 bg-press-surface border border-press-border rounded-lg shadow-press-overlay py-1 z-press-dropdown"
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
                  class="w-full flex items-center gap-3 px-3 py-2 text-press-ui text-press-text hover:bg-press-sunken transition-colors"
                >
                  <Download class="w-4 h-4 text-press-muted" />
                  Export
                </button>
                <button
                  data-testid="snapshots-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showSnapshotsPanel = true;
                  }}
                  class="w-full flex items-center gap-3 px-3 py-2 text-press-ui text-press-text hover:bg-press-sunken transition-colors"
                >
                  <Clock class="w-4 h-4 text-press-muted" />
                  Snapshots
                </button>
                <button
                  data-testid="archive-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showArchivePanel = true;
                  }}
                  class="w-full flex items-center gap-3 px-3 py-2 text-press-ui text-press-text hover:bg-press-sunken transition-colors"
                >
                  <Archive class="w-4 h-4 text-press-muted" />
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
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-press-muted">Loading...</span>
      </div>
    {:else if currentProject.chapters.length === 0}
      <div class="flex items-center justify-center p-4">
        <span class="text-press-muted text-press-ui">No chapters found</span>
      </div>
    {:else}
      <nav class="space-y-1" aria-label="Project outline">
        {#each partGroups as group}
          <!-- Part header (if this group has a Part) -->
          {#if group.part}
            {@const part = group.part}
            {@const isPartExpanded = checkPartExpanded(part.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              data-testid="part-item"
              data-drag-chapter={part.id}
              class="select-none relative rounded-lg mt-4"
              class:ring-2={dragOverId === part.id}
              class:ring-press-focus={dragOverId === part.id}
              onmouseenter={() => (hoveredChapterId = part.id)}
              onmouseleave={() => (hoveredChapterId = null)}
            >
              <!-- Part row -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="w-full flex items-center gap-1 px-1 py-1.5 rounded-lg transition-colors group bg-press-accent-wash border-l-2 border-press-accent"
                oncontextmenu={(e) => openContextMenu(e, "chapter", part)}
              >
                <!-- Drag handle -->
                <div
                  data-testid="drag-handle"
                  onmousedown={(e) => onDragHandleMouseDown(e, "chapter", part.id)}
                  class="cursor-grab active:cursor-grabbing p-0.5 text-press-muted hover:text-press-text transition-opacity"
                  class:opacity-0={hoveredChapterId !== part.id}
                  class:opacity-100={hoveredChapterId === part.id}
                  role="button"
                  tabindex="-1"
                  aria-label="Drag to reorder"
                >
                  <GripVertical class="w-3.5 h-3.5" />
                </div>

                <button
                  onclick={() => togglePartExpanded(part.id)}
                  class="flex-1 flex items-center gap-1.5 text-left min-w-0"
                  aria-expanded={isPartExpanded}
                >
                  <ChevronRight
                    class="w-4 h-4 text-press-accent-text transition-transform shrink-0 {isPartExpanded
                      ? 'rotate-90'
                      : ''}"
                  />
                  {#if part.locked}
                    <Lock class="w-3 h-3 text-press-warning shrink-0" />
                  {:else if (part.planning_status ?? "fixed") === "flexible"}
                    <CircleDot class="w-3 h-3 shrink-0 text-press-warning" />
                  {:else if (part.planning_status ?? "fixed") === "undefined"}
                    <CircleDashed class="w-3 h-3 shrink-0 text-press-muted" />
                  {/if}
                  <span
                    data-testid="part-title"
                    class="font-semibold text-press-eyebrow uppercase tracking-wider truncate text-press-accent-text"
                    class:text-press-disabled-text={part.locked}>{part.title}</span
                  >
                </button>

                <!-- Three-dot menu button -->
                <button
                  data-testid="menu-button"
                  onclick={(e) => openContextMenu(e, "chapter", part)}
                  class="p-1 text-press-muted hover:text-press-text transition-opacity shrink-0"
                  class:opacity-0={hoveredChapterId !== part.id}
                  class:opacity-100={hoveredChapterId === part.id}
                  aria-label="{partLabel} menu"
                >
                  <MoreVertical class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          {/if}

          <!-- Chapters in this group (collapsible under Part) -->
          {#if !group.part || checkPartExpanded(group.part.id)}
            <div class={group.part ? "ml-2" : ""}>
              {#each group.chapters as chapter}
                {@const isExpanded = isChapterExpanded(chapter.id)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  data-testid="chapter-item"
                  data-drag-chapter={chapter.id}
                  class="select-none relative rounded-lg"
                  class:ring-2={dragOverId === chapter.id}
                  class:ring-press-focus={dragOverId === chapter.id}
                  onmouseenter={() => (hoveredChapterId = chapter.id)}
                  onmouseleave={() => (hoveredChapterId = null)}
                >
                  <!-- Chapter row -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="w-full flex flex-col gap-1 px-1 py-1.5 rounded-lg transition-colors group"
                    class:bg-press-sunken={isExpanded}
                    class:hover:bg-press-sunken={!isExpanded}
                    oncontextmenu={(e) => openContextMenu(e, "chapter", chapter)}
                  >
                    <div class="flex items-center gap-1">
                      <!-- Drag handle -->
                      <div
                        data-testid="drag-handle"
                        onmousedown={(e) => onDragHandleMouseDown(e, "chapter", chapter.id)}
                        class="cursor-grab active:cursor-grabbing p-0.5 text-press-muted hover:text-press-text transition-opacity"
                        class:opacity-0={hoveredChapterId !== chapter.id}
                        class:opacity-100={hoveredChapterId === chapter.id}
                        role="button"
                        tabindex="-1"
                        aria-label="Drag to reorder"
                      >
                        <GripVertical class="w-3.5 h-3.5" />
                      </div>

                      <button
                        onclick={() => toggleChapter(chapter)}
                        class="flex-1 flex items-center gap-1.5 text-left min-w-0"
                        aria-expanded={isExpanded}
                      >
                        <ChevronRight
                          class="w-4 h-4 text-press-muted transition-transform shrink-0 {isExpanded
                            ? 'rotate-90'
                            : ''}"
                        />
                        {#if chapter.locked}
                          <Lock class="w-3 h-3 text-press-warning shrink-0" />
                        {:else if (chapter.planning_status ?? "fixed") === "flexible"}
                          <CircleDot class="w-3 h-3 shrink-0 text-press-warning" />
                        {:else if (chapter.planning_status ?? "fixed") === "undefined"}
                          <CircleDashed class="w-3 h-3 shrink-0 text-press-muted" />
                        {/if}
                        <span
                          data-testid="chapter-title"
                          class="font-medium text-press-ui truncate text-press-text"
                          class:text-press-disabled-text={chapter.locked}>{chapter.title}</span
                        >
                        {#if writing.value?.chapter_words?.[chapter.id] !== undefined}
                          <span class="text-press-eyebrow text-press-muted shrink-0"
                            >{writing.value.chapter_words[chapter.id].toLocaleString()} words</span
                          >
                        {/if}
                      </button>

                      <!-- Three-dot menu button -->
                      <button
                        data-testid="menu-button"
                        onclick={(e) => openContextMenu(e, "chapter", chapter)}
                        class="p-1 text-press-muted hover:text-press-text transition-opacity shrink-0"
                        class:opacity-0={hoveredChapterId !== chapter.id}
                        class:opacity-100={hoveredChapterId === chapter.id}
                        aria-label="{chapterLabel} menu"
                      >
                        <MoreVertical class="w-3.5 h-3.5" />
                      </button>
                    </div>

                    {#if isExpanded && currentProject.currentChapter?.id === chapter.id}
                      {@const chapterPlanning = chapter.planning_status ?? "fixed"}

                      <!-- Chapter synopsis (Flexible/Undefined only) -->
                      {#if chapterPlanning !== "fixed"}
                        <div class="pl-6 pr-1 mt-1">
                          {#if editingChapterSynopsisId === chapter.id}
                            <!-- svelte-ignore a11y_autofocus -->
                            <textarea
                              bind:value={chapterSynopsisText}
                              oninput={() => handleChapterSynopsisInput(chapter.id)}
                              onblur={() => finishEditingChapterSynopsis(chapter.id)}
                              placeholder="{chapterLabel} synopsis..."
                              class="w-full text-press-eyebrow text-press-text bg-press-sunken border border-press-accent rounded-md px-2.5 py-1.5 resize-none focus:outline-none focus:border-press-accent"
                              rows="2"
                              autofocus
                            ></textarea>
                          {:else}
                            <button
                              onclick={() => startEditingChapterSynopsis(chapter)}
                              class="w-full text-left text-press-eyebrow rounded-md px-2.5 py-1.5 transition-colors hover:bg-press-sunken {chapter.synopsis
                                ? 'text-press-muted'
                                : 'text-press-muted italic'}"
                            >
                              {chapter.synopsis || "Add synopsis..."}
                            </button>
                          {/if}
                        </div>
                      {/if}

                      <!-- Undefined chapter: placeholder, no scene list -->
                      {#if chapterPlanning === "undefined"}
                        <div class="ml-5 mt-2 pl-2 border-l border-press-border/60">
                          <div
                            class="px-2 py-3 rounded-md bg-press-surface/50 border border-dashed border-press-border text-center"
                          >
                            <CircleDashed class="w-5 h-5 text-press-muted mx-auto mb-1.5" />
                            <p class="text-press-eyebrow text-press-muted">
                              This chapter is undefined. Add a synopsis and scenes will appear when
                              you promote it to Flexible or Fixed.
                            </p>
                            {#if !chapter.locked}
                              <button
                                onclick={() => setPlanningStatus("chapter", chapter, "flexible")}
                                class="mt-2 px-2.5 py-1 rounded-md bg-press-accent-wash text-press-accent-text text-press-eyebrow font-medium hover:text-press-text transition-colors"
                              >
                                Switch to Flexible
                              </button>
                            {/if}
                          </div>
                        </div>

                        <!-- Flexible chapter: simplified scene titles, no filters/drag -->
                      {:else if chapterPlanning === "flexible"}
                        <div class="ml-5 mt-1.5 space-y-0.5 border-l border-press-border/60 pl-2">
                          {#each filteredScenes as scene}
                            {@const isSelected = currentProject.currentScene?.id === scene.id}
                            <button
                              onclick={() => selectScene(scene)}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                              class="w-full flex items-center gap-1.5 text-left px-2 py-1 rounded-md text-press-ui transition-colors min-w-0"
                              class:bg-press-accent={isSelected}
                              class:text-press-on-accent={isSelected}
                              class:text-press-muted={!isSelected}
                              class:hover:bg-press-sunken={!isSelected}
                              class:hover:text-press-text={!isSelected}
                            >
                              {#if (scene.planning_status ?? "fixed") === "flexible"}
                                <CircleDot
                                  class="w-3 h-3 shrink-0 {isSelected
                                    ? 'text-press-on-accent'
                                    : 'text-press-warning'}"
                                />
                              {:else if (scene.planning_status ?? "fixed") === "undefined"}
                                <CircleDashed
                                  class="w-3 h-3 shrink-0 {isSelected
                                    ? 'text-press-on-accent'
                                    : 'text-press-muted'}"
                                />
                              {/if}
                              <span class="truncate">{scene.title}</span>
                              {#if writing.value?.scene_words?.[scene.id] !== undefined}
                                <span class="text-press-eyebrow text-press-muted"
                                  >{writing.value.scene_words[scene.id].toLocaleString()} words</span
                                >
                              {/if}
                            </button>
                          {/each}

                          {#if creatingScene}
                            <div class="px-2 py-1">
                              <!-- svelte-ignore a11y_autofocus -->
                              <input
                                data-testid="title-input"
                                type="text"
                                bind:value={newTitle}
                                onkeydown={handleCreateKeydown}
                                onblur={cancelCreate}
                                placeholder="Scene title..."
                                class="w-full px-2 py-1 text-press-ui bg-press-sunken border border-press-accent rounded focus:outline-none text-press-text"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="w-full flex items-center gap-2 px-2 py-1 rounded text-press-eyebrow text-press-muted hover:text-press-text hover:bg-press-sunken transition-colors"
                            >
                              <Plus class="w-3 h-3" />
                              New Scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="text-press-muted text-press-eyebrow px-2 py-1 italic"
                              >No scenes yet</span
                            >
                          {/if}

                          {#if !chapter.locked}
                            <div class="px-2 pt-1">
                              <button
                                onclick={() => setPlanningStatus("chapter", chapter, "fixed")}
                                class="text-press-eyebrow text-press-accent-text hover:underline"
                              >
                                Define full structure
                              </button>
                            </div>
                          {/if}
                        </div>

                        <!-- Fixed chapter: full scene list with filters, drag, icons -->
                      {:else}
                        <div class="flex items-center gap-1 pl-6 mt-1">
                          <!-- View toggle pills -->
                          <div
                            class="flex bg-press-sunken rounded-md overflow-hidden border border-press-border"
                          >
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-press-eyebrow font-medium transition-colors ${
                                outlineViewFilter === "all"
                                  ? "bg-press-accent text-press-on-accent"
                                  : "text-press-muted hover:text-press-text"
                              }`}
                              onclick={() => (outlineViewFilter = "all")}
                              title="Show all scenes"
                            >
                              All
                            </button>
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-press-eyebrow font-medium transition-colors ${
                                outlineViewFilter === "planned_only"
                                  ? "bg-press-accent text-press-on-accent"
                                  : "text-press-muted hover:text-press-text"
                              }`}
                              onclick={() => (outlineViewFilter = "planned_only")}
                              title="Show only planned scenes"
                            >
                              Planned
                            </button>
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-press-eyebrow font-medium transition-colors ${
                                outlineViewFilter === "next_5"
                                  ? "bg-press-accent text-press-on-accent"
                                  : "text-press-muted hover:text-press-text"
                              }`}
                              onclick={() => (outlineViewFilter = "next_5")}
                              title="Show next 5 scenes"
                            >
                              Next 5
                            </button>
                          </div>

                          <!-- Filter popover trigger -->
                          <div class="relative ml-auto" bind:this={filterPopoverRef}>
                            <button
                              type="button"
                              onclick={() => (showFilterPopover = !showFilterPopover)}
                              class={`p-1.5 rounded transition-colors ${
                                hasActiveFilters
                                  ? "text-press-accent-text bg-press-accent-wash"
                                  : "text-press-muted hover:text-press-text hover:bg-press-sunken"
                              }`}
                              title="Filter by type & status"
                            >
                              <Filter class="w-3.5 h-3.5" />
                            </button>
                            {#if showFilterPopover}
                              <div
                                class="absolute right-0 top-full mt-1 w-56 bg-press-surface border border-press-border rounded-lg shadow-press-overlay p-3 z-press-dropdown space-y-3"
                              >
                                <div class="flex items-center justify-between">
                                  <span
                                    class="text-press-eyebrow font-semibold text-press-text uppercase tracking-wide"
                                    >Filters</span
                                  >
                                  {#if hasActiveFilters}
                                    <button
                                      onclick={() => {
                                        sceneStatusFilter = "all";
                                        showNotesScenes = true;
                                        showTodoScenes = true;
                                        showUnusedScenes = true;
                                      }}
                                      class="text-press-eyebrow text-press-accent-text hover:underline"
                                    >
                                      Reset
                                    </button>
                                  {/if}
                                </div>

                                <!-- Type filter -->
                                <div class="space-y-1.5">
                                  <span class="text-press-eyebrow text-press-muted">Scene type</span
                                  >
                                  <div class="flex flex-wrap gap-1.5">
                                    {#each sceneTypeFilterOptions as option}
                                      {@const TypeIcon = option.icon}
                                      <button
                                        type="button"
                                        class={`flex items-center gap-1.5 px-2 py-1 rounded-md text-press-eyebrow transition-colors ${
                                          isSceneTypeVisible(option.type)
                                            ? "bg-press-accent-wash text-press-accent-text border border-press-accent"
                                            : "bg-press-sunken text-press-muted border border-transparent hover:text-press-text"
                                        }`}
                                        onclick={() => toggleSceneTypeVisible(option.type)}
                                        aria-pressed={isSceneTypeVisible(option.type)}
                                      >
                                        <TypeIcon class="w-3 h-3" />
                                        {option.label}
                                      </button>
                                    {/each}
                                  </div>
                                </div>

                                <!-- Status filter -->
                                <div class="space-y-1.5">
                                  <span class="text-press-eyebrow text-press-muted">Status</span>
                                  <div class="relative">
                                    <select
                                      bind:value={sceneStatusFilter}
                                      class="w-full appearance-none bg-press-sunken text-press-text text-press-eyebrow border border-press-border rounded-md px-2.5 py-1.5 focus:outline-none focus:border-press-accent cursor-pointer"
                                      aria-label="Scene status filter"
                                    >
                                      {#each sceneStatusOptions as option}
                                        <option value={option.value}>{option.label}</option>
                                      {/each}
                                    </select>
                                    <ChevronDown
                                      class="absolute right-2 top-1/2 -translate-y-1/2 w-3 h-3 text-press-muted pointer-events-none"
                                    />
                                  </div>
                                </div>

                                <!-- Saved filters -->
                                {#if savedFilters.length > 0}
                                  <div class="border-t border-press-border pt-2 space-y-1">
                                    <span class="text-press-eyebrow text-press-muted"
                                      >Saved filters</span
                                    >
                                    {#each savedFilters as filter}
                                      <div class="flex items-center gap-1">
                                        <button
                                          onclick={() => applySavedFilter(filter)}
                                          class="flex-1 text-left text-press-eyebrow px-2 py-1 rounded hover:bg-press-sunken text-press-text truncate"
                                        >
                                          {filter.name}
                                        </button>
                                        <button
                                          onclick={() => deleteSavedFilter(filter.id)}
                                          class="p-0.5 text-press-muted hover:text-press-error shrink-0"
                                          aria-label="Delete saved filter {filter.name}"
                                        >
                                          <Trash2 class="w-3 h-3" />
                                        </button>
                                      </div>
                                    {/each}
                                  </div>
                                {/if}

                                <!-- Save current filter -->
                                {#if hasActiveFilters}
                                  <div class="border-t border-press-border pt-2">
                                    {#if showSaveFilterInput}
                                      <div class="flex items-center gap-1">
                                        <input
                                          type="text"
                                          bind:value={savedFilterName}
                                          placeholder="Filter name..."
                                          class="flex-1 bg-press-sunken text-press-text text-press-eyebrow rounded px-2 py-1 focus:outline-none focus:ring-1 focus:ring-press-focus"
                                          onkeydown={(e) =>
                                            e.key === "Enter" && saveCurrentFilter()}
                                        />
                                        <button
                                          onclick={saveCurrentFilter}
                                          disabled={!savedFilterName.trim()}
                                          class="text-press-eyebrow text-press-accent-text hover:underline disabled:no-underline px-1"
                                        >
                                          Save
                                        </button>
                                      </div>
                                    {:else}
                                      <button
                                        onclick={() => (showSaveFilterInput = true)}
                                        class="text-press-eyebrow text-press-accent-text hover:underline"
                                      >
                                        Save current filter...
                                      </button>
                                    {/if}
                                  </div>
                                {/if}
                              </div>
                            {/if}
                          </div>
                        </div>

                        <div class="ml-5 mt-1.5 space-y-0.5 border-l border-press-border/60 pl-2">
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
                              class="relative flex items-center gap-1 py-0.5"
                              class:ring-2={dragOverId === scene.id}
                              class:ring-press-focus={dragOverId === scene.id}
                              onmouseenter={() => (hoveredSceneId = scene.id)}
                              onmouseleave={() => (hoveredSceneId = null)}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                            >
                              <!-- Scene drag handle -->
                              <div
                                data-testid="drag-handle"
                                onmousedown={(e) => onDragHandleMouseDown(e, "scene", scene.id)}
                                class="cursor-grab active:cursor-grabbing p-0.5 transition-opacity shrink-0"
                                class:text-press-on-accent={isSelected}
                                class:text-press-muted={!isSelected}
                                class:opacity-0={hoveredSceneId !== scene.id}
                                class:opacity-100={hoveredSceneId === scene.id}
                                role="button"
                                tabindex="-1"
                                aria-label="Drag to reorder"
                              >
                                <GripVertical class="w-3 h-3" />
                              </div>

                              <button
                                onclick={() => selectScene(scene)}
                                class="flex-1 flex items-center gap-1.5 text-left px-2 py-1.5 rounded-md text-press-ui transition-colors min-w-0"
                                class:bg-press-accent={isSelected}
                                class:text-press-on-accent={isSelected}
                                class:text-press-muted={!isSelected}
                                class:hover:bg-press-sunken={!isSelected}
                                class:hover:text-press-text={!isSelected}
                              >
                                <!-- Planning status / lock indicator (single leading icon) -->
                                {#if isLocked}
                                  <Lock
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-press-on-accent'
                                      : 'text-press-warning'}"
                                  />
                                {:else if planningStatus === "flexible"}
                                  <CircleDot
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-press-on-accent'
                                      : 'text-press-warning'}"
                                  />
                                {:else if planningStatus === "undefined"}
                                  <CircleDashed
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-press-on-accent'
                                      : 'text-press-muted'}"
                                  />
                                {/if}
                                <span
                                  data-testid="scene-title"
                                  class="truncate flex-1"
                                  class:text-press-disabled-text={isLocked}>{scene.title}</span
                                >
                                {#if writing.value?.scene_words?.[scene.id] !== undefined}
                                  <span class="text-press-eyebrow shrink-0"
                                    >{writing.value.scene_words[scene.id].toLocaleString()} words</span
                                  >
                                {/if}
                                <!-- Trailing badges: scene type + status dot -->
                                <span class="flex items-center gap-1 shrink-0 ml-auto">
                                  {#if sceneType !== "normal"}
                                    {@const SceneTypeIcon = sceneTypeFilterOptions.find(
                                      (option) => option.type === sceneType
                                    )?.icon}
                                    {#if SceneTypeIcon}
                                      <SceneTypeIcon
                                        class={`w-3 h-3 ${isSelected ? "text-press-on-accent" : "text-press-muted"}`}
                                        title={sceneTypeLabels[sceneType as SceneType]}
                                      />
                                    {/if}
                                  {/if}
                                  <span
                                    class={`w-1.5 h-1.5 rounded-full ${sceneStatusClasses[sceneStatus]} ${sceneStatus === "draft" ? "opacity-40" : ""}`}
                                    title={sceneStatusLabels[sceneStatus]}
                                  ></span>
                                </span>
                              </button>

                              <!-- Scene menu button -->
                              <button
                                data-testid="menu-button"
                                onclick={(e) => openContextMenu(e, "scene", scene)}
                                class="p-0.5 transition-opacity shrink-0"
                                class:text-press-on-accent={isSelected}
                                class:text-press-muted={!isSelected}
                                class:opacity-0={hoveredSceneId !== scene.id}
                                class:opacity-100={hoveredSceneId === scene.id}
                                aria-label="Scene menu"
                              >
                                <MoreVertical class="w-3 h-3" />
                              </button>
                            </div>
                          {/each}

                          <!-- New Scene Button or Input -->
                          {#if creatingScene}
                            <div class="px-2 py-1">
                              <!-- svelte-ignore a11y_autofocus -->
                              <input
                                data-testid="title-input"
                                type="text"
                                bind:value={newTitle}
                                onkeydown={handleCreateKeydown}
                                onblur={cancelCreate}
                                placeholder="Scene title..."
                                class="w-full px-2 py-1 text-press-ui bg-press-sunken border border-press-accent rounded focus:outline-none text-press-text"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="w-full flex items-center gap-2 px-2 py-1 rounded text-press-eyebrow text-press-muted hover:text-press-text hover:bg-press-sunken transition-colors"
                            >
                              <Plus class="w-3 h-3" />
                              New Scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="text-press-muted text-press-eyebrow px-2 py-1 italic"
                              >No scenes yet</span
                            >
                          {:else if filteredScenes.length === 0 && !creatingScene}
                            <span class="text-press-muted text-press-eyebrow px-2 py-1 italic"
                              >No scenes match filters</span
                            >
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

        <!-- New Chapter/Part Button or Input -->
        {#if creatingChapter || creatingPart}
          <div class="px-2 py-1">
            <!-- svelte-ignore a11y_autofocus -->
            <input
              data-testid="title-input"
              type="text"
              bind:value={newTitle}
              onkeydown={handleCreateKeydown}
              onblur={cancelCreate}
              placeholder={creatingPart ? `${partLabel} title...` : `${chapterLabel} title...`}
              class="w-full px-2 py-1 text-press-ui bg-press-sunken border border-press-accent rounded focus:outline-none text-press-text"
              autofocus
            />
          </div>
        {:else}
          <!-- Split button: New Chapter (default) with dropdown for New Part -->
          <div class="relative mt-2" bind:this={newButtonRef}>
            <div
              class="flex items-stretch rounded-lg bg-press-sunken border border-press-border hover:border-press-accent transition-colors"
            >
              <!-- Main action: New Chapter/Sequence -->
              <button
                data-testid="new-chapter-button"
                onclick={startCreatingChapter}
                class="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors rounded-l-lg"
              >
                <Plus class="w-4 h-4" />
                New {chapterLabel}
              </button>
              <!-- Dropdown trigger -->
              <button
                data-testid="new-dropdown-button"
                onclick={() => (showNewDropdown = !showNewDropdown)}
                class="px-2 py-2 text-press-muted hover:text-press-text transition-colors border-l border-press-border hover:bg-press-surface rounded-r-lg"
                aria-label="More options"
              >
                <ChevronDown class="w-4 h-4" />
              </button>
            </div>

            <!-- Dropdown menu -->
            {#if showNewDropdown}
              <div
                class="absolute left-0 right-0 mt-1 bg-press-surface border border-press-border rounded-lg shadow-press-overlay py-1 z-press-dropdown"
              >
                <button
                  data-testid="dropdown-new-chapter"
                  onclick={startCreatingChapter}
                  class="w-full flex items-center gap-2 px-3 py-2 text-press-ui text-press-text hover:bg-press-sunken transition-colors"
                >
                  <Folder class="w-4 h-4" />
                  New {chapterLabel}
                </button>
                <button
                  data-testid="dropdown-new-part"
                  onclick={startCreatingPart}
                  class="w-full flex items-center gap-2 px-3 py-2 text-press-ui text-press-text hover:bg-press-sunken transition-colors"
                >
                  <BookOpen class="w-4 h-4" />
                  New {partLabel}
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </nav>
    {/if}
  </div>
  {#if onOpenSettings}
    <footer class="shrink-0 border-t border-press-border px-3 py-2" inert={ui.sidebarCollapsed}>
      <button
        type="button"
        data-testid="sidebar-settings-button"
        onclick={onOpenSettings}
        class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-press-ui text-press-muted hover:bg-press-sunken hover:text-press-text transition-colors"
      >
        <Settings class="w-4 h-4" />
        Settings
      </button>
    </footer>
  {/if}
</aside>

<!-- Collapsed sidebar toggle -->
{#if ui.sidebarCollapsed}
  <Tooltip text="Expand sidebar" position="right">
    <button
      onclick={toggleSidebar}
      class="fixed left-0 top-1/2 -translate-y-1/2 bg-press-surface p-2 rounded-r-lg text-press-muted hover:text-press-text z-press-raised"
      aria-label="Expand sidebar"
    >
      <ChevronsRight class="w-5 h-5" />
    </button>
  </Tooltip>
{/if}

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
    title="Delete {deleteDialog.type === 'chapter' ? 'Chapter' : 'Scene'}"
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
  <SnapshotsPanel onClose={() => (showSnapshotsPanel = false)} />
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
