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
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Chapter, Scene } from "../types";
  import ConfirmDialog from "./ConfirmDialog.svelte";

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

  // Drag-and-drop state
  let draggedItem: { type: "chapter" | "scene"; id: string } | null = $state(null);
  let dropTarget: { type: "chapter" | "scene"; id: string; position: "before" | "after" } | null =
    $state(null);

  // Hover state for showing action buttons
  let hoveredChapterId: string | null = $state(null);
  let hoveredSceneId: string | null = $state(null);

  // Reimport state
  let reimporting = $state(false);
  let reimportSummary: {
    chapters_added: number;
    chapters_updated: number;
    scenes_added: number;
    scenes_updated: number;
    beats_added: number;
    beats_updated: number;
    prose_preserved: number;
  } | null = $state(null);

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
        await invoke("delete_scene", { sceneId: deleteDialog.id });
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

  // === Drag and Drop ===
  function handleDragStart(type: "chapter" | "scene", id: string) {
    draggedItem = { type, id };
  }

  function handleDragOver(e: DragEvent, type: "chapter" | "scene", id: string) {
    e.preventDefault();
    if (!draggedItem || draggedItem.type !== type || draggedItem.id === id) return;

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const midY = rect.top + rect.height / 2;
    const position = e.clientY < midY ? "before" : "after";
    dropTarget = { type, id, position };
  }

  function handleDragLeave() {
    dropTarget = null;
  }

  async function handleDrop() {
    if (!draggedItem || !dropTarget || draggedItem.type !== dropTarget.type) {
      draggedItem = null;
      dropTarget = null;
      return;
    }

    const items = draggedItem.type === "chapter" ? currentProject.chapters : currentProject.scenes;
    const draggedIndex = items.findIndex((item) => item.id === draggedItem!.id);
    let targetIndex = items.findIndex((item) => item.id === dropTarget!.id);

    if (dropTarget.position === "after") targetIndex++;
    if (draggedIndex < targetIndex) targetIndex--;

    const newOrder = [...items];
    const [removed] = newOrder.splice(draggedIndex, 1);
    newOrder.splice(targetIndex, 0, removed);

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
    } finally {
      draggedItem = null;
      dropTarget = null;
    }
  }

  function handleDragEnd() {
    draggedItem = null;
    dropTarget = null;
  }

  // === Reimport ===
  async function handleReimport() {
    if (!currentProject.value) return;
    reimporting = true;
    try {
      const summary = await invoke<typeof reimportSummary>("reimport_project", {
        projectId: currentProject.value.id,
      });
      reimportSummary = summary;
      // Reload content
      await loadChapters();
    } catch (e) {
      console.error("Failed to reimport:", e);
    } finally {
      reimporting = false;
    }
  }

  function closeReimportSummary() {
    reimportSummary = null;
  }

  $effect(() => {
    if (currentProject.value) {
      // Skip loading if import is in progress (importProject will load chapters)
      // or if chapters are already loaded
      if (!ui.isImporting && currentProject.chapters.length === 0) {
        loadChapters();
      }
    }
  });
</script>

