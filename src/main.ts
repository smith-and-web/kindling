import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import { currentProject } from "./lib/stores/project.svelte";
import { ui } from "./lib/stores/ui.svelte";
import type { Project, Chapter } from "./lib/types";

const app = mount(App, {
  target: document.getElementById("app")!,
});

// Expose Tauri invoke and store helpers for E2E testing
// This allows WebDriver tests to call Tauri commands and update stores directly
declare global {
  interface Window {
    __KINDLING_TEST__?: {
      invoke: typeof invoke;
      importProject: (path: string) => Promise<Project>;
    };
  }
}

// Helper to import a project and update the frontend state
// For E2E testing, this also loads chapters directly rather than relying on $effect
async function importProject(
  path: string
): Promise<
  Project & { _debug?: { chapterCount: number; storeChapterCount: number; hasProject: boolean } }
> {
  ui.startImport();
  try {
    const project = await invoke<Project>("import_plottr", { path });
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
    await new Promise((r) => requestAnimationFrame(r));
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
window.__KINDLING_TEST__ = { invoke, importProject };

export default app;
