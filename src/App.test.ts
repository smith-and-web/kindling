import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { updateState } from "./lib/updater";
import type { Editor } from "@tiptap/core";
import { ui } from "./lib/stores/ui.svelte";
import { proseSaves } from "./lib/utils/proseSaves";
import { exit, relaunch } from "@tauri-apps/plugin-process";
import { synopsisSaves } from "./lib/stores/synopsisSaves.svelte";
import { session } from "./lib/stores/session.svelte";
import { runImport } from "./lib/utils/import";
import { currentProject } from "./lib/stores/project.svelte";
import { mockProject, mockScenes, mockChapters } from "./dev/mock-data";
import App from "./App.svelte";
import { EditorialSaves } from "./lib/utils/editorialSaves";

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
beforeEach(async () => {
  ui.clearToast();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents" ? [doc] : []
  );
  vi.mocked(listen).mockClear();
  vi.mocked(exit).mockClear();
  vi.mocked(relaunch).mockReset();
  updateState.set(null);
  vi.mocked(getCurrentWindow().onCloseRequested).mockClear();
  currentProject.setProject(mockProject);
  currentProject.setCurrentScene({ ...mockScenes[0], planning_status: "undefined" });
  // Drain the real debounce timer before individual tests install fake timers or flush spies.
  await session.flush();
  HTMLElement.prototype.scrollIntoView = vi.fn();
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
});
afterEach(async () => {
  cleanup();
  updateState.set(null);
  vi.mocked(invoke).mockResolvedValue([]);
  await synopsisSaves.flush();
  await proseSaves.flush(mockProject.id);
  await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  ui.setExpandedBeat(null);
  vi.restoreAllMocks();
  currentProject.setProject(null);
  await session.flush();
  vi.useRealTimers();
});
async function menu(payload: string) {
  const callback = vi.mocked(listen).mock.calls.find(([event]) => event === "menu-event")![1];
  callback({ event: "menu-event", id: 1, payload });
  await tick();
}

it.each(["menu", "native"])("flushes a debounced synopsis before %s quit", async (path) => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
  await fireEvent.input(screen.getByPlaceholderText("Write a brief synopsis for this scene..."), {
    target: { value: "Last words before quitting" },
  });
  let finish!: () => void;
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis")
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
    return [];
  });
  const preventDefault = vi.fn();
  let closed = false;
  let closing: Promise<unknown> | undefined;
  if (path === "menu") await menu("quit");
  else {
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    closing = Promise.resolve(handler({ preventDefault } as never)).then(() => {
      closed = true;
    });
  }
  await vi.advanceTimersByTimeAsync(0);
  try {
    expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
      sceneId: mockScenes[0].id,
      synopsis: "Last words before quitting",
    });
    expect(exit).not.toHaveBeenCalled();
    expect(closed).toBe(false);
    expect(screen.getByRole("main", { hidden: true }).inert).toBe(true);
  } finally {
    finish?.();
  }
  await vi.advanceTimersByTimeAsync(0);
  await closing;
  if (path === "menu") expect(exit).toHaveBeenCalledWith(0);
  else expect(closed).toBe(true);
  expect(preventDefault).not.toHaveBeenCalled();
  expect(screen.getByRole("main").inert).toBe(false);
});

