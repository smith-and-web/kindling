import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import type { Editor } from "@tiptap/core";
import { invoke } from "@tauri-apps/api/core";
import BeatView from "./BeatView.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { proseSaves } from "../utils/proseSaves";
import { mockProject } from "../../dev/mock-data";

vi.hoisted(() => {
  vi.stubGlobal("localStorage", { getItem: () => null, setItem: () => {}, removeItem: () => {} });
});

afterEach(async () => {
  vi.useRealTimers();
  await proseSaves.discard(proseSaves.draftsForRecovery());
  cleanup();
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.restoreAllMocks();
});

it("updates the beat title and scroll target on scene switch without needing a hover", async () => {
  const first = {
    id: "first-beat",
    scene_id: "first-scene",
    content: "First scene's beat",
    prose: null,
    position: 0,
  };
  const second = {
    ...first,
    id: "second-beat",
    scene_id: "second-scene",
    content: "Second scene's beat",
  };
  const view = render(BeatView, { beats: [first] });
  await view.rerender({ beats: [second] });
  expect(screen.queryByText(first.content)).toBeNull();
  expect(screen.getByTestId("beat-header").textContent).toContain(second.content);
  const row = screen.getByTestId("beat-item");
  const scroll = vi.fn();
  row.scrollIntoView = scroll;
  await fireEvent.click(screen.getByTestId("beat-header"));
  expect(scroll).toHaveBeenCalledWith({ behavior: "smooth", block: "start" });
  expect(ui.expandedBeatId).toBe(second.id);
});

it.each([0, 1])("explains whether deleting beat %s discards or merges its prose", async (index) => {
  const beats = [0, 1].map((position) => ({
    id: `beat-${position}`,
    scene_id: "scene",
    content: `Beat ${position}`,
    prose: "<p>Draft</p>",
    position,
  }));
  render(BeatView, { beats });
  await fireEvent.contextMenu(screen.getAllByTestId("beat-header")[index]);
  await fireEvent.click(screen.getByText("Delete", { exact: true }));
  expect(
    screen.getByText(
      index === 0
        ? /first beat.*permanently deleted/
        : /prose will be merged into the previous beat/
    )
  ).toBeTruthy();
});

it.each(["collapsing the beat", "leaving the scene"])(
  "reopens a beat from its failed-save draft after %s, and the next keystroke keeps it",
  async (how) => {
    vi.useFakeTimers();
    vi.spyOn(console, "error").mockImplementation(() => {});
    const beat = {
      id: "beat",
      scene_id: "scene",
      content: "Greeting",
      prose: "<p>Saved</p>",
      position: 0,
    };
    const editor = () =>
      (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
    currentProject.setProject(mockProject);
    currentProject.setBeats([beat]);
    ui.setExpandedBeat(beat.id);
    const { component } = render(BeatView, { beats: [beat] });
    await vi.advanceTimersByTimeAsync(0);
    vi.mocked(invoke).mockRejectedValue("Disk full");
    editor().commands.setContent("<p>Unsaved draft</p>");
    await vi.advanceTimersByTimeAsync(500);
    expect(proseSaves.draftsForRecovery(mockProject.id)).toHaveLength(1);

    if (how === "leaving the scene") component.flushOnSceneChange();
    else await fireEvent.click(screen.getByTestId("beat-header"));
    await tick();
    expect(document.querySelector(".tiptap")).toBeNull();
    expect(screen.getByTestId("beat-item").textContent).toContain("Unsaved draft");
    screen.getByTestId("beat-item").scrollIntoView = vi.fn();

    await fireEvent.click(screen.getByTestId("beat-header"));
    await vi.advanceTimersByTimeAsync(0);
    expect(editor().getText()).toBe("Unsaved draft");

    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue(undefined);
    editor().commands.insertContentAt(editor().state.doc.content.size - 1, " and more");
    await vi.advanceTimersByTimeAsync(500);
    expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
      beatId: beat.id,
      prose: "<p>Unsaved draft and more</p>",
    });
    expect(proseSaves.draftsForRecovery(mockProject.id)).toEqual([]);
  }
);
