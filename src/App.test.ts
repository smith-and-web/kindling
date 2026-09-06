import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process";
import { session } from "./lib/stores/session.svelte";
import { runImport } from "./lib/utils/import";
import { currentProject } from "./lib/stores/project.svelte";
import { mockProject, mockScenes } from "./dev/mock-data";
import App from "./App.svelte";

vi.hoisted(() => {
  const values = new Map([
    ["kindling:onboardingCompleted", "true"],
    ["kindling:guidanceEnabled", "false"],
  ]);
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock("./lib/updater", async (original) => ({
  ...(await original<object>()),
  checkForUpdate: vi.fn(),
}));
vi.mock("./lib/utils/import", async (original) => ({
  ...(await original<object>()),
  runImport: vi.fn(),
}));

const doc = {
  id: "page",
  scene_id: mockScenes[0].id,
  chapter_id: "chapter",
  chapter_title: "Chapter",
  scene_title: "Original scene",
  beat_title: null,
  prose: "<p>Alice Alice</p>",
  locked: false,
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents" ? [doc] : []
  );
  vi.mocked(listen).mockClear();
  vi.mocked(exit).mockClear();
  vi.mocked(getCurrentWindow().onCloseRequested).mockClear();
  currentProject.setProject(mockProject);
  currentProject.setCurrentScene({ ...mockScenes[0], planning_status: "undefined" });
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  vi.restoreAllMocks();
});
async function menu(payload: string) {
  const callback = vi.mocked(listen).mock.calls.find(([event]) => event === "menu-event")![1];
  callback({ event: "menu-event", id: 1, payload });
  await tick();
}

it("waits for pending position saves before quitting from the menu", async () => {
  let finish!: () => void;
  vi.spyOn(session, "flush").mockReturnValue(
    new Promise<void>((resolve) => {
      finish = resolve;
    })
  );
  render(App);
  await menu("quit");
  expect(exit).not.toHaveBeenCalled();
  finish();
  await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
});

it("waits for pending position saves in the native close handler", async () => {
  render(App);
  let finish!: () => void;
  vi.spyOn(session, "flush").mockReturnValue(
    new Promise<void>((resolve) => {
      finish = resolve;
    })
  );
  const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
  let closed = false;
  const closing = Promise.resolve(handler({} as never)).then(() => {
    closed = true;
  });
  await tick();
  expect(closed).toBe(false);
  finish();
  await closing;
  expect(closed).toBe(true);
});

it.each(["menu", "native"])(
  "still closes through %s when a position flush rejects",
  async (path) => {
    render(App);
    await tick();
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(session, "flush").mockRejectedValue(new Error("disk full"));
    if (path === "menu") {
      await menu("quit");
      await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
    } else {
      const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
      await expect(handler({} as never)).resolves.toBeUndefined();
    }
    expect(error).toHaveBeenCalledWith(
      "Failed to save writing position before closing:",
      expect.any(Error)
    );
  }
);
async function find() {
  await menu("find");
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Alice" } });
  expect(screen.getByText("2 matches")).toBeTruthy();
}

it("updates an open dialog from native Find commands without losing the query or undo", async () => {
  render(App);
  await find();
  expect(screen.queryByLabelText("Replace with")).toBeNull();
  await menu("find_project");
  expect((screen.getByLabelText("Search in") as HTMLSelectElement).value).toBe("project");
  expect((screen.getByLabelText("Find") as HTMLInputElement).value).toBe("Alice");
  await fireEvent.input(screen.getByLabelText("Replace with"), { target: { value: "Bob" } });
  await fireEvent.click(screen.getByText("Replace match"));
  await screen.findByText("Replacement saved.");
  await menu("find");
  expect((screen.getByLabelText("Search in") as HTMLSelectElement).value).toBe("scene");
  expect(screen.queryByLabelText("Replace with")).toBeNull();
  await menu("find_replace");
  expect((screen.getByLabelText("Replace with") as HTMLInputElement).value).toBe("Bob");
  await fireEvent.click(screen.getByText("Undo replacement"));
  await screen.findByText("Replacement undone.");
  expect(screen.getByText("2 matches")).toBeTruthy();
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "get_search_documents")
  ).toHaveLength(1);
  screen.getByLabelText("Replace with").focus();
  await menu("find_replace");
  expect(document.activeElement).toBe(screen.getByLabelText("Find"));
});

it.each(["import", "new"])(
  "closes old search when the native %s flow activates another project",
  async (flow) => {
    render(App);
    await find();
    const next = { ...mockProject, id: "new-project", name: "New project" };
    if (flow === "import") {
      vi.mocked(runImport).mockResolvedValue(next);
      await menu("import_ywriter");
    } else {
      await menu("new_project");
      vi.mocked(invoke).mockImplementation(async (cmd) =>
        cmd === "create_blank_project" ? next : []
      );
      await fireEvent.click(screen.getByRole("button", { name: "Create" }));
    }
    await waitFor(() => expect(currentProject.value?.id).toBe(next.id));
    expect(screen.queryByLabelText("Find")).toBeNull();
    // Returning to the original project must not revive its old modal.
    currentProject.setProject(mockProject);
    await tick();
    expect(screen.queryByLabelText("Find")).toBeNull();
  }
);

it("ignores an old project's search response after switching projects during load", async () => {
  let finish!: (value: (typeof doc)[]) => void;
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents"
      ? new Promise<(typeof doc)[]>((resolve) => (finish = resolve))
      : []
  );
  render(App);
  await menu("find_project");
  await waitFor(() => expect(finish).toBeDefined());
  currentProject.setProject({ ...mockProject, id: "another" });
  await tick();
  finish([doc]);
  await tick();
  expect(screen.queryByLabelText("Find")).toBeNull();
  vi.mocked(invoke).mockResolvedValue([]);
  await menu("find_project");
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("get_search_documents", { projectId: "another" })
  );
  expect((screen.getByLabelText("Find") as HTMLInputElement).value).toBe("");
});
