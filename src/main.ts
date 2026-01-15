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
async function importProject(path: string): Promise<Project> {
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
    await tick();

    return project;
  } finally {
    ui.finishImport();
  }
}

// Always expose for E2E testing - the test helper checks for this
window.__KINDLING_TEST__ = { invoke, importProject };

export default app;
