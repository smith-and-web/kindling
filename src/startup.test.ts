import { afterEach, beforeEach, expect, it, vi } from "vitest";
import startupHtml from "../index.html?raw";
import tauriConfig from "../src-tauri/tauri.conf.json";

const mocks = vi.hoisted(() => ({
  isTauri: vi.fn(),
  loadMain: vi.fn(),
  focus: vi.fn(),
  setupPluginListeners: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ isTauri: mocks.isTauri }));
vi.mock("tauri-plugin-mcp", () => ({ setupPluginListeners: mocks.setupPluginListeners }));

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => (resolve = done));
  return { promise, resolve };
}
let frames: FrameRequestCallback[];
let content: ReturnType<typeof deferred>;
let fonts: ReturnType<typeof deferred>;
const originalFonts = Object.getOwnPropertyDescriptor(document, "fonts");

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  document.body.innerHTML = new DOMParser().parseFromString(
    startupHtml,
    "text/html"
  ).body.innerHTML;
  frames = [];
  vi.stubGlobal("PerformanceObserver", undefined);
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    frames.push(callback);
    return frames.length;
  });
  content = deferred();
  fonts = deferred();
  Object.defineProperty(document, "fonts", {
    configurable: true,
    value: {
      get ready() {
        return mocks.loadMain.mock.calls.length ? fonts.promise : Promise.resolve();
      },
    },
  });
  mocks.isTauri.mockReturnValue(true);
  mocks.setupPluginListeners.mockResolvedValue(undefined);
  mocks.focus.mockImplementation(() => {
    expect(document.getElementById("app")?.inert).toBe(false);
    expect(document.getElementById("startup-loading")).toBeNull();
  });
  mocks.loadMain.mockReturnValue({ ready: content.promise, focusAfterStartup: mocks.focus });
  vi.doMock("./main", () => mocks.loadMain());
});
afterEach(() => {
  document.body.innerHTML = "";
  if (originalFonts) Object.defineProperty(document, "fonts", originalFonts);
  else Reflect.deleteProperty(document, "fonts");
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});
function frame() {
  for (const callback of frames.splice(0)) callback(performance.now());
}
async function begin() {
  const { startup } = await import("./startup");
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  return { startup };
}

async function shellFrames() {
  frame();
  frame();
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  frame();
  frame();
}

it("ships the shell and small entry point with the app initially inert", () => {
  expect(tauriConfig.app.windows[0].visible).toBe(false);
  const page = new DOMParser().parseFromString(startupHtml, "text/html");
  expect(page.querySelector('script[type="module"]')?.getAttribute("src")).toBe("/src/startup.ts");
  expect(page.querySelector('link[href="/src/startup.css"]')).not.toBeNull();
  expect(page.querySelector("#app")?.hasAttribute("inert")).toBe(true);
  expect(page.querySelector("#startup-loading")?.getAttribute("role")).toBe("status");
  expect(page.querySelector("#startup-loading")?.textContent).toContain("Starting Kindling…");
});
it("paints before loading the app and waits for content, assets and another paint", async () => {
  const decoded = deferred();
  const image = document.createElement("img");
  image.decode = vi.fn(() => decoded.promise);
  document.getElementById("app")!.append(image);
  const { startup } = await begin();
  expect(mocks.loadMain).not.toHaveBeenCalled();
  frame();
  expect(mocks.loadMain).not.toHaveBeenCalled();
  frame();
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  expect(mocks.loadMain).not.toHaveBeenCalled();
  frame();
  frame();
  await vi.waitFor(() => expect(mocks.loadMain).toHaveBeenCalledOnce());
  expect(document.getElementById("startup-loading")).not.toBeNull();
  expect(frames).toHaveLength(0);
  content.resolve();
  await Promise.resolve();
  expect(image.decode).not.toHaveBeenCalled();
  fonts.resolve();
  await vi.waitFor(() => expect(image.decode).toHaveBeenCalledOnce());
  expect(frames).toHaveLength(0);
  decoded.resolve();
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  frame();
  expect(document.getElementById("startup-loading")).not.toBeNull();
  frame();
  await startup;
  expect(document.getElementById("startup-loading")).toBeNull();
  expect(document.getElementById("app")?.inert).toBe(false);
  expect(document.getElementById("app")?.hasAttribute("aria-busy")).toBe(false);
});
it("tolerates failed images", async () => {
  const image = document.createElement("img");
  image.decode = vi.fn().mockRejectedValue(new Error("Missing image"));
  document.getElementById("app")!.append(image);
  const { startup } = await begin();
  await shellFrames();
  content.resolve();
  fonts.resolve();
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  frame();
  frame();
  await startup;
  expect(document.getElementById("startup-loading")).toBeNull();
});
it("offers retry when the application module fails instead of spinning forever", async () => {
  mocks.isTauri.mockReturnValue(false);
  mocks.loadMain.mockImplementation(() => {
    throw new Error("Bundle unavailable");
  });
  vi.spyOn(console, "error").mockImplementation(() => {});
  const { startup } = await begin();
  await shellFrames();
  await startup;
  expect(mocks.setupPluginListeners).not.toHaveBeenCalled();
  expect(document.getElementById("startup-loading")?.getAttribute("role")).toBe("alert");
  expect(document.getElementById("startup-message")?.textContent).toContain("couldn’t start");
  expect(document.querySelector<HTMLElement>(".startup-spinner")?.hidden).toBe(true);
  expect(document.getElementById("startup-retry")?.hidden).toBe(false);
  expect(document.getElementById("startup-retry")?.onclick).toBeTypeOf("function");
  expect(document.getElementById("app")?.hasAttribute("inert")).toBe(true);
});

it("waits for reported contentful paint when the browser supports paint timing", async () => {
  let report!: (entries: { getEntries: () => { name: string }[] }) => void;
  const observe = vi.fn();
  const disconnect = vi.fn();
  vi.stubGlobal(
    "PerformanceObserver",
    class {
      static supportedEntryTypes = ["paint"];
      constructor(callback: typeof report) {
        report = callback;
      }
      observe = observe;
      disconnect = disconnect;
    }
  );
  const { startup } = await import("./startup");
  await vi.waitFor(() => expect(observe).toHaveBeenCalledWith({ type: "paint", buffered: true }));
  expect(mocks.loadMain).not.toHaveBeenCalled();
  report({ getEntries: () => [{ name: "first-paint" }] });
  await Promise.resolve();
  expect(mocks.loadMain).not.toHaveBeenCalled();
  report({ getEntries: () => [{ name: "first-contentful-paint" }] });
  await vi.waitFor(() => expect(mocks.loadMain).toHaveBeenCalledOnce());
  expect(disconnect).toHaveBeenCalledOnce();
  content.resolve();
  fonts.resolve();
  await vi.waitFor(() => expect(frames).toHaveLength(1));
  frame();
  frame();
  await startup;
  expect(mocks.focus).toHaveBeenCalledOnce();
});
