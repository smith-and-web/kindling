<script lang="ts">
  import { IMPORT_FORMATS, importTypeForCommand, isImportType } from "./lib/importFormats";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { exit } from "@tauri-apps/plugin-process";
  import { onMount, tick } from "svelte";
  import { runImport, type ImportType } from "./lib/utils/import";
  import AboutDialog from "./lib/components/AboutDialog.svelte";
  import FeedbackDialog from "./lib/components/FeedbackDialog.svelte";
  import Onboarding from "./lib/components/Onboarding.svelte";
  import ReferencesPanel from "./lib/components/ReferencesPanel.svelte";
  import FindReplaceDialog from "./lib/components/FindReplaceDialog.svelte";
  import ScenePanel from "./lib/components/ScenePanel.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StartScreen from "./lib/components/StartScreen.svelte";
  import KindlingSettingsDialog from "./lib/components/KindlingSettingsDialog.svelte";
  import ProjectSettingsDialog from "./lib/components/ProjectSettingsDialog.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import ExportSuccessDialog from "./lib/components/ExportSuccessDialog.svelte";
  import ErrorToast from "./lib/components/ErrorToast.svelte";
  import ConfirmDialog from "./lib/components/ConfirmDialog.svelte";
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
  import { session } from "./lib/stores/session.svelte";
  import { synopsisSaves, type SynopsisDraft } from "./lib/stores/synopsisSaves.svelte";
  import { proseSaves, type ProseSave } from "./lib/utils/proseSaves";
  import { ui } from "./lib/stores/ui.svelte";
  import type { ProseDocument } from "./lib/utils/proseSearch";
  import type { Project, ExportResult, Chapter, Scene, Beat } from "./lib/types";

  let scenePanel: ReturnType<typeof ScenePanel> | undefined = $state();
  let searchDialog: ReturnType<typeof FindReplaceDialog> | undefined = $state();
  let search = $state<{ projectId: string; scope: "scene" | "project"; replace: boolean } | null>(
    null
  );

  // Native New/Import commands can change projects while a modal is open.
  $effect(() => {
    if (search && search.projectId !== currentProject.value?.id) search = null;
  });

  function openSearch(scope: "scene" | "project", replace = false) {
    const projectId = currentProject.value?.id;
    if (!projectId) return;
    if (search?.projectId === projectId) searchDialog?.configure(scope, replace);
    search = { projectId, scope, replace };
  }

  async function openSearchScene(doc: ProseDocument) {
    if (!currentProject.value) return;
    const projectId = currentProject.value.id;
    const chapters = await invoke<Chapter[]>("get_chapters", {
      projectId: currentProject.value.id,
    });
    const chapter = chapters.find((chapter) => chapter.id === doc.chapter_id);
    const scenes = await invoke<Scene[]>("get_scenes", { chapterId: doc.chapter_id });
    const scene = scenes.find((scene) => scene.id === doc.scene_id);
    if (!chapter || !scene) throw new Error("This scene is no longer available.");
    const beats = await invoke<Beat[]>("get_beats", { sceneId: scene.id });
    if (currentProject.value?.id !== projectId) return;
    currentProject.setChapters(chapters);
    currentProject.setCurrentChapter(chapter);
    currentProject.setScenes(scenes);
    currentProject.setCurrentScene(scene);
    await tick();
    currentProject.setBeats(beats);
    if (doc.beat_title !== null) ui.setExpandedBeat(doc.id);
  }

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
  let showFeedbackDialog = $state(false);

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

  async function handleImport(type: ImportType) {
    const project = await runImport(type);
    if (!project) return;
    activateImportedProject(project);
    if (IMPORT_FORMATS[type].references) {
      openReferenceClassificationDialog(project);
    }
  }

  const importLongform = () => handleImport("longform");
  const importLongformVault = () => handleImport("longformVault");

  function handleImportCommand(command: string): boolean {
    const type = importTypeForCommand(command);
    if (!type) return false;
    if (type === "longform") openLongformImportDialog();
    else void handleImport(type);
    return true;
  }

  function openLongformImportDialog() {
    showLongformImportDialog = true;
  }

  function closeProject() {
    search = null;
    currentProject.setProject(null);
  }

  let closePending = $state(false);
  let updatePending = $state(false);
  let closeRequest: Promise<boolean> | null = null;
  let discardQuitDrafts = $state.raw<SynopsisDraft[] | null>(null);
  let discardQuitProse = $state.raw<ProseSave[]>([]);

  const interactionBlocked = $derived(closePending || updatePending || discardQuitDrafts !== null);

  let quitConfirmation: HTMLDialogElement | undefined = $state();

  function focusQuitConfirmation(node: HTMLDialogElement) {
    node.showModal();
    node.querySelector<HTMLButtonElement>("button")?.focus();
  }

  // Capture before any child/window shortcut handler can act on the same event.
  onMount(() => {
    const guardKeyboard = (event: KeyboardEvent) => {
      if (!interactionBlocked) return;
      if (discardQuitDrafts && !closePending && event.key === "Tab") {
        event.preventDefault();
        event.stopImmediatePropagation();
        const buttons = Array.from(
          quitConfirmation?.querySelectorAll<HTMLButtonElement>("button") ?? []
        );
        const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
        buttons[(current + (event.shiftKey ? buttons.length - 1 : 1)) % buttons.length]?.focus();
        return;
      }
      const inConfirmation =
        event.target instanceof HTMLElement &&
        event.target.closest("[data-quit-confirmation]") !== null;
      if (
        inConfirmation &&
        !closePending &&
        !event.metaKey &&
        !event.ctrlKey &&
        event.key !== "Escape"
      )
        return;
      event.preventDefault();
      event.stopImmediatePropagation();
      if (event.key === "Escape" && !closePending && !updatePending) discardQuitDrafts = null;
    };
    window.addEventListener("keydown", guardKeyboard, true);
    return () => window.removeEventListener("keydown", guardKeyboard, true);
  });

  function flushBeforeClose() {
    if (discardQuitDrafts || updatePending) return Promise.resolve(false);
    if (closeRequest) return closeRequest;
    closePending = true;
    closeRequest = saveBeforeClose().finally(() => {
      closePending = false;
      closeRequest = null;
    });
    return closeRequest;
  }

  async function flushProseBeforeExit() {
    // This existing editor hook submits debounced page and beat edits to proseSaves.
    await scenePanel?.prepareForSearch();
    await proseSaves.flush();
    if (proseSaves.draftsForRecovery().length) {
      throw new Error(
        "Prose changes could not be saved. Review the unsaved drafts in Find and Replace."
      );
    }
  }

  async function saveBeforeClose() {
    let failed = false;
    try {
      await flushProseBeforeExit();
    } catch {
      failed = true;
    }
    try {
      await synopsisSaves.flush();
    } catch {
      failed = true;
    }
    if (failed) {
      discardQuitProse = proseSaves.draftsForRecovery();
      discardQuitDrafts = synopsisSaves.snapshot();
      return false;
    }
    await flushPositionBeforeClose();
    return true;
  }

  async function flushPositionBeforeClose() {
    try {
      await session.flush();
    } catch (error) {
      console.error("Failed to save writing position before closing:", error);
    }
  }

  async function quit() {
    if (!(await flushBeforeClose())) return;
    try {
      await exit(0);
    } catch (error) {
      console.error("Failed to quit:", error);
    }
  }

  async function quitAndDiscard() {
    const approved = discardQuitDrafts;
    if (!approved || closePending) return;
    closePending = true;
    try {
      await proseSaves.discard(discardQuitProse, () =>
        scenePanel?.discardProseDraftsForClose(discardQuitProse)
      );
      await synopsisSaves.discardAll(approved);
      await flushPositionBeforeClose();
      discardQuitDrafts = null;
      await exit(0);
    } catch (error) {
      discardQuitDrafts = null;
      ui.showError(`Could not quit: ${String(error)}`);
    } finally {
      closePending = false;
    }
  }

  onMount(() => {
    // Tauri awaits this handler before destroying the window.
    const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await flushBeforeClose())) event.preventDefault();
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  });

  let retryingSynopses = $state(false);
  async function retrySynopses() {
    retryingSynopses = true;
    try {
      await synopsisSaves.flush();
    } catch {
      // The banner and per-scene errors remain visible until saving succeeds.
    } finally {
      retryingSynopses = false;
    }
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
      if (interactionBlocked) return;
      const menuId = event.payload;

      if (handleImportCommand(menuId)) return;
      if (["find", "find_replace", "find_project"].includes(menuId)) {
        runCommand(menuId);
        return;
      }
      switch (menuId) {
        case "new_project":
          showNewProjectDialog = true;
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
        case "send_feedback":
          showFeedbackDialog = true;
          break;
        case "quit":
          void quit();
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
    if (interactionBlocked) return;
    if (handleImportCommand(id)) return;
    switch (id) {
      case "find":
        openSearch("scene");
        break;
      case "find_replace":
        openSearch("scene", true);
        break;
      case "find_project":
        openSearch("project", true);
        break;
      case "export":
        if (currentProject.value) showExportDialog = true;
        break;
      case "close_project":
        closeProject();
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
      case "detect_references":
        window.dispatchEvent(new CustomEvent("kindling:detectReferences"));
        break;
      case "detect_all_references":
        window.dispatchEvent(new CustomEvent("kindling:detectAllReferences"));
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
        void quit();
        break;
    }
  }

  // Global keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
      if (!currentProject.value) return;
      event.preventDefault();
      openSearch(event.shiftKey ? "project" : "scene", event.altKey || event.shiftKey);
      return;
    }
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

