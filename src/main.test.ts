import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import startupHtml from "../index.html?raw";

const { mount, tick, setupPluginListeners } = vi.hoisted(() => ({
  mount: vi.fn(),
  tick: vi.fn(),
  setupPluginListeners: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("svelte", () => ({ mount, tick }));
vi.mock("./App.svelte", () => ({ default: {} }));
vi.mock("tauri-plugin-mcp", () => ({ setupPluginListeners }));
vi.mock("./lib/stores/project.svelte", () => ({ currentProject: {} }));
vi.mock("./lib/stores/ui.svelte", () => ({ ui: {} }));

describe("startup loading screen", () => {
  beforeEach(() => {
    vi.resetModules();
    mount.mockReset();
    tick.mockReset();
    const page = new DOMParser().parseFromString(startupHtml, "text/html");
    document.body.innerHTML = page.body.innerHTML;
  });

  afterEach(() => {
    document.body.innerHTML = "";
    delete window.__KINDLING_TEST__;
  });

  it("shows accessible feedback before JavaScript mounts the app", () => {
    const loader = document.getElementById("startup-loading")!;
    expect(loader.getAttribute("role")).toBe("status");
    expect(loader.textContent).toContain("Starting Kindling…");
    expect(loader.querySelector(".startup-spinner")?.getAttribute("aria-hidden")).toBe("true");
    expect(document.getElementById("app")!.contains(loader)).toBe(false);
    expect(mount).not.toHaveBeenCalled();
  });

  it("keeps feedback until Svelte's first render completes, then removes it", async () => {
    let rendered!: () => void;
    tick.mockImplementation(() => new Promise<void>((resolve) => (rendered = resolve)));
    mount.mockImplementation((_component: unknown, { target }: { target: HTMLElement }) => {
      target.innerHTML = "<main>Ready to write</main>";
    });

    const startup = import("./main");
    await vi.waitFor(() => expect(tick).toHaveBeenCalledOnce());
    expect(document.getElementById("startup-loading")).not.toBeNull();
    rendered();
    await startup;

    expect(document.getElementById("startup-loading")).toBeNull();
    expect(document.querySelector("#app main")?.textContent).toBe("Ready to write");
  });

  it("also mounts when the loading element is absent", async () => {
    document.getElementById("startup-loading")!.remove();
    tick.mockResolvedValue(undefined);
    await import("./main");
    expect(mount).toHaveBeenCalledOnce();
  });
});
