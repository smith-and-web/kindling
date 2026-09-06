<script lang="ts">
  import { IMPORT_FORMATS, type ImportType } from "../importFormats";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowDownAZ,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronsRight,
    FileText,
    ListChevronsDownUp,
    Loader2,
    Kanban,
    BookOpen,
    CircleDot,
    FilePlus,
    PenTool,
    Scroll,
    User,
    Users,
    Zap,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import BrandMark from "./BrandMark.svelte";
  import { ui, type OnboardingStep } from "../stores/ui.svelte";
  import type { ImportPreview, Project } from "../types";
  import { pickScrivenerProjectPath } from "$lib/utils/import";

  interface Props {
    onImportLongform?: () => void;
    onImportComplete?: (project: Project, type: string) => void;
  }

  let { onImportComplete, onImportLongform }: Props = $props();

  // Guided import wizard state (within import step)
  type GuidedFormat = Exclude<ImportType, "longformVault">;
  let guidedPreview = $state<ImportPreview | null>(null);
  let guidedPath = $state<string | null>(null);
  let guidedFormat = $state<GuidedFormat | null>(null);
  let guidedError = $state<string | null>(null);
  let guidedLoading = $state(false);

  const STEP_ORDER: OnboardingStep[] = [
    "welcome",
    "tour-sidebar",
    "tour-editor",
    "tour-references",
    "import",
  ];

  async function trySampleProject() {
    ui.startImport();
    try {
      const project = await invoke<Project>("create_sample_project");
      currentProject.setProject(project);
      ui.completeOnboarding();
      ui.setView("editor");
    } catch (e) {
      console.error("Failed to create sample project:", e);
      ui.showError(`Failed to create sample project: ${e}`);
    } finally {
      ui.finishImport();
    }
  }

  async function startGuidedImport(format: GuidedFormat) {
    guidedError = null;
    guidedPreview = null;
    guidedPath = null;
    guidedFormat = format;
    const config = IMPORT_FORMATS[format];
    const path =
      format === "scrivener"
        ? await pickScrivenerProjectPath()
        : await open({
            multiple: false,
            ...(config.directory ? {} : { filters: config.filters }),
            directory: config.directory ?? false,
          });
    if (path) {
      await loadPreview(path, format);
    } else {
      guidedFormat = null;
    }
  }

  function handleLongformImportClick() {
    if (onImportLongform) {
      onImportLongform();
      return;
    }
    void startGuidedImport("longform");
  }

  async function loadPreview(path: string, format: GuidedFormat) {
    guidedLoading = true;
    guidedError = null;
    try {
      const preview = await invoke<ImportPreview>("preview_import", {
        path,
        format,
      });
      guidedPath = path;
      guidedPreview = preview;
    } catch (e) {
      guidedError = e instanceof Error ? e.message : String(e);
      guidedPreview = null;
    } finally {
      guidedLoading = false;
    }
  }

  async function confirmGuidedImport() {
    if (!guidedPath || !guidedFormat) return;
    const path = guidedPath;
    const format = guidedFormat;
    guidedPreview = null;
    guidedPath = null;
    guidedFormat = null;
    guidedError = null;

    ui.startImport();
    try {
      const project = await invoke<Project>(IMPORT_FORMATS[format].command, { path });
      currentProject.setProject(project);
      ui.completeOnboarding();
      ui.setView("editor");
      onImportComplete?.(project, format);
    } catch (e) {
      console.error("Import failed:", e);
      ui.showError(`Import failed: ${e}`);
    } finally {
      ui.finishImport();
    }
  }

  function backFromGuidedPreview() {
    guidedPreview = null;
    guidedPath = null;
    guidedFormat = null;
    guidedError = null;
  }

  function skipOnboarding() {
    ui.completeOnboarding();
  }
</script>