it.each(["menu", "native"])(
  "blocks %s quit on a failed synopsis save and allows retry",
  async (path) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    await fireEvent.input(screen.getByPlaceholderText("Write a brief synopsis for this scene..."), {
      target: { value: "Do not lose this synopsis" },
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") throw "disk full";
      return [];
    });
    const preventDefault = vi.fn();
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    if (path === "menu") await menu("quit");
    else await handler({ preventDefault } as never);
    await vi.advanceTimersByTimeAsync(0);
    expect(exit).not.toHaveBeenCalled();
    if (path === "native") expect(preventDefault).toHaveBeenCalledOnce();
    expect(
      screen.getByRole("dialog", { name: "Quit without saving synopsis changes?" })
    ).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
    expect(screen.getByRole("main").inert).toBe(false);
    expect(synopsisSaves.getState(mockProject.id, mockScenes[0].id).draft?.synopsis).toBe(
      "Do not lose this synopsis"
    );
    expect(screen.getByRole("button", { name: "Retry all synopsis saves" })).toBeTruthy();
    vi.mocked(invoke).mockResolvedValue([]);
    await fireEvent.click(screen.getByRole("button", { name: "Retry all synopsis saves" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(screen.queryByRole("button", { name: "Retry all synopsis saves" })).toBeNull();
    if (path === "menu") {
      await menu("quit");
      await vi.advanceTimersByTimeAsync(0);
      expect(exit).toHaveBeenCalledWith(0);
    } else {
      preventDefault.mockClear();
      await handler({ preventDefault } as never);
      expect(preventDefault).not.toHaveBeenCalled();
    }
  }
);

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

it.each([
  ["menu", "Cannot edit a locked scene"],
  ["native", "Cannot edit a locked scene"],
  ["menu", "disk full"],
  ["native", "disk full"],
])("allows explicit quit-and-discard via %s after %s", async (path, failure) => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsavable draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw failure;
    return [];
  });
  const preventDefault = vi.fn();
  if (path === "menu") await menu("quit");
  else {
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    await handler({ preventDefault } as never);
    expect(preventDefault).toHaveBeenCalledOnce();
  }
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).not.toHaveBeenCalled();
  expect(synopsisSaves.failedCount).toBe(1);
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).toHaveBeenCalledWith(0);
  expect(synopsisSaves.getState(mockProject.id, mockScenes[0].id).draft).toBeUndefined();
  await vi.advanceTimersByTimeAsync(2000);
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_synopsis")
  ).toHaveLength(1);
});

it.each(["menu", "native"])(
  "does not block %s quit after a scene is deleted during synopsis debounce",
  async (path) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    await fireEvent.input(screen.getByPlaceholderText("Write a brief synopsis for this scene..."), {
      target: { value: "Deleted with its scene" },
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") throw "Query returned no rows";
      return [];
    });
    await invoke("delete_scene", { sceneId: mockScenes[0].id });
    currentProject.setCurrentScene(null);
    await tick();
    const preventDefault = vi.fn();
    if (path === "menu") await menu("quit");
    else {
      const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
      await handler({ preventDefault } as never);
    }
    await vi.advanceTimersByTimeAsync(0);
    if (path === "menu") expect(exit).toHaveBeenCalledWith(0);
    expect(preventDefault).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(synopsisSaves.failedCount).toBe(0);
  }
);

it("keeps both synopsis error warnings visible while typing continues", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
  const textarea = screen.getByPlaceholderText("Write a brief synopsis for this scene...");
  await fireEvent.input(textarea, { target: { value: "Unsaved" } });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw "disk full";
    return [];
  });
  await vi.advanceTimersByTimeAsync(1000);
  for (const value of ["Unsaved changes", "Unsaved changes continue"]) {
    await fireEvent.input(textarea, { target: { value } });
    await vi.advanceTimersByTimeAsync(500);
    expect(screen.getByText(/Synopsis not saved: disk full/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Retry all synopsis saves" })).toBeTruthy();
  }
});

it("requires fresh discard confirmation if synopsis drafts change while the quit dialog is open", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  const draft = {
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Original draft",
  };
  synopsisSaves.stage(draft);
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw "disk full";
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  synopsisSaves.stage({ ...draft, synopsis: "New changes after the dialog opened" });
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).not.toHaveBeenCalled();
  expect(synopsisSaves.getState(draft.projectId, draft.sceneId).draft?.synopsis).toBe(
    "New changes after the dialog opened"
  );
  expect(screen.getByRole("main").inert).toBe(false);
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).toHaveBeenCalledWith(0);
});

it.each([false, true])(
  "flushes writing position before quit-and-discard (position save fails: %s)",
  async (fails) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    synopsisSaves.stage({
      projectId: mockProject.id,
      sceneId: mockScenes[0].id,
      synopsis: "Locked draft",
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") throw "Cannot edit a locked scene";
      return [];
    });
    await menu("quit");
    await vi.advanceTimersByTimeAsync(0);
    let finish!: () => void;
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    const flush = vi.spyOn(session, "flush").mockImplementation(async () => {
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
      if (fails) throw new Error("disk full");
    });
    await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
    await vi.advanceTimersByTimeAsync(0);
    try {
      expect(flush).toHaveBeenCalledOnce();
      expect(exit).not.toHaveBeenCalled();
    } finally {
      finish?.();
    }
    await vi.advanceTimersByTimeAsync(0);
    expect(exit).toHaveBeenCalledWith(0);
    if (fails)
      expect(error).toHaveBeenCalledWith(
        "Failed to save writing position before closing:",
        expect.any(Error)
      );
  }
);

