import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import type { Editor } from "@tiptap/core";
import { proseSaves } from "../utils/proseSaves";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import BeatView from "./BeatView.svelte";
import ScenePanel from "./ScenePanel.svelte";
import SnapshotsPanel from "./SnapshotsPanel.svelte";
import FindReplaceDialog from "./FindReplaceDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { mockProject, mockScenes, mockChapters } from "../../dev/mock-data";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});
const beat = {
  id: "beat",
  scene_id: "scene",
  content: "Greeting",
  prose: "<p>Alice</p>",
  position: 0,
};
function editor() {
  return (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
}
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue([]);
  currentProject.setProject(mockProject);
  currentProject.setCurrentChapter(mockChapters[0]);
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.useRealTimers();
  vi.restoreAllMocks();
});

it("flushes the newest beat draft after an in-flight save without restoring older editor text", async () => {
  currentProject.setBeats([beat]);
  ui.setExpandedBeat(beat.id);
  const { component } = render(BeatView, { beats: [beat] });
  await vi.advanceTimersByTimeAsync(0);
  let finish!: () => void;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      })
  );
  editor().commands.setContent("<p>First draft</p>");
  await vi.advanceTimersByTimeAsync(500);
  editor().commands.setContent("<p>Latest Alice</p>");
  const pending = component.flushForSearch();
  finish();
  await pending;
  await tick();
  expect(invoke).toHaveBeenNthCalledWith(1, "save_beat_prose", {
    beatId: beat.id,
    prose: "<p>First draft</p>",
  });
  expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
    beatId: beat.id,
    prose: "<p>Latest Alice</p>",
  });
  expect(currentProject.beats[0].prose).toBe("<p>Latest Alice</p>");
  expect(editor().getText()).toBe("Latest Alice");
});

it("blocks search on failed beat saves and retains the draft for retry", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.setExpandedBeat(beat.id);
  const { component } = render(BeatView, { beats: [beat] });
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Unsaved Alice</p>");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(component.flushForSearch()).rejects.toThrow("Could not save");
  vi.mocked(invoke).mockResolvedValue(undefined);
  await component.flushForSearch();
  expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
    beatId: beat.id,
    prose: "<p>Unsaved Alice</p>",
  });
});

it.each([false, true])(
  "keeps draft ownership across an in-place project switch (old save fails: %s)",
  async (oldSaveFails) => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    currentProject.setBeats([beat]);
    ui.setExpandedBeat(beat.id);
    const view = render(BeatView, { beats: [beat] });
    await vi.advanceTimersByTimeAsync(0);
    editor().commands.setContent("<p>Old project draft</p>");
    const nextProject = { ...mockProject, id: "project-B" };
    const nextBeat = { ...beat, id: "beat-B", scene_id: "scene-B" };
    currentProject.setProject(null);
    currentProject.setProject(nextProject);
    currentProject.setBeats([nextBeat]);
    await view.rerender({ beats: [nextBeat] });
    vi.mocked(invoke).mockImplementation(async () => {
      if (oldSaveFails) throw "Cannot edit beats in a locked scene";
    });
    await vi.advanceTimersByTimeAsync(500);
    expect(proseSaves.draftsForRecovery(mockProject.id)).toHaveLength(oldSaveFails ? 1 : 0);
    expect(proseSaves.draftsForRecovery(nextProject.id)).toEqual([]);

    ui.setExpandedBeat(nextBeat.id);
    await tick();
    await vi.advanceTimersByTimeAsync(0);
    editor().commands.setContent("<p>New project draft</p>");
    vi.mocked(invoke).mockRejectedValue("Cannot edit beats in a locked scene");
    await vi.advanceTimersByTimeAsync(500);
    const drafts = proseSaves.draftsForRecovery(nextProject.id);
    expect(drafts).toEqual([
      {
        projectId: nextProject.id,
        kind: "beat",
        id: nextBeat.id,
        prose: "<p>New project draft</p>",
      },
    ]);

    HTMLDialogElement.prototype.showModal = function () {
      this.setAttribute("open", "");
    };
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_beat_prose") throw "Cannot edit beats in a locked scene";
      return [
        {
          id: nextBeat.id,
          scene_id: nextBeat.scene_id,
          chapter_id: "chapter-B",
          chapter_title: "New chapter",
          scene_title: "New scene",
          beat_title: nextBeat.content,
          prose: nextBeat.prose,
          locked: false,
        },
      ];
    });
    const dialog = render(FindReplaceDialog, {
      projectId: nextProject.id,
      sceneId: nextBeat.scene_id,
      showReplace: true,
      prepare: () => view.component.flushForSearch(),
      onApplied: vi.fn(),
      onClose: vi.fn(),
      onDiscardDrafts: async (discarded) => {
        await proseSaves.discard(discarded);
        view.component.discardFailedDrafts(discarded);
      },
    });
    await vi.advanceTimersByTimeAsync(0);
    expect(
      (screen.getByLabelText("Draft text (select to copy)") as HTMLTextAreaElement).value
    ).toBe("New project draft\n");
    expect(screen.getByText("Retry saving drafts")).toBeTruthy();
    await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Alice" } });
    expect((screen.getByText("Replace match") as HTMLButtonElement).disabled).toBe(true);
    await fireEvent.click(screen.getByText("Discard unsaved drafts…"));
    await fireEvent.click(screen.getByText("Confirm discard drafts"));
    await vi.advanceTimersByTimeAsync(0);
    expect(proseSaves.draftsForRecovery(nextProject.id)).toEqual([]);
    dialog.unmount();
    await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  }
);