{#if ui.showOnboarding}
  <div
    data-testid="onboarding"
    class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal p-4"
  >
    <div class="max-w-2xl w-full">
      <!-- Progress indicator -->
      <div class="flex justify-center gap-2 mb-8">
        {#each STEP_ORDER as step, i}
          <button
            onclick={() => ui.goToStep(step)}
            class="w-2 h-2 rounded-full transition-all {i === ui.currentStepIndex
              ? 'bg-press-accent w-6'
              : i < ui.currentStepIndex
                ? 'bg-press-accent-wash'
                : 'bg-press-sunken'}"
            aria-label="Go to step {i + 1}"
          ></button>
        {/each}
      </div>

      <!-- Step content -->
      <div class="app-dialog-surface bg-press-surface rounded-xl p-8 shadow-press-overlay">
        {#if ui.onboardingStep === "welcome"}
          <!-- STEP 1: Welcome -->
          <div class="text-center">
            <!-- Logo -->
            <div class="flex justify-center mb-6">
              <BrandMark size={100} />
            </div>

            <h1 class="text-press-h1 font-heading font-semibold text-press-accent-text mb-3">
              Welcome to Kindling
            </h1>
            <p class="font-prose text-press-text text-press-body-lg mb-8 max-w-md mx-auto">
              Transform your outline into a finished draft. Import your story structure and start
              writing scene by scene.
            </p>

            <div class="flex flex-col gap-3">
              <button
                onclick={trySampleProject}
                class="w-full py-3 px-6 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors"
              >
                Try Sample Project
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="w-full py-3 px-6 bg-press-sunken hover:bg-press-sunken text-press-text font-medium rounded-lg transition-colors border border-press-border"
              >
                Take the Tour
              </button>
              <button
                onclick={skipOnboarding}
                class="text-press-muted hover:text-press-text text-press-ui transition-colors"
              >
                Skip and start importing
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "tour-sidebar"}
          <!-- STEP 2: Chapters & Scenes Tour -->
          <div>
            <div class="flex items-center gap-3 mb-4">
              <div
                class="w-10 h-10 rounded-lg bg-press-accent-wash flex items-center justify-center"
              >
                <ArrowDownAZ class="w-5 h-5 text-press-accent-text" />
              </div>
              <div>
                <h2 class="text-press-h2 font-heading font-semibold text-press-text">
                  Chapters & Scenes
                </h2>
                <p class="text-press-muted text-press-ui">The left sidebar</p>
              </div>
            </div>

            <!-- Visual mockup of sidebar -->
            <div class="bg-press-sunken rounded-lg p-4 mb-6">
              <div class="space-y-3">
                <!-- Chapter example -->
                <div class="relative">
                  <div class="flex items-center gap-2 p-2 rounded bg-press-surface">
                    <ChevronDown class="w-4 h-4 text-press-muted" />
                    <span class="text-press-text font-medium">The Letter</span>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-press-accent"></div>
                    <span
                      class="text-press-accent-text text-press-eyebrow font-medium whitespace-nowrap"
                      >Chapter</span
                    >
                  </div>
                </div>

                <!-- Scene examples -->
                <div class="ml-6 space-y-2">
                  <div class="relative">
                    <div
                      class="flex items-center gap-2 p-2 rounded bg-press-accent-wash border border-press-accent"
                    >
                      <CircleDot class="w-3 h-3 text-press-accent-text" />
                      <span class="text-press-text">Opening Scene</span>
                    </div>
                    <!-- Label -->
                    <div
                      class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                    >
                      <div class="w-8 h-px bg-press-warning"></div>
                      <span
                        class="text-press-warning text-press-eyebrow font-medium whitespace-nowrap"
                        >Active Scene</span
                      >
                    </div>
                  </div>

                  <div class="relative">
                    <div class="flex items-center gap-2 p-2 rounded hover:bg-press-surface">
                      <CircleDot class="w-3 h-3 text-press-muted" />
                      <span class="text-press-muted">The Discovery</span>
                    </div>
                    <!-- Label -->
                    <div
                      class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                    >
                      <div class="w-8 h-px bg-press-muted"></div>
                      <span
                        class="text-press-muted text-press-eyebrow font-medium whitespace-nowrap"
                        >Other Scene</span
                      >
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-press-sunken rounded-lg p-4 mb-6">
              <ul class="text-press-muted text-press-ui space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Click a chapter</strong> to expand or collapse its
                    scenes</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Click a scene</strong> to load it in the editor</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >The <strong class="text-press-text">highlighted scene</strong> is your current working
                    scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >Imported and template projects may include <strong class="text-press-text"
                      >Parts</strong
                    > (Acts) as collapsible groups above chapters</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >Screenplay projects use <strong class="text-press-text"
                      >Acts → Sequences → Scenes</strong
                    >
                    instead of Chapters → Scenes, with
                    <strong class="text-press-text">page count</strong> estimates alongside each act</span
                  >
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-press-border">
              <button
                onclick={() => ui.previousStep()}
                class="text-press-muted hover:text-press-text transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="py-2 px-4 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors flex items-center gap-1"
              >
                Next
                <ChevronRight class="w-4 h-4" />
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "tour-editor"}
          <!-- STEP 3: Synopsis & Beats Tour -->
          <div>
            <div class="flex items-center gap-3 mb-4">
              <div
                class="w-10 h-10 rounded-lg bg-press-warning/20 flex items-center justify-center"
              >
                <Zap class="w-5 h-5 text-press-warning" />
              </div>
              <div>
                <h2 class="text-press-h2 font-heading font-semibold text-press-text">
                  Synopsis & Beats
                </h2>
                <p class="text-press-muted text-press-ui">The main editor area</p>
              </div>
            </div>

            <!-- Visual mockup of editor -->
            <div class="bg-press-sunken rounded-lg p-4 mb-6 space-y-4">
              <!-- Synopsis section -->
              <div class="relative">
                <div class="bg-press-surface rounded-lg p-3">
                  <h4 class="text-press-eyebrow uppercase tracking-wide text-press-muted mb-2">
                    Synopsis
                  </h4>
                  <p class="text-press-text text-press-ui">
                    The hero receives the call to adventure and must decide whether to leave their
                    ordinary world behind...
                  </p>
                </div>
                <!-- Label -->
                <div
                  class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                >
                  <div class="w-8 h-px bg-press-accent"></div>
                  <span
                    class="text-press-accent-text text-press-eyebrow font-medium whitespace-nowrap"
                    >Scene Synopsis</span
                  >
                </div>
              </div>

              <!-- Beats section -->
              <div class="space-y-2">
                <div class="relative">
                  <div class="bg-press-sunken rounded-lg p-3 border-l-2 border-press-accent">
                    <div class="flex items-center justify-between mb-1">
                      <span class="text-press-text font-medium text-press-ui"
                        >The Messenger Arrives</span
                      >
                      <span class="text-press-eyebrow text-press-muted">Beat 1</span>
                    </div>
                    <p class="text-press-muted text-press-eyebrow">
                      A stranger appears at the door with urgent news...
                    </p>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-press-warning"></div>
                    <span
                      class="text-press-warning text-press-eyebrow font-medium whitespace-nowrap"
                      >Story Beat</span
                    >
                  </div>
                </div>

                <div class="relative">
                  <div class="bg-press-sunken rounded-lg p-3 border-l-2 border-transparent">
                    <div class="flex items-center justify-between mb-1">
                      <span class="text-press-text font-medium text-press-ui">The Decision</span>
                      <span class="text-press-eyebrow text-press-muted">Beat 2</span>
                    </div>
                    <p class="text-press-muted text-press-eyebrow">
                      Our hero weighs their options and makes a choice...
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-press-sunken rounded-lg p-4 mb-6">
              <ul class="text-press-muted text-press-ui space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >The <strong class="text-press-text">synopsis</strong> gives you the scene overview
                    from your outline</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Beats</strong> are the key story moments within each
                    scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span>Use beats as your writing prompts to draft each section</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >Switch between <strong class="text-press-text">Beat view</strong>
                    (outline-guided) and
                    <strong class="text-press-text">Page view</strong> (free-form writing) using the toggle
                    above</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >In screenplays, scene titles are <strong class="text-press-text"
                      >sluglines</strong
                    >
                    (e.g.,
                    <code class="text-press-eyebrow bg-press-sunken px-1 py-0.5 rounded"
                      >INT. CASTLE - NIGHT</code
                    >). Kindling auto-suggests locations from your reference panel</span
                  >
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-press-border">
              <button
                onclick={() => ui.previousStep()}
                class="text-press-muted hover:text-press-text transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="py-2 px-4 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors flex items-center gap-1"
              >
                Next
                <ChevronRight class="w-4 h-4" />
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "tour-references"}
          <!-- STEP 4: Reference Panel Tour -->
          <div>
            <div class="flex items-center gap-3 mb-4">
              <div
                class="w-10 h-10 rounded-lg bg-press-success-wash flex items-center justify-center"
              >
                <Users class="w-5 h-5 text-press-success" />
              </div>
              <div>
                <h2 class="text-press-h2 font-heading font-semibold text-press-text">
                  Reference Panel
                </h2>
                <p class="text-press-muted text-press-ui">The right sidebar</p>
              </div>
            </div>

            <!-- Visual mockup of reference panel -->
            <div class="bg-press-sunken rounded-lg p-4 mb-6">
              <!-- Tab buttons mockup -->
              <div class="relative flex gap-1 mb-4 bg-press-surface rounded-lg p-1">
                <div class="flex-1 py-2 px-3 rounded bg-press-sunken text-center">
                  <span class="text-press-text text-press-ui font-medium">Characters</span>
                </div>
                <div class="flex-1 py-2 px-3 rounded text-center">
                  <span class="text-press-muted text-press-ui">Locations</span>
                </div>
                <!-- Label -->
                <div
                  class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                >
                  <div class="w-8 h-px bg-press-accent"></div>
                  <span
                    class="text-press-accent-text text-press-eyebrow font-medium whitespace-nowrap"
                    >Tab Switcher</span
                  >
                </div>
              </div>

              <!-- Character cards mockup -->
              <div class="space-y-2">
                <div class="relative">
                  <div class="bg-press-surface rounded-lg p-3">
                    <div class="flex items-center gap-3">
                      <div
                        class="w-10 h-10 rounded-full bg-press-accent-wash flex items-center justify-center"
                      >
                        <User class="w-5 h-5 text-press-accent-text" />
                      </div>
                      <div>
                        <span class="text-press-text font-medium">Elena</span>
                        <p class="text-press-muted text-press-eyebrow">Protagonist</p>
                      </div>
                    </div>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-press-warning"></div>
                    <span
                      class="text-press-warning text-press-eyebrow font-medium whitespace-nowrap"
                      >Character Card</span
                    >
                  </div>
                </div>

                <div class="bg-press-surface rounded-lg p-3">
                  <div class="flex items-center gap-3">
                    <div
                      class="w-10 h-10 rounded-full bg-press-warning/20 flex items-center justify-center"
                    >
                      <User class="w-5 h-5 text-press-warning" />
                    </div>
                    <div>
                      <span class="text-press-text font-medium">Marcus</span>
                      <p class="text-press-muted text-press-eyebrow">Mentor</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Toolbar buttons -->
              <div class="mt-4 pt-4 border-t border-press-border space-y-2">
                <!-- Collapse All button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-press-surface flex items-center justify-center shrink-0"
                  >
                    <ListChevronsDownUp class="w-4 h-4 text-press-muted" />
                  </div>
                  <span class="text-press-muted text-press-ui"
                    ><strong class="text-press-text">Collapse All</strong> — close all expanded cards</span
                  >
                </div>

                <!-- Sort A-Z button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-press-surface flex items-center justify-center shrink-0"
                  >
                    <ArrowDownAZ class="w-4 h-4 text-press-muted" />
                  </div>
                  <span class="text-press-muted text-press-ui"
                    ><strong class="text-press-text">Sort A-Z</strong> — alphabetize the list</span
                  >
                </div>

                <!-- Hide Panel button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-press-surface flex items-center justify-center shrink-0"
                  >
                    <ChevronsRight class="w-4 h-4 text-press-muted" />
                  </div>
                  <span class="text-press-muted text-press-ui"
                    ><strong class="text-press-text">Hide Panel</strong> — collapse to focus on writing</span
                  >
                </div>
              </div>
            </div>

            <div class="bg-press-sunken rounded-lg p-4 mb-6">
              <ul class="text-press-muted text-press-ui space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Characters tab</strong> shows who appears in the
                    current scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Locations tab</strong> shows where the scene takes
                    place</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >Use the <strong class="text-press-text">+</strong> button to search and link references
                    to a scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    >Kindling can <strong class="text-press-text">auto-detect</strong> character and location
                    mentions in your prose — look for ⚡ suggestions</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-press-accent-text mt-0.5">•</span>
                  <span
                    ><strong class="text-press-text">Hide Panel</strong> collapses the sidebar to focus
                    on writing</span
                  >
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-press-border">
              <button
                onclick={() => ui.previousStep()}
                class="text-press-muted hover:text-press-text transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => {
                  localStorage.setItem("kindling:onboardingCompleted", "true");
                  ui.nextStep();
                }}
                class="py-2 px-4 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors flex items-center gap-1"
              >
                Start Importing
                <ChevronRight class="w-4 h-4" />
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "import"}
          <!-- STEP 5: Import (Final Step) - Guided wizard -->
          <div>
            {#if guidedLoading}
              <div class="flex flex-col items-center py-12">
                <Loader2 class="w-12 h-12 text-press-accent-text animate-spin mb-4" />
                <p class="text-press-muted">Reading your outline...</p>
              </div>
            {:else if guidedPreview}
              <!-- Preview step -->
              <div class="text-center mb-6">
                <div
                  class="w-16 h-16 rounded-full bg-press-success-wash flex items-center justify-center mx-auto mb-4"
                >
                  <Check class="w-8 h-8 text-press-success" />
                </div>
                <h2 class="text-press-h2 font-heading font-semibold text-press-text mb-2">
                  Preview: {guidedPreview.project_name}
                </h2>
                <p class="text-press-muted mb-6">
                  This outline contains the following. Ready to import?
                </p>
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-6 text-left">
                  <div class="bg-press-sunken rounded-lg p-3">
                    <span class="text-press-h2 font-semibold text-press-accent-text"
                      >{guidedPreview.chapter_count}</span
                    >
                    <span class="text-press-muted text-press-ui block">Chapters</span>
                  </div>
                  <div class="bg-press-sunken rounded-lg p-3">
                    <span class="text-press-h2 font-semibold text-press-accent-text"
                      >{guidedPreview.scene_count}</span
                    >
                    <span class="text-press-muted text-press-ui block">Scenes</span>
                  </div>
                  <div class="bg-press-sunken rounded-lg p-3">
                    <span class="text-press-h2 font-semibold text-press-accent-text"
                      >{guidedPreview.beat_count}</span
                    >
                    <span class="text-press-muted text-press-ui block">Beats</span>
                  </div>
                  {#if guidedPreview.character_count > 0}
                    <div class="bg-press-sunken rounded-lg p-3">
                      <span class="text-press-h2 font-semibold text-press-accent-text"
                        >{guidedPreview.character_count}</span
                      >
                      <span class="text-press-muted text-press-ui block">Characters</span>
                    </div>
                  {/if}
                  {#if guidedPreview.location_count > 0}
                    <div class="bg-press-sunken rounded-lg p-3">
                      <span class="text-press-h2 font-semibold text-press-accent-text"
                        >{guidedPreview.location_count}</span
                      >
                      <span class="text-press-muted text-press-ui block">Locations</span>
                    </div>
                  {/if}
                </div>
                <div class="flex justify-center gap-3">
                  <button
                    onclick={backFromGuidedPreview}
                    class="px-4 py-2 text-press-muted hover:text-press-text transition-colors flex items-center gap-1"
                  >
                    <ChevronLeft class="w-4 h-4" />
                    Choose different file
                  </button>
                  <button
                    data-testid="guided-import-confirm"
                    onclick={confirmGuidedImport}
                    class="px-6 py-2 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors"
                  >
                    Import Project
                  </button>
                </div>
              </div>
            {:else if guidedError}
              <div class="mb-6">
                <p class="text-press-error text-press-ui mb-4">{guidedError}</p>
                <button
                  onclick={backFromGuidedPreview}
                  class="px-4 py-2 text-press-muted hover:text-press-text transition-colors"
                >
                  ← Try a different file
                </button>
              </div>
            {:else}
              <!-- Choose format -->
              <div class="text-center mb-6">
                <div
                  class="w-16 h-16 rounded-full bg-press-success-wash flex items-center justify-center mx-auto mb-4"
                >
                  <Check class="w-8 h-8 text-press-success" />
                </div>
                <h2 class="text-press-h2 font-heading font-semibold text-press-text mb-2">
                  Ready to Import
                </h2>
                <p class="text-press-muted">
                  Choose your outline format. We'll preview it before importing.
                </p>
              </div>

              <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 mb-6">
                <button
                  onclick={trySampleProject}
                  class="flex flex-col items-center p-4 bg-press-accent-wash border-2 border-press-accent rounded-lg hover:bg-press-sunken transition-colors cursor-pointer"
                >
                  <BookOpen class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">Try Sample</span>
                  <span class="text-press-muted text-press-eyebrow">Explore first</span>
                </button>

                <button
                  onclick={() => startGuidedImport("plottr")}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <Kanban class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">Plottr</span>
                  <span class="text-press-muted text-press-eyebrow">.pltr</span>
                </button>

                <button
                  onclick={() => startGuidedImport("novelwriter")}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <Scroll class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">novelWriter</span>
                  <span class="text-press-muted text-press-eyebrow">Project folder</span>
                </button>

                <button
                  onclick={() => startGuidedImport("scrivener")}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <Scroll class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">Scrivener</span>
                  <span class="text-press-muted text-press-eyebrow">.scriv</span>
                </button>

                <button
                  onclick={() => startGuidedImport("markdown")}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <FileText class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">Markdown</span>
                  <span class="text-press-muted text-press-eyebrow">.md file</span>
                </button>

                <button
                  onclick={handleLongformImportClick}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <BookOpen class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">Longform</span>
                  <span class="text-press-muted text-press-eyebrow">Index file</span>
                </button>

                <button
                  onclick={() => startGuidedImport("ywriter")}
                  class="flex flex-col items-center p-4 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors cursor-pointer border border-transparent hover:border-press-accent"
                >
                  <PenTool class="w-10 h-10 text-press-accent-text mb-2" />
                  <span class="text-press-text font-medium text-press-ui">yWriter</span>
                  <span class="text-press-muted text-press-eyebrow">.yw7</span>
                </button>
              </div>

              <div class="text-center mb-4">
                <button
                  onclick={skipOnboarding}
                  class="inline-flex items-center gap-2 px-4 py-2 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors border border-transparent hover:border-press-accent"
                >
                  <FilePlus class="w-4 h-4 text-press-accent-text" />
                  <span class="text-press-text text-press-ui font-medium"
                    >Start a new project from scratch</span
                  >
                </button>
              </div>

              <div class="flex justify-between items-center pt-4 border-t border-press-border">
                <button
                  onclick={() => ui.previousStep()}
                  class="text-press-muted hover:text-press-text transition-colors flex items-center gap-1"
                >
                  <ChevronLeft class="w-4 h-4" />
                  Back to tour
                </button>
                <button
                  onclick={skipOnboarding}
                  class="text-press-muted hover:text-press-text text-press-ui transition-colors"
                >
                  I'll import later
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Skip button (always visible except on import step) -->
      {#if ui.onboardingStep !== "import"}
        <div class="text-center mt-4">
          <button
            data-testid="skip-onboarding"
            onclick={skipOnboarding}
            class="text-press-muted hover:text-press-text text-press-ui transition-colors"
          >
            Skip onboarding
          </button>
        </div>
      {/if}
    </div>

    <!-- Import Progress Modal -->
    {#if ui.isImporting}
      <div class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-popover">
        <div class="bg-press-surface rounded-lg p-6 max-w-md w-full mx-4">
          <h3 class="text-press-body-lg font-heading font-medium text-press-text mb-4">
            Importing...
          </h3>
          <div class="w-full bg-press-sunken rounded-full h-2 mb-2">
            <div
              class="bg-press-accent h-2 rounded-full transition-all"
              style="width: {ui.importProgress}%"
            ></div>
          </div>
          <p class="text-press-muted text-press-ui">{ui.importStatus}</p>
        </div>
      </div>
    {/if}
  </div>
{/if}
