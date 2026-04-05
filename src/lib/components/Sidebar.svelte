<!--
  Sidebar.svelte - Main navigation sidebar

  Displays the project chapter/scene tree with:
  - Drag-and-drop reordering
  - Context menus for actions
  - Create/delete functionality
  - Sync button for reimporting
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
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
    Settings,
    BookOpen,
    StickyNote,
    CheckSquare,
    EyeOff,
    CircleDot,
    CircleDashed,
    Filter,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
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
    Project,
  } from "../types";
  import ArchivePanel from "./ArchivePanel.svelte";
  import ProjectSettingsDialog from "./ProjectSettingsDialog.svelte";
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

  let loading = $state(false);
  let chaptersRequestId = 0;
  let scenesRequestId = 0;
  let beatsRequestId = 0;
  let expandedChapters = new SvelteSet<string>();
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
    draft: "bg-text-secondary/40",
    revised: "bg-warning",
    final: "bg-success",
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

  // Project settings dialog state
  let showSettingsDialog = $state(false);

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

  async function loadChapters() {
    if (!currentProject.value) return;
    const projectId = currentProject.value.id;
    const requestId = ++chaptersRequestId;

    loading = true;
    try {
      const chapters = await invoke<Chapter[]>("get_chapters", {
        projectId,
      });
      if (requestId !== chaptersRequestId || currentProject.value?.id !== projectId) return;

      currentProject.setChapters(chapters);

      // Auto-expand all Parts
      expandedParts.clear();
      for (const chapter of chapters) {
        if (chapter.is_part) {
          expandedParts.add(chapter.id);
        }
      }

      // Auto-expand first non-Part chapter if any exist
      const firstChapter = chapters.find((c) => !c.is_part);
      if (firstChapter) {
        expandedChapters.clear();
        expandedChapters.add(firstChapter.id);
        await loadScenes(firstChapter);
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

  async function loadScenes(chapter: Chapter) {
    const requestId = ++scenesRequestId;
    const chapterId = chapter.id;
    currentProject.setCurrentChapter(chapter);
    try {
      const scenes = await invoke<Scene[]>("get_scenes", {
        chapterId: chapter.id,
      });
      if (requestId !== scenesRequestId || currentProject.currentChapter?.id !== chapterId) return;
      currentProject.setScenes(scenes);
      if (scenes.length === 1 && currentProject.value?.project_type === "screenplay") {
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

  function goHome() {
    currentProject.setProject(null);
    ui.setView("start");
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
    return () => window.removeEventListener("kindling:sync", handler);
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
    contextMenu = {
      type,
      id: item.id,
      x: e.clientX,
      y: e.clientY,
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
      loadChapters();
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
  class="bg-bg-panel border-r border-bg-card flex flex-col h-full transition-all duration-200"
  class:w-80={!ui.sidebarCollapsed}
  class:w-0={ui.sidebarCollapsed}
  class:overflow-hidden={ui.sidebarCollapsed}
  class:opacity-0={ui.sidebarCollapsed}
  class:border-r-0={ui.sidebarCollapsed}
  class:p-0={ui.sidebarCollapsed}
>
  <!-- Header -->
  <div class="p-4 border-b border-bg-card">
    <div class="flex items-center justify-between">
      <span class="flex items-center gap-2 text-accent font-heading font-medium text-lg">
        <!-- Mini Logo Mark -->
        <svg width="24" height="24" viewBox="0 0 1024 1024" class="shrink-0">
          <defs>
            <linearGradient
              id="sidebarBookGrad"
              x1="509"
              y1="739"
              x2="512"
              y2="609"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0" stop-color="#501D0F" />
              <stop offset="1" stop-color="#89492B" />
            </linearGradient>
          </defs>
          <path
            fill="#E25227"
            d="M495.154 288.138C498.378 289.608 505.914 297.445 508.313 300.3C526.269 321.669 539.502 342.79 542.378 370.879C549.115 436.662 490.007 467.903 476.848 526.209C472.415 545.849 474.731 568.443 482.366 587.122C483.763 590.541 490.702 602.324 490.569 604.62L489.492 604.081C466.698 587.526 440.031 561.25 430.639 534.248C403.556 456.377 485.481 402.143 496.346 330.247C498.679 314.804 498.133 303.222 495.154 288.138Z"
          />
          <path
            fill="url(#sidebarBookGrad)"
            d="M679.512 611.655C679.948 623.671 679.803 636.504 679.711 648.539C679.819 650.345 679.874 650.354 679.431 652.203C678.578 653.105 645.852 669.482 641.946 671.541L551.504 719.091C543.78 723.161 536.109 727.33 528.491 731.597C523.974 734.127 516.055 738.826 511.383 740.578C504.39 737.13 495.509 731.912 488.494 728.114L438.452 701.202C418.928 690.993 399.491 680.618 380.143 670.078C368.598 663.83 355.674 656.975 344.543 650.136C344.526 637.556 344.602 624.446 344.219 611.898C359.414 619.412 379.065 631.083 394.357 639.52L470.64 681.021C479.247 685.796 487.81 690.649 496.33 695.578C500.794 698.136 506.902 701.896 511.48 703.945C532.487 690.677 560.415 676.473 582.602 664.63C615.066 647.267 647.37 629.608 679.512 611.655Z"
          />
          <path
            fill="#F0912D"
            d="M567.225 404.156C568.003 404.556 568.509 404.868 568.965 405.666C588.192 439.301 602.938 484.462 595.366 523.183C587.91 561.316 558.078 585.951 527.823 605.935L518.591 611.429C510.152 597.693 506.392 586.985 503.912 571.209C497.26 528.911 522.684 499.522 542.221 465.408C552.466 447.518 562.786 424.502 567.225 404.156Z"
          />
          <path
            fill="#F0912D"
            d="M359.24 550.125C365.269 552.715 379.71 564.412 385.223 568.751C425.497 600.45 464.809 634.729 496.049 675.611C499.494 680.119 508.175 690.937 510.126 695.939C503.857 692.741 497.548 689.208 491.532 685.547C448.751 659.511 402.641 638.037 359.663 612.561C359.387 591.884 359.872 570.732 359.24 550.125Z"
          />
          <path
            fill="#F0912D"
            d="M664.174 549.059L664.428 593.205C664.417 599.159 664.625 607.179 664.213 612.947C655.817 616.909 647.229 621.897 639.067 626.408L603.341 646.032C582.669 657.264 562.058 668.608 541.509 680.063C534.744 683.835 526.75 687.959 520.246 691.793C518.071 693.047 515.906 694.089 513.66 695.2L519.513 687.005C556.887 634.717 612.459 587.041 664.174 549.059Z"
          />
        </svg>
        kindling
      </span>
      <Tooltip text="Collapse sidebar" position="bottom">
        <button
          onclick={toggleSidebar}
          class="text-text-secondary hover:text-text-primary p-1"
          aria-label="Collapse sidebar"
        >
          <ChevronsLeft class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>
    {#if currentProject.value}
      <!-- Project name with action icons -->
      <div class="flex items-center justify-between mt-2 gap-2">
        <div class="flex items-center gap-2 min-w-0 flex-1">
          <p class="text-text-primary text-base font-semibold truncate">
            {currentProject.value.name}
          </p>
          {#if currentProject.value.project_type === "screenplay" && pageCountEstimate}
            <span
              class="shrink-0 text-xs text-text-secondary bg-bg-card px-1.5 py-0.5 rounded"
              title="{pageCountEstimate.words} words · target: {pageCountEstimate.target}"
            >
              {pageCountEstimate.pages.toFixed(1)} / {pageCountEstimate.target}
            </span>
          {/if}
        </div>
        <!-- Action icons (primary only; secondary behind more menu) -->
        <div class="flex items-center gap-0.5 shrink-0">
          <Tooltip text="Project settings" position="bottom">
            <button
              data-testid="settings-button"
              onclick={() => (showSettingsDialog = true)}
              class="p-1.5 text-text-secondary hover:text-text-primary hover:bg-bg-card rounded transition-colors"
              aria-label="Project settings"
            >
              <Settings class="w-4 h-4" />
            </button>
          </Tooltip>
          {#if currentProject.value.source_path}
            <Tooltip text="Sync from source" position="bottom">
              <button
                data-testid="sync-button"
                onclick={handleSyncClick}
                disabled={loadingSyncPreview}
                class="p-1.5 text-text-secondary hover:text-text-primary hover:bg-bg-card rounded transition-colors disabled:opacity-50"
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
                class="p-1.5 text-text-secondary hover:text-text-primary hover:bg-bg-card rounded transition-colors"
                aria-label="More actions"
              >
                <MoreVertical class="w-4 h-4" />
              </button>
            </Tooltip>
            {#if showMoreMenu}
              <div
                class="absolute right-0 mt-1 w-48 bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 z-50"
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
                  class="w-full flex items-center gap-3 px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors"
                >
                  <Download class="w-4 h-4 text-text-secondary" />
                  Export
                </button>
                <button
                  data-testid="snapshots-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showSnapshotsPanel = true;
                  }}
                  class="w-full flex items-center gap-3 px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors"
                >
                  <Clock class="w-4 h-4 text-text-secondary" />
                  Snapshots
                </button>
                <button
                  data-testid="archive-button"
                  onclick={() => {
                    showMoreMenu = false;
                    showArchivePanel = true;
                  }}
                  class="w-full flex items-center gap-3 px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors"
                >
                  <Archive class="w-4 h-4 text-text-secondary" />
                  Archive
                </button>
              </div>
            {/if}
          </div>
        </div>
      </div>
      <button
        onclick={goHome}
        class="mt-3 w-full flex items-center gap-2 px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary rounded-md hover:bg-bg-card transition-colors"
        aria-label="Close project"
      >
        <Home class="w-3.5 h-3.5" />
        All Projects
      </button>
    {/if}
  </div>

  <!-- Chapter/Scene Tree -->
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary">Loading...</span>
      </div>
    {:else if currentProject.chapters.length === 0}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary text-sm">No chapters found</span>
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
              class:ring-accent={dragOverId === part.id}
              onmouseenter={() => (hoveredChapterId = part.id)}
              onmouseleave={() => (hoveredChapterId = null)}
            >
              <!-- Part row -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="w-full flex items-center gap-1 px-1 py-1.5 rounded-lg transition-colors group bg-accent/10 border-l-2 border-accent"
                oncontextmenu={(e) => openContextMenu(e, "chapter", part)}
              >
                <!-- Drag handle -->
                <div
                  data-testid="drag-handle"
                  onmousedown={(e) => onDragHandleMouseDown(e, "chapter", part.id)}
                  class="cursor-grab active:cursor-grabbing p-0.5 text-text-secondary hover:text-text-primary transition-opacity"
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
                    class="w-4 h-4 text-accent transition-transform shrink-0 {isPartExpanded
                      ? 'rotate-90'
                      : ''}"
                  />
                  {#if part.locked}
                    <Lock class="w-3 h-3 text-amber-500 shrink-0" />
                  {:else if (part.planning_status ?? "fixed") === "flexible"}
                    <CircleDot class="w-3 h-3 shrink-0 text-amber-500/70" />
                  {:else if (part.planning_status ?? "fixed") === "undefined"}
                    <CircleDashed class="w-3 h-3 shrink-0 text-text-secondary/50" />
                  {/if}
                  <span
                    data-testid="part-title"
                    class="font-semibold text-xs uppercase tracking-wider truncate text-accent"
                    class:opacity-60={part.locked}>{part.title}</span
                  >
                </button>

                <!-- Three-dot menu button -->
                <button
                  data-testid="menu-button"
                  onclick={(e) => openContextMenu(e, "chapter", part)}
                  class="p-1 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
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
                  class:ring-accent={dragOverId === chapter.id}
                  onmouseenter={() => (hoveredChapterId = chapter.id)}
                  onmouseleave={() => (hoveredChapterId = null)}
                >
                  <!-- Chapter row -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="w-full flex flex-col gap-1 px-1 py-1.5 rounded-lg transition-colors group"
                    class:bg-bg-card={isExpanded}
                    class:hover:bg-bg-card={!isExpanded}
                    oncontextmenu={(e) => openContextMenu(e, "chapter", chapter)}
                  >
                    <div class="flex items-center gap-1">
                      <!-- Drag handle -->
                      <div
                        data-testid="drag-handle"
                        onmousedown={(e) => onDragHandleMouseDown(e, "chapter", chapter.id)}
                        class="cursor-grab active:cursor-grabbing p-0.5 text-text-secondary hover:text-text-primary transition-opacity"
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
                          class="w-4 h-4 text-text-secondary transition-transform shrink-0 {isExpanded
                            ? 'rotate-90'
                            : ''}"
                        />
                        {#if chapter.locked}
                          <Lock class="w-3 h-3 text-amber-500 shrink-0" />
                        {:else if (chapter.planning_status ?? "fixed") === "flexible"}
                          <CircleDot class="w-3 h-3 shrink-0 text-amber-500/70" />
                        {:else if (chapter.planning_status ?? "fixed") === "undefined"}
                          <CircleDashed class="w-3 h-3 shrink-0 text-text-secondary/50" />
                        {/if}
                        <span
                          data-testid="chapter-title"
                          class="font-medium text-sm truncate text-text-primary"
                          class:opacity-60={chapter.locked}>{chapter.title}</span
                        >
                      </button>

                      <!-- Three-dot menu button -->
                      <button
                        data-testid="menu-button"
                        onclick={(e) => openContextMenu(e, "chapter", chapter)}
                        class="p-1 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
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
                              class="w-full text-xs text-text-primary bg-bg-card border border-accent/40 rounded-md px-2.5 py-1.5 resize-none focus:outline-none focus:border-accent"
                              rows="2"
                              autofocus
                            ></textarea>
                          {:else}
                            <button
                              onclick={() => startEditingChapterSynopsis(chapter)}
                              class="w-full text-left text-xs rounded-md px-2.5 py-1.5 transition-colors hover:bg-bg-card {chapter.synopsis
                                ? 'text-text-secondary'
                                : 'text-text-secondary/50 italic'}"
                            >
                              {chapter.synopsis || "Add synopsis..."}
                            </button>
                          {/if}
                        </div>
                      {/if}

                      <!-- Undefined chapter: placeholder, no scene list -->
                      {#if chapterPlanning === "undefined"}
                        <div class="ml-5 mt-2 pl-2 border-l border-bg-card/60">
                          <div
                            class="px-2 py-3 rounded-md bg-bg-panel/50 border border-dashed border-bg-card text-center"
                          >
                            <CircleDashed class="w-5 h-5 text-text-secondary/40 mx-auto mb-1.5" />
                            <p class="text-xs text-text-secondary">
                              This chapter is undefined. Add a synopsis and scenes will appear when
                              you promote it to Flexible or Fixed.
                            </p>
                            {#if !chapter.locked}
                              <button
                                onclick={() => setPlanningStatus("chapter", chapter, "flexible")}
                                class="mt-2 px-2.5 py-1 rounded-md bg-accent/10 text-accent text-xs font-medium hover:bg-accent/20 transition-colors"
                              >
                                Switch to Flexible
                              </button>
                            {/if}
                          </div>
                        </div>

                        <!-- Flexible chapter: simplified scene titles, no filters/drag -->
                      {:else if chapterPlanning === "flexible"}
                        <div class="ml-5 mt-1.5 space-y-0.5 border-l border-bg-card/60 pl-2">
                          {#each filteredScenes as scene}
                            {@const isSelected = currentProject.currentScene?.id === scene.id}
                            <button
                              onclick={() => selectScene(scene)}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                              class="w-full flex items-center gap-1.5 text-left px-2 py-1 rounded-md text-sm transition-colors min-w-0"
                              class:bg-accent={isSelected}
                              class:text-white={isSelected}
                              class:text-text-secondary={!isSelected}
                              class:hover:bg-bg-card={!isSelected}
                              class:hover:text-text-primary={!isSelected}
                            >
                              {#if (scene.planning_status ?? "fixed") === "flexible"}
                                <CircleDot
                                  class="w-3 h-3 shrink-0 {isSelected
                                    ? 'text-white/80'
                                    : 'text-amber-500/70'}"
                                />
                              {:else if (scene.planning_status ?? "fixed") === "undefined"}
                                <CircleDashed
                                  class="w-3 h-3 shrink-0 {isSelected
                                    ? 'text-white/80'
                                    : 'text-text-secondary/50'}"
                                />
                              {/if}
                              <span class="truncate">{scene.title}</span>
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
                                class="w-full px-2 py-1 text-sm bg-bg-card border border-accent rounded focus:outline-none text-text-primary"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-text-secondary hover:text-text-primary hover:bg-bg-card transition-colors"
                            >
                              <Plus class="w-3 h-3" />
                              New Scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="text-text-secondary text-xs px-2 py-1 italic"
                              >No scenes yet</span
                            >
                          {/if}

                          {#if !chapter.locked}
                            <div class="px-2 pt-1">
                              <button
                                onclick={() => setPlanningStatus("chapter", chapter, "fixed")}
                                class="text-xs text-accent hover:underline"
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
                            class="flex bg-bg-card rounded-md overflow-hidden border border-text-secondary/20"
                          >
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-xs font-medium transition-colors ${
                                outlineViewFilter === "all"
                                  ? "bg-accent text-white"
                                  : "text-text-secondary hover:text-text-primary"
                              }`}
                              onclick={() => (outlineViewFilter = "all")}
                              title="Show all scenes"
                            >
                              All
                            </button>
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-xs font-medium transition-colors ${
                                outlineViewFilter === "planned_only"
                                  ? "bg-accent text-white"
                                  : "text-text-secondary hover:text-text-primary"
                              }`}
                              onclick={() => (outlineViewFilter = "planned_only")}
                              title="Show only planned scenes"
                            >
                              Planned
                            </button>
                            <button
                              type="button"
                              class={`px-2.5 py-1 text-xs font-medium transition-colors ${
                                outlineViewFilter === "next_5"
                                  ? "bg-accent text-white"
                                  : "text-text-secondary hover:text-text-primary"
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
                                  ? "text-accent bg-accent/10"
                                  : "text-text-secondary hover:text-text-primary hover:bg-bg-card"
                              }`}
                              title="Filter by type & status"
                            >
                              <Filter class="w-3.5 h-3.5" />
                            </button>
                            {#if showFilterPopover}
                              <div
                                class="absolute right-0 top-full mt-1 w-56 bg-bg-panel border border-bg-card rounded-lg shadow-lg p-3 z-50 space-y-3"
                              >
                                <div class="flex items-center justify-between">
                                  <span
                                    class="text-xs font-semibold text-text-primary uppercase tracking-wide"
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
                                      class="text-xs text-accent hover:underline"
                                    >
                                      Reset
                                    </button>
                                  {/if}
                                </div>

                                <!-- Type filter -->
                                <div class="space-y-1.5">
                                  <span class="text-xs text-text-secondary">Scene type</span>
                                  <div class="flex flex-wrap gap-1.5">
                                    {#each sceneTypeFilterOptions as option}
                                      {@const TypeIcon = option.icon}
                                      <button
                                        type="button"
                                        class={`flex items-center gap-1.5 px-2 py-1 rounded-md text-xs transition-colors ${
                                          isSceneTypeVisible(option.type)
                                            ? "bg-accent/15 text-accent border border-accent/30"
                                            : "bg-bg-card text-text-secondary border border-transparent hover:text-text-primary"
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
                                  <span class="text-xs text-text-secondary">Status</span>
                                  <div class="relative">
                                    <select
                                      bind:value={sceneStatusFilter}
                                      class="w-full appearance-none bg-bg-card text-text-primary text-xs border border-text-secondary/20 rounded-md px-2.5 py-1.5 focus:outline-none focus:border-accent cursor-pointer"
                                      aria-label="Scene status filter"
                                    >
                                      {#each sceneStatusOptions as option}
                                        <option value={option.value}>{option.label}</option>
                                      {/each}
                                    </select>
                                    <ChevronDown
                                      class="absolute right-2 top-1/2 -translate-y-1/2 w-3 h-3 text-text-secondary pointer-events-none"
                                    />
                                  </div>
                                </div>

                                <!-- Saved filters -->
                                {#if savedFilters.length > 0}
                                  <div class="border-t border-bg-card pt-2 space-y-1">
                                    <span class="text-xs text-text-secondary">Saved filters</span>
                                    {#each savedFilters as filter}
                                      <div class="flex items-center gap-1">
                                        <button
                                          onclick={() => applySavedFilter(filter)}
                                          class="flex-1 text-left text-xs px-2 py-1 rounded hover:bg-bg-card text-text-primary truncate"
                                        >
                                          {filter.name}
                                        </button>
                                        <button
                                          onclick={() => deleteSavedFilter(filter.id)}
                                          class="p-0.5 text-text-secondary hover:text-red-400 shrink-0"
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
                                  <div class="border-t border-bg-card pt-2">
                                    {#if showSaveFilterInput}
                                      <div class="flex items-center gap-1">
                                        <input
                                          type="text"
                                          bind:value={savedFilterName}
                                          placeholder="Filter name..."
                                          class="flex-1 bg-bg-card text-text-primary text-xs rounded px-2 py-1 focus:outline-none focus:ring-1 focus:ring-accent"
                                          onkeydown={(e) =>
                                            e.key === "Enter" && saveCurrentFilter()}
                                        />
                                        <button
                                          onclick={saveCurrentFilter}
                                          disabled={!savedFilterName.trim()}
                                          class="text-xs text-accent hover:underline disabled:opacity-40 disabled:no-underline px-1"
                                        >
                                          Save
                                        </button>
                                      </div>
                                    {:else}
                                      <button
                                        onclick={() => (showSaveFilterInput = true)}
                                        class="text-xs text-accent hover:underline"
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

                        <div class="ml-5 mt-1.5 space-y-0.5 border-l border-bg-card/60 pl-2">
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
                              class:ring-accent={dragOverId === scene.id}
                              onmouseenter={() => (hoveredSceneId = scene.id)}
                              onmouseleave={() => (hoveredSceneId = null)}
                              oncontextmenu={(e) => openContextMenu(e, "scene", scene)}
                            >
                              <!-- Scene drag handle -->
                              <div
                                data-testid="drag-handle"
                                onmousedown={(e) => onDragHandleMouseDown(e, "scene", scene.id)}
                                class="cursor-grab active:cursor-grabbing p-0.5 transition-opacity shrink-0"
                                class:text-white={isSelected}
                                class:text-text-secondary={!isSelected}
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
                                class="flex-1 flex items-center gap-1.5 text-left px-2 py-1.5 rounded-md text-sm transition-colors min-w-0"
                                class:bg-accent={isSelected}
                                class:text-white={isSelected}
                                class:text-text-secondary={!isSelected}
                                class:hover:bg-bg-card={!isSelected}
                                class:hover:text-text-primary={!isSelected}
                              >
                                <!-- Planning status / lock indicator (single leading icon) -->
                                {#if isLocked}
                                  <Lock
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-white'
                                      : 'text-amber-500'}"
                                  />
                                {:else if planningStatus === "flexible"}
                                  <CircleDot
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-white/80'
                                      : 'text-amber-500/70'}"
                                  />
                                {:else if planningStatus === "undefined"}
                                  <CircleDashed
                                    class="w-3 h-3 shrink-0 {isSelected
                                      ? 'text-white/80'
                                      : 'text-text-secondary/50'}"
                                  />
                                {/if}
                                <span
                                  data-testid="scene-title"
                                  class="truncate flex-1"
                                  class:opacity-60={isLocked}>{scene.title}</span
                                >
                                <!-- Trailing badges: scene type + status dot -->
                                <span class="flex items-center gap-1 shrink-0 ml-auto">
                                  {#if sceneType !== "normal"}
                                    {@const SceneTypeIcon = sceneTypeFilterOptions.find(
                                      (option) => option.type === sceneType
                                    )?.icon}
                                    {#if SceneTypeIcon}
                                      <SceneTypeIcon
                                        class={`w-3 h-3 ${isSelected ? "text-white/70" : "text-text-secondary/60"}`}
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
                                class:text-white={isSelected}
                                class:text-text-secondary={!isSelected}
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
                                class="w-full px-2 py-1 text-sm bg-bg-card border border-accent rounded focus:outline-none text-text-primary"
                                autofocus
                              />
                            </div>
                          {:else}
                            <button
                              data-testid="new-scene-button"
                              onclick={startCreatingScene}
                              class="w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-text-secondary hover:text-text-primary hover:bg-bg-card transition-colors"
                            >
                              <Plus class="w-3 h-3" />
                              New Scene
                            </button>
                          {/if}

                          {#if currentProject.scenes.length === 0 && !creatingScene}
                            <span class="text-text-secondary text-xs px-2 py-1 italic"
                              >No scenes yet</span
                            >
                          {:else if filteredScenes.length === 0 && !creatingScene}
                            <span class="text-text-secondary text-xs px-2 py-1 italic"
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
              class="w-full px-2 py-1 text-sm bg-bg-card border border-accent rounded focus:outline-none text-text-primary"
              autofocus
            />
          </div>
        {:else}
          <!-- Split button: New Chapter (default) with dropdown for New Part -->
          <div class="relative mt-2" bind:this={newButtonRef}>
            <div
              class="flex items-stretch rounded-lg bg-bg-card border border-bg-card hover:border-accent/50 transition-colors"
            >
              <!-- Main action: New Chapter/Sequence -->
              <button
                data-testid="new-chapter-button"
                onclick={startCreatingChapter}
                class="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors rounded-l-lg"
              >
                <Plus class="w-4 h-4" />
                New {chapterLabel}
              </button>
              <!-- Dropdown trigger -->
              <button
                data-testid="new-dropdown-button"
                onclick={() => (showNewDropdown = !showNewDropdown)}
                class="px-2 py-2 text-text-secondary hover:text-text-primary transition-colors border-l border-bg-panel hover:bg-bg-panel rounded-r-lg"
                aria-label="More options"
              >
                <ChevronDown class="w-4 h-4" />
              </button>
            </div>

            <!-- Dropdown menu -->
            {#if showNewDropdown}
              <div
                class="absolute left-0 right-0 mt-1 bg-bg-panel border border-bg-card rounded-lg shadow-lg py-1 z-50"
              >
                <button
                  data-testid="dropdown-new-chapter"
                  onclick={startCreatingChapter}
                  class="w-full flex items-center gap-2 px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors"
                >
                  <Folder class="w-4 h-4" />
                  New {chapterLabel}
                </button>
                <button
                  data-testid="dropdown-new-part"
                  onclick={startCreatingPart}
                  class="w-full flex items-center gap-2 px-3 py-2 text-sm text-text-primary hover:bg-bg-card transition-colors"
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
</aside>

<!-- Collapsed sidebar toggle -->
{#if ui.sidebarCollapsed}
  <Tooltip text="Expand sidebar" position="right">
    <button
      onclick={toggleSidebar}
      class="fixed left-0 top-1/2 -translate-y-1/2 bg-bg-panel p-2 rounded-r-lg text-text-secondary hover:text-text-primary z-10"
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

<!-- Project Settings Dialog -->
{#if showSettingsDialog}
  <ProjectSettingsDialog
    onClose={() => (showSettingsDialog = false)}
    onSave={(updatedProject: Project) => {
      currentProject.setProject(updatedProject);
      showSettingsDialog = false;
    }}
  />
{/if}
