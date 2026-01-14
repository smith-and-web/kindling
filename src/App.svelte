<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ReferencesPanel from "./lib/components/ReferencesPanel.svelte";
  import ScenePanel from "./lib/components/ScenePanel.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StartScreen from "./lib/components/StartScreen.svelte";
  import { currentProject } from "./lib/stores/project.svelte";

  let recentProjects = $state<any[]>([]);

  async function loadRecentProjects() {
    try {
      recentProjects = await invoke("get_recent_projects");
    } catch (e) {
      console.error("Failed to load recent projects:", e);
      recentProjects = [];
    }
  }

  $effect(() => {
    loadRecentProjects();
  });
</script>

<main class="flex h-screen w-screen overflow-hidden bg-bg-primary">
  {#if currentProject.value}
    <Sidebar />
    <ScenePanel />
    <ReferencesPanel />
  {:else}
    <StartScreen {recentProjects} />
  {/if}
</main>
