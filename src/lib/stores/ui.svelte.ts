/**
 * UI Store - Manages application UI state
 *
 * This store handles all UI-related state that isn't directly tied to project data:
 * - Current view (start screen vs editor)
 * - Panel visibility and sizing
 * - Focus mode
 * - Onboarding flow
 * - Import progress indicators
 *
 * State is persisted to localStorage where appropriate (panel widths, onboarding completion).
 *
 * Usage:
 *   import { ui } from "$lib/stores/ui.svelte";
 *   ui.setView("editor");
 *   ui.toggleSidebar();
 *
 * @see project.svelte.ts for project data state
 */

export type View = "start" | "editor";
export type Panel = "sidebar" | "editor" | "references";
export type OnboardingStep =
  | "welcome"
  | "tour-sidebar"
  | "tour-editor"
  | "tour-references"
  | "import";

// Panel width constraints
const REFERENCES_PANEL_MIN_WIDTH = 200;
const REFERENCES_PANEL_DEFAULT_WIDTH = 288; // w-72
const REFERENCES_PANEL_STORAGE_KEY = "kindling:referencesPanelWidth";
const SIDEBAR_WIDTH = 256; // w-64

// Onboarding storage
const ONBOARDING_COMPLETED_KEY = "kindling:onboardingCompleted";

// Guidance preferences (Phase C)
const GUIDANCE_ENABLED_KEY = "kindling:guidanceEnabled";
const TOOLTIP_SEEN_PREFIX = "kindling:tooltipSeen:";

export type GuidanceArea = "sidebar" | "scenePanel" | "references" | "sync" | "planningStatus";

const ONBOARDING_STEPS: OnboardingStep[] = [
  "welcome",
  "tour-sidebar",
  "tour-editor",
  "tour-references",
  "import",
];

class UIStore {
  private _currentView = $state<View>("start");
  private _sidebarCollapsed = $state(false);
  private _referencesPanelCollapsed = $state(false);
  private _referencesPanelWidth = $state(REFERENCES_PANEL_DEFAULT_WIDTH);
  private _focusMode = $state(false);
  private _expandedBeatId = $state<string | null>(null);
  private _beatSaveStatus = $state<"idle" | "saving" | "saved" | "error">("idle");
  private _sceneReferenceRefreshId = $state(0);
  private _isImporting = $state(false);
  private _importProgress = $state(0);
  private _importStatus = $state("");
  private _toast = $state<{ id: number; message: string } | null>(null);
  private _toastIdCounter = 0;

  // Onboarding state
  private _showOnboarding = $state(false);
  private _onboardingStep = $state<OnboardingStep>("welcome");
  private _onboardingCompleted = $state(false);

  // Guidance state (Phase C)
  private _guidanceEnabled = $state(true);
  private _tooltipSeenVersion = $state(0); // Bump to trigger re-check of localStorage

  constructor() {
    const saved = localStorage.getItem(REFERENCES_PANEL_STORAGE_KEY);
    if (saved) {
      const width = parseInt(saved, 10);
      if (!isNaN(width) && width >= REFERENCES_PANEL_MIN_WIDTH) {
        this._referencesPanelWidth = width;
      }
    }

    const onboardingCompleted = localStorage.getItem(ONBOARDING_COMPLETED_KEY);
    this._onboardingCompleted = onboardingCompleted === "true";
    this._showOnboarding = !this._onboardingCompleted;

    const guidanceStored = localStorage.getItem(GUIDANCE_ENABLED_KEY);
    this._guidanceEnabled = guidanceStored === null ? true : guidanceStored === "true";

    window.addEventListener("resize", () => {
      const maxWidth = this.referencesPanelMaxWidth;
      if (this._referencesPanelWidth > maxWidth) {
        this._referencesPanelWidth = maxWidth;
      }
    });
  }

  get currentView() {
    return this._currentView;
  }

  get sidebarCollapsed() {
    return this._sidebarCollapsed;
  }

  set sidebarCollapsed(value: boolean) {
    this._sidebarCollapsed = value;
  }

  get referencesPanelCollapsed() {
    return this._referencesPanelCollapsed;
  }

  set referencesPanelCollapsed(value: boolean) {
    this._referencesPanelCollapsed = value;
  }

  get referencesPanelWidth() {
    return this._referencesPanelWidth;
  }

  get referencesPanelMinWidth() {
    return REFERENCES_PANEL_MIN_WIDTH;
  }

  get referencesPanelMaxWidth() {
    const sidebarWidth = this._sidebarCollapsed ? 0 : SIDEBAR_WIDTH;
    const availableWidth = window.innerWidth - sidebarWidth;
    return Math.max(REFERENCES_PANEL_MIN_WIDTH, Math.floor(availableWidth * 0.5));
  }

