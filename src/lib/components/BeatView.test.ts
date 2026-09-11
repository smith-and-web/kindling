import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import BeatView from "./BeatView.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";

vi.hoisted(() => {
  vi.stubGlobal("localStorage", { getItem: () => null, setItem: () => {}, removeItem: () => {} });
});

afterEach(() => {
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