it("disables update restart while a quit-discard decision is pending", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  const install = vi.fn().mockResolvedValue(undefined);
  updateState.set({ ready: true, version: "1.2.1", body: null, update: { install } as never });
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsaved draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw "disk full";
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  const restart = screen.getByRole("button", { name: "Restart" }) as HTMLButtonElement;
  expect(restart.disabled).toBe(true);
  expect(screen.queryByText("Your synopsis changes have not been saved.")).toBeNull();
  await fireEvent.click(restart);
  await vi.advanceTimersByTimeAsync(0);
  expect(install).not.toHaveBeenCalled();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect(restart.disabled).toBe(false);
  expect(screen.getByText("Your synopsis changes have not been saved.")).toBeTruthy();
  await fireEvent.click(restart);
  await vi.advanceTimersByTimeAsync(0);
  expect(install).not.toHaveBeenCalled();
  expect(screen.getByText(/Could not restart to update/)).toBeTruthy();
});

it("prevents editing and duplicate restart or quit while an update is saving", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  const install = vi.fn().mockResolvedValue(undefined);
  updateState.set({ ready: true, version: "1.2.1", body: null, update: { install } as never });
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  let finish!: () => void;
  vi.spyOn(session, "flush").mockImplementation(async () => {
    await new Promise<void>((resolve) => {
      finish = resolve;
    });
  });
  const restart = screen.getByRole("button", { name: "Restart" }) as HTMLButtonElement;
  await fireEvent.click(restart);
  await vi.advanceTimersByTimeAsync(0);
  try {
    expect(restart.disabled).toBe(true);
    expect(screen.getByRole("main", { hidden: true }).inert).toBe(true);
    await fireEvent.click(restart);
    await menu("quit");
    const preventDefault = vi.fn();
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    await handler({ preventDefault } as never);
    expect(preventDefault).toHaveBeenCalledOnce();
    expect(exit).not.toHaveBeenCalled();
  } finally {
    finish?.();
  }
  await vi.advanceTimersByTimeAsync(0);
  expect(install).toHaveBeenCalledOnce();
  expect(screen.getByRole("main").inert).toBe(false);
});

async function editProse(mode: "beat" | "page") {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  currentProject.setCurrentChapter(mockChapters[0]);
  currentProject.setCurrentScene({
    ...mockScenes[0],
    planning_status: "fixed",
    editor_mode: mode,
    prose: "<p>Saved page</p>",
  });
  currentProject.setBeats([
    {
      id: "exit-beat",
      scene_id: mockScenes[0].id,
      content: "Beat",
      prose: "<p>Saved beat</p>",
      position: 0,
    },
  ]);
  render(App);
  await tick();
  if (mode === "beat") ui.setExpandedBeat("exit-beat");
  await vi.advanceTimersByTimeAsync(0);
  const editor = (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
  editor.commands.setContent("<p>Final words before exit</p>");
}

it.each([
  ["beat", "menu"],
  ["page", "menu"],
  ["beat", "native"],
  ["page", "native"],
  ["beat", "update"],
  ["page", "update"],
] as const)("flushes debounced prose in %s mode before %s exit", async (mode, path) => {
  const install = vi.fn().mockResolvedValue(undefined);
  if (path === "update")
    updateState.set({ ready: true, version: "1.2.1", body: null, update: { install } as never });
  await editProse(mode);
  let finish!: () => void;
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_beat_prose" || cmd === "save_scene_page_prose")
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
    return [];
  });
  let closing: Promise<unknown> | undefined;
  let closed = false;
  const preventDefault = vi.fn();
  if (path === "menu") await menu("quit");
  else if (path === "update")
    await fireEvent.click(screen.getByRole("button", { name: "Restart" }));
  else {
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    closing = Promise.resolve(handler({ preventDefault } as never)).then(() => {
      closed = true;
    });
  }
  await vi.advanceTimersByTimeAsync(0);
  try {
    expect(invoke).toHaveBeenCalledWith(
      mode === "beat" ? "save_beat_prose" : "save_scene_page_prose",
      {
        [mode === "beat" ? "beatId" : "sceneId"]: mode === "beat" ? "exit-beat" : mockScenes[0].id,
        prose: "<p>Final words before exit</p>",
      }
    );
    expect(exit).not.toHaveBeenCalled();
    expect(install).not.toHaveBeenCalled();
    expect(closed).toBe(false);
  } finally {
    finish?.();
  }
  await vi.advanceTimersByTimeAsync(0);
  await closing;
  if (path === "menu") expect(exit).toHaveBeenCalledWith(0);
  if (path === "update") expect(install).toHaveBeenCalledOnce();
  if (path === "native") expect(closed).toBe(true);
  expect(preventDefault).not.toHaveBeenCalled();
});