it.each(["beat", "page"] as const)(
  "applies a successful %s retry even when another pending draft fails",
  async (kind) => {
    const page = {
      ...mockScenes[0],
      editor_mode: "page" as const,
      planning_status: "fixed" as const,
      prose: "<p>Old page</p>",
    };
    currentProject.setScenes([page]);
    currentProject.setCurrentScene(page);
    currentProject.setBeats([beat]);
    const flush =
      kind === "beat"
        ? render(BeatView, { beats: [beat] }).component.flushForSearch
        : render(ScenePanel).component.prepareForSearch;
    await vi.advanceTimersByTimeAsync(0);
    const recovered = {
      projectId: mockProject.id,
      kind,
      id: kind === "beat" ? beat.id : page.id,
      prose: "<p>Successfully recovered prose</p>",
    };
    const failing = {
      projectId: mockProject.id,
      kind: "beat" as const,
      id: "still-failing",
      prose: "Unsaved",
    };
    vi.mocked(invoke).mockRejectedValue("Disk full");
    await expect(proseSaves.save(recovered)).rejects.toBe("Disk full");
    await expect(proseSaves.save(failing)).rejects.toBe("Disk full");
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    await expect(flush()).rejects.toThrow("Could not save");
    expect(
      kind === "beat" ? currentProject.beats[0].prose : currentProject.currentScene?.prose
    ).toBe(recovered.prose);
    if (kind === "page") {
      await tick();
      expect(editor().getText()).toBe("Successfully recovered prose");
    }
    expect(proseSaves.pendingFor(mockProject.id)).toEqual([failing]);
    vi.mocked(invoke).mockClear();
    await expect(flush()).rejects.toThrow("Could not save");
    expect(invoke).toHaveBeenCalledExactlyOnceWith("save_beat_prose", {
      beatId: failing.id,
      prose: failing.prose,
    });
    await proseSaves.discard([failing]);
  }
);

