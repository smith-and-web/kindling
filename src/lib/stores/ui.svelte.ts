// UI state management using Svelte 5 runes

export type View = "start" | "editor";
export type Panel = "sidebar" | "editor" | "references";

class UIStore {
  currentView = $state<View>("start");
  sidebarCollapsed = $state(false);
  referencesPanelCollapsed = $state(false);
  focusMode = $state(false);
  expandedBeatId = $state<string | null>(null);
  isImporting = $state(false);
  importProgress = $state(0);
  importStatus = $state("");

  setView(view: View) {
    this.currentView = view;
  }

  toggleSidebar() {
    this.sidebarCollapsed = !this.sidebarCollapsed;
  }

  toggleReferencesPanel() {
    this.referencesPanelCollapsed = !this.referencesPanelCollapsed;
  }

  toggleFocusMode() {
    this.focusMode = !this.focusMode;
    if (this.focusMode) {
      this.sidebarCollapsed = true;
      this.referencesPanelCollapsed = true;
    }
  }

  setExpandedBeat(beatId: string | null) {
    this.expandedBeatId = beatId;
  }

  startImport() {
    this.isImporting = true;
    this.importProgress = 0;
    this.importStatus = "Starting import...";
  }

  updateImportProgress(progress: number, status: string) {
    this.importProgress = progress;
    this.importStatus = status;
  }

  finishImport() {
    this.isImporting = false;
    this.importProgress = 100;
    this.importStatus = "Import complete!";
  }
}

export const ui = new UIStore();