it.each([
  ["beat", "disk full"],
  ["page", "disk full"],
  ["beat", "Cannot edit beats in a locked scene"],
  ["page", "Cannot edit a locked scene"],
] as const)("allows explicit discard when %s prose cannot save: %s", async (mode, error) => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  await editProse(mode);
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_beat_prose" || cmd === "save_scene_page_prose") throw error;
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).not.toHaveBeenCalled();
  expect(screen.getByRole("dialog", { name: "Quit without saving writing changes?" })).toBeTruthy();
  expect(proseSaves.draftsForRecovery(mockProject.id)[0].prose).toBe(
    "<p>Final words before exit</p>"
  );
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).toHaveBeenCalledWith(0);
  expect(proseSaves.draftsForRecovery(mockProject.id)).toEqual([]);
  const calls = vi.mocked(invoke).mock.calls.length;
  await vi.advanceTimersByTimeAsync(1000);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.slice(calls)
      .some(([cmd]) => cmd === "save_beat_prose" || cmd === "save_scene_page_prose")
  ).toBe(false);
});

it.each(["install", "relaunch"])("shows an actionable update error when %s fails", async (step) => {
  vi.useFakeTimers();
  vi.spyOn(console, "error").mockImplementation(() => {});
  currentProject.setChapters(mockChapters);
  const install = vi.fn().mockResolvedValue(undefined);
  if (step === "install") install.mockRejectedValue(new Error("Installer failed"));
  else vi.mocked(relaunch).mockRejectedValue(new Error("Relaunch failed"));
  updateState.set({ ready: true, version: "1.2.1", body: null, update: { install } as never });
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Restart" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(
    screen.getByText(
      new RegExp(
        `Could not restart to update.*${step === "install" ? "Installer" : "Relaunch"} failed`
      )
    )
  ).toBeTruthy();
  expect(screen.queryByText(/Retry saving your synopsis first/)).toBeNull();
  expect((screen.getByRole("button", { name: "Restart" }) as HTMLButtonElement).disabled).toBe(
    false
  );
});

it.each(["quit", "update"])(
  "gives %s preparation exclusive keyboard and command ownership",
  async (mode) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    updateState.set({
      ready: true,
      version: "1.2.1",
      body: null,
      update: { install: vi.fn() } as never,
    });
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    const editor = screen.getByPlaceholderText("Write a brief synopsis for this scene...");
    await fireEvent.input(editor, { target: { value: "Keep this editor open" } });
    let fail!: (error: Error) => void;
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") {
        if (mode === "quit") throw new Error("disk full");
        await new Promise((_, reject) => {
          fail = reject;
        });
      }
      return [];
    });
    if (mode === "quit") await menu("quit");
    else await fireEvent.click(screen.getByRole("button", { name: "Restart" }));
    await vi.advanceTimersByTimeAsync(0);
    try {
      if (mode === "quit") {
        expect(document.activeElement).toBe(screen.getByRole("button", { name: "Keep editing" }));
        await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
        expect(document.activeElement).toBe(
          screen.getByRole("button", { name: "Quit and discard" })
        );
        await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
        expect(document.activeElement).toBe(screen.getByRole("button", { name: "Keep editing" }));
      }
      for (const key of ["e", "k", "f", "H"]) {
        await fireEvent.keyDown(window, { key, ctrlKey: true, shiftKey: key === "H" });
        await fireEvent.keyDown(window, { key, metaKey: true, shiftKey: key === "H" });
      }
      for (const command of [
        "export",
        "command_palette",
        "find",
        "new_project",
        "close_project",
        "import_plottr",
      ])
        await menu(command);
      expect(currentProject.value?.id).toBe(mockProject.id);
      expect(screen.queryByPlaceholderText("Type a command or search...")).toBeNull();
      expect(screen.queryByText("Export Project")).toBeNull();
      await fireEvent.keyDown(window, { key: "Escape" });
      expect(editor.isConnected).toBe(true);
      if (mode === "quit")
        expect(screen.queryByRole("button", { name: "Keep editing" })).toBeNull();
      else expect(screen.getByRole("main", { hidden: true }).inert).toBe(true);
    } finally {
      fail?.(new Error("disk full"));
    }
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(screen.getByPlaceholderText("Type a command or search...")).toBeTruthy();
    await vi.advanceTimersByTimeAsync(0);
  }
);