it("clears a locally failed beat draft after a partial recovery so later flushes do not resave it", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  currentProject.setBeats([beat]);
  ui.setExpandedBeat(beat.id);
  const { component } = render(BeatView, { beats: [beat] });
  await vi.advanceTimersByTimeAsync(0);
  vi.mocked(invoke).mockRejectedValue("Disk full");
  editor().commands.setContent("<p>Recovered local draft</p>");
  await vi.advanceTimersByTimeAsync(500);
  const failing = {
    projectId: mockProject.id,
    kind: "beat" as const,
    id: "still-failing",
    prose: "Unsaved",
  };
  await expect(proseSaves.save(failing)).rejects.toBe("Disk full");
  let attempts = 0;
  vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
    if (args && "beatId" in args && args.beatId === beat.id && ++attempts > 1) return;
    throw "Disk full";
  });
  await expect(component.flushForSearch()).rejects.toThrow("Could not save");
  expect(currentProject.beats[0].prose).toBe("<p>Recovered local draft</p>");
  expect(attempts).toBe(2);
  await expect(component.flushForSearch()).rejects.toThrow("Could not save");
  expect(attempts).toBe(2);
  await proseSaves.discard([failing]);
});

it("flushes page prose before searching and refreshes the open editor after replacement", async () => {
  const scene = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Alice</p>",
  };
  currentProject.setCurrentScene(scene);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Unsaved Alice</p>");
  await component.prepareForSearch();
  expect(invoke).toHaveBeenCalledWith("save_scene_page_prose", {
    sceneId: scene.id,
    prose: "<p>Unsaved Alice</p>",
  });
  component.applySearchChanges([{ id: scene.id, prose: "<p>Unsaved Bob</p>" }]);
  await tick();
  expect(editor().getText()).toBe("Unsaved Bob");
  await vi.advanceTimersByTimeAsync(1000);
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_page_prose")
  ).toHaveLength(1);
});

it("retains a beat save failure after Escape clears local drafts, and retries before search", async () => {
  currentProject.setBeats([beat]);
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.setExpandedBeat(beat.id);
  const { component } = render(BeatView, { beats: [beat] });
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Escaped draft</p>");
  let fail!: (e: string) => void;
  vi.mocked(invoke).mockImplementationOnce(
    () => new Promise((_resolve, reject) => (fail = reject))
  );
  component.flushOnSceneChange();
  await vi.advanceTimersByTimeAsync(0);
  fail("Disk full");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(component.flushForSearch()).rejects.toThrow("Could not save");
  vi.mocked(invoke).mockResolvedValue(undefined);
  await component.flushForSearch();
  expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
    beatId: beat.id,
    prose: "<p>Escaped draft</p>",
  });
  expect(currentProject.beats[0].prose).toBe("<p>Escaped draft</p>");
});

it("preserves failed beat saves across editor teardown", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.setExpandedBeat(beat.id);
  const { component, unmount } = render(BeatView, { beats: [beat] });
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Departed draft</p>");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  component.flushOnSceneChange();
  unmount();
  const panel = render(ScenePanel).component;
  await expect(panel.prepareForSearch()).rejects.toThrow("Could not save");
  vi.mocked(invoke).mockResolvedValue(undefined);
  await panel.prepareForSearch();
  expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
    beatId: beat.id,
    prose: "<p>Departed draft</p>",
  });
});

it("recovers from a failed page save after changing scenes without losing the old draft", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const scene = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Alice</p>",
  };
  currentProject.setCurrentScene(scene);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Failed page draft</p>");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(component.prepareForSearch()).rejects.toThrow("Could not save");
  vi.mocked(invoke).mockResolvedValue([]);
  currentProject.setCurrentScene({ ...scene, id: "another-scene", prose: "<p>Other prose</p>" });
  await tick();
  await component.prepareForSearch();
  expect(invoke).toHaveBeenCalledWith("save_scene_page_prose", {
    sceneId: scene.id,
    prose: "<p>Failed page draft</p>",
  });
  expect(editor().getText()).toBe("Other prose");
});

