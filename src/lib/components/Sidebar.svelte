<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
    FileText,
    Folder,
    Home,
    Plus,
    Trash2,
    GripVertical,
    RefreshCw,
    Loader2,
    X,
    Pencil,
    MoreVertical,
    Copy,
    Archive,
    Lock,
    Unlock,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Chapter, Scene, SyncPreview, ReimportSummary } from "../types";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import RenameDialog from "./RenameDialog.svelte";
  import ArchivePanel from "./ArchivePanel.svelte";

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type IconComponent = any;

  interface MenuItem {
    label: string;
    icon?: IconComponent;
    action: () => void | Promise<void>;
    disabled?: boolean;
    danger?: boolean;
    divider?: boolean;
  }

  let loading = $state(false);
  let expandedChapters = new SvelteSet<string>();

  // Create new content state
  let creatingChapter = $state(false);
  let creatingScene = $state(false);
  let newTitle = $state("");

  // Delete confirmation state
  let deleteDialog: {
    type: "chapter" | "scene";
    id: string;
    title: string;
    message: string;
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

  // Sync state
  let syncing = $state(false);
  let loadingSyncPreview = $state(false);
  let showSyncDialog = $state(false);
  let syncPreview: SyncPreview | null = $state(null);
  let selectedChanges = new SvelteSet<string>();
  let selectedAdditions = new SvelteSet<string>();
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

  async function loadChapters() {
    if (!currentProject.value) return;

    loading = true;
    try {
      const chapters = await invoke<Chapter[]>("get_chapters", {
        projectId: currentProject.value.id,
      });
      currentProject.setChapters(chapters);
      // Auto-expand first chapter if any exist
      if (chapters.length > 0) {
        expandedChapters.clear();
        expandedChapters.add(chapters[0].id);
        await loadScenes(chapters[0]);
      }
    } catch (e) {
      console.error("Failed to load chapters:", e);
    } finally {
      loading = false;
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
    currentProject.setCurrentChapter(chapter);
    try {
      const scenes = await invoke<Scene[]>("get_scenes", {
        chapterId: chapter.id,
      });
      currentProject.setScenes(scenes);
    } catch (e) {
      console.error("Failed to load scenes:", e);
    }
  }

  async function selectScene(scene: Scene) {
    currentProject.setCurrentScene(scene);
    try {
      const beats = await invoke("get_beats", { sceneId: scene.id });
      currentProject.setBeats(beats as any[]);
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

  // === Create Chapter/Scene ===
  function startCreatingChapter() {
    creatingChapter = true;
    creatingScene = false;
    newTitle = "";
  }

  function startCreatingScene() {
    creatingScene = true;
    creatingChapter = false;
    newTitle = "";
  }

  function cancelCreate() {
    creatingChapter = false;
    creatingScene = false;
    newTitle = "";
  }

  async function createChapter() {
    if (!newTitle.trim() || !currentProject.value) return;
    try {
      const chapter = await invoke<Chapter>("create_chapter", {
        projectId: currentProject.value.id,
        title: newTitle.trim(),
      });
      currentProject.addChapter(chapter);
      expandedChapters.clear();
      expandedChapters.add(chapter.id);
      await loadScenes(chapter);
      cancelCreate();
    } catch (e) {
      console.error("Failed to create chapter:", e);
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
      else if (creatingScene) createScene();
    } else if (e.key === "Escape") {
      cancelCreate();
    }
  }

  // === Delete Chapter/Scene ===
  async function confirmDeleteChapter(chapter: Chapter) {
    try {
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
      // Default: all additions selected, no changes selected
      selectedChanges.clear();
      selectedAdditions.clear();
      for (const addition of preview.additions) {
        selectedAdditions.add(addition.id);
      }
      showSyncDialog = true;
    } catch (e) {
      console.error("Failed to get sync preview:", e);
    } finally {
      loadingSyncPreview = false;
    }
  }

  function cancelSync() {
    showSyncDialog = false;
    syncPreview = null;
    selectedChanges.clear();
    selectedAdditions.clear();
  }

  function toggleChange(changeId: string) {
    if (selectedChanges.has(changeId)) {
      selectedChanges.delete(changeId);
    } else {
      selectedChanges.add(changeId);
    }
  }

  function selectAllChanges() {
    if (!syncPreview) return;
    selectedChanges.clear();
    for (const change of syncPreview.changes) {
      selectedChanges.add(change.id);
    }
  }

  function deselectAllChanges() {
    selectedChanges.clear();
  }

  function toggleAddition(additionId: string) {
    if (selectedAdditions.has(additionId)) {
      selectedAdditions.delete(additionId);
    } else {
      selectedAdditions.add(additionId);
    }
  }

  function selectAllAdditions() {
    if (!syncPreview) return;
    selectedAdditions.clear();
    for (const addition of syncPreview.additions) {
      selectedAdditions.add(addition.id);
    }
  }

  function deselectAllAdditions() {
    selectedAdditions.clear();
  }

  async function applySync() {
    if (!currentProject.value) return;
    syncing = true;
    try {
      const summary = await invoke<ReimportSummary>("apply_sync", {
        projectId: currentProject.value.id,
        acceptedChangeIds: Array.from(selectedChanges),
        acceptedAdditionIds: Array.from(selectedAdditions),
      });
      syncSummary = summary;
      showSyncDialog = false;
      syncPreview = null;
      selectedChanges.clear();
      selectedAdditions.clear();

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
    } catch (e) {
      console.error("Failed to apply sync:", e);
    } finally {
      syncing = false;
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
        label: "Duplicate",
        icon: Copy,
        action: () => handleDuplicate(type, item.id),
      },
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
        <svg width="24" height="24" viewBox="0 0 1024 1024" class="flex-shrink-0">
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
      <button
        onclick={toggleSidebar}
        class="text-text-secondary hover:text-text-primary p-1"
        aria-label="Collapse sidebar"
      >
        <ChevronsLeft class="w-5 h-5" />
      </button>
    </div>
    {#if currentProject.value}
      <p class="text-text-secondary text-sm mt-1 truncate">
        {currentProject.value.name}
      </p>
      <div class="flex gap-2 mt-3">
        <button
          onclick={goHome}
          class="flex-1 flex items-center justify-center gap-2 px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary bg-bg-card hover:bg-beat-header rounded-lg transition-colors"
          aria-label="Close project"
        >
          <Home class="w-3.5 h-3.5" />
          All Projects
        </button>
        <button
          data-testid="archive-button"
          onclick={() => (showArchivePanel = true)}
          class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary bg-bg-card hover:bg-beat-header rounded-lg transition-colors"
          aria-label="View archive"
          title="View archived items"
        >
          <Archive class="w-3.5 h-3.5" />
        </button>
        {#if currentProject.value.source_path}
          <button
            data-testid="reimport-button"
            onclick={handleSyncClick}
            disabled={loadingSyncPreview || syncing}
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary bg-bg-card hover:bg-beat-header rounded-lg transition-colors disabled:opacity-50"
            aria-label="Sync from source"
            title="Sync from source file"
          >
            {#if loadingSyncPreview || syncing}
              <Loader2 data-testid="reimport-spinner" class="w-3.5 h-3.5 animate-spin" />
            {:else}
              <RefreshCw class="w-3.5 h-3.5" />
            {/if}
            <span>Sync</span>
          </button>
        {/if}
      </div>
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
        {#each currentProject.chapters as chapter (chapter.id)}
          {@const isExpanded = isChapterExpanded(chapter.id)}
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
            <div
              class="w-full flex items-center gap-1 px-1 py-1.5 rounded-lg transition-colors group"
              class:bg-bg-card={isExpanded}
              class:hover:bg-bg-card={!isExpanded}
              oncontextmenu={(e) => openContextMenu(e, "chapter", chapter)}
            >
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
                class="flex-1 flex items-center gap-2 text-left min-w-0"
                aria-expanded={isExpanded}
              >
                <!-- Expand/collapse chevron -->
                <ChevronRight
                  class="w-4 h-4 text-text-secondary transition-transform flex-shrink-0 {isExpanded
                    ? 'rotate-90'
                    : ''}"
                />
                <!-- Chapter icon -->
                <Folder class="w-4 h-4 text-text-secondary flex-shrink-0" />
                <!-- Lock indicator -->
                {#if chapter.locked}
                  <Lock class="w-3 h-3 text-amber-500 flex-shrink-0" />
                {/if}
                <span
                  data-testid="chapter-title"
                  class="text-text-primary font-medium text-sm truncate"
                  class:opacity-60={chapter.locked}>{chapter.title}</span
                >
              </button>

              <!-- Three-dot menu button -->
              <button
                data-testid="menu-button"
                onclick={(e) => openContextMenu(e, "chapter", chapter)}
                class="p-1 text-text-secondary hover:text-text-primary transition-opacity flex-shrink-0"
                class:opacity-0={hoveredChapterId !== chapter.id}
                class:opacity-100={hoveredChapterId === chapter.id}
                aria-label="Chapter menu"
              >
                <MoreVertical class="w-3.5 h-3.5" />
              </button>
            </div>

            <!-- Scenes (collapsible) -->
            {#if isExpanded && currentProject.currentChapter?.id === chapter.id}
              <div class="ml-6 mt-1 space-y-0.5 border-l border-bg-card pl-2">
                {#each currentProject.scenes as scene (scene.id)}
                  {@const isSelected = currentProject.currentScene?.id === scene.id}
                  {@const isLocked = scene.locked || chapter.locked}
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
                      class="cursor-grab active:cursor-grabbing p-0.5 transition-opacity flex-shrink-0"
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
                      class="flex-1 flex items-center gap-2 text-left px-2 py-1 rounded text-sm transition-colors min-w-0"
                      class:bg-accent={isSelected}
                      class:text-white={isSelected}
                      class:text-text-secondary={!isSelected}
                      class:hover:bg-bg-card={!isSelected}
                      class:hover:text-text-primary={!isSelected}
                    >
                      <!-- Scene icon -->
                      <FileText
                        class="w-3.5 h-3.5 flex-shrink-0 {isSelected
                          ? 'text-white'
                          : 'text-text-secondary'}"
                      />
                      <!-- Lock indicator -->
                      {#if isLocked}
                        <Lock
                          class="w-3 h-3 flex-shrink-0 {isSelected
                            ? 'text-white'
                            : 'text-amber-500'}"
                        />
                      {/if}
                      <span data-testid="scene-title" class="truncate" class:opacity-60={isLocked}
                        >{scene.title}</span
                      >
                    </button>

                    <!-- Scene menu button -->
                    <button
                      data-testid="menu-button"
                      onclick={(e) => openContextMenu(e, "scene", scene)}
                      class="p-0.5 transition-opacity flex-shrink-0"
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
                  <span class="text-text-secondary text-xs px-2 py-1 italic">No scenes yet</span>
                {/if}
              </div>
            {/if}
          </div>
        {/each}

        <!-- New Chapter Button or Input -->
        {#if creatingChapter}
          <div class="px-2 py-1">
            <input
              data-testid="title-input"
              type="text"
              bind:value={newTitle}
              onkeydown={handleCreateKeydown}
              onblur={cancelCreate}
              placeholder="Chapter title..."
              class="w-full px-2 py-1 text-sm bg-bg-card border border-accent rounded focus:outline-none text-text-primary"
              autofocus
            />
          </div>
        {:else}
          <button
            data-testid="new-chapter-button"
            onclick={startCreatingChapter}
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-text-secondary hover:text-text-primary hover:bg-bg-card transition-colors"
          >
            <Plus class="w-4 h-4" />
            New Chapter
          </button>
        {/if}
      </nav>
    {/if}
  </div>
</aside>

<!-- Collapsed sidebar toggle -->
{#if ui.sidebarCollapsed}
  <button
    onclick={toggleSidebar}
    class="fixed left-0 top-1/2 -translate-y-1/2 bg-bg-panel p-2 rounded-r-lg text-text-secondary hover:text-text-primary z-10"
    aria-label="Expand sidebar"
  >
    <ChevronsRight class="w-5 h-5" />
  </button>
{/if}

<!-- Sync Preview Dialog - Full Window -->
{#if showSyncDialog && syncPreview}
  <div
    data-testid="sync-preview-dialog"
    class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-6 md:p-10"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="bg-bg-panel rounded-2xl w-full h-full max-w-7xl flex flex-col shadow-2xl border border-white/5 overflow-hidden"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-8 py-6 border-b border-bg-card/50">
        <div>
          <h2 class="text-2xl font-heading font-semibold text-text-primary">Sync from Source</h2>
          <p class="text-text-secondary text-sm mt-1">Review and select items to import</p>
        </div>
        <button
          onclick={cancelSync}
          class="p-2 text-text-secondary hover:text-text-primary rounded-lg hover:bg-bg-card transition-colors"
        >
          <X class="w-6 h-6" />
        </button>
      </div>

      <!-- Content - Two Column Layout -->
      {#if syncPreview.additions.length === 0 && syncPreview.changes.length === 0}
        <!-- No changes message -->
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center py-12">
            <div
              class="w-16 h-16 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4"
            >
              <RefreshCw class="w-8 h-8 text-green-500" />
            </div>
            <p class="text-text-primary text-lg font-medium">All synced!</p>
            <p class="text-text-secondary text-sm mt-1">
              Your project is up to date with the source file.
            </p>
          </div>
        </div>
      {:else}
        <div
          class="flex-1 overflow-hidden grid grid-cols-1 lg:grid-cols-2 divide-y lg:divide-y-0 lg:divide-x divide-bg-card/50"
        >
          <!-- Left Column: Additions -->
          <div class="flex flex-col min-h-0">
            <div class="flex items-center justify-between px-6 py-4 border-b border-bg-card/30">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-500/10 flex items-center justify-center">
                  <Plus class="w-4 h-4 text-green-500" />
                </div>
                <div>
                  <h3 class="text-sm font-medium text-text-primary">New Items</h3>
                  <p class="text-xs text-text-secondary">
                    {selectedAdditions.size} of {syncPreview.additions.length} selected
                  </p>
                </div>
              </div>
              {#if syncPreview.additions.length > 0}
                <div class="flex gap-2 text-xs">
                  <button
                    onclick={selectAllAdditions}
                    class="text-text-secondary hover:text-accent transition-colors">All</button
                  >
                  <span class="text-text-secondary/30">|</span>
                  <button
                    onclick={deselectAllAdditions}
                    class="text-text-secondary hover:text-accent transition-colors">None</button
                  >
                </div>
              {/if}
            </div>

            <div class="flex-1 overflow-y-auto p-4 space-y-2">
              {#if syncPreview.additions.length === 0}
                <div class="text-center py-12 text-text-secondary">
                  <p>No new items to import</p>
                </div>
              {:else}
                {#each syncPreview.additions as addition (addition.id)}
                  <label
                    class="flex items-center gap-4 p-4 bg-bg-card/50 rounded-xl cursor-pointer hover:bg-bg-card transition-colors group"
                  >
                    <input
                      type="checkbox"
                      checked={selectedAdditions.has(addition.id)}
                      onchange={() => toggleAddition(addition.id)}
                      class="w-5 h-5 rounded border-2 border-bg-card bg-transparent text-accent focus:ring-accent focus:ring-offset-0 cursor-pointer"
                    />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span
                          class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-500/10 text-green-500 uppercase"
                        >
                          {addition.item_type}
                        </span>
                        <span class="text-text-primary font-medium truncate">{addition.title}</span>
                      </div>
                      {#if addition.parent_title}
                        <p class="text-xs text-text-secondary mt-1">in {addition.parent_title}</p>
                      {/if}
                    </div>
                  </label>
                {/each}
              {/if}
            </div>
          </div>

          <!-- Right Column: Changes -->
          <div class="flex flex-col min-h-0">
            <div class="flex items-center justify-between px-6 py-4 border-b border-bg-card/30">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center">
                  <Pencil class="w-4 h-4 text-amber-500" />
                </div>
                <div>
                  <h3 class="text-sm font-medium text-text-primary">Changes</h3>
                  <p class="text-xs text-text-secondary">
                    {selectedChanges.size} of {syncPreview.changes.length} selected
                  </p>
                </div>
              </div>
              {#if syncPreview.changes.length > 0}
                <div class="flex gap-2 text-xs">
                  <button
                    onclick={selectAllChanges}
                    class="text-text-secondary hover:text-accent transition-colors">All</button
                  >
                  <span class="text-text-secondary/30">|</span>
                  <button
                    onclick={deselectAllChanges}
                    class="text-text-secondary hover:text-accent transition-colors">None</button
                  >
                </div>
              {/if}
            </div>

            <div class="flex-1 overflow-y-auto p-4 space-y-2">
              {#if syncPreview.changes.length === 0}
                <div class="text-center py-12 text-text-secondary">
                  <p>No changes detected</p>
                </div>
              {:else}
                {#each syncPreview.changes as change (change.id)}
                  <label
                    class="flex items-start gap-4 p-4 bg-bg-card/50 rounded-xl cursor-pointer hover:bg-bg-card transition-colors"
                  >
                    <input
                      type="checkbox"
                      checked={selectedChanges.has(change.id)}
                      onchange={() => toggleChange(change.id)}
                      class="mt-0.5 w-5 h-5 rounded border-2 border-bg-card bg-transparent text-accent focus:ring-accent focus:ring-offset-0 cursor-pointer"
                    />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2 mb-2">
                        <span
                          class="px-2 py-0.5 text-xs font-medium rounded-full bg-amber-500/10 text-amber-500 uppercase"
                        >
                          {change.item_type}
                        </span>
                        <span class="text-text-primary font-medium truncate"
                          >{change.item_title}</span
                        >
                        <span class="text-text-secondary/60 text-xs">({change.field})</span>
                      </div>
                      <div class="text-sm space-y-1 font-mono">
                        <div class="flex gap-2 text-red-400/80">
                          <span class="flex-shrink-0">−</span>
                          <span class="line-through opacity-60 truncate"
                            >{change.current_value || "(empty)"}</span
                          >
                        </div>
                        <div class="flex gap-2 text-green-400/80">
                          <span class="flex-shrink-0">+</span>
                          <span class="truncate">{change.new_value || "(empty)"}</span>
                        </div>
                      </div>
                    </div>
                  </label>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- Footer -->
      <div
        class="flex items-center justify-between px-8 py-5 border-t border-bg-card/50 bg-bg-card/20"
      >
        <p class="text-text-secondary text-sm">
          {selectedAdditions.size + selectedChanges.size} item{selectedAdditions.size +
            selectedChanges.size !==
          1
            ? "s"
            : ""} selected
        </p>
        <div class="flex gap-4">
          <button
            onclick={cancelSync}
            class="px-6 py-2.5 text-text-secondary hover:text-text-primary rounded-lg hover:bg-bg-card transition-colors"
          >
            Cancel
          </button>
          <button
            data-testid="sync-confirm"
            onclick={applySync}
            disabled={syncing || (selectedAdditions.size === 0 && selectedChanges.size === 0)}
            class="px-6 py-2.5 bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
          >
            {#if syncing}
              <Loader2 class="w-4 h-4 animate-spin" />
              Syncing...
            {:else}
              Apply Sync
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
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

<!-- Sync Summary Dialog -->
{#if syncSummary}
  <div
    data-testid="reimport-summary-dialog"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
      <h3 class="text-lg font-heading font-medium text-text-primary mb-4">Sync Complete</h3>
      <div data-testid="reimport-summary" class="text-text-secondary text-sm space-y-2 mb-6">
        {#if syncSummary.chapters_added === 0 && syncSummary.chapters_updated === 0 && syncSummary.scenes_added === 0 && syncSummary.scenes_updated === 0 && syncSummary.beats_added === 0 && syncSummary.beats_updated === 0}
          <p>No changes were applied.</p>
        {:else}
          {#if syncSummary.chapters_added > 0 || syncSummary.chapters_updated > 0}
            <p>
              Chapters: {syncSummary.chapters_added} added, {syncSummary.chapters_updated} updated
            </p>
          {/if}
          {#if syncSummary.scenes_added > 0 || syncSummary.scenes_updated > 0}
            <p>
              Scenes: {syncSummary.scenes_added} added, {syncSummary.scenes_updated} updated
            </p>
          {/if}
          {#if syncSummary.beats_added > 0 || syncSummary.beats_updated > 0}
            <p>
              Beats: {syncSummary.beats_added} added, {syncSummary.beats_updated} updated
            </p>
          {/if}
          {#if syncSummary.prose_preserved > 0}
            <p class="text-text-secondary/80 italic">
              {syncSummary.prose_preserved} prose item{syncSummary.prose_preserved !== 1 ? "s" : ""} preserved
            </p>
          {/if}
        {/if}
      </div>
      <button
        data-testid="dialog-close"
        onclick={closeSyncSummary}
        class="w-full px-4 py-2 bg-accent text-white rounded hover:bg-accent/80 transition-colors"
      >
        Close
      </button>
    </div>
  </div>
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
    title="Rename {renameDialog.type === 'chapter' ? 'Chapter' : 'Scene'}"
    currentName={renameDialog.title}
    onSave={(newName) => handleRename(renameDialog!.type, renameDialog!.id, newName)}
    onClose={() => (renameDialog = null)}
  />
{/if}

<!-- Archive Panel -->
{#if showArchivePanel}
  <ArchivePanel onClose={() => (showArchivePanel = false)} />
{/if}