it.each(["same scene", "navigation", "other control", "unmount"])(
  "restores update-failure selection only for the original writing target: %s",
  async (target) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    updateState.set({
      ready: true,
      version: "1.2.1",
      body: null,
      update: { install: vi.fn() } as never,
    });
    const app = render(App);
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    const editor = screen.getByPlaceholderText(
      "Write a brief synopsis for this scene..."
    ) as HTMLTextAreaElement;
    await fireEvent.input(editor, { target: { value: "My unsaved writing" } });
    editor.focus();
    editor.setSelectionRange(3, 7, "backward");
    let fail!: (error: Error) => void;
    const saving = new Promise((_, reject) => {
      fail = reject;
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") await saving;
      return [];
    });
    await fireEvent.click(screen.getByRole("button", { name: "Restart" }));
    await vi.advanceTimersByTimeAsync(0);
    editor.blur(); // jsdom does not implement the browser's inert-induced blur.
    if (target === "navigation") {
      currentProject.setCurrentScene({ ...mockScenes[1], planning_status: "undefined" });
      await tick();
    }
    const other = document.createElement("button");
    if (target === "other control") {
      document.body.append(other);
      other.focus();
    }
    if (target === "unmount") app.unmount();
    fail(new Error("disk full"));
    await vi.advanceTimersByTimeAsync(0);
    if (target === "same scene") {
      expect(document.activeElement).toBe(editor);
      expect([editor.selectionStart, editor.selectionEnd, editor.selectionDirection]).toEqual([
        3,
        7,
        "backward",
      ]);
      expect(editor.value).toBe("My unsaved writing");
    } else expect(document.activeElement).not.toBe(editor);
    other.remove();
  }
);

it.each(["export", "export success"])(
  "keeps confirmation keys out of the background %s dialog",
  async (background) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    localStorage.setItem("kindling:lastExportPath", "/tmp/kindling-review-export");
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") throw new Error("disk full");
      if (cmd === "export_to_markdown")
        return {
          output_path: "/tmp/kindling-review-export",
          chapters_exported: 1,
          scenes_exported: 1,
          files_created: 1,
        };
      return [];
    });
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    await menu("export");
    await fireEvent.click(screen.getByText("Markdown"));
    expect((screen.getByTestId("export-confirm") as HTMLButtonElement).disabled).toBe(false);
    if (background === "export success")
      await fireEvent.click(screen.getByTestId("export-confirm"));
    synopsisSaves.stage({
      projectId: mockProject.id,
      sceneId: mockScenes[0].id,
      synopsis: "Unsaved draft",
    });
    await menu("quit");
    await vi.advanceTimersByTimeAsync(0);
    vi.mocked(invoke).mockClear();
    const keepEditing = screen.getByRole("button", { name: "Keep editing" });
    const backgroundKey = vi.fn();
    window.addEventListener("keydown", backgroundKey);
    try {
      for (const key of ["Enter", " ", "ArrowDown"]) {
        const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
        await fireEvent(keepEditing, event);
        expect(event.defaultPrevented).toBe(false);
      }
      expect(backgroundKey).not.toHaveBeenCalled();
      expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "export_to_markdown")).toBe(
        false
      );
      // jsdom does not perform the native button click generated by Enter.
      await fireEvent.click(keepEditing);
      expect(screen.queryByRole("button", { name: "Keep editing" })).toBeNull();
      if (background === "export success") expect(screen.getByText("Export Complete")).toBeTruthy();
      else expect(screen.getByTestId("export-confirm")).toBeTruthy();
    } finally {
      window.removeEventListener("keydown", backgroundKey);
      localStorage.removeItem("kindling:lastExportPath");
    }
  }
);