it("reveals a chapter and its collapsed Part after navigation from a search result", async () => {
  const first = {
    ...mockChapters[0],
    id: "first",
    title: "First chapter",
    is_part: false,
    planning_status: "fixed" as const,
  };
  const part = { ...first, id: "part", title: "Part Two", is_part: true, position: 1 };
  const target = { ...first, id: "target", title: "Target chapter", position: 2 };
  const scene = { ...mockScenes[0], chapter_id: target.id, title: "Search destination" };
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_chapters" ? [first, part, target] : []
  );
  render(Sidebar);
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Part Two" }));
  expect(screen.queryByRole("button", { name: "Target chapter" })).toBeNull();
  currentProject.setCurrentChapter(target);
  currentProject.setScenes([scene]);
  currentProject.setCurrentScene(scene);
  await tick();
  expect(screen.getByText("Search destination")).toBeTruthy();
  // Manual collapse remains possible; expansion responds only to navigation.
  await fireEvent.click(screen.getByRole("button", { name: "Target chapter" }));
  expect(screen.queryByText("Search destination")).toBeNull();
});

it("saves a pending page draft to its original scene when selection changes", async () => {
  const scene = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Alice</p>",
  };
  currentProject.setCurrentScene(scene);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Before navigation</p>");
  currentProject.setCurrentScene({ ...scene, id: "destination", prose: "<p>Destination</p>" });
  await tick();
  await component.prepareForSearch();
  expect(invoke).toHaveBeenCalledWith("save_scene_page_prose", {
    sceneId: scene.id,
    prose: "<p>Before navigation</p>",
  });
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_page_prose")
  ).toHaveLength(1);
  expect(editor().getText()).toBe("Destination");
});

it.each([0, 500])(
  "waits for page prose before switching to beats after %i ms of debounce",
  async (delay) => {
    const scene = {
      ...mockScenes[0],
      editor_mode: "page" as const,
      planning_status: "fixed" as const,
      prose: "<p>Original</p>",
    };
    currentProject.setCurrentScene(scene);
    let persisted = scene.prose;
    let finish!: () => void;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "save_scene_page_prose")
        return new Promise<void>((resolve) => {
          finish = () => {
            persisted = (args as { prose: string }).prose;
            resolve();
          };
        });
      if (cmd === "switch_scene_editor_mode")
        return { ...scene, editor_mode: "beat", prose: persisted };
      if (cmd === "get_beats") return [{ ...beat, prose: persisted }];
      return [];
    });
    render(ScenePanel);
    await vi.advanceTimersByTimeAsync(0);
    editor().commands.setContent("<p>Last edits before switching</p>");
    await vi.advanceTimersByTimeAsync(delay);
    window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Switch" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "switch_scene_editor_mode")).toBe(
      false
    );
    expect(editor().isEditable).toBe(false);
    finish();
    await vi.advanceTimersByTimeAsync(0);
    expect(currentProject.currentScene?.editor_mode).toBe("beat");
    expect(currentProject.beats[0].prose).toBe("<p>Last edits before switching</p>");
    expect(
      vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_page_prose")
    ).toHaveLength(1);
  }
);

it("refuses a mode switch after save failure and succeeds after retry", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const showError = vi.spyOn(ui, "showError");
  const scene = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Original</p>",
  };
  currentProject.setCurrentScene(scene);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Do not lose this</p>");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Switch" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(showError).toHaveBeenCalledWith(expect.stringContaining("Disk full"));
  expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "switch_scene_editor_mode")).toBe(
    false
  );
  expect(currentProject.currentScene?.editor_mode).toBe("page");
  expect(editor().isEditable).toBe(true);
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "switch_scene_editor_mode" ? { ...scene, editor_mode: "beat" } : []
  );
  await component.prepareForSearch();
  window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Switch" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(currentProject.currentScene?.editor_mode).toBe("beat");
});

