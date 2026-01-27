<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { FileText, Kanban, Trash2, Loader2, Settings, PenTool } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Project } from "../types";
  import Tooltip from "./Tooltip.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import KindlingSettingsDialog from "./KindlingSettingsDialog.svelte";

  interface Props {
    recentProjects: Project[];
  }

  let { recentProjects = $bindable() }: Props = $props();

  let deletingProjectId = $state<string | null>(null);
  let hoveredProjectId = $state<string | null>(null);
  let projectToDelete = $state<Project | null>(null);
  let showSettingsDialog = $state(false);

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
        ui.showError(`Import failed: ${e}`);
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
        ui.showError(`Import failed: ${e}`);
      } finally {
        ui.finishImport();
      }
    }
  }

  async function importYWriter() {
    const path = await open({
      multiple: false,
      filters: [{ name: "yWriter 7", extensions: ["yw7"] }],
    });

    if (path) {
      ui.startImport();
      try {
        const project = await invoke<Project>("import_ywriter", { path });
        currentProject.setProject(project);
        ui.setView("editor");
      } catch (e) {
        console.error("Failed to import yWriter file:", e);
        ui.showError(`Import failed: ${e}`);
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
      ui.showError(`Failed to open project: ${e}`);
    }
  }

  function showDeleteConfirmation(event: MouseEvent, project: Project) {
    event.stopPropagation();
    projectToDelete = project;
  }

  async function confirmDeleteProject() {
    if (!projectToDelete) return;

    const project = projectToDelete;
    projectToDelete = null;
    deletingProjectId = project.id;

    try {
      await invoke("delete_project", { projectId: project.id });
      recentProjects = recentProjects.filter((p) => p.id !== project.id);
    } catch (e) {
      console.error("Failed to delete project:", e);
      ui.showError(`Failed to delete project: ${e}`);
    } finally {
      deletingProjectId = null;
    }
  }

  function cancelDeleteProject() {
    projectToDelete = null;
  }
</script>

<div class="flex-1 flex flex-col items-center justify-center p-8 relative">
  <!-- Settings button in corner -->
  <div class="absolute top-4 right-4">
    <Tooltip text="Kindling Settings" position="left">
      <button
        onclick={() => (showSettingsDialog = true)}
        class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-card rounded-lg transition-colors"
        aria-label="Kindling Settings"
      >
        <Settings class="w-5 h-5" />
      </button>
    </Tooltip>
  </div>

  <div class="max-w-2xl w-full">
    <!-- Logo & Tagline -->
    <div class="text-center mb-12">
      <div class="flex justify-center mb-4">
        <!-- Kindling Brand Logo Mark -->
        <svg width="120" height="120" viewBox="0 0 1024 1024" class="drop-shadow-lg">
          <defs>
            <linearGradient
              id="bookGradient"
              x1="509"
              y1="739"
              x2="512"
              y2="609"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0" stop-color="#501D0F" />
              <stop offset="1" stop-color="#89492B" />
            </linearGradient>
          </defs>
          <path
            fill="#E25227"
            d="M495.154 288.138C498.378 289.608 505.914 297.445 508.313 300.3C526.269 321.669 539.502 342.79 542.378 370.879C549.115 436.662 490.007 467.903 476.848 526.209C472.415 545.849 474.731 568.443 482.366 587.122C483.763 590.541 490.702 602.324 490.569 604.62L489.492 604.081C466.698 587.526 440.031 561.25 430.639 534.248C403.556 456.377 485.481 402.143 496.346 330.247C498.679 314.804 498.133 303.222 495.154 288.138Z"
          />
          <path
            fill="url(#bookGradient)"
            d="M679.512 611.655C679.948 623.671 679.803 636.504 679.711 648.539C679.819 650.345 679.874 650.354 679.431 652.203C678.578 653.105 645.852 669.482 641.946 671.541L551.504 719.091C543.78 723.161 536.109 727.33 528.491 731.597C523.974 734.127 516.055 738.826 511.383 740.578C504.39 737.13 495.509 731.912 488.494 728.114L438.452 701.202C418.928 690.993 399.491 680.618 380.143 670.078C368.598 663.83 355.674 656.975 344.543 650.136C344.526 637.556 344.602 624.446 344.219 611.898C359.414 619.412 379.065 631.083 394.357 639.52L470.64 681.021C479.247 685.796 487.81 690.649 496.33 695.578C500.794 698.136 506.902 701.896 511.48 703.945C532.487 690.677 560.415 676.473 582.602 664.63C615.066 647.267 647.37 629.608 679.512 611.655Z"
          />
          <path
            fill="#F0912D"
            d="M567.225 404.156C568.003 404.556 568.509 404.868 568.965 405.666C588.192 439.301 602.938 484.462 595.366 523.183C587.91 561.316 558.078 585.951 527.823 605.935L518.591 611.429C510.152 597.693 506.392 586.985 503.912 571.209C497.26 528.911 522.684 499.522 542.221 465.408C552.466 447.518 562.786 424.502 567.225 404.156Z"
          />
          <path
            fill="#F0912D"
            d="M359.24 550.125C365.269 552.715 379.71 564.412 385.223 568.751C425.497 600.45 464.809 634.729 496.049 675.611C499.494 680.119 508.175 690.937 510.126 695.939C503.857 692.741 497.548 689.208 491.532 685.547C448.751 659.511 402.641 638.037 359.663 612.561C359.387 591.884 359.872 570.732 359.24 550.125Z"
          />
          <path
            fill="#F0912D"
            d="M664.174 549.059L664.428 593.205C664.417 599.159 664.625 607.179 664.213 612.947C655.817 616.909 647.229 621.897 639.067 626.408L603.341 646.032C582.669 657.264 562.058 668.608 541.509 680.063C534.744 683.835 526.75 687.959 520.246 691.793C518.071 693.047 515.906 694.089 513.66 695.2L519.513 687.005C556.887 634.717 612.459 587.041 664.174 549.059Z"
          />
        </svg>
      </div>
      <h1 class="text-5xl font-heading font-semibold text-accent mb-2">kindling</h1>
      <p class="text-text-secondary text-lg">Spark your draft</p>
    </div>

    <!-- Import Options -->
    <div data-testid="import-section" class="bg-bg-panel rounded-lg p-6 mb-8">
      <h2 class="text-xl font-heading font-medium text-text-primary mb-4">Import Your Outline</h2>
      <div class="grid grid-cols-3 gap-4">
        <button
          onclick={importPlottr}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <Kanban class="w-10 h-10 text-accent mb-2" />
          <span class="text-text-primary font-medium">Plottr</span>
          <span class="text-text-secondary text-sm">.pltr</span>
        </button>

        <button
          onclick={importYWriter}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <PenTool class="w-10 h-10 text-accent mb-2" />
          <span class="text-text-primary font-medium">yWriter</span>
          <span class="text-text-secondary text-sm">.yw7</span>
        </button>

        <button
          onclick={importMarkdown}
          class="flex flex-col items-center p-4 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer"
        >
          <FileText class="w-10 h-10 text-accent mb-2" />
          <span class="text-text-primary font-medium">Markdown</span>
          <span class="text-text-secondary text-sm">.md</span>
        </button>
      </div>
    </div>

    <!-- Recent Projects -->
    {#if recentProjects.length > 0}
      <div data-testid="recent-projects" class="bg-bg-panel rounded-lg p-6">
        <h2 class="text-xl font-heading font-medium text-text-primary mb-4">Recent Projects</h2>
        <div class="space-y-2 max-h-[40vh] overflow-y-auto pr-1">
          {#each recentProjects as project (project.id)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="relative flex items-center bg-bg-card rounded-lg hover:bg-beat-header transition-colors"
              onmouseenter={() => (hoveredProjectId = project.id)}
              onmouseleave={() => (hoveredProjectId = null)}
            >
              <button
                onclick={() => openProject(project)}
                class="flex-1 flex items-center justify-between p-3 cursor-pointer text-left"
              >
                <div>
                  <span class="text-text-primary font-medium">{project.name}</span>
                  <span class="text-text-secondary text-sm ml-2">({project.source_type})</span>
                </div>
                <span class="text-text-secondary text-sm">
                  {new Date(project.modified_at).toLocaleDateString()}
                </span>
              </button>

              <!-- Delete button - visible on hover -->
              <div
                class="pr-3 transition-opacity"
                class:opacity-0={hoveredProjectId !== project.id &&
                  deletingProjectId !== project.id}
                class:opacity-100={hoveredProjectId === project.id ||
                  deletingProjectId === project.id}
              >
                <Tooltip text="Delete project" position="left">
                  <button
                    onclick={(e) => showDeleteConfirmation(e, project)}
                    disabled={deletingProjectId === project.id}
                    class="p-1.5 text-text-secondary hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors disabled:opacity-50"
                    aria-label="Delete project"
                  >
                    {#if deletingProjectId === project.id}
                      <Loader2 class="w-4 h-4 animate-spin" />
                    {:else}
                      <Trash2 class="w-4 h-4" />
                    {/if}
                  </button>
                </Tooltip>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Import Progress -->
    {#if ui.isImporting}
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4">
          <h3 class="text-lg font-heading font-medium text-text-primary mb-4">Importing...</h3>
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

<!-- Delete Project Confirmation -->
{#if projectToDelete}
  <ConfirmDialog
    title="Delete Project"
    message="Are you sure you want to delete &quot;{projectToDelete.name}&quot;? This will permanently delete the project and all its chapters, scenes, beats, and snapshots. This cannot be undone."
    confirmLabel="Delete Project"
    onConfirm={confirmDeleteProject}
    onCancel={cancelDeleteProject}
  />
{/if}

<!-- Kindling Settings Dialog -->
{#if showSettingsDialog}
  <KindlingSettingsDialog
    onClose={() => (showSettingsDialog = false)}
    onSave={() => (showSettingsDialog = false)}
  />
{/if}