<aside
  data-testid="sidebar"
  class="bg-bg-panel border-r border-bg-card flex flex-col h-full transition-all duration-200"
  class:w-64={!ui.sidebarCollapsed}
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
        {#if currentProject.value.source_path}
          <button
            data-testid="reimport-button"
            onclick={handleReimport}
            disabled={reimporting}
            class="flex items-center justify-center px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary bg-bg-card hover:bg-beat-header rounded-lg transition-colors disabled:opacity-50"
            aria-label="Re-import from source"
            title="Re-import from source file"
          >
            {#if reimporting}
              <Loader2 data-testid="reimport-spinner" class="w-3.5 h-3.5 animate-spin" />
            {:else}
              <RefreshCw class="w-3.5 h-3.5" />
            {/if}
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
          {@const isDragging = draggedItem?.type === "chapter" && draggedItem?.id === chapter.id}
          {@const isDropTarget = dropTarget?.type === "chapter" && dropTarget?.id === chapter.id}
          <div
            data-testid="chapter-item"
            class="select-none relative"
            class:opacity-50={isDragging}
            onmouseenter={() => (hoveredChapterId = chapter.id)}
            onmouseleave={() => (hoveredChapterId = null)}
            ondragover={(e) => handleDragOver(e, "chapter", chapter.id)}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}
          >
            <!-- Drop indicator -->
            {#if isDropTarget && dropTarget?.position === "before"}
              <div class="absolute -top-0.5 left-0 right-0 h-0.5 bg-accent rounded"></div>
            {/if}

            <!-- Chapter row -->
            <div
              class="w-full flex items-center gap-1 px-1 py-1.5 rounded-lg transition-colors group"
              class:bg-bg-card={isExpanded}
              class:hover:bg-bg-card={!isExpanded}
            >
              <!-- Drag handle -->
              <div
                data-testid="drag-handle"
                draggable="true"
                ondragstart={() => handleDragStart("chapter", chapter.id)}
                ondragend={handleDragEnd}
                class="cursor-grab p-0.5 text-text-secondary hover:text-text-primary transition-opacity"
                class:opacity-0={hoveredChapterId !== chapter.id}
                class:opacity-100={hoveredChapterId === chapter.id}
              >
                <GripVertical class="w-3.5 h-3.5" />
              </div>

              <button
                onclick={() => toggleChapter(chapter)}
                class="flex-1 flex items-center gap-2 text-left"
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
                <span
                  data-testid="chapter-title"
                  class="text-text-primary font-medium text-sm truncate">{chapter.title}</span
                >
              </button>

              <!-- Delete button -->
              <button
                data-testid="delete-button"
                onclick={(e) => {
                  e.stopPropagation();
                  confirmDeleteChapter(chapter);
                }}
                class="p-1 text-text-secondary hover:text-red-500 transition-opacity"
                class:opacity-0={hoveredChapterId !== chapter.id}
                class:opacity-100={hoveredChapterId === chapter.id}
                aria-label="Delete chapter"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>

            <!-- Drop indicator after -->
            {#if isDropTarget && dropTarget?.position === "after"}
              <div class="absolute -bottom-0.5 left-0 right-0 h-0.5 bg-accent rounded"></div>
            {/if}

            <!-- Scenes (collapsible) -->
            {#if isExpanded && currentProject.currentChapter?.id === chapter.id}
              <div class="ml-6 mt-1 space-y-0.5 border-l border-bg-card pl-2">
                {#each currentProject.scenes as scene (scene.id)}
                  {@const isSelected = currentProject.currentScene?.id === scene.id}
                  {@const isSceneDragging =
                    draggedItem?.type === "scene" && draggedItem?.id === scene.id}
                  {@const isSceneDropTarget =
                    dropTarget?.type === "scene" && dropTarget?.id === scene.id}
                  <div
                    class="relative"
                    class:opacity-50={isSceneDragging}
                    onmouseenter={() => (hoveredSceneId = scene.id)}
                    onmouseleave={() => (hoveredSceneId = null)}
                    ondragover={(e) => handleDragOver(e, "scene", scene.id)}
                    ondragleave={handleDragLeave}
                    ondrop={handleDrop}
                  >
                    <!-- Scene drop indicator before -->
                    {#if isSceneDropTarget && dropTarget?.position === "before"}
                      <div class="absolute -top-0.5 left-0 right-0 h-0.5 bg-accent rounded"></div>
                    {/if}

                    <div
                      data-testid="scene-item"
                      class="w-full flex items-center gap-1 px-1 py-1 rounded text-sm transition-colors"
                      class:bg-accent={isSelected}
                      class:text-white={isSelected}
                      class:text-text-secondary={!isSelected}
                      class:hover:bg-bg-card={!isSelected}
                      class:hover:text-text-primary={!isSelected}
                    >
                      <!-- Scene drag handle -->
                      <div
                        data-testid="drag-handle"
                        draggable="true"
                        ondragstart={() => handleDragStart("scene", scene.id)}
                        ondragend={handleDragEnd}
                        class="cursor-grab p-0.5 transition-opacity"
                        class:text-white={isSelected}
                        class:text-text-secondary={!isSelected}
                        class:opacity-0={hoveredSceneId !== scene.id}
                        class:opacity-100={hoveredSceneId === scene.id}
                      >
                        <GripVertical class="w-3 h-3" />
                      </div>

                      <button
                        onclick={() => selectScene(scene)}
                        class="flex-1 flex items-center gap-2 text-left"
                      >
                        <!-- Scene icon -->
                        <FileText
                          class="w-3.5 h-3.5 flex-shrink-0 {isSelected
                            ? 'text-white'
                            : 'text-text-secondary'}"
                        />
                        <span data-testid="scene-title" class="truncate">{scene.title}</span>
                      </button>

                      <!-- Scene delete button -->
                      <button
                        data-testid="delete-button"
                        onclick={(e) => {
                          e.stopPropagation();
                          confirmDeleteScene(scene);
                        }}
                        class="p-0.5 transition-opacity"
                        class:text-white={isSelected}
                        class:hover:text-red-300={isSelected}
                        class:text-text-secondary={!isSelected}
                        class:hover:text-red-500={!isSelected}
                        class:opacity-0={hoveredSceneId !== scene.id}
                        class:opacity-100={hoveredSceneId === scene.id}
                        aria-label="Delete scene"
                      >
                        <Trash2 class="w-3 h-3" />
                      </button>
                    </div>

                    <!-- Scene drop indicator after -->
                    {#if isSceneDropTarget && dropTarget?.position === "after"}
                      <div
                        class="absolute -bottom-0.5 left-0 right-0 h-0.5 bg-accent rounded"
                      ></div>
                    {/if}
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

<!-- Delete Confirmation Dialog -->
{#if deleteDialog}
  <ConfirmDialog
    title="Delete {deleteDialog.type === 'chapter' ? 'Chapter' : 'Scene'}"
    message={deleteDialog.message}
    onConfirm={executeDelete}
    onCancel={() => (deleteDialog = null)}
  />
{/if}

<!-- Reimport Summary Dialog -->
{#if reimportSummary}
  <div
    data-testid="reimport-summary-dialog"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
      <h3 class="text-lg font-heading font-medium text-text-primary mb-4">Re-import Complete</h3>
      <div data-testid="reimport-summary" class="text-text-secondary text-sm space-y-2 mb-6">
        {#if reimportSummary.chapters_added === 0 && reimportSummary.chapters_updated === 0 && reimportSummary.scenes_added === 0 && reimportSummary.scenes_updated === 0 && reimportSummary.beats_added === 0 && reimportSummary.beats_updated === 0}
          <p>No changes detected.</p>
        {:else}
          {#if reimportSummary.chapters_added > 0 || reimportSummary.chapters_updated > 0}
            <p>
              Chapters: {reimportSummary.chapters_added} added, {reimportSummary.chapters_updated} updated
            </p>
          {/if}
          {#if reimportSummary.scenes_added > 0 || reimportSummary.scenes_updated > 0}
            <p>
              Scenes: {reimportSummary.scenes_added} added, {reimportSummary.scenes_updated} updated
            </p>
          {/if}
          {#if reimportSummary.beats_added > 0 || reimportSummary.beats_updated > 0}
            <p>
              Beats: {reimportSummary.beats_added} added, {reimportSummary.beats_updated} updated
            </p>
          {/if}
          {#if reimportSummary.prose_preserved > 0}
            <p class="text-green-500">
              {reimportSummary.prose_preserved} prose item{reimportSummary.prose_preserved !== 1
                ? "s"
                : ""} preserved
            </p>
          {/if}
        {/if}
      </div>
      <button
        data-testid="dialog-close"
        onclick={closeReimportSummary}
        class="w-full px-4 py-2 bg-accent text-white rounded hover:bg-accent/80 transition-colors"
      >
        Close
      </button>
    </div>
  </div>
{/if}