it("waits for an in-flight beat save before switching to page mode", async () => {
  const scene = {
    ...mockScenes[0],
    editor_mode: "beat" as const,
    planning_status: "fixed" as const,
  };
  currentProject.setCurrentScene(scene);
  currentProject.setBeats([beat]);
  ui.setExpandedBeat(beat.id);
  let finish!: () => void;
  let persisted = beat.prose;
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "save_beat_prose")
      return new Promise<void>((resolve) => {
        finish = () => {
          persisted = (args as { prose: string }).prose;
          resolve();
        };
      });
    if (cmd === "switch_scene_editor_mode")
      return { ...scene, editor_mode: "page", prose: persisted };
    return [];
  });
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Newest beat draft</p>");
  await vi.advanceTimersByTimeAsync(500);
  window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
  await vi.advanceTimersByTimeAsync(0);
  expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "switch_scene_editor_mode")).toBe(
    false
  );
  finish();
  await vi.advanceTimersByTimeAsync(0);
  expect(currentProject.currentScene?.editor_mode).toBe("page");
  expect(editor().getText()).toBe("Newest beat draft");
});

async function beatMutationMenu(action: "split" | "merge") {
  await fireEvent.click(screen.getAllByTestId("beat-menu-button")[0]);
  await fireEvent.click(
    screen.getByRole("menuitem", {
      name: action === "split" ? "Split at cursor" : "Merge with next",
    })
  );
}

it.each([
  { action: "split", inFlight: false },
  { action: "split", inFlight: true },
  { action: "merge", inFlight: false },
  { action: "merge", inFlight: true },
] as const)(
  "waits for deferred prose before $action (save in flight: $inFlight)",
  async ({ action, inFlight }) => {
    const scene = { ...mockScenes[0], id: beat.scene_id };
    const second = { ...beat, id: "second", position: 1, prose: "<p>Tail</p>" };
    currentProject.setCurrentScene(scene);
    currentProject.setBeats([beat, second]);
    ui.setExpandedBeat(beat.id);
    render(BeatView, { beats: [beat, second] });
    await vi.advanceTimersByTimeAsync(0);
    const latest = "<p>Newest first</p><p>Newest second</p>";
    editor().commands.setContent(latest);
    editor().commands.setTextSelection(15);
    let finish!: () => void;
    let persisted = beat.prose;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "save_beat_prose") {
        if (!finish) await new Promise<void>((resolve) => (finish = resolve));
        persisted = (args as { prose: string }).prose;
        return;
      }
      if (cmd === "split_beat" || cmd === "merge_beats") {
        expect(persisted).toBe(latest);
        persisted = "<p>Transformed prose</p>";
        return { ...beat, id: "split-result" };
      }
      return [{ ...beat, prose: persisted }, second];
    });
    if (inFlight) await vi.advanceTimersByTimeAsync(500);
    await beatMutationMenu(action);
    expect(
      vi
        .mocked(invoke)
        .mock.calls.map(([cmd]) => cmd)
        .filter((cmd) => cmd !== "save_session_state")
    ).toEqual(["save_beat_prose"]);
    expect(editor().isEditable).toBe(false);
    finish();
    await vi.advanceTimersByTimeAsync(0);
    expect(
      vi
        .mocked(invoke)
        .mock.calls.map(([cmd]) => cmd)
        .filter((cmd) => cmd !== "save_session_state")
    ).toEqual([
      "save_beat_prose",
      ...(inFlight ? ["save_beat_prose"] : []),
      action === "split" ? "split_beat" : "merge_beats",
      "get_beats",
    ]);
    expect(currentProject.beats[0].prose).toBe("<p>Transformed prose</p>");
    await vi.advanceTimersByTimeAsync(1000);
    expect(persisted).toBe("<p>Transformed prose</p>");
  }
);