it("keeps errors visible and dismissable within the active quit modal", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  ui.showError("Error before quitting");
  await tick();
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsaved draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw new Error("disk full");
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  const confirmation = screen.getByRole("dialog", {
    name: "Quit without saving synopsis changes?",
  });
  expect(confirmation.tagName).toBe("DIALOG");
  expect(confirmation.querySelector('[role="dialog"]')).toBeNull();
  expect(
    screen.getByText("Error before quitting").closest("[data-quit-confirmation]")
  ).toBeTruthy();
  ui.showError("Error during confirmation");
  await tick();
  const dismiss = screen.getByRole("button", { name: "Dismiss error" });
  expect(dismiss.closest("[data-quit-confirmation]")).toBeTruthy();
  expect(screen.getAllByRole("button", { name: "Dismiss error" })).toHaveLength(1);
  await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
  await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
  expect(document.activeElement).toBe(dismiss);
  await fireEvent.click(dismiss);
  expect(screen.queryByText("Error during confirmation")).toBeNull();
  expect(screen.getByRole("button", { name: "Keep editing" })).toBeTruthy();
  ui.showError("Still relevant after cancelling");
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect(
    screen.getByText("Still relevant after cancelling").closest("[data-quit-confirmation]")
  ).toBeNull();
  expect(screen.getAllByRole("button", { name: "Dismiss error" })).toHaveLength(1);
  ui.clearToast();
});

it.each(["quit", "update"])("blocks retry saving while %s preparation is pending", async (mode) => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  updateState.set({
    ready: true,
    version: "1.2.1",
    body: null,
    update: { install: vi.fn() } as never,
  });
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsaved draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw new Error("disk full");
    return [];
  });
  await expect(synopsisSaves.flush()).rejects.toThrow();
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  const retry = screen.getByRole("button", {
    name: "Retry all synopsis saves",
  }) as HTMLButtonElement;
  let fail!: (error: Error) => void;
  const flush = vi.spyOn(synopsisSaves, "flush").mockImplementation(
    () =>
      new Promise((_, reject) => {
        fail = reject;
      })
  );
  try {
    if (mode === "quit") await menu("quit");
    else await fireEvent.click(screen.getByRole("button", { name: "Restart" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(retry.disabled).toBe(true);
    await fireEvent.click(retry);
    expect(flush).toHaveBeenCalledOnce();
    fail(new Error("disk full"));
    await vi.advanceTimersByTimeAsync(0);
  } finally {
    flush.mockRestore();
  }
  if (mode === "quit") await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  const enabledRetry = screen.getByRole("button", {
    name: "Retry all synopsis saves",
  }) as HTMLButtonElement;
  expect(enabledRetry.disabled).toBe(false);
  vi.mocked(invoke).mockResolvedValue([]);
  await fireEvent.click(enabledRetry);
  await vi.advanceTimersByTimeAsync(0);
  expect(synopsisSaves.failedCount).toBe(0);
});

it("attempts writing saves and offers explicit recovery when an editorial save blocks quitting", async () => {
  const round = {
    id: "quit-review",
    project_id: "project",
    title: "Letter",
    name: "Review",
    brief: "",
    created_at: "today",
    sources: [
      {
        id: "source",
        scene_id: "scene",
        chapter_id: "chapter",
        chapter: "Chapter",
        scene: "Letter",
        mode: "page",
        html: "<p>Eleanor waited.</p>",
        locked: false,
      },
    ],
  };
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "take_editorial_open_files") return ["/review.kindling-review"];
    if (cmd === "open_editorial_package")
      return { format: "kindling-editorial", version: 1, kind: "review", round, session: null };
    return [];
  });
  Range.prototype.getClientRects = vi.fn().mockReturnValue([]);
  Range.prototype.getBoundingClientRect = vi
    .fn()
    .mockReturnValue({ left: 0, right: 0, top: 0, bottom: 0 });
  render(App);
  await screen.findByRole("region", { name: "Editorial workspace" });
  await fireEvent.input(screen.getByLabelText("Your name"), { target: { value: "Rowan" } });
  vi.spyOn(EditorialSaves.prototype, "flush").mockRejectedValue(new Error("disk full"));
  const writing = vi.spyOn(proseSaves, "flush").mockResolvedValue([]);
  const synopsis = vi.spyOn(synopsisSaves, "flush").mockResolvedValue(undefined);
  await menu("quit");
  await screen.findByRole("dialog", { name: "Quit without saving review changes?" });
  expect(writing).toHaveBeenCalled();
  expect(synopsis).toHaveBeenCalled();
  expect(exit).not.toHaveBeenCalled();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect(localStorage.getItem("kindling.editorial.recovery.quit-review")).toContain("Rowan");
  localStorage.removeItem("kindling.editorial.recovery.quit-review");
});

