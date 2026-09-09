import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import Previously from "./Previously.svelte";
import ScenePanel from "./ScenePanel.svelte";
import { session } from "../stores/session.svelte";
import type { SceneReview } from "../utils/revisions";
import { currentProject } from "../stores/project.svelte";
import { synopsisSaves } from "../stores/synopsisSaves.svelte";
import { mockProject, mockChapters, mockScenes } from "../../dev/mock-data";
import type { Scene } from "../types";
vi.hoisted(() => {
  const data = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => data.set(key, value),
    removeItem: (key: string) => data.delete(key),
  });
});
const chapter = mockChapters[0];
const previous: Scene = {
  ...mockScenes[0],
  id: "previous",
  chapter_id: chapter.id,
  title: "A prior scene",
  synopsis: "The bridge collapsed.",
  prose: "<p>One. Two. Three. Four.</p>",
  position: 0,
  editor_mode: "page",
};
const current = { ...previous, id: "current", title: "Current scene", position: 1 };
beforeEach(() => {
  localStorage.removeItem("kindling:previouslyCollapsed");
  currentProject.setProject(mockProject);
  currentProject.setChapters([chapter]);
  currentProject.setScenes([previous, current]);
  currentProject.setCurrentScene(current);
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_scenes" ? currentProject.scenes : []
  );
});
afterEach(async () => {
  cleanup();
  await synopsisSaves.discardAll(synopsisSaves.snapshot());
  currentProject.setProject(null);
  vi.restoreAllMocks();
});
it("shows title, synopsis and closing prose in an accessible disclosure", async () => {
  render(Previously);
  const button = await screen.findByRole("button", { name: "Previously" });
  expect(button.getAttribute("aria-expanded")).toBe("true");
  expect(screen.getByRole("heading", { name: previous.title })).toBeTruthy();
  expect(screen.getByText(previous.synopsis!)).toBeTruthy();
  expect(screen.getByText("Two. Three. Four.").tagName).toBe("BLOCKQUOTE");
});
it("remembers collapse across navigation and remount, and allows expansion", async () => {
  const view = render(Previously);
  await fireEvent.click(await screen.findByRole("button", { name: "Previously" }));
  expect(screen.queryByText(previous.title)).toBeNull();
  expect(localStorage.getItem("kindling:previouslyCollapsed")).toBe("true");
  currentProject.setCurrentScene(previous);
  await tick();
  expect(screen.queryByRole("button", { name: "Previously" })).toBeNull();
  currentProject.setCurrentScene(current);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Previously" }).getAttribute("aria-expanded")).toBe(
      "false"
    )
  );
  view.unmount();
  render(Previously);
  await fireEvent.click(await screen.findByRole("button", { name: "Previously" }));
  expect(screen.getByText(previous.title)).toBeTruthy();
  expect(localStorage.getItem("kindling:previouslyCollapsed")).toBe("false");
});
it("hides for the first scene and when nothing is selected", async () => {
  currentProject.setCurrentScene(previous);
  render(Previously);
  await tick();
  expect(screen.queryByRole("region")).toBeNull();
  currentProject.setCurrentScene(null);
  await tick();
  expect(screen.queryByRole("region")).toBeNull();
});
it("keeps title-only summaries and renders synopsis as text", async () => {
  currentProject.setScenes([{ ...previous, synopsis: null, prose: null }, current]);
  render(Previously);
  await screen.findByText(previous.title);
  expect(document.querySelector("blockquote")).toBeNull();
  currentProject.updateSceneSynopsis(previous.id, "<img src=x onerror=alert(1)>");
  await screen.findByText("<img src=x onerror=alert(1)>");
  expect(document.querySelector("img")).toBeNull();
});
it("reflects pending synopsis drafts and title updates", async () => {
  render(Previously);
  await screen.findByText(previous.title);
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: previous.id,
    synopsis: "Newest synopsis",
  });
  currentProject.updateScene(previous.id, { title: "Renamed scene" });
  await screen.findByText("Newest synopsis");
  expect(screen.getByText("Renamed scene")).toBeTruthy();
});
it("recomputes after scene reordering", async () => {
  render(Previously);
  await screen.findByText(previous.title);
  currentProject.reorderScenes([current.id, previous.id]);
  await waitFor(() => expect(screen.queryByRole("region")).toBeNull());
});
it("ignores stale responses on rapid navigation, including failures", async () => {
  let reject!: (error: Error) => void;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((_resolve, fail) => {
        reject = fail;
      })
  );
  render(Previously);
  await waitFor(() => expect(reject).toBeTypeOf("function"));
  currentProject.setCurrentScene(previous);
  await tick();
  reject(new Error("Late failure"));
  await tick();
  expect(screen.queryByRole("region")).toBeNull();
  expect(screen.queryByRole("status")).toBeNull();
});
it("discards a successful response belonging to the old project", async () => {
  let resolve!: (scenes: Scene[]) => void;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      })
  );
  render(Previously);
  await waitFor(() => expect(resolve).toBeTypeOf("function"));
  currentProject.setProject(null);
  await tick();
  resolve([previous, current]);
  await tick();
  expect(screen.queryByRole("region")).toBeNull();
});
it("offers a retry after a read failure without showing stale context", async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error("Read failed"));
  render(Previously);
  await screen.findByRole("status");
  expect(screen.queryByRole("region")).toBeNull();
  await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await screen.findByText(previous.title);
  expect(screen.queryByRole("status")).toBeNull();
});

