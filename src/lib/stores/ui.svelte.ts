// UI state management using Svelte 5 runes

export type View = "start" | "editor";
export type Panel = "sidebar" | "editor" | "references";

// Panel width constraints
const REFERENCES_PANEL_MIN_WIDTH = 200;
const REFERENCES_PANEL_MAX_WIDTH = 600;
const REFERENCES_PANEL_DEFAULT_WIDTH = 288; // w-72
const REFERENCES_PANEL_STORAGE_KEY = "kindling:referencesPanelWidth";

class UIStore {
  private _currentView = $state<View>("start");
  private _sidebarCollapsed = $state(false);
  private _referencesPanelCollapsed = $state(false);
  private _referencesPanelWidth = $state(REFERENCES_PANEL_DEFAULT_WIDTH);
  private _focusMode = $state(false);
  private _expandedBeatId = $state<string | null>(null);
  private _isImporting = $state(false);
  private _importProgress = $state(0);
  private _importStatus = $state("");

  constructor() {
    // Load saved panel width from localStorage
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem(REFERENCES_PANEL_STORAGE_KEY);
      if (saved) {
        const width = parseInt(saved, 10);
        if (
          !isNaN(width) &&
          width >= REFERENCES_PANEL_MIN_WIDTH &&
          width <= REFERENCES_PANEL_MAX_WIDTH
        ) {
          this._referencesPanelWidth = width;
        }
      }
    }
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
    return REFERENCES_PANEL_MAX_WIDTH;
  }

  setReferencesPanelWidth(width: number) {
    const clamped = Math.max(
      REFERENCES_PANEL_MIN_WIDTH,
      Math.min(REFERENCES_PANEL_MAX_WIDTH, width)
    );
    this._referencesPanelWidth = clamped;
    // Persist to localStorage
    if (typeof window !== "undefined") {
      localStorage.setItem(REFERENCES_PANEL_STORAGE_KEY, String(clamped));
    }
  }

  get focusMode() {
    return this._focusMode;
  }

  get expandedBeatId() {
    return this._expandedBeatId;
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
}

export const ui = new UIStore();
