<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import { ChevronRight, ChevronsLeft, ChevronsRight, FileText, Folder, Home } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Chapter, Scene } from "../types";

  let loading = $state(false);
  let expandedChapters = new SvelteSet<string>();

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

  $effect(() => {
    if (currentProject.value) {
      loadChapters();
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
      <button
        onclick={goHome}
        class="w-full mt-3 flex items-center justify-center gap-2 px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary bg-bg-card hover:bg-beat-header rounded-lg transition-colors"
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
        {#each currentProject.chapters as chapter (chapter.id)}
          {@const isExpanded = isChapterExpanded(chapter.id)}
          <div data-testid="chapter-item" class="select-none">
            <!-- Chapter row -->
            <button
              onclick={() => toggleChapter(chapter)}
              class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors group"
              class:bg-bg-card={isExpanded}
              class:hover:bg-bg-card={!isExpanded}
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

            <!-- Scenes (collapsible) -->
            {#if isExpanded && currentProject.currentChapter?.id === chapter.id}
              <div class="ml-6 mt-1 space-y-0.5 border-l border-bg-card pl-2">
                {#each currentProject.scenes as scene (scene.id)}
                  {@const isSelected = currentProject.currentScene?.id === scene.id}
                  <button
                    data-testid="scene-item"
                    onclick={() => selectScene(scene)}
                    class="w-full flex items-center gap-2 px-2 py-1 rounded text-sm transition-colors"
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
                    <span data-testid="scene-title" class="truncate">{scene.title}</span>
                  </button>
                {/each}
                {#if currentProject.scenes.length === 0}
                  <span class="text-text-secondary text-xs px-2 py-1 italic">No scenes</span>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
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
