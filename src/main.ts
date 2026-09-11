import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import { currentProject } from "./lib/stores/project.svelte";
import { ui } from "./lib/stores/ui.svelte";
import type { Project, Chapter } from "./lib/types";
import { backgroundQA } from "./lib/qaMode";

let contentReady!: () => void;
const initialContent = new Promise<void>((resolve) => (contentReady = resolve));
const app = mount(App, {
  target: document.getElementById("app")!,
  props: { onReady: contentReady },
});

// The bootstrap keeps its overlay until initial content has settled and Svelte
// has flushed it, then waits for fonts, images and a paint opportunity.
export const ready = initialContent.then(() => tick());
export const focusAfterStartup = app.focusAfterStartup;

import { IMPORT_COMMANDS, isImportType, type ImportType } from "./lib/importFormats";

// Expose Tauri invoke and store helpers for E2E testing
// This allows WebDriver tests to call Tauri commands and update stores directly
declare global {
  interface Window {
    __KINDLING_TEST__?: {
      invoke: typeof invoke;
      importCommands: typeof IMPORT_COMMANDS;
      importProject: (path: string, format?: ImportType) => Promise<Project>;
      disableGuidance: () => void;
      visualPreferences: () => {
        guidanceEnabled: boolean;
        referencesPanelWidth: number;
        sidebarCollapsed: boolean;
        referencesPanelCollapsed: boolean;
      };
      setVisualPreferences: (
        value: ReturnType<NonNullable<Window["__KINDLING_TEST__"]>["visualPreferences"]>
      ) => void;
      creationObserverVersion: 1;
      onProjectCreated?: (command: string, project: Project) => void;
    };
  }
}

// Helper to import a project and update the frontend state
// For E2E testing, this also loads chapters directly rather than relying on $effect
async function importProject(
  path: string,
  format: ImportType = "plottr"
): Promise<
  Project & { _debug?: { chapterCount: number; storeChapterCount: number; hasProject: boolean } }
> {
  // Capture the observer synchronously for this action, before any async work.
  const onCreated = window.__KINDLING_TEST__?.onProjectCreated;
  ui.startImport();
  try {
    const command = isImportType(format) ? IMPORT_COMMANDS[format] : undefined;
    if (!command) throw new Error(`Unsupported test import format: ${format}`);
    const project = await invoke<Project>(command, { path });
    onCreated?.(command, project);
    currentProject.setProject(project);

    // Load and set chapters directly for E2E testing
    // The Sidebar $effect would normally do this, but it runs async
    const chapters = await invoke<Chapter[]>("get_chapters", {
      projectId: project.id,
    });
    currentProject.setChapters(chapters);

    ui.setView("editor");

    // Wait for Svelte to update the DOM before returning
    // This ensures E2E tests can find the rendered chapter elements
    // Multiple ticks and RAF needed for Svelte 5 reactivity to fully propagate
    await tick();
    if (backgroundQA()) await new Promise((r) => setTimeout(r, 0));
    else await new Promise((r) => requestAnimationFrame(r));
    await tick();

    // Add debug info - verify chapters are in the store, not just the local variable
    return {
      ...project,
      _debug: {
        chapterCount: chapters.length,
        storeChapterCount: currentProject.chapters.length,
        hasProject: !!currentProject.value,
      },
    };
  } finally {
    ui.finishImport();
  }
}

// Always expose for E2E testing - the test helper checks for this
window.__KINDLING_TEST__ = {
  creationObserverVersion: 1,
  invoke,
  importCommands: IMPORT_COMMANDS,
  importProject,
  disableGuidance: () => ui.setGuidanceEnabled(false),
  visualPreferences: () => ({
    guidanceEnabled: ui.guidanceEnabled,
    referencesPanelWidth: ui.referencesPanelWidth,
    sidebarCollapsed: ui.sidebarCollapsed,
    referencesPanelCollapsed: ui.referencesPanelCollapsed,
  }),
  setVisualPreferences: (value) => {
    ui.sidebarCollapsed = value.sidebarCollapsed;
    ui.referencesPanelCollapsed = value.referencesPanelCollapsed;
    ui.setReferencesPanelWidth(value.referencesPanelWidth);
    ui.setGuidanceEnabled(value.guidanceEnabled);
  },
};

export default app;