it.each(["Newest cross-chapter synopsis", null])(
  "retains a pending synopsis (%s) after saving across chapters",
  async (synopsis) => {
    const nextChapter = { ...chapter, id: "next-chapter", position: chapter.position + 1 };
    const selected = { ...current, chapter_id: nextChapter.id, position: 0 };
    currentProject.setChapters([chapter, nextChapter]);
    currentProject.setScenes([selected]);
    currentProject.setCurrentScene(selected);
    let releaseSave!: () => void;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "get_scenes")
        return (args as { chapterId: string }).chapterId === chapter.id ? [previous] : [selected];
      if (cmd === "save_scene_synopsis")
        await new Promise<void>((done) => {
          releaseSave = done;
        });
      return [];
    });
    synopsisSaves.stage({ projectId: mockProject.id, sceneId: previous.id, synopsis });
    const saving = synopsisSaves.flush(mockProject.id, previous.id);
    render(Previously);
    await screen.findByText(previous.title);
    expect(screen.queryByText(previous.synopsis!)).toBeNull();
    if (synopsis) expect(screen.getByText(synopsis)).toBeTruthy();
    releaseSave();
    await saving;
    await tick();
    expect(screen.queryByText(previous.synopsis!)).toBeNull();
    if (synopsis) expect(screen.getByText(synopsis)).toBeTruthy();
  }
);

it("reloads changed prose on explicit refresh without navigation", async () => {
  const view = render(Previously);
  await screen.findByText("Two. Three. Four.");
  vi.mocked(invoke).mockResolvedValue([{ ...previous, prose: "Replacement ending." }, current]);
  await view.rerender({ refreshVersion: 1 });
  await screen.findByText("Replacement ending.");
  expect(screen.queryByText("Two. Three. Four.")).toBeNull();
});

it.each(["replacement", "discard", "editorial"])(
  "refreshes predecessor prose when ScenePanel applies %s",
  async (operation) => {
    const view = render(ScenePanel);
    await screen.findByText("Two. Three. Four.");
    vi.mocked(invoke).mockImplementation(async (cmd) =>
      cmd === "get_scenes" ? [{ ...previous, prose: "Refreshed ending." }, current] : []
    );
    if (operation === "replacement")
      view.component.applySearchChanges([{ id: previous.id, prose: "Refreshed ending." }]);
    else if (operation === "discard") await view.component.discardFailedSaves([]);
    else
      view.component.applyRevision({
        scene_id: current.id,
        mode: "page",
        documents: [{ id: current.id, html: current.prose }],
        data: {},
      } as SceneReview);
    await screen.findByText("Refreshed ending.");
    expect(screen.queryByText("Two. Three. Four.")).toBeNull();
  }
);

it.each(["success", "failure"])(
  "waits for previous context %s before restoring the saved viewport",
  async (outcome) => {
    let finish!: () => void;
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_scenes") {
        await new Promise<void>((done) => {
          finish = done;
        });
        if (outcome === "failure") throw new Error("Context read failed");
        return [previous, current];
      }
      return [];
    });
    session.restoreViewport(mockProject.id, current.id, 250);
    const view = render(ScenePanel);
    const scroll = view.getByTestId("scene-panel").firstElementChild as HTMLElement;
    await waitFor(() => expect(finish).toBeTypeOf("function"));
    // Allow the one-shot restore's animation frame to run if it was scheduled early.
    await new Promise<void>((done) => requestAnimationFrame(() => done()));
    expect(scroll.scrollTop).toBe(0);
    finish();
    await waitFor(() => expect(scroll.scrollTop).toBe(250));
    if (outcome === "success") expect(screen.getByText("Two. Three. Four.")).toBeTruthy();
    else expect(screen.getByText(/Could not load previous scene context/)).toBeTruthy();
  }
);