it.each(["split", "merge"] as const)("aborts %s if the prose save fails", async (action) => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const second = { ...beat, id: "second", position: 1 };
  currentProject.setCurrentScene({ ...mockScenes[0], id: beat.scene_id });
  currentProject.setBeats([beat, second]);
  ui.setExpandedBeat(beat.id);
  render(BeatView, { beats: [beat, second] });
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>New first</p><p>New second</p>");
  editor().commands.setTextSelection(12);
  vi.mocked(invoke).mockRejectedValue("Cannot edit beats in a locked scene");
  await beatMutationMenu(action);
  await vi.advanceTimersByTimeAsync(0);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.map(([cmd]) => cmd)
      .filter((cmd) => cmd !== "save_session_state")
  ).toEqual(["save_beat_prose"]);
  expect(ui.toast?.message).toContain("Save the affected beats");
  expect(editor().isEditable).toBe(true);
  await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  ui.clearToast();
});

it.each(["page", "beat"] as const)(
  "preserves an unrelated open editor when discarding a %s draft",
  async (kind) => {
    currentProject.setBeats([beat]);
    ui.setExpandedBeat(beat.id);
    const { component } = render(BeatView, { beats: [beat] });
    await vi.advanceTimersByTimeAsync(0);
    const activeEditor = editor();
    component.discardFailedDrafts([
      { projectId: mockProject.id, id: "unrelated", kind, prose: "Discard me" },
    ]);
    await tick();
    expect(ui.expandedBeatId).toBe(beat.id);
    expect(editor()).toBe(activeEditor);
  }
);

it("clears local drafts atomically when the scene changes as queue discard completes", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const scene = {
    ...mockScenes[0],
    editor_mode: "beat" as const,
    planning_status: "fixed" as const,
  };
  const other = { ...scene, id: "other-scene" };
  currentProject.setCurrentScene(scene);
  currentProject.setBeats([beat]);
  ui.setExpandedBeat(beat.id);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Discard this draft</p>");
  vi.mocked(invoke).mockRejectedValue("Cannot edit beats in a locked scene");
  await component.prepareForSearch();
  const drafts = proseSaves.draftsForRecovery(mockProject.id);
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_scenes" ? [scene, other] : cmd === "get_beats" ? [beat] : []
  );
  const discard = proseSaves.discard.bind(proseSaves);
  vi.mocked(invoke).mockClear();
  vi.spyOn(proseSaves, "discard").mockImplementation(async (discarded, onDiscarded) => {
    await discard(discarded, onDiscarded);
    // Native navigation can happen between queue deletion and the caller's continuation.
    currentProject.setCurrentScene(other);
  });
  await component.discardFailedSaves(drafts);
  await vi.advanceTimersByTimeAsync(0);
  await component.prepareForSearch();
  expect(currentProject.currentScene?.id).toBe(other.id);
  expect(proseSaves.draftsForRecovery(mockProject.id)).toEqual([]);
  expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "save_beat_prose")).toBe(false);
});

it("can discard a locked beat draft explicitly without requeueing it from local editor state", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const scene = {
    ...mockScenes[0],
    editor_mode: "beat" as const,
    planning_status: "fixed" as const,
  };
  currentProject.setCurrentScene(scene);
  currentProject.setBeats([beat]);
  ui.setExpandedBeat(beat.id);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Draft before locking</p>");
  vi.mocked(invoke).mockRejectedValue("Cannot edit beats in a locked scene");
  await component.prepareForSearch();
  expect(proseSaves.pendingFor(mockProject.id)).toEqual([]);
  const pending = proseSaves.draftsForRecovery(mockProject.id);
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_scenes" ? [{ ...scene, locked: true }] : cmd === "get_beats" ? [beat] : []
  );
  await component.discardFailedSaves(pending);
  await component.prepareForSearch();
  expect(proseSaves.pendingFor(mockProject.id)).toEqual([]);
  expect(currentProject.beats[0].prose).toBe(beat.prose);
  const calls = vi.mocked(invoke).mock.calls.length;
  await vi.advanceTimersByTimeAsync(1000);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.slice(calls)
      .some(([cmd]) => cmd === "save_beat_prose")
  ).toBe(false);
});

