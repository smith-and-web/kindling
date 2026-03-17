<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { exit } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import { runImport, type ImportType } from "./lib/utils/import";
  import AboutDialog from "./lib/components/AboutDialog.svelte";
  import Onboarding from "./lib/components/Onboarding.svelte";
  import ReferencesPanel from "./lib/components/ReferencesPanel.svelte";
  import ScenePanel from "./lib/components/ScenePanel.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StartScreen from "./lib/components/StartScreen.svelte";
  import KindlingSettingsDialog from "./lib/components/KindlingSettingsDialog.svelte";
  import ProjectSettingsDialog from "./lib/components/ProjectSettingsDialog.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import ExportSuccessDialog from "./lib/components/ExportSuccessDialog.svelte";
  import ErrorToast from "./lib/components/ErrorToast.svelte";
  import ImportLongformDialog from "./lib/components/ImportLongformDialog.svelte";
  import ReferenceClassificationDialog from "./lib/components/ReferenceClassificationDialog.svelte";
  import QuickStartDialog from "./lib/components/QuickStartDialog.svelte";
  import GuidanceOverlay from "./lib/components/GuidanceOverlay.svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import UpdateBanner from "./lib/components/UpdateBanner.svelte";
  import { checkForUpdate } from "./lib/updater";
  import NewProjectDialog from "./lib/components/NewProjectDialog.svelte";
  import { COMMAND_DEFS } from "./lib/commands";
  import { currentProject } from "./lib/stores/project.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import type { Project, ExportResult } from "./lib/types";

  let recentProjects = $state<Project[]>([]);

  // Dialog states triggered by menu
  let showKindlingSettings = $state(false);
  let showProjectSettings = $state(false);
  let showExportDialog = $state(false);
  let exportResult = $state<ExportResult | null>(null);
  let showLongformImportDialog = $state(false);
  let showReferenceClassificationDialog = $state(false);
  let referenceClassificationProjectId = $state<string | null>(null);
  let showQuickStart = $state(false);
  let showCommandPalette = $state(false);
  let showNewProjectDialog = $state(false);
  let showAboutDialog = $state(false);

  async function loadRecentProjects() {
    try {
      recentProjects = await invoke("get_recent_projects");
    } catch (e) {
      console.error("Failed to load recent projects:", e);
      recentProjects = [];
    }
  }

  function activateImportedProject(project: Project) {
    // Clear prior project state so panels reload for the new project.
    currentProject.setProject(null);
    currentProject.setProject(project);
    ui.setView("editor");
  }

  function openReferenceClassificationDialog(project: Project) {
    referenceClassificationProjectId = project.id;
    showReferenceClassificationDialog = true;
  }

  function closeReferenceClassificationDialog() {
    showReferenceClassificationDialog = false;
    referenceClassificationProjectId = null;
  }

  function handleReferenceClassificationComplete(project: Project) {
    currentProject.setProject(project);
    closeReferenceClassificationDialog();
  }

  // Reload projects when returning to start screen (currentProject becomes null)
  // or on initial load
  $effect(() => {
    if (!currentProject.value) {
      loadRecentProjects();
    }
  });

  const HAS_REFERENCES: ImportType[] = ["plottr", "ywriter", "longform", "longformVault"];

  async function handleImport(type: ImportType) {
    const project = await runImport(type);
    if (!project) return;
    activateImportedProject(project);
    if (HAS_REFERENCES.includes(type)) {
      openReferenceClassificationDialog(project);
    }
  }

  const importPlottr = () => handleImport("plottr");
  const importMarkdown = () => handleImport("markdown");
  const importYWriter = () => handleImport("ywriter");
  const importLongform = () => handleImport("longform");
  const importLongformVault = () => handleImport("longformVault");

  function openLongformImportDialog() {
    showLongformImportDialog = true;
  }

  function closeProject() {
    currentProject.setProject(null);
  }

  // Check for updates on launch (delayed so it doesn't block startup)
  onMount(() => {
    const t = setTimeout(() => {
      checkForUpdate();
    }, 3000);
    return () => clearTimeout(t);
  });

  // Handle menu events from Tauri
  onMount(() => {
    const unlisten = listen<string>("menu-event", (event) => {
      const menuId = event.payload;

      switch (menuId) {
        case "new_project":
          showNewProjectDialog = true;
          break;
        case "import_plottr":
          importPlottr();
          break;
        case "import_ywriter":
          importYWriter();
          break;
        case "import_markdown":
          importMarkdown();
          break;
        case "import_longform":
          openLongformImportDialog();
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
        case "command_palette":
          showCommandPalette = true;
          break;
        case "quick_start":
          showQuickStart = true;
          break;
        case "toggle_sidebar":
          ui.toggleSidebar();
          break;
        case "toggle_references":
          ui.toggleReferencesPanel();
          break;
        case "sync":
          window.dispatchEvent(new CustomEvent("kindling:sync"));
          break;
        case "about":
          showAboutDialog = true;
          break;
        case "quit":
          exit(0);
          break;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  // Build command list with actions, filtered by context
  const paletteCommands = $derived(
    COMMAND_DEFS.filter((def) => {
      if (def.requiresProject && !currentProject.value) return false;
      if (def.requiresSourcePath && !currentProject.value?.source_path) return false;
      return true;
    }).map((def) => ({
      ...def,
      action: () => runCommand(def.id),
    }))
  );

  function runCommand(id: string) {
    switch (id) {
      case "export":
        if (currentProject.value) showExportDialog = true;
        break;
      case "close_project":
        closeProject();
        break;
      case "import_plottr":
        importPlottr();
        break;
      case "import_markdown":
        importMarkdown();
        break;
      case "import_longform":
        openLongformImportDialog();
        break;
      case "import_ywriter":
        importYWriter();
        break;
      case "project_settings":
        if (currentProject.value) showProjectSettings = true;
        break;
      case "sync":
        window.dispatchEvent(new CustomEvent("kindling:sync"));
        break;
      case "toggle_sidebar":
        ui.toggleSidebar();
        break;
      case "toggle_references":
        ui.toggleReferencesPanel();
        break;
      case "toggle_discovery_notes":
        window.dispatchEvent(new CustomEvent("kindling:toggleDiscoveryNotes"));
        break;
      case "toggle_editor_mode":
        window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
        break;
      case "quick_start":
        showQuickStart = true;
        break;
      case "kindling_settings":
        showKindlingSettings = true;
        break;
      case "about":
        showAboutDialog = true;
        break;
      case "quit":
        exit(0);
        break;
    }
  }

  // Global keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Cmd/Ctrl+K: Open command palette
    if ((event.metaKey || event.ctrlKey) && event.key === "k") {
      event.preventDefault();
      showCommandPalette = true;
      return;
    }
    // Cmd/Ctrl+E: Open export dialog
    if ((event.metaKey || event.ctrlKey) && event.key === "e") {
      event.preventDefault();
      if (currentProject.value && !showExportDialog) {
        showExportDialog = true;
      }
      return;
    }
    // Cmd/Ctrl+Shift+H: Open Quick Start
    if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === "H") {
      event.preventDefault();
      showQuickStart = true;
      return;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<UpdateBanner />

<main class="flex h-screen w-screen overflow-hidden bg-bg-primary">
  {#if currentProject.value}
    <Sidebar />
    <ScenePanel />
    <ReferencesPanel />
  {:else}
    <StartScreen
      {recentProjects}
      onImportLongform={openLongformImportDialog}
      onImportComplete={openReferenceClassificationDialog}
      onOpenQuickStart={() => (showQuickStart = true)}
      onNewProject={() => (showNewProjectDialog = true)}
    />
  {/if}
</main>

{#if ui.toast}
  {#key ui.toast.id}
    <ErrorToast message={ui.toast.message} onDismiss={() => ui.clearToast()} />
  {/key}
{/if}

<!-- Command palette (⌘K) -->
<CommandPalette
  bind:open={showCommandPalette}
  commands={paletteCommands}
  onClose={() => (showCommandPalette = false)}
/>

<!-- Guidance overlay (first-visit tips, one at a time, modal-style) -->
<GuidanceOverlay />

<!-- Onboarding overlay (shown on first launch) -->
<Onboarding
  onImportLongform={openLongformImportDialog}
  onImportComplete={openReferenceClassificationDialog}
/>

{#if showReferenceClassificationDialog && referenceClassificationProjectId}
  <ReferenceClassificationDialog
    projectId={referenceClassificationProjectId}
    onClose={closeReferenceClassificationDialog}
    onComplete={handleReferenceClassificationComplete}
  />
{/if}

<!-- New Project Dialog (triggered by File menu or StartScreen) -->
{#if showNewProjectDialog}
  <NewProjectDialog onClose={() => (showNewProjectDialog = false)} />
{/if}

<!-- Quick Start Dialog (triggered by Help menu) -->
{#if showQuickStart}
  <QuickStartDialog onClose={() => (showQuickStart = false)} />
{/if}

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

{#if showLongformImportDialog}
  <ImportLongformDialog
    onSelectIndex={() => {
      showLongformImportDialog = false;
      importLongform();
    }}
    onSelectVault={() => {
      showLongformImportDialog = false;
      importLongformVault();
    }}
    onClose={() => (showLongformImportDialog = false)}
  />
{/if}

<!-- Export Success Dialog -->
{#if exportResult}
  <ExportSuccessDialog result={exportResult} onClose={() => (exportResult = null)} />
{/if}

<!-- About Dialog -->
{#if showAboutDialog}
  <AboutDialog onClose={() => (showAboutDialog = false)} />
{/if}
