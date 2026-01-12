<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project } from "../types";

  interface Props {
    recentProjects: Project[];
  }

  let { recentProjects }: Props = $props();

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

  async function importScrivener() {
    const path = await open({
      multiple: false,
      directory: true,
    });

    if (path) {
      ui.startImport();
      try {
        const project = await invoke<Project>("import_scrivener", { path });
        currentProject.setProject(project);
        ui.setView("editor");
      } catch (e) {
        console.error("Failed to import Scrivener project:", e);
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

  async function openProject(project: Project) {
    try {
      const loaded = await invoke<Project>("get_project", { id: project.id });
      currentProject.setProject(loaded);
      ui.setView("editor");
    } catch (e) {
      console.error("Failed to open project:", e);
      alert(`Failed to open project: ${e}`);
    }
  }
</script>

<div class="flex-1 flex flex-col items-center justify-center p-8">
  <div class="max-w-2xl w-full">
    <!-- Logo & Tagline -->
    <div class="text-center mb-12">
      <h1 class="text-5xl font-bold text-accent mb-2">Kindling</h1>
      <p class="text-text-secondary text-lg">Spark your draft</p>
    </div>

    <!-- Import Options -->
    <div class="bg-bg-panel rounded-lg p-6 mb-8">
      <h2 class="text-xl font-semibold text-text-primary mb-4">
        Import Your Outline
      </h2>
      <div class="grid grid-cols-3 gap-4">
        <button
          onclick={importPlottr}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <svg
            class="w-10 h-10 text-accent mb-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
            />
          </svg>
          <span class="text-text-primary font-medium">Plottr</span>
          <span class="text-text-secondary text-sm">.pltr</span>
        </button>

        <button
          onclick={importScrivener}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <svg
            class="w-10 h-10 text-accent mb-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
            />
          </svg>
          <span class="text-text-primary font-medium">Scrivener</span>
          <span class="text-text-secondary text-sm">.scriv</span>
        </button>

        <button
          onclick={importMarkdown}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <svg
            class="w-10 h-10 text-accent mb-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <span class="text-text-primary font-medium">Markdown</span>
          <span class="text-text-secondary text-sm">.md</span>
        </button>
      </div>
    </div>

    <!-- Recent Projects -->
    {#if recentProjects.length > 0}
      <div class="bg-bg-panel rounded-lg p-6">
        <h2 class="text-xl font-semibold text-text-primary mb-4">
          Recent Projects
        </h2>
        <div class="space-y-2">
          {#each recentProjects as project}
            <button
              onclick={() => openProject(project)}
              class="w-full flex items-center justify-between p-3 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer text-left"
            >
              <div>
                <span class="text-text-primary font-medium">{project.name}</span
                >
                <span class="text-text-secondary text-sm ml-2"
                  >({project.source_type})</span
                >
              </div>
              <span class="text-text-secondary text-sm">
                {new Date(project.modified_at).toLocaleDateString()}
              </span>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Import Progress -->
    {#if ui.isImporting}
      <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      >
        <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4">
          <h3 class="text-lg font-semibold text-text-primary mb-4">
            Importing...
          </h3>
          <div class="w-full bg-bg-card rounded-full h-2 mb-2">
            <div
              class="bg-accent h-2 rounded-full transition-all"
              style="width: {ui.importProgress}%"
            ></div>
          </div>
          <p class="text-text-secondary text-sm">{ui.importStatus}</p>
        </div>
      </div>
    {/if}
  </div>
</div>