it("keeps drafts if recovery reload fails, then restores the page editor after explicit discard", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const scene = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Saved page</p>",
  };
  currentProject.setCurrentScene(scene);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Unsaved page</p>");
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(component.prepareForSearch()).rejects.toThrow("Disk full");
  const pending = proseSaves.pendingFor(mockProject.id);
  await expect(component.discardFailedSaves(pending)).rejects.toBe("Disk full");
  expect(proseSaves.pendingFor(mockProject.id)).toEqual(pending);
  vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === "get_scenes" ? [scene] : []));
  await component.discardFailedSaves(pending);
  await component.prepareForSearch();
  await tick();
  expect(editor().getText()).toBe("Saved page");
  expect(proseSaves.pendingFor(mockProject.id)).toEqual([]);
});

it.each([false, true])(
  "preserves page edits across page A → beat B → page A (save pending: %s)",
  async (delayed) => {
    const page = {
      ...mockScenes[0],
      id: "page-A",
      editor_mode: "page" as const,
      planning_status: "fixed" as const,
      prose: "<p>Before</p>",
    };
    const other = { ...page, id: "beat-B", editor_mode: "beat" as const, prose: null };
    currentProject.setScenes([page, other]);
    currentProject.setCurrentScene(page);
    const { component } = render(ScenePanel);
    await vi.advanceTimersByTimeAsync(0);
    let finish!: () => void;
    if (delayed)
      vi.mocked(invoke).mockImplementationOnce(
        () => new Promise<void>((resolve) => (finish = resolve))
      );
    editor().commands.setContent("<p>Latest page draft</p>");
    await vi.advanceTimersByTimeAsync(500);
    expect(currentProject.scenes.find((s) => s.id === page.id)?.prose).toBe(
      "<p>Latest page draft</p>"
    );
    currentProject.setCurrentScene(other);
    await tick();
    currentProject.setCurrentScene(currentProject.scenes.find((s) => s.id === page.id)!);
    await tick();
    await vi.advanceTimersByTimeAsync(0);
    expect(editor().getText()).toBe("Latest page draft");
    editor().commands.setContent("<p>Latest page draft continued</p>");
    if (delayed) finish();
    await component.prepareForSearch();
    expect(editor().getText()).toBe("Latest page draft continued");
    expect(currentProject.currentScene?.prose).toBe("<p>Latest page draft continued</p>");
    expect(invoke).toHaveBeenLastCalledWith("save_scene_page_prose", {
      sceneId: page.id,
      prose: "<p>Latest page draft continued</p>",
    });
  }
);

it("retires the actual SQLite error when a page scene is deleted during its debounce", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  const page = {
    ...mockScenes[0],
    editor_mode: "page" as const,
    planning_status: "fixed" as const,
    prose: "<p>Saved</p>",
  };
  const other = { ...page, id: "other-page" };
  currentProject.setScenes([page, other]);
  currentProject.setCurrentScene(page);
  const { component } = render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  editor().commands.setContent("<p>Draft from deleted scene</p>");
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_page_prose") throw "Query returned no rows";
    if (cmd === "switch_scene_editor_mode") return { ...other, editor_mode: "beat" };
    return [];
  });
  currentProject.setScenes([other]);
  currentProject.setCurrentScene(other);
  await tick();
  await vi.advanceTimersByTimeAsync(500);
  await expect(component.prepareForSearch()).resolves.toBeUndefined();
  await expect(component.prepareForSearch()).resolves.toBeUndefined();
  window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Switch" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(currentProject.currentScene?.editor_mode).toBe("beat");
  expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_page_prose")).toEqual([
    ["save_scene_page_prose", { sceneId: page.id, prose: "<p>Draft from deleted scene</p>" }],
  ]);
  expect(proseSaves.pendingFor(mockProject.id)).toEqual([]);
  expect(proseSaves.draftsForRecovery(mockProject.id)[0]?.prose).toBe(
    "<p>Draft from deleted scene</p>"
  );
  await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
});

