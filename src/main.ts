import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { currentProject } from "./lib/stores/project.svelte";
import { ui } from "./lib/stores/ui.svelte";
import type { Project } from "./lib/types";

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
async function importProject(path: string): Promise<Project> {
  ui.startImport();
  try {
    const project = await invoke<Project>("import_plottr", { path });
    currentProject.setProject(project);
    ui.setView("editor");
    return project;
  } finally {
    ui.finishImport();
  }
}

// Always expose for E2E testing - the test helper checks for this
window.__KINDLING_TEST__ = { invoke, importProject };

export default app;