<div inert={interactionBlocked} hidden={interactionBlocked}>
  {#if search && search.projectId === currentProject.value?.id}
    {#key search.projectId}
      <FindReplaceDialog
        bind:this={searchDialog}
        projectId={search.projectId}
        sceneId={currentProject.currentScene?.id ?? null}
        initialScope={search.scope}
        showReplace={search.replace}
        prepare={async () => {
          await scenePanel?.prepareForSearch();
        }}
        onDiscardDrafts={async (drafts) => {
          await scenePanel?.discardFailedSaves(drafts);
        }}
        onApplied={(changes) => scenePanel?.applySearchChanges(changes)}
        onOpenScene={openSearchScene}
        onClose={() => (search = null)}
      />
    {/key}
  {/if}
</div>

<UpdateBanner
  disabled={closePending || discardQuitDrafts !== null}
  bind:restarting={updatePending}
  prepare={flushProseBeforeExit}
/>

{#if discardQuitDrafts}
  <dialog
    data-quit-confirmation
    bind:this={quitConfirmation}
    use:focusQuitConfirmation
    oncancel={(event) => event.preventDefault()}
    class="border-0 p-0 max-w-none max-h-none"
  >
    <ConfirmDialog
      title={discardQuitProse.length
        ? "Quit without saving writing changes?"
        : "Quit without saving synopsis changes?"}
      message={discardQuitProse.length
        ? "Some prose or synopsis changes could not be saved. Quit and discard these unsaved writing changes, or keep editing to retry saving."
        : "Some synopsis changes could not be saved. Quit and discard these unsaved synopsis changes, or keep editing to retry saving."}
      confirmLabel="Quit and discard"
      cancelLabel="Keep editing"
      onConfirm={quitAndDiscard}
      onCancel={() => {
        if (!closePending) discardQuitDrafts = null;
      }}
    />
  </dialog>
{/if}

{#if synopsisSaves.failedCount && !discardQuitDrafts}
  <div
    role="alert"
    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-press-toast rounded-lg bg-press-surface border border-press-error p-4 shadow-lg text-press-ui"
  >
    <p class="text-press-error">Your synopsis changes have not been saved.</p>
    <button
      onclick={retrySynopses}
      disabled={retryingSynopses || discardQuitDrafts !== null}
      aria-label="Retry all synopsis saves"
      class="mt-2 underline text-press-text disabled:opacity-50"
      >{retryingSynopses ? "Saving..." : "Retry saving"}</button
    >
  </div>
{/if}

<main
  inert={interactionBlocked}
  aria-busy={closePending || updatePending}
  class="flex h-screen w-screen overflow-hidden bg-press-bg"
>
  {#if currentProject.value}
    <Sidebar />
    <ScenePanel bind:this={scenePanel} />
    <ReferencesPanel />
  {:else}
    <StartScreen
      {recentProjects}
      onImportLongform={openLongformImportDialog}
      onImportComplete={(project, type) => {
        if (IMPORT_FORMATS[type].references) {
          openReferenceClassificationDialog(project);
        }
      }}
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

<div inert={interactionBlocked} hidden={interactionBlocked}>
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
    onImportComplete={(project: Project, type: string) => {
      if (isImportType(type) && IMPORT_FORMATS[type].references) {
        openReferenceClassificationDialog(project);
      }
    }}
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
    <AboutDialog
      onClose={() => (showAboutDialog = false)}
      onSendFeedback={() => {
        showAboutDialog = false;
        showFeedbackDialog = true;
      }}
    />
  {/if}

  <!-- Feedback Dialog (opened from the Help menu or the About dialog) -->
  {#if showFeedbackDialog}
    <FeedbackDialog onClose={() => (showFeedbackDialog = false)} />
  {/if}
</div>
