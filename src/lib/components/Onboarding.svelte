<script lang="ts">
  import { IMPORT_FORMATS, type ImportType } from "../importFormats";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "../utils/nativeDialog";
  import {
    ArrowDownAZ,
    ChevronDown,
    CircleAlert,
    FilePlus,
    ListChevronsDownUp,
    PanelRightClose,
    Plus,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui, type OnboardingStep } from "../stores/ui.svelte";
  import type { ImportPreview, Project } from "../types";
  import { pickScrivenerProjectPath } from "$lib/utils/import";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";

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

  const STEP_HEADINGS: Record<OnboardingStep, { title: string; subtitle?: string }> = {
    welcome: { title: "Welcome to kindling" },
    "tour-sidebar": { title: "Chapters and scenes", subtitle: "The left sidebar" },
    "tour-editor": { title: "Synopsis and beats", subtitle: "The main editor" },
    "tour-references": { title: "References", subtitle: "The right panel" },
    import: { title: "Import your outline" },
  };

  const IMPORTS: { label: string; hint: string; run: () => void }[] = [
    { label: "Plottr", hint: ".pltr", run: () => void startGuidedImport("plottr") },
    { label: "Scrivener", hint: ".scriv", run: () => void startGuidedImport("scrivener") },
    { label: "yWriter", hint: ".yw7", run: () => void startGuidedImport("ywriter") },
    {
      label: "novelWriter",
      hint: "Project folder",
      run: () => void startGuidedImport("novelwriter"),
    },
    { label: "Markdown", hint: ".md", run: () => void startGuidedImport("markdown") },
    { label: "Longform", hint: "Index or vault", run: () => handleLongformImportClick() },
  ];

  const stepNumber = $derived(Math.max(0, STEP_ORDER.indexOf(ui.onboardingStep)) + 1);
  const progressLabel = $derived(
    ui.onboardingStep === "welcome"
      ? "Getting started"
      : ui.onboardingStep === "import"
        ? "Import"
        : "Tour"
  );
  const heading = $derived(
    ui.onboardingStep === "import" && guidedPreview
      ? { title: `Ready to import “${guidedPreview.project_name}”` }
      : STEP_HEADINGS[ui.onboardingStep]
  );

  async function trySampleProject() {
    ui.startImport("Opening sample project", "Creating the sample project…");
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

  function finishTour() {
    localStorage.setItem("kindling:onboardingCompleted", "true");
    ui.nextStep();
  }

  function handleEscape() {
    if (!guidedLoading) skipOnboarding();
  }

  /** Each step lands keyboard focus on its main action. */
  function focusOnMount(node: HTMLElement) {
    node.focus();
  }
</script>

{#if ui.showOnboarding}
  <div
    data-testid="onboarding"
    class="dialog-scrim"
    role="dialog"
    aria-modal="true"
    aria-labelledby="onboarding-title"
    tabindex="-1"
    use:modalFocus={{ onEscape: handleEscape }}
  >
    <div class="app-dialog-surface ka-dialog-default dialog-shell onboarding">
      <DialogHeader
        title={heading.title}
        subtitle={heading.subtitle}
        titleId="onboarding-title"
        onClose={skipOnboarding}
        closeLabel="Skip onboarding"
        closeTestId="skip-onboarding"
      />

      <div class="ka-dialog-body onboarding-body">
        {#if ui.onboardingStep === "welcome"}
          <img class="onboarding-mark mark-light" src="/brand/kindling-mark.svg" alt="" />
          <img class="onboarding-mark mark-dark" src="/brand/kindling-mark-reversed.svg" alt="" />
          <p class="onboarding-lede">
            Transform your outline into a finished draft. Import your story structure and start
            writing scene by scene.
          </p>
        {:else if ui.onboardingStep === "tour-sidebar"}
          <div class="tour-figure" aria-hidden="true">
            <div class="tour-sheet">
              <div class="tour-row">
                <ChevronDown class="w-4 h-4 ka-icon" />
                <span class="tour-strong">The Letter</span>
                <span class="ka-badge">Chapter</span>
              </div>
              <div class="tour-row tour-child is-current">
                <span>On the Cliff</span>
                <small>Current scene</small>
              </div>
              <div class="tour-row tour-child">
                <span class="tour-muted">The Seventh Step</span>
                <span class="ka-badge">Scene</span>
              </div>
            </div>
          </div>
          <ul class="tour-points">
            <li><strong>Click a chapter</strong> to expand or collapse its scenes.</li>
            <li><strong>Click a scene</strong> to load it in the editor.</li>
            <li>The <strong>highlighted scene</strong> is your current working scene.</li>
            <li>
              Imported and template projects may include <strong>Parts</strong> (acts) as collapsible
              groups above chapters.
            </li>
            <li>
              Screenplay projects use <strong>Acts → Sequences → Scenes</strong> instead of Chapters
              → Scenes, with <strong>page count</strong> estimates alongside each act.
            </li>
          </ul>
        {:else if ui.onboardingStep === "tour-editor"}
          <div class="tour-figure" aria-hidden="true">
            <div class="tour-sheet tour-stack">
              <div class="tour-block">
                <div class="tour-row-head">
                  <span class="tour-strong">Synopsis</span>
                  <span class="ka-badge">Scene synopsis</span>
                </div>
                <p class="tour-prose">
                  The hero receives the call to adventure and must decide whether to leave their
                  ordinary world behind…
                </p>
              </div>
              <div class="tour-beat is-open">
                <span class="tour-beat-num">1</span>
                <span class="tour-strong">The messenger arrives</span>
                <span class="ka-badge ka-badge--accent">Story beat</span>
              </div>
              <div class="tour-beat">
                <span class="tour-beat-num">2</span>
                <span class="tour-muted">The decision</span>
              </div>
            </div>
          </div>
          <ul class="tour-points">
            <li>The <strong>synopsis</strong> gives you the scene overview from your outline.</li>
            <li><strong>Beats</strong> are the key story moments within each scene.</li>
            <li>Use beats as your writing prompts to draft each section.</li>
            <li>
              Switch between <strong>Beats</strong> (outline-guided) and <strong>Page</strong>
              (free-form writing) at the top of the scene.
            </li>
            <li>
              In screenplays, scene titles are <strong>sluglines</strong> (for example,
              <code>INT. CASTLE - NIGHT</code>). kindling suggests locations from your references.
            </li>
          </ul>
        {:else if ui.onboardingStep === "tour-references"}
          <div class="tour-figure" aria-hidden="true">
            <div class="tour-sheet tour-stack">
              <div class="tour-tabs">
                <span class="is-selected">Characters</span>
                <span>Locations</span>
                <span>Items</span>
              </div>
              <div class="tour-row">
                <span class="tour-strong">Elena</span>
                <span class="tour-muted">Protagonist</span>
                <span class="ka-badge">Linked</span>
              </div>
              <div class="tour-row">
                <span class="tour-strong">Marcus</span>
                <span class="tour-muted">Mentor</span>
              </div>
              <div class="tour-tools">
                <span><Plus class="w-4 h-4 ka-icon" /> Add</span>
                <span><ListChevronsDownUp class="w-4 h-4 ka-icon" /> Collapse all</span>
                <span><ArrowDownAZ class="w-4 h-4 ka-icon" /> Sort A–Z</span>
                <span><PanelRightClose class="w-4 h-4 ka-icon" /> Hide panel</span>
              </div>
            </div>
          </div>
          <ul class="tour-points">
            <li>
              <strong>Characters</strong> shows who appears in the current scene;
              <strong>Locations</strong> shows where it takes place.
            </li>
            <li>Use <strong>Add</strong> to search and link references to a scene.</li>
            <li>
              kindling can <strong>detect</strong> character and location mentions in your prose.
              Look for the <strong>Suggested</strong> group.
            </li>
            <li><strong>Hide panel</strong> collapses the sidebar so you can focus on writing.</li>
          </ul>
        {:else if ui.onboardingStep === "import"}
          {#if guidedLoading}
            <div class="ka-progress od-field" role="status">
              <span>Reading your outline…</span>
              <progress aria-label="Reading your outline"></progress>
            </div>
          {:else if guidedPreview}
            <p class="onboarding-lede">This outline contains the following.</p>
            <dl class="ka-facts">
              <div>
                <dt>Chapters</dt>
                <dd>{guidedPreview.chapter_count}</dd>
              </div>
              <div>
                <dt>Scenes</dt>
                <dd>{guidedPreview.scene_count}</dd>
              </div>
              <div>
                <dt>Beats</dt>
                <dd>{guidedPreview.beat_count}</dd>
              </div>
              {#if guidedPreview.character_count > 0}
                <div>
                  <dt>Characters</dt>
                  <dd>{guidedPreview.character_count}</dd>
                </div>
              {/if}
              {#if guidedPreview.location_count > 0}
                <div>
                  <dt>Locations</dt>
                  <dd>{guidedPreview.location_count}</dd>
                </div>
              {/if}
            </dl>
          {:else if guidedError}
            <div class="ka-notice ka-notice--error od-row-top" role="alert">
              <CircleAlert class="w-5 h-5" aria-hidden="true" />
              <div class="od-field od-fill">
                <strong>Couldn’t read that outline</strong>
                <p>{guidedError}</p>
              </div>
            </div>
          {:else}
            <p class="onboarding-lede">
              Choose your outline format. kindling previews it before importing.
            </p>
            <ul class="import-list">
              {#each IMPORTS as format (format.label)}
                <li>
                  <button
                    type="button"
                    class="ka-button ka-button--ghost import-format"
                    onclick={format.run}
                  >
                    {format.label}
                    <small>{format.hint}</small>
                  </button>
                </li>
              {/each}
            </ul>
            <div class="import-alternatives">
              <button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={trySampleProject}
              >
                Try the sample project
              </button>
              <button type="button" class="ka-button ka-button--ghost" onclick={skipOnboarding}>
                <FilePlus class="w-5 h-5" aria-hidden="true" />
                Start a new project from scratch
              </button>
            </div>
          {/if}
        {/if}

        <div class="ka-progress od-field onboarding-progress">
          <div class="ka-between">
            <span>{progressLabel}</span>
            <span class="od-nowrap">Step {stepNumber} of {STEP_ORDER.length}</span>
          </div>
          <progress
            aria-label={`${progressLabel}, step ${stepNumber} of ${STEP_ORDER.length}`}
            value={stepNumber}
            max={STEP_ORDER.length}
          ></progress>
        </div>
      </div>

      <footer class="ka-dialog-footer">
        {#key `${ui.onboardingStep}:${guidedPreview ? "preview" : guidedError ? "error" : ""}`}
          {#if ui.onboardingStep === "welcome"}
            <button
              type="button"
              class="ka-button ka-button--ghost ka-dialog-footer-start"
              onclick={skipOnboarding}
            >
              Skip and start importing
            </button>
            <button
              type="button"
              class="ka-button ka-button--secondary"
              onclick={() => ui.nextStep()}
            >
              Take the tour
            </button>
            <button type="button" class="ka-button" onclick={trySampleProject} use:focusOnMount>
              Try sample project
            </button>
          {:else if ui.onboardingStep === "import"}
            {#if guidedPreview}
              <button
                type="button"
                class="ka-button ka-button--ghost ka-dialog-footer-start"
                onclick={backFromGuidedPreview}
              >
                Choose a different file
              </button>
              <button
                type="button"
                class="ka-button"
                data-testid="guided-import-confirm"
                onclick={confirmGuidedImport}
                use:focusOnMount
              >
                Import project
              </button>
            {:else if guidedError}
              <button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={backFromGuidedPreview}
                use:focusOnMount
              >
                Try a different file
              </button>
            {:else}
              <button
                type="button"
                class="ka-button ka-button--ghost ka-dialog-footer-start"
                onclick={() => ui.previousStep()}
                disabled={guidedLoading}
              >
                Back to tour
              </button>
              <button
                type="button"
                class="ka-button ka-button--secondary"
                onclick={skipOnboarding}
                disabled={guidedLoading}
              >
                I’ll import later
              </button>
            {/if}
          {:else}
            <button
              type="button"
              class="ka-button ka-button--ghost ka-dialog-footer-start"
              onclick={() => ui.goToStep("import")}
            >
              Skip tour
            </button>
            <button
              type="button"
              class="ka-button ka-button--secondary"
              onclick={() => ui.previousStep()}
            >
              Back
            </button>
            {#if ui.onboardingStep === "tour-references"}
              <button type="button" class="ka-button" onclick={finishTour} use:focusOnMount>
                Start importing
              </button>
            {:else}
              <button
                type="button"
                class="ka-button"
                onclick={() => ui.nextStep()}
                use:focusOnMount
              >
                Next
              </button>
            {/if}
          {/if}
        {/key}
      </footer>
    </div>
  </div>
{/if}

<style>
  .onboarding-body {
    display: grid;
    gap: var(--space-s);
  }
  .onboarding-mark {
    width: 112px;
    height: auto;
  }
  .mark-dark,
  :global([data-theme="dark"]) .mark-light {
    display: none;
  }
  :global([data-theme="dark"]) .mark-dark {
    display: block;
  }
  .onboarding-lede {
    margin: 0;
    max-width: 48ch;
    font: var(--text-base) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
  .onboarding-progress {
    margin-top: var(--space-2xs);
    color: var(--color-text-muted);
  }

  /* Tour illustrations: a quiet sketch of the real surface, on the desk. */
  .tour-figure {
    padding: var(--space-s);
    border-radius: var(--radius-m);
    background: var(--color-surface-sunken);
  }
  .tour-sheet {
    display: grid;
    padding: var(--space-2xs);
    border: var(--border-hair);
    border-radius: var(--radius-m);
    background: var(--color-surface);
  }
  .tour-stack {
    gap: var(--space-2xs);
  }
  .tour-row,
  .tour-beat,
  .tour-row-head {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    min-height: 40px;
    padding-inline: var(--space-xs);
    font: var(--text-ui) / 1.4 var(--font-ui);
    color: var(--color-text);
  }
  .tour-row .ka-badge,
  .tour-beat .ka-badge,
  .tour-row-head .ka-badge {
    margin-left: auto;
  }
  .tour-child {
    margin-left: var(--space-m);
    border-radius: var(--radius-xs);
  }
  /* The real sidebar's selection: accent-text fill, on-accent text. */
  .tour-row.is-current {
    background: var(--color-accent-text);
    color: var(--color-on-accent);
  }
  .tour-row.is-current small {
    margin-left: auto;
    font: var(--text-small) / 1.4 var(--font-ui);
  }
  .tour-strong {
    font-weight: 500;
  }
  .tour-muted {
    color: var(--color-text-muted);
  }
  .tour-block {
    padding-bottom: var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .tour-prose {
    margin: 0;
    padding-inline: var(--space-xs);
    font: italic var(--text-ui) / 1.6 var(--font-body);
    color: var(--color-text-muted);
  }
  .tour-beat-num {
    display: inline-grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    font: 500 var(--text-small) / 1 var(--font-ui);
    color: var(--color-text-muted);
  }
  .tour-beat.is-open .tour-beat-num {
    border-color: var(--color-accent-text);
    background: var(--color-accent-text);
    color: var(--color-on-accent);
  }
  .tour-tabs {
    display: flex;
    gap: var(--space-s);
    padding-inline: var(--space-xs);
    border-bottom: var(--border-hair);
    font: var(--text-ui) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
  }
  .tour-tabs span {
    padding-block: var(--space-2xs);
  }
  .tour-tabs .is-selected {
    color: var(--color-accent-text);
    box-shadow: inset 0 -2px 0 var(--color-accent-text);
  }
  .tour-tools {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs) var(--space-s);
    padding: var(--space-2xs) var(--space-xs) 0;
    border-top: var(--border-hair);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .tour-tools span {
    display: inline-flex;
    align-items: center;
    gap: var(--space-3xs);
  }

  .tour-points {
    display: grid;
    gap: var(--space-2xs);
    margin: 0;
    padding-left: 1.25em;
    list-style: disc;
    font: var(--text-ui) / 1.6 var(--font-ui);
    color: var(--color-text-muted);
  }
  .tour-points li::marker {
    color: var(--color-text-muted);
  }
  .tour-points strong {
    font-weight: 600;
    color: var(--color-text);
  }
  .tour-points code {
    white-space: nowrap;
    padding: 0 var(--space-3xs);
    border-radius: var(--radius-xs);
    background: var(--color-surface-sunken);
    font: var(--text-small) / 1.5 var(--font-mono);
    color: var(--color-text);
  }

  /* Import formats: the start screen's hairline rows. */
  .import-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--space-m);
  }
  .import-list li {
    border-bottom: var(--border-hair);
  }
  .import-list li:nth-child(-n + 2) {
    border-top: var(--border-hair);
  }
  .import-format {
    width: 100%;
    justify-content: space-between;
    padding-inline: var(--space-xs);
    margin-block: var(--space-3xs);
    border-radius: var(--radius-xs);
  }
  .import-format small {
    font: 400 var(--text-small) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
  }
  .import-alternatives {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-xs);
  }
</style>