it.each(["Beat not found", "Cannot edit beats in a locked scene"])(
  "does not let terminal failure %s block another scene's mode switch",
  async (error) => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValue(error);
    await expect(
      proseSaves.save({
        projectId: mockProject.id,
        id: "removed-beat",
        kind: "beat",
        prose: "<p>Recover me</p>",
      })
    ).rejects.toBe(error);
    const scene = {
      ...mockScenes[0],
      editor_mode: "page" as const,
      planning_status: "fixed" as const,
      prose: "<p>Other scene</p>",
    };
    currentProject.setCurrentScene(scene);
    vi.mocked(invoke).mockImplementation(async (cmd) =>
      cmd === "switch_scene_editor_mode" ? { ...scene, editor_mode: "beat" } : []
    );
    const { component } = render(ScenePanel);
    await vi.advanceTimersByTimeAsync(0);
    await component.prepareForSearch();
    window.dispatchEvent(new CustomEvent("kindling:toggleEditorMode"));
    await tick();
    await fireEvent.click(screen.getByRole("button", { name: "Switch" }));
    await vi.advanceTimersByTimeAsync(0);
    expect(currentProject.currentScene?.editor_mode).toBe("beat");
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_beat_prose")).toHaveLength(
      1
    );
    expect(proseSaves.pendingFor(mockProject.id)).toEqual([]);
    expect(proseSaves.draftsForRecovery(mockProject.id)).toHaveLength(1);
    await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  }
);

it.each(["page", "beat"] as const)(
  "drains pending %s prose before restoring a snapshot",
  async (mode) => {
    const scene = {
      ...mockScenes[0],
      editor_mode: mode,
      planning_status: "fixed" as const,
      prose: "<p>Original</p>",
    };
    const sceneBeat = { ...beat, scene_id: scene.id };
    currentProject.setCurrentScene(scene);
    currentProject.setBeats([sceneBeat]);
    ui.setExpandedBeat(mode === "beat" ? beat.id : null);
    let disk = "Original";
    const operations: string[] = [];
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_beats") return [sceneBeat];
      if (command === "list_snapshots")
        return [
          {
            id: "snapshot",
            name: "Earlier",
            created_at: "2026-01-01T00:00:00Z",
            file_size: 100,
            trigger_type: "manual",
          },
        ];
      if (command === "save_scene_page_prose" || command === "save_beat_prose") {
        operations.push("save");
        disk = "Draft";
        return;
      }
      if (command === "restore_snapshot") {
        operations.push("restore");
        disk = "Snapshot";
        return mockProject;
      }
      return [];
    });
    const panel = render(ScenePanel).component;
    await vi.advanceTimersByTimeAsync(0);
    editor().commands.setContent("<p>Pending draft</p>");
    render(SnapshotsPanel, { onClose: vi.fn(), prepareRestore: () => panel.prepareForSearch() });
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByRole("button", { name: "Restore snapshot" }));
    await fireEvent.click(screen.getByRole("button", { name: "Restore" }));
    await vi.advanceTimersByTimeAsync(1000);
    expect(operations).toEqual(["save", "restore"]);
    expect(disk).toBe("Snapshot");
  }
);

it("does not restore when preparing pending drafts fails", async () => {
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === "list_snapshots"
      ? [
          {
            id: "snapshot",
            name: "Earlier",
            created_at: "2026-01-01T00:00:00Z",
            file_size: 100,
            trigger_type: "manual",
          },
        ]
      : []
  );
  render(SnapshotsPanel, {
    onClose: vi.fn(),
    prepareRestore: async () => {
      throw new Error("Could not save draft");
    },
  });
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Restore snapshot" }));
  await fireEvent.click(screen.getByRole("button", { name: "Restore" }));
  await tick();
  expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "restore_snapshot")).toBe(
    false
  );
  const alert = screen.getByRole("alert");
  expect(alert.textContent).toBe("Could not save draft");
  const dialogs = screen.getAllByRole("dialog");
  expect(dialogs[dialogs.length - 1].contains(alert)).toBe(true);
});