it("opens one settings window from the menu and reserves native project commands until it closes", async () => {
  render(App);
  await menu("settings");
  expect(await screen.findByRole("dialog", { name: "Settings" })).toBeTruthy();
  await menu("settings");
  expect(screen.getAllByRole("dialog", { name: "Settings" })).toHaveLength(1);
  await menu("new_project");
  expect(screen.queryByText("Create New Project")).toBeNull();
  await menu("close_project");
  expect(currentProject.value?.id).toBe(mockProject.id);
});

it("opens the shared settings window from the pinned project sidebar footer", async () => {
  render(App);
  const link = screen.getByTestId("sidebar-settings-button");
  expect(link.closest("footer")?.parentElement).toBe(screen.getByTestId("sidebar"));
  await fireEvent.click(link);
  expect(await screen.findByRole("dialog", { name: "Settings" })).toBeTruthy();
  await menu("settings");
  expect(screen.getAllByRole("dialog", { name: "Settings" })).toHaveLength(1);
});

it("keeps Settings visible and reachable from every entry point during local editorial review", async () => {
  const source = {
    id: mockScenes[0].id,
    scene_id: mockScenes[0].id,
    chapter_id: mockScenes[0].chapter_id,
    chapter: "Chapter",
    scene: "Scene",
    mode: "page",
    html: "<p>Manuscript for settings navigation.</p>",
    locked: false,
  };
  const round = {
    id: "settings-review",
    project_id: mockProject.id,
    title: "Settings test",
    name: "Local review",
    brief: "",
    created_at: "2026-09-09",
    sources: [source],
  };
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "open_local_editorial_review")
      return { format: "kindling-editorial", version: 1, kind: "review", round, session: null };
    if (command === "get_editorial_feedback")
      return { round, sources: [source], entries: [], version: 1 };
    if (command === "editorial_sources") return [source];
    return [];
  });
  Range.prototype.getClientRects = vi.fn().mockReturnValue([]);
  Range.prototype.getBoundingClientRect = vi.fn().mockReturnValue(new DOMRect());
  render(App);
  await fireEvent.click(await screen.findByRole("button", { name: "Revisions" }));
  await screen.findByRole("button", { name: "Return to writing" });
  const footer = screen.getByTestId("sidebar-settings-button");
  await fireEvent.click(footer);
  let settings = await screen.findByRole("dialog", { name: "Settings" });
  expect(settings.closest("[hidden]")).toBeNull();
  await fireEvent.click(screen.getByRole("button", { name: "Close settings" }));
  await menu("settings");
  settings = await screen.findByRole("dialog", { name: "Settings" });
  expect(settings.closest("[hidden]")).toBeNull();
  await fireEvent.click(screen.getByRole("button", { name: "Close settings" }));
  await fireEvent.keyDown(window, { key: ",", metaKey: true });
  settings = await screen.findByRole("dialog", { name: "Settings" });
  expect(settings.closest("[hidden]")).toBeNull();
  expect(currentProject.value?.id).toBe(mockProject.id);
});
