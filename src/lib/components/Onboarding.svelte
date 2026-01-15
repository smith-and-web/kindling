<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowDownAZ,
    BookOpen,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronsRight,
    FileText,
    ListChevronsDownUp,
    Kanban,
    MapPin,
    User,
    Users,
    Zap,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui, type OnboardingStep } from "../stores/ui.svelte";
  import type { Project } from "../types";

  const STEP_ORDER: OnboardingStep[] = [
    "welcome",
    "tour-sidebar",
    "tour-editor",
    "tour-references",
    "import",
  ];

  // Import handlers (same logic as StartScreen)
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
        ui.completeOnboarding();
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
        ui.completeOnboarding();
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
        ui.completeOnboarding();
        ui.setView("editor");
      } catch (e) {
        console.error("Failed to import Markdown file:", e);
        alert(`Import failed: ${e}`);
      } finally {
        ui.finishImport();
      }
    }
  }

  function skipOnboarding() {
    ui.completeOnboarding();
  }
</script>

{#if ui.showOnboarding}
  <div
    data-testid="onboarding"
    class="fixed inset-0 bg-bg-primary/95 flex items-center justify-center z-50 p-4"
  >
    <div class="max-w-2xl w-full">
      <!-- Progress indicator -->
      <div class="flex justify-center gap-2 mb-8">
        {#each STEP_ORDER as step, i (step)}
          <button
            onclick={() => ui.goToStep(step)}
            class="w-2 h-2 rounded-full transition-all {i === ui.currentStepIndex
              ? 'bg-accent w-6'
              : i < ui.currentStepIndex
                ? 'bg-accent/50'
                : 'bg-bg-card'}"
            aria-label="Go to step {i + 1}"
          ></button>
        {/each}
      </div>

      <!-- Step content -->
      <div class="bg-bg-panel rounded-xl p-8 shadow-xl">
        {#if ui.onboardingStep === "welcome"}
          <!-- STEP 1: Welcome -->
          <div class="text-center">
            <!-- Logo -->
            <div class="flex justify-center mb-6">
              <svg width="100" height="100" viewBox="0 0 1024 1024" class="drop-shadow-lg">
                <defs>
                  <linearGradient
                    id="bookGradientOnboarding"
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
                  fill="url(#bookGradientOnboarding)"
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

            <h1 class="text-4xl font-heading font-semibold text-accent mb-3">
              Welcome to Kindling
            </h1>
            <p class="text-text-secondary text-lg mb-8 max-w-md mx-auto">
              Transform your outline into a finished draft. Import your story structure and start
              writing scene by scene.
            </p>

            <div class="flex flex-col gap-3">
              <button
                onclick={() => ui.nextStep()}
                class="w-full py-3 px-6 bg-accent hover:bg-flame-orange text-white font-medium rounded-lg transition-colors"
              >
                Take the Tour
              </button>
              <button
                onclick={skipOnboarding}
                class="text-text-secondary hover:text-text-primary text-sm transition-colors"
              >
                Skip and start importing
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "tour-sidebar"}
          <!-- STEP 2: Chapters & Scenes Tour -->
          <div>
            <div class="flex items-center gap-3 mb-4">
              <div class="w-10 h-10 rounded-lg bg-accent/20 flex items-center justify-center">
                <ArrowDownAZ class="w-5 h-5 text-accent" />
              </div>
              <div>
                <h2 class="text-2xl font-heading font-semibold text-text-primary">
                  Chapters & Scenes
                </h2>
                <p class="text-text-secondary text-sm">The left sidebar</p>
              </div>
            </div>

            <!-- Visual mockup of sidebar -->
            <div class="bg-bg-card rounded-lg p-4 mb-6">
              <div class="space-y-3">
                <!-- Chapter example -->
                <div class="relative">
                  <div class="flex items-center gap-2 p-2 rounded bg-bg-panel">
                    <ChevronDown class="w-4 h-4 text-text-secondary" />
                    <span class="text-text-primary font-medium">Chapter 1: The Beginning</span>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-accent"></div>
                    <span class="text-accent text-xs font-medium whitespace-nowrap">Chapter</span>
                  </div>
                </div>

                <!-- Scene examples -->
                <div class="ml-6 space-y-2">
                  <div class="relative">
                    <div
                      class="flex items-center gap-2 p-2 rounded bg-accent/10 border border-accent/30"
                    >
                      <FileText class="w-4 h-4 text-accent" />
                      <span class="text-text-primary">Opening Scene</span>
                    </div>
                    <!-- Label -->
                    <div
                      class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                    >
                      <div class="w-8 h-px bg-spark-gold"></div>
                      <span class="text-spark-gold text-xs font-medium whitespace-nowrap"
                        >Active Scene</span
                      >
                    </div>
                  </div>

                  <div class="relative">
                    <div class="flex items-center gap-2 p-2 rounded hover:bg-bg-panel">
                      <FileText class="w-4 h-4 text-text-secondary" />
                      <span class="text-text-secondary">The Discovery</span>
                    </div>
                    <!-- Label -->
                    <div
                      class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                    >
                      <div class="w-8 h-px bg-text-secondary"></div>
                      <span class="text-text-secondary text-xs font-medium whitespace-nowrap"
                        >Other Scene</span
                      >
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-bg-card/50 rounded-lg p-4 mb-6">
              <ul class="text-text-secondary text-sm space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Click a chapter</strong> to expand or collapse
                    its scenes</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Click a scene</strong> to load it in the editor</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    >The <strong class="text-text-primary">highlighted scene</strong> is your current
                    working scene</span
                  >
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-bg-card">
              <button
                onclick={() => ui.previousStep()}
                class="text-text-secondary hover:text-text-primary transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="py-2 px-4 bg-accent hover:bg-flame-orange text-white font-medium rounded-lg transition-colors flex items-center gap-1"
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
              <div class="w-10 h-10 rounded-lg bg-spark-gold/20 flex items-center justify-center">
                <Zap class="w-5 h-5 text-spark-gold" />
              </div>
              <div>
                <h2 class="text-2xl font-heading font-semibold text-text-primary">
                  Synopsis & Beats
                </h2>
                <p class="text-text-secondary text-sm">The main editor area</p>
              </div>
            </div>

            <!-- Visual mockup of editor -->
            <div class="bg-bg-card rounded-lg p-4 mb-6 space-y-4">
              <!-- Synopsis section -->
              <div class="relative">
                <div class="bg-bg-panel rounded-lg p-3">
                  <h4 class="text-xs uppercase tracking-wide text-text-secondary mb-2">Synopsis</h4>
                  <p class="text-text-primary text-sm">
                    The hero receives the call to adventure and must decide whether to leave their
                    ordinary world behind...
                  </p>
                </div>
                <!-- Label -->
                <div
                  class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                >
                  <div class="w-8 h-px bg-accent"></div>
                  <span class="text-accent text-xs font-medium whitespace-nowrap"
                    >Scene Synopsis</span
                  >
                </div>
              </div>

              <!-- Beats section -->
              <div class="space-y-2">
                <div class="relative">
                  <div class="bg-beat-header rounded-lg p-3 border-l-2 border-accent">
                    <div class="flex items-center justify-between mb-1">
                      <span class="text-text-primary font-medium text-sm"
                        >The Messenger Arrives</span
                      >
                      <span class="text-xs text-text-secondary">Beat 1</span>
                    </div>
                    <p class="text-text-secondary text-xs">
                      A stranger appears at the door with urgent news...
                    </p>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-spark-gold"></div>
                    <span class="text-spark-gold text-xs font-medium whitespace-nowrap"
                      >Story Beat</span
                    >
                  </div>
                </div>

                <div class="relative">
                  <div class="bg-beat-header rounded-lg p-3 border-l-2 border-transparent">
                    <div class="flex items-center justify-between mb-1">
                      <span class="text-text-primary font-medium text-sm">The Decision</span>
                      <span class="text-xs text-text-secondary">Beat 2</span>
                    </div>
                    <p class="text-text-secondary text-xs">
                      Our hero weighs their options and makes a choice...
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-bg-card/50 rounded-lg p-4 mb-6">
              <ul class="text-text-secondary text-sm space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    >The <strong class="text-text-primary">synopsis</strong> gives you the scene overview
                    from your outline</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Beats</strong> are the key story moments within
                    each scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span>Use beats as your writing prompts to draft each section</span>
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-bg-card">
              <button
                onclick={() => ui.previousStep()}
                class="text-text-secondary hover:text-text-primary transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="py-2 px-4 bg-accent hover:bg-flame-orange text-white font-medium rounded-lg transition-colors flex items-center gap-1"
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
              <div class="w-10 h-10 rounded-lg bg-success/20 flex items-center justify-center">
                <Users class="w-5 h-5 text-success" />
              </div>
              <div>
                <h2 class="text-2xl font-heading font-semibold text-text-primary">
                  Reference Panel
                </h2>
                <p class="text-text-secondary text-sm">The right sidebar</p>
              </div>
            </div>

            <!-- Visual mockup of reference panel -->
            <div class="bg-bg-card rounded-lg p-4 mb-6">
              <!-- Tab buttons mockup -->
              <div class="relative flex gap-1 mb-4 bg-bg-panel rounded-lg p-1">
                <div class="flex-1 py-2 px-3 rounded bg-bg-card text-center">
                  <span class="text-text-primary text-sm font-medium">Characters</span>
                </div>
                <div class="flex-1 py-2 px-3 rounded text-center">
                  <span class="text-text-secondary text-sm">Locations</span>
                </div>
                <!-- Label -->
                <div
                  class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                >
                  <div class="w-8 h-px bg-accent"></div>
                  <span class="text-accent text-xs font-medium whitespace-nowrap">Tab Switcher</span
                  >
                </div>
              </div>

              <!-- Character cards mockup -->
              <div class="space-y-2">
                <div class="relative">
                  <div class="bg-bg-panel rounded-lg p-3">
                    <div class="flex items-center gap-3">
                      <div
                        class="w-10 h-10 rounded-full bg-accent/20 flex items-center justify-center"
                      >
                        <User class="w-5 h-5 text-accent" />
                      </div>
                      <div>
                        <span class="text-text-primary font-medium">Elena</span>
                        <p class="text-text-secondary text-xs">Protagonist</p>
                      </div>
                    </div>
                  </div>
                  <!-- Label -->
                  <div
                    class="absolute -right-2 top-1/2 -translate-y-1/2 translate-x-full flex items-center gap-2"
                  >
                    <div class="w-8 h-px bg-spark-gold"></div>
                    <span class="text-spark-gold text-xs font-medium whitespace-nowrap"
                      >Character Card</span
                    >
                  </div>
                </div>

                <div class="bg-bg-panel rounded-lg p-3">
                  <div class="flex items-center gap-3">
                    <div
                      class="w-10 h-10 rounded-full bg-spark-gold/20 flex items-center justify-center"
                    >
                      <MapPin class="w-5 h-5 text-spark-gold" />
                    </div>
                    <div>
                      <span class="text-text-primary font-medium">Marcus</span>
                      <p class="text-text-secondary text-xs">Mentor</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Toolbar buttons -->
              <div class="mt-4 pt-4 border-t border-bg-panel space-y-2">
                <!-- Collapse All button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-bg-panel flex items-center justify-center flex-shrink-0"
                  >
                    <ListChevronsDownUp class="w-4 h-4 text-text-secondary" />
                  </div>
                  <span class="text-text-secondary text-sm"
                    ><strong class="text-text-primary">Collapse All</strong> — close all expanded cards</span
                  >
                </div>

                <!-- Sort A-Z button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-bg-panel flex items-center justify-center flex-shrink-0"
                  >
                    <ArrowDownAZ class="w-4 h-4 text-text-secondary" />
                  </div>
                  <span class="text-text-secondary text-sm"
                    ><strong class="text-text-primary">Sort A-Z</strong> — alphabetize the list</span
                  >
                </div>

                <!-- Hide Panel button -->
                <div class="flex items-center gap-3">
                  <div
                    class="w-7 h-7 rounded bg-bg-panel flex items-center justify-center flex-shrink-0"
                  >
                    <ChevronsRight class="w-4 h-4 text-text-secondary" />
                  </div>
                  <span class="text-text-secondary text-sm"
                    ><strong class="text-text-primary">Hide Panel</strong> — collapse to focus on writing</span
                  >
                </div>
              </div>
            </div>

            <div class="bg-bg-card/50 rounded-lg p-4 mb-6">
              <ul class="text-text-secondary text-sm space-y-2">
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Characters tab</strong> shows who appears in the
                    current scene</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Locations tab</strong> shows where the scene takes
                    place</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Collapse All</strong> closes all expanded character/location
                    cards</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Sort A-Z</strong> alphabetizes the list</span
                  >
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-accent mt-0.5">•</span>
                  <span
                    ><strong class="text-text-primary">Hide Panel</strong> collapses the sidebar to focus
                    on writing</span
                  >
                </li>
              </ul>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-bg-card">
              <button
                onclick={() => ui.previousStep()}
                class="text-text-secondary hover:text-text-primary transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back
              </button>
              <button
                onclick={() => ui.nextStep()}
                class="py-2 px-4 bg-accent hover:bg-flame-orange text-white font-medium rounded-lg transition-colors flex items-center gap-1"
              >
                Start Importing
                <ChevronRight class="w-4 h-4" />
              </button>
            </div>
          </div>
        {:else if ui.onboardingStep === "import"}
          <!-- STEP 5: Import (Final Step) -->
          <div>
            <div class="text-center mb-6">
              <div
                class="w-16 h-16 rounded-full bg-success/20 flex items-center justify-center mx-auto mb-4"
              >
                <Check class="w-8 h-8 text-success" />
              </div>
              <h2 class="text-2xl font-heading font-semibold text-text-primary mb-2">
                Ready to Import
              </h2>
              <p class="text-text-secondary">Choose your outline format to get started.</p>
            </div>

            <div class="grid grid-cols-3 gap-4 mb-6">
              <button
                onclick={importPlottr}
                class="flex flex-col items-center p-5 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer border border-transparent hover:border-accent/30"
              >
                <Kanban class="w-12 h-12 text-accent mb-3" />
                <span class="text-text-primary font-medium">Plottr</span>
                <span class="text-text-secondary text-sm">.pltr</span>
              </button>

              <button
                onclick={importScrivener}
                class="flex flex-col items-center p-5 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer border border-transparent hover:border-accent/30"
              >
                <BookOpen class="w-12 h-12 text-accent mb-3" />
                <span class="text-text-primary font-medium">Scrivener</span>
                <span class="text-text-secondary text-sm">.scriv</span>
              </button>

              <button
                onclick={importMarkdown}
                class="flex flex-col items-center p-5 bg-bg-card rounded-lg hover:bg-beat-header transition-colors cursor-pointer border border-transparent hover:border-accent/30"
              >
                <FileText class="w-12 h-12 text-accent mb-3" />
                <span class="text-text-primary font-medium">Markdown</span>
                <span class="text-text-secondary text-sm">.md</span>
              </button>
            </div>

            <div class="flex justify-between items-center pt-4 border-t border-bg-card">
              <button
                onclick={() => ui.previousStep()}
                class="text-text-secondary hover:text-text-primary transition-colors flex items-center gap-1"
              >
                <ChevronLeft class="w-4 h-4" />
                Back to tour
              </button>
              <button
                onclick={skipOnboarding}
                class="text-text-secondary hover:text-text-primary text-sm transition-colors"
              >
                I'll import later
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Skip button (always visible except on import step) -->
      {#if ui.onboardingStep !== "import"}
        <div class="text-center mt-4">
          <button
            data-testid="skip-onboarding"
            onclick={skipOnboarding}
            class="text-text-secondary hover:text-text-primary text-sm transition-colors"
          >
            Skip onboarding
          </button>
        </div>
      {/if}
    </div>

    <!-- Import Progress Modal -->
    {#if ui.isImporting}
      <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-[60]">
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
{/if}
