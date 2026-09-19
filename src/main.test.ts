import { afterEach, beforeEach, expect, it, vi } from "vitest";

const { mount, tick } = vi.hoisted(() => ({ mount: vi.fn(), tick: vi.fn() }));
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("svelte", () => ({ mount, tick }));
vi.mock("./App.svelte", () => ({ default: {} }));
vi.mock("./lib/stores/project.svelte", () => ({ currentProject: { setProject: vi.fn() } }));
const { ui } = vi.hoisted(() => ({
  ui: {
    startImport: vi.fn(),
    finishImport: vi.fn(),
    guidanceEnabled: true,
    referencesPanelWidth: 400,
    sidebarCollapsed: true,
    referencesPanelCollapsed: true,
    setGuidanceEnabled(value: boolean) {
      this.guidanceEnabled = value;
    },
    setReferencesPanelWidth(value: number) {
      this.referencesPanelWidth = value;
    },
  },
}));
vi.mock("./lib/stores/ui.svelte", () => ({ ui }));

beforeEach(() => {
  vi.resetModules();
  mount.mockReset().mockReturnValue({ focusAfterStartup: vi.fn() });
  tick.mockReset();
  invoke.mockReset();
  document.body.innerHTML = '<div id="app"></div><div id="startup-loading"></div>';
});

it("reports the created project before later import UI work fails", async () => {
  await import("./main");
  const project = { id: "owned-fixture", name: "QA" };
  invoke.mockResolvedValueOnce(project).mockRejectedValueOnce(new Error("chapter load failed"));
  const observer = vi.fn();
  window.__KINDLING_TEST__!.onProjectCreated = observer;
  const importing = window.__KINDLING_TEST__!.importProject("fixture.pltr");
  // The bridge captures this callback for this action before yielding to IPC.
  delete window.__KINDLING_TEST__!.onProjectCreated;
  await expect(importing).rejects.toThrow("chapter load failed");
  expect(observer).toHaveBeenCalledExactlyOnceWith("import_plottr", project);
});
afterEach(() => {
  document.body.innerHTML = "";
  delete window.__KINDLING_TEST__;
});

it("restores live visual preferences from an independent snapshot", async () => {
  await import("./main");
  const bridge = window.__KINDLING_TEST__!;
  const original = bridge.visualPreferences();
  bridge.setVisualPreferences({
    guidanceEnabled: false,
    referencesPanelWidth: 288,
    sidebarCollapsed: false,
    referencesPanelCollapsed: false,
  });
  expect(ui.referencesPanelWidth).toBe(288);
  expect(ui.guidanceEnabled).toBe(false);
  expect(original.referencesPanelWidth).toBe(400);
  bridge.setVisualPreferences(original);
  expect(bridge.visualPreferences()).toEqual(original);
});

it("reports readiness only after the app's content hook and Svelte flush", async () => {
  let flushed!: () => void;
  tick.mockImplementation(() => new Promise<void>((resolve) => (flushed = resolve)));
  const { ready } = await import("./main");
  const completed = vi.fn();
  void ready.then(completed);
  expect(mount).toHaveBeenCalledOnce();
  expect(tick).not.toHaveBeenCalled();
  expect(window.__KINDLING_TEST__).toBeDefined();
  mount.mock.calls[0][1].props.onReady();
  await vi.waitFor(() => expect(tick).toHaveBeenCalledOnce());
  expect(completed).not.toHaveBeenCalled();
  flushed();
  await ready;
  expect(completed).toHaveBeenCalledOnce();
  // Only the bootstrap may uncover the app after its paint opportunity.
  expect(document.getElementById("startup-loading")).not.toBeNull();
});
