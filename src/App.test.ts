import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { updateState } from "./lib/updater";
import { exit } from "@tauri-apps/plugin-process";
import { synopsisSaves } from "./lib/stores/synopsisSaves.svelte";
import { session } from "./lib/stores/session.svelte";
import { runImport } from "./lib/utils/import";
import { currentProject } from "./lib/stores/project.svelte";
import { mockProject, mockScenes, mockChapters } from "./dev/mock-data";
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
  updateState.set(null);
  vi.mocked(getCurrentWindow().onCloseRequested).mockClear();
  currentProject.setProject(mockProject);
  currentProject.setCurrentScene({ ...mockScenes[0], planning_status: "undefined" });
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
});
afterEach(async () => {
  cleanup();
  updateState.set(null);
  vi.mocked(invoke).mockResolvedValue([]);
  await synopsisSaves.flush();
  currentProject.setProject(null);
  vi.useRealTimers();
  vi.restoreAllMocks();
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
