<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import Onboarding from "./lib/components/Onboarding.svelte";
  import ReferencesPanel from "./lib/components/ReferencesPanel.svelte";
  import ScenePanel from "./lib/components/ScenePanel.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StartScreen from "./lib/components/StartScreen.svelte";
  import KindlingSettingsDialog from "./lib/components/KindlingSettingsDialog.svelte";
  import ProjectSettingsDialog from "./lib/components/ProjectSettingsDialog.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import ExportSuccessDialog from "./lib/components/ExportSuccessDialog.svelte";
  import { currentProject } from "./lib/stores/project.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import type { Project, ExportResult } from "./lib/types";

  let recentProjects = $state<any[]>([]);

  // Dialog states triggered by menu
  let showKindlingSettings = $state(false);
  let showProjectSettings = $state(false);
  let showExportDialog = $state(false);
  let exportResult = $state<ExportResult | null>(null);

  async function loadRecentProjects() {
    try {
      recentProjects = await invoke("get_recent_projects");
    } catch (e) {
      console.error("Failed to load recent projects:", e);
      recentProjects = [];
    }
  }

  // Reload projects when returning to start screen (currentProject becomes null)
  // or on initial load
  $effect(() => {
    if (!currentProject.value) {
      loadRecentProjects();
    }
  });

  // Import functions (same as StartScreen)
  async function importPlottr() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Plottr", extensions: ["pltr"] }],
    });

    if (path) {
      ui.startImport();
      try {
        const project = await invoke<Project>("import_plottr", { path });
        currentProject.setProject(project);
        ui.setView("editor");
      } catch (e) {
        console.error("Failed to import Plottr file:", e);
        alert(`Import failed: ${e}`);
      } finally {
        ui.finishImport();
      }
    }
  }

  async function importMarkdown() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Markdown", extensions: ["md", "markdown"] }],
    });

    if (path) {
      ui.startImport();
      try {
        const project = await invoke<Project>("import_markdown", { path });
        currentProject.setProject(project);
        ui.setView("editor");
      } catch (e) {
        console.error("Failed to import Markdown file:", e);
        alert(`Import failed: ${e}`);
      } finally {
        ui.finishImport();
      }
    }
  }

  function closeProject() {
    currentProject.setProject(null);
  }

  // Handle menu events from Tauri
  onMount(() => {
    const unlisten = listen<string>("menu-event", (event) => {
      const menuId = event.payload;

      switch (menuId) {
        case "import_plottr":
          importPlottr();
          break;
        case "import_markdown":
          importMarkdown();
          break;
        case "export":
          if (currentProject.value) {
            showExportDialog = true;
          }
          break;
        case "close_project":
          closeProject();
          break;
        case "project_settings":
          if (currentProject.value) {
            showProjectSettings = true;
          }
          break;
        case "kindling_settings":
          showKindlingSettings = true;
          break;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  // Global keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Cmd/Ctrl+E: Open export dialog
    if ((event.metaKey || event.ctrlKey) && event.key === "e") {
      event.preventDefault();
      if (currentProject.value && !showExportDialog) {
        showExportDialog = true;
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="flex h-screen w-screen overflow-hidden bg-bg-primary">
  {#if currentProject.value}
    <Sidebar />
    <ScenePanel />
    <ReferencesPanel />
  {:else}
    <StartScreen {recentProjects} />
  {/if}
</main>

<!-- Onboarding overlay (shown on first launch) -->
<Onboarding />

<!-- Kindling Settings Dialog (triggered by menu) -->
{#if showKindlingSettings}
  <KindlingSettingsDialog
    onClose={() => (showKindlingSettings = false)}
    onSave={() => (showKindlingSettings = false)}
  />
{/if}

<!-- Project Settings Dialog (triggered by menu) -->
{#if showProjectSettings && currentProject.value}
  <ProjectSettingsDialog
    onClose={() => (showProjectSettings = false)}
    onSave={(project) => {
      currentProject.setProject(project);
      showProjectSettings = false;
    }}
  />
{/if}

<!-- Export Dialog (triggered by menu) -->
{#if showExportDialog && currentProject.value}
  <ExportDialog
    scope="project"
    scopeId={null}
    scopeTitle={currentProject.value.name}
    onClose={() => (showExportDialog = false)}
    onSuccess={(result) => {
      showExportDialog = false;
      exportResult = result;
    }}
  />
{/if}

<!-- Export Success Dialog -->
{#if exportResult}
  <ExportSuccessDialog result={exportResult} onClose={() => (exportResult = null)} />
{/if}
