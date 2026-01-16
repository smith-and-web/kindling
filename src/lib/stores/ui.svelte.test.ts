import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";

// Mock localStorage before importing ui store
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

vi.stubGlobal("localStorage", localStorageMock);

// Import after mocking
const { ui } = await import("./ui.svelte");

describe("ui store", () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
    // Reset UI state
    ui.setView("start");
    ui.sidebarCollapsed = false;
    ui.referencesPanelCollapsed = false;
    ui.setExpandedBeat(null);
    ui.setBeatSaveStatus("idle");
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("view management", () => {
    it("should initialize with start view", () => {
      expect(ui.currentView).toBe("start");
    });

    it("should set view", () => {
      ui.setView("editor");
      expect(ui.currentView).toBe("editor");
    });
  });

  describe("sidebar management", () => {
    it("should initialize with sidebar expanded", () => {
      expect(ui.sidebarCollapsed).toBe(false);
    });

    it("should toggle sidebar", () => {
      ui.toggleSidebar();
      expect(ui.sidebarCollapsed).toBe(true);

      ui.toggleSidebar();
      expect(ui.sidebarCollapsed).toBe(false);
    });

    it("should set sidebar collapsed directly", () => {
      ui.sidebarCollapsed = true;
      expect(ui.sidebarCollapsed).toBe(true);
    });
  });

  describe("references panel management", () => {
    it("should initialize with references panel expanded", () => {
      expect(ui.referencesPanelCollapsed).toBe(false);
    });

    it("should toggle references panel", () => {
      ui.toggleReferencesPanel();
      expect(ui.referencesPanelCollapsed).toBe(true);

      ui.toggleReferencesPanel();
      expect(ui.referencesPanelCollapsed).toBe(false);
    });

    it("should set references panel collapsed directly", () => {
      ui.referencesPanelCollapsed = true;
      expect(ui.referencesPanelCollapsed).toBe(true);
    });

    it("should return panel width", () => {
      expect(ui.referencesPanelWidth).toBeGreaterThan(0);
    });

    it("should return min width constraint", () => {
      expect(ui.referencesPanelMinWidth).toBe(200);
    });

    it("should return max width constraint", () => {
      expect(ui.referencesPanelMaxWidth).toBeGreaterThanOrEqual(200);
    });

    it("should set references panel width with clamping", () => {
      ui.setReferencesPanelWidth(300);
      expect(ui.referencesPanelWidth).toBe(300);

      // Test min clamping
      ui.setReferencesPanelWidth(50);
      expect(ui.referencesPanelWidth).toBe(200); // min width

      // Test persistence
      expect(localStorageMock.setItem).toHaveBeenCalled();
    });
  });

  describe("focus mode", () => {
    it("should toggle focus mode on and collapse panels", () => {
      // Start with focus mode off
      const initialFocusMode = ui.focusMode;

      // Toggle to opposite state
      ui.toggleFocusMode();
      expect(ui.focusMode).toBe(!initialFocusMode);

      // If we're now in focus mode, panels should be collapsed
      if (ui.focusMode) {
        expect(ui.sidebarCollapsed).toBe(true);
        expect(ui.referencesPanelCollapsed).toBe(true);
      }
    });

    it("should toggle focus mode back and forth", () => {
      const initial = ui.focusMode;
      ui.toggleFocusMode();
      expect(ui.focusMode).toBe(!initial);
      ui.toggleFocusMode();
      expect(ui.focusMode).toBe(initial);
    });
  });

  describe("beat expansion", () => {
    it("should initialize with no expanded beat", () => {
      expect(ui.expandedBeatId).toBeNull();
    });

    it("should set expanded beat", () => {
      ui.setExpandedBeat("beat-1");
      expect(ui.expandedBeatId).toBe("beat-1");
    });

    it("should clear expanded beat", () => {
      ui.setExpandedBeat("beat-1");
      ui.setExpandedBeat(null);
      expect(ui.expandedBeatId).toBeNull();
    });
  });

  describe("beat save status", () => {
    it("should initialize with idle status", () => {
      expect(ui.beatSaveStatus).toBe("idle");
    });

    it("should set save status to saving", () => {
      ui.setBeatSaveStatus("saving");
      expect(ui.beatSaveStatus).toBe("saving");
    });

    it("should set save status to saved", () => {
      ui.setBeatSaveStatus("saved");
      expect(ui.beatSaveStatus).toBe("saved");
    });

    it("should set save status to error", () => {
      ui.setBeatSaveStatus("error");
      expect(ui.beatSaveStatus).toBe("error");
    });
  });

  describe("import progress", () => {
    it("should initialize with not importing", () => {
      expect(ui.isImporting).toBe(false);
      expect(ui.importProgress).toBe(0);
    });

    it("should start import", () => {
      ui.startImport();
      expect(ui.isImporting).toBe(true);
      expect(ui.importProgress).toBe(0);
      expect(ui.importStatus).toBe("Starting import...");
    });

    it("should update import progress", () => {
      ui.startImport();
      ui.updateImportProgress(50, "Processing chapters...");
      expect(ui.importProgress).toBe(50);
      expect(ui.importStatus).toBe("Processing chapters...");
    });

    it("should finish import", () => {
      ui.startImport();
      ui.finishImport();
      expect(ui.isImporting).toBe(false);
      expect(ui.importProgress).toBe(100);
      expect(ui.importStatus).toBe("Import complete!");
    });
  });

  describe("onboarding", () => {
    it("should return onboarding completed status", () => {
      expect(typeof ui.onboardingCompleted).toBe("boolean");
    });

    it("should return current step", () => {
      expect(ui.onboardingStep).toBeDefined();
    });

    it("should return current step index", () => {
      expect(ui.currentStepIndex).toBeGreaterThanOrEqual(0);
    });

    it("should return total steps", () => {
      expect(ui.totalSteps).toBe(5);
    });

    it("should start onboarding", () => {
      ui.completeOnboarding();
      ui.startOnboarding();
      expect(ui.showOnboarding).toBe(true);
      expect(ui.onboardingStep).toBe("welcome");
    });

    it("should go to next step", () => {
      ui.startOnboarding();
      ui.nextStep();
      expect(ui.onboardingStep).toBe("tour-sidebar");
    });

    it("should go to previous step", () => {
      ui.startOnboarding();
      ui.nextStep();
      ui.previousStep();
      expect(ui.onboardingStep).toBe("welcome");
    });

    it("should not go before first step", () => {
      ui.startOnboarding();
      ui.previousStep();
      expect(ui.onboardingStep).toBe("welcome");
    });

    it("should go to specific step", () => {
      ui.startOnboarding();
      ui.goToStep("tour-editor");
      expect(ui.onboardingStep).toBe("tour-editor");
    });

    it("should complete onboarding", () => {
      ui.startOnboarding();
      ui.completeOnboarding();
      expect(ui.showOnboarding).toBe(false);
      expect(ui.onboardingCompleted).toBe(true);
      expect(localStorageMock.setItem).toHaveBeenCalledWith("kindling:onboardingCompleted", "true");
    });

    it("should skip onboarding (same as complete)", () => {
      ui.startOnboarding();
      ui.skipOnboarding();
      expect(ui.showOnboarding).toBe(false);
      expect(ui.onboardingCompleted).toBe(true);
    });

    it("should reset onboarding", () => {
      ui.completeOnboarding();
      ui.resetOnboarding();
      expect(ui.showOnboarding).toBe(true);
      expect(ui.onboardingStep).toBe("welcome");
      expect(ui.onboardingCompleted).toBe(false);
      expect(localStorageMock.removeItem).toHaveBeenCalledWith("kindling:onboardingCompleted");
    });
  });
});
