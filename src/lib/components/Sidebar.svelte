<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Chapter, Scene } from "../types";

  let loading = $state(false);

  async function loadChapters() {
    if (!currentProject.value) return;

    loading = true;
    try {
      const chapters = await invoke<Chapter[]>("get_chapters", {
        projectId: currentProject.value.id,
      });
      currentProject.setChapters(chapters);
    } catch (e) {
      console.error("Failed to load chapters:", e);
    } finally {
      loading = false;
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

  $effect(() => {
    if (currentProject.value) {
      loadChapters();
    }
  });
</script>

<aside
  class="w-64 bg-bg-panel border-r border-bg-card flex flex-col h-full transition-all"
  class:w-0={ui.sidebarCollapsed}
  class:overflow-hidden={ui.sidebarCollapsed}
>
  <!-- Header -->
  <div class="p-4 border-b border-bg-card">
    <div class="flex items-center justify-between">
      <button
        onclick={goHome}
        class="text-accent font-bold text-lg hover:text-text-primary transition-colors"
      >
        Kindling
      </button>
      <button
        onclick={() => ui.toggleSidebar()}
        class="text-text-secondary hover:text-text-primary p-1"
        aria-label="Collapse sidebar"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
          />
        </svg>
      </button>
    </div>
    {#if currentProject.value}
      <p class="text-text-secondary text-sm mt-1 truncate">
        {currentProject.value.name}
      </p>
    {/if}
  </div>

  <!-- Chapter/Scene List -->
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary">Loading...</span>
      </div>
    {:else}
      {#each currentProject.chapters as chapter}
        <div class="mb-2">
          <button
            onclick={() => loadScenes(chapter)}
            class="w-full text-left px-3 py-2 rounded-lg transition-colors"
            class:bg-bg-card={currentProject.currentChapter?.id === chapter.id}
            class:hover:bg-bg-card={currentProject.currentChapter?.id !==
              chapter.id}
          >
            <span class="text-text-primary font-medium">{chapter.title}</span>
          </button>

          {#if currentProject.currentChapter?.id === chapter.id}
            <div class="ml-4 mt-1 space-y-1">
              {#each currentProject.scenes as scene}
                <button
                  onclick={() => selectScene(scene)}
                  class="w-full text-left px-3 py-1.5 rounded text-sm transition-colors"
                  class:bg-accent={currentProject.currentScene?.id === scene.id}
                  class:text-white={currentProject.currentScene?.id ===
                    scene.id}
                  class:text-text-secondary={currentProject.currentScene?.id !==
                    scene.id}
                  class:hover:text-text-primary={currentProject.currentScene
                    ?.id !== scene.id}
                >
                  {scene.title}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</aside>

<!-- Collapsed sidebar toggle -->
{#if ui.sidebarCollapsed}
  <button
    onclick={() => ui.toggleSidebar()}
    class="fixed left-0 top-1/2 -translate-y-1/2 bg-bg-panel p-2 rounded-r-lg text-text-secondary hover:text-text-primary z-10"
    aria-label="Expand sidebar"
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M13 5l7 7-7 7M5 5l7 7-7 7"
      />
    </svg>
  </button>
{/if}
