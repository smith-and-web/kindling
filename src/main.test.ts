import { afterEach, beforeEach, expect, it, vi } from "vitest";

const { mount, tick } = vi.hoisted(() => ({ mount: vi.fn(), tick: vi.fn() }));
vi.mock("svelte", () => ({ mount, tick }));
vi.mock("./App.svelte", () => ({ default: {} }));
vi.mock("./lib/stores/project.svelte", () => ({ currentProject: {} }));
vi.mock("./lib/stores/ui.svelte", () => ({ ui: {} }));

beforeEach(() => {
  vi.resetModules();
  mount.mockReset().mockReturnValue({ focusAfterStartup: vi.fn() });
  tick.mockReset();
  document.body.innerHTML = '<div id="app"></div><div id="startup-loading"></div>';
});
afterEach(() => {
  document.body.innerHTML = "";
  delete window.__KINDLING_TEST__;
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