  setReferencesPanelWidth(width: number) {
    const maxWidth = this.referencesPanelMaxWidth;
    const clamped = Math.max(REFERENCES_PANEL_MIN_WIDTH, Math.min(maxWidth, width));
    this._referencesPanelWidth = clamped;
    localStorage.setItem(REFERENCES_PANEL_STORAGE_KEY, String(clamped));
  }

  get focusMode() {
    return this._focusMode;
  }

  get expandedBeatId() {
    return this._expandedBeatId;
  }

  get beatSaveStatus() {
    return this._beatSaveStatus;
  }

  get sceneReferenceRefreshId() {
    return this._sceneReferenceRefreshId;
  }

  get isImporting() {
    return this._isImporting;
  }

  get importProgress() {
    return this._importProgress;
  }

  get importStatus() {
    return this._importStatus;
  }

  get toast() {
    return this._toast;
  }

  setView(view: View) {
    this._currentView = view;
  }

  toggleSidebar() {
    this._sidebarCollapsed = !this._sidebarCollapsed;
  }

  toggleReferencesPanel() {
    this._referencesPanelCollapsed = !this._referencesPanelCollapsed;
  }

  toggleFocusMode() {
    this._focusMode = !this._focusMode;
    if (this._focusMode) {
      this._sidebarCollapsed = true;
      this._referencesPanelCollapsed = true;
    }
  }

  setExpandedBeat(beatId: string | null) {
    this._expandedBeatId = beatId;
  }

  setBeatSaveStatus(status: "idle" | "saving" | "saved" | "error") {
    this._beatSaveStatus = status;
  }

  bumpSceneReferenceRefresh() {
    this._sceneReferenceRefreshId += 1;
  }

  startImport() {
    this._isImporting = true;
    this._importProgress = 0;
    this._importStatus = "Starting import...";
  }

  updateImportProgress(progress: number, status: string) {
    this._importProgress = progress;
    this._importStatus = status;
  }

  finishImport() {
    this._isImporting = false;
    this._importProgress = 100;
    this._importStatus = "Import complete!";
  }

  showError(message: string) {
    this._toast = { id: ++this._toastIdCounter, message };
  }

  clearToast() {
    this._toast = null;
  }

  // Onboarding getters
  get showOnboarding() {
    return this._showOnboarding;
  }

  get onboardingStep() {
    return this._onboardingStep;
  }

  get onboardingCompleted() {
    return this._onboardingCompleted;
  }

  get currentStepIndex() {
    return ONBOARDING_STEPS.indexOf(this._onboardingStep);
  }

  get totalSteps() {
    return ONBOARDING_STEPS.length;
  }

  // Onboarding methods
  startOnboarding() {
    this._showOnboarding = true;
    this._onboardingStep = "welcome";
  }

  nextStep() {
    const currentIndex = ONBOARDING_STEPS.indexOf(this._onboardingStep);
    if (currentIndex < ONBOARDING_STEPS.length - 1) {
      this._onboardingStep = ONBOARDING_STEPS[currentIndex + 1];
    }
  }

  previousStep() {
    const currentIndex = ONBOARDING_STEPS.indexOf(this._onboardingStep);
    if (currentIndex > 0) {
      this._onboardingStep = ONBOARDING_STEPS[currentIndex - 1];
    }
  }

  goToStep(step: OnboardingStep) {
    this._onboardingStep = step;
  }

  completeOnboarding() {
    this._showOnboarding = false;
    this._onboardingCompleted = true;
    localStorage.setItem(ONBOARDING_COMPLETED_KEY, "true");
  }

  skipOnboarding() {
    this.completeOnboarding();
  }

  // For testing/development: reset onboarding
  resetOnboarding() {
    this._showOnboarding = true;
    this._onboardingStep = "welcome";
    this._onboardingCompleted = false;
    localStorage.removeItem(ONBOARDING_COMPLETED_KEY);
  }

  // Guidance getters/setters (Phase C)
  get guidanceEnabled() {
    return this._guidanceEnabled;
  }

  setGuidanceEnabled(enabled: boolean) {
    this._guidanceEnabled = enabled;
    localStorage.setItem(GUIDANCE_ENABLED_KEY, String(enabled));
  }

  hasSeenTooltip(area: GuidanceArea): boolean {
    void this._tooltipSeenVersion;
    return localStorage.getItem(TOOLTIP_SEEN_PREFIX + area) === "true";
  }

  markTooltipSeen(area: GuidanceArea) {
    localStorage.setItem(TOOLTIP_SEEN_PREFIX + area, "true");
    this._tooltipSeenVersion += 1;
  }

  // For testing: reset guidance tooltips
  resetGuidanceTooltips() {
    (["sidebar", "scenePanel", "references", "sync", "planningStatus"] as GuidanceArea[]).forEach(
      (area) => {
        localStorage.removeItem(TOOLTIP_SEEN_PREFIX + area);
      }
    );
    this._tooltipSeenVersion += 1;
  }
}

export const ui = new UIStore();
