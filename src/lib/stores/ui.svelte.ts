// UI state management using Svelte 5 runes

export type View = "start" | "editor";
export type Panel = "sidebar" | "editor" | "references";

class UIStore {
  private _currentView = $state<View>("start");
  private _sidebarCollapsed = $state(false);
  private _referencesPanelCollapsed = $state(false);
  private _focusMode = $state(false);
  private _expandedBeatId = $state<string | null>(null);
  private _isImporting = $state(false);
  private _importProgress = $state(0);
  private _importStatus = $state("");

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
