import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import BeatView from "./BeatView.svelte";
import ReferencesPanel from "./ReferencesPanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import type { Beat, Chapter, ReferenceItem, Scene, SceneReferenceState } from "../types";
import { mockProject } from "../../dev/mock-data";

// M21: chapters, scenes, beats and references can be reordered without a mouse.

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

function chapter(id: string, title: string, position: number): Chapter {
  return {
    id,
    project_id: mockProject.id,
    title,
    position,
    source_id: null,
    archived: false,
    locked: false,
    is_part: false,
    synopsis: null,
    planning_status: "fixed",
  };
}
function scene(id: string, title: string, position: number, locked = false): Scene {
  return {
    id,
    chapter_id: "one",
    title,
    synopsis: null,
    prose: null,
    position,
    source_id: null,
    archived: false,
    locked,
    scene_type: "normal",
    scene_status: "draft",
    planning_status: "fixed",
    editor_mode: "beat",
  };
}
const chapters = [chapter("one", "Arrival", 0), chapter("two", "Departure", 1)];

let responses: Record<string, unknown> = {};
let failing: string | null = null;
beforeEach(() => {
  HTMLElement.prototype.scrollIntoView = vi.fn();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === failing) throw "Cannot move the locked scene “Gate”. Unlock it first.";
    return command in responses ? responses[command] : [];
  });
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.clearToast();
  currentProject.setProject(mockProject);
});
afterEach(() => {
  responses = {};
  failing = null;
  cleanup();
  currentProject.setProject(null);
  ui.clearToast();
  vi.restoreAllMocks();
});

const status = () => screen.getAllByRole("status").map((el) => el.textContent?.trim());
const menuItem = (name: string) => screen.getByRole("menuitem", { name }) as HTMLButtonElement;

it("moves a chapter up from its menu, announces it and keeps focus on it", async () => {
  responses = { get_chapters: chapters };
  render(Sidebar);
  const [first, second] = await screen.findAllByRole("button", { name: /menu$/ });
  await fireEvent.click(first);
  expect(menuItem("Move up").disabled).toBe(true);
  await fireEvent.keyDown(window, { key: "Escape" });
  await fireEvent.click(second);
  await fireEvent.click(menuItem("Move up"));
  await waitFor(() => expect(currentProject.chapters.map((c) => c.id)).toEqual(["two", "one"]));
  expect(invoke).toHaveBeenCalledWith("reorder_chapters", {
    projectId: mockProject.id,
    chapterIds: ["two", "one"],
  });
  await waitFor(() => expect(status()).toContain("Moved “Departure” up, to position 1 of 2."));
  await waitFor(() =>
    expect(
      document.activeElement?.closest("[data-outline-row]")?.getAttribute("data-outline-row")
    ).toBe("two")
  );
});

it("moves a scene down, shows the backend's lock refusal and leaves locked scenes pinned", async () => {
  const scenes = [scene("a", "Gate", 0), scene("b", "Well", 1), scene("c", "Tower", 2, true)];
  responses = { get_chapters: chapters, get_scenes: scenes };
  render(Sidebar);
  await screen.findByRole("button", { name: /Tower/ });

  const sceneMenus = () => screen.getAllByRole("button", { name: "Scene menu" });
  await fireEvent.click(sceneMenus()[0]);
  await fireEvent.click(menuItem("Move down"));
  await waitFor(() => expect(currentProject.scenes.map((s) => s.id)).toEqual(["b", "a", "c"]));
  expect(invoke).toHaveBeenCalledWith("reorder_scenes", {
    chapterId: "one",
    sceneIds: ["b", "a", "c"],
  });
  await waitFor(() => expect(status()).toContain("Moved “Gate” down, to position 2 of 3."));

  // The locked scene itself can't be moved from its menu.
  await fireEvent.click(sceneMenus()[2]);
  expect(menuItem("Move up").disabled).toBe(true);
  expect(menuItem("Move down").disabled).toBe(true);
  await fireEvent.keyDown(window, { key: "Escape" });

  // Moving a neighbour past it would shift the pinned scene, so that move isn't offered.
  await fireEvent.click(sceneMenus()[1]);
  expect(menuItem("Move down").disabled).toBe(true);
  expect(menuItem("Move up").disabled).toBe(false);

  // A move the backend still refuses is reported, and the order is left alone.
  failing = "reorder_scenes";
  await fireEvent.click(menuItem("Move up"));
  await waitFor(() =>
    expect(ui.toast?.message).toBe(
      "Failed to reorder: Cannot move the locked scene “Gate”. Unlock it first."
    )
  );
  expect(currentProject.scenes.map((s) => s.id)).toEqual(["b", "a", "c"]);
});

it.each([
  ["scenes", ["a", "l", "c"]],
  ["chapters", ["one", "mid", "two"]],
])("doesn't offer a move that would shift a locked neighbour (%s)", async (kind, ids) => {
  if (kind === "scenes") {
    const scenes = [scene("a", "Gate", 0), scene("l", "Keep", 1, true), scene("c", "Tower", 2)];
    responses = { get_chapters: chapters, get_scenes: scenes };
  } else {
    const mid = { ...chapter("mid", "Interlude", 1), locked: true };
    responses = { get_chapters: [chapters[0], mid, { ...chapters[1], position: 2 }] };
  }
  render(Sidebar);
  const label = kind === "scenes" ? "Scene menu" : "Chapter menu";
  await waitFor(() => expect(screen.getAllByRole("button", { name: label })).toHaveLength(3));
  const menus = () => screen.getAllByRole("button", { name: label });
  await fireEvent.click(menus()[0]);
  expect(menuItem("Move down").disabled).toBe(true);
  await fireEvent.keyDown(window, { key: "Escape" });
  await fireEvent.click(menus()[2]);
  expect(menuItem("Move up").disabled).toBe(true);
  await fireEvent.keyDown(window, { key: "Escape" });
  const order = kind === "scenes" ? currentProject.scenes : currentProject.chapters;
  expect(order.map((item) => item.id)).toEqual(ids);
  expect(invoke).not.toHaveBeenCalledWith(
    kind === "scenes" ? "reorder_scenes" : "reorder_chapters",
    expect.anything()
  );
});

it("moves a scene past the scenes a filter hides, relative to the visible order", async () => {
  const notes: Scene = { ...scene("n", "Aside", 1), scene_type: "notes" };
  const scenes = [scene("a", "Gate", 0), notes, scene("b", "Well", 2)];
  responses = { get_chapters: chapters, get_scenes: scenes };
  render(Sidebar);
  await screen.findByRole("button", { name: /Aside/ });
  await fireEvent.click(screen.getByRole("button", { name: "Filter by type & status" }));
  await fireEvent.click(screen.getByRole("button", { name: "Notes" }));
  await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("button", { name: /Aside/ })).toBeNull());

  await fireEvent.click(screen.getAllByRole("button", { name: "Scene menu" })[0]);
  await fireEvent.click(menuItem("Move down"));
  // Gate lands after Well, the next scene the writer can see; the hidden note keeps its place.
  await waitFor(() => expect(currentProject.scenes.map((s) => s.id)).toEqual(["n", "b", "a"]));
  expect(invoke).toHaveBeenCalledWith("reorder_scenes", {
    chapterId: "one",
    sceneIds: ["n", "b", "a"],
  });
  await waitFor(() => expect(status()).toContain("Moved “Gate” down, to position 2 of 2."));
  expect(screen.getAllByTestId("scene-title").map((title) => title.textContent)).toEqual([
    "Well",
    "Gate",
  ]);
});

it("moves a beat from its menu, announces it and keeps focus on the beat", async () => {
  const beats: Beat[] = ["Knock", "Answer"].map((content, position) => ({
    id: `beat-${position}`,
    scene_id: "a",
    content,
    prose: null,
    position,
  }));
  currentProject.setCurrentScene(scene("a", "Gate", 0));
  currentProject.setBeats(beats);
  render(BeatView, { beats });
  const [firstMenu] = screen.getAllByRole("button", { name: "Beat menu" });
  await fireEvent.click(firstMenu);
  expect(menuItem("Move up").disabled).toBe(true);
  await fireEvent.click(menuItem("Move down"));
  await waitFor(() => expect(currentProject.beats.map((b) => b.id)).toEqual(["beat-1", "beat-0"]));
  expect(invoke).toHaveBeenCalledWith("reorder_beats", {
    sceneId: "a",
    beatIds: ["beat-1", "beat-0"],
  });
  await waitFor(() => expect(status()).toContain("Moved beat “Knock” down, to position 2 of 2."));
  expect(document.activeElement).toBe(firstMenu);
});

it("opens the beat menu under its button when opened from the keyboard", async () => {
  const beats: Beat[] = [{ id: "b", scene_id: "a", content: "Knock", prose: null, position: 0 }];
  render(BeatView, { beats });
  const button = screen.getByRole("button", { name: "Beat menu" });
  button.getBoundingClientRect = () => ({ left: 120, bottom: 64 }) as DOMRect;
  await fireEvent.click(button, { clientX: 0, clientY: 0 });
  const menu = screen.getByTestId("context-menu");
  expect(menu.style.left).toBe("120px");
  expect(menu.style.top).toBe("64px");
});

function reference(id: string, name: string): ReferenceItem {
  return {
    id,
    project_id: mockProject.id,
    name,
    reference_type: "characters",
    description: null,
    attributes: {},
    source_id: null,
  };
}
const references = [reference("ada", "Ada"), reference("bram", "Bram")];

it("moves a reference up in the project list and announces it", async () => {
  currentProject.setCurrentScene(null);
  ui.referencesPanelCollapsed = false;
  responses = { get_references: references };
  render(ReferencesPanel);
  await fireEvent.click(await screen.findByRole("button", { name: "Bram" }));
  expect(
    (screen.getByRole("button", { name: "Move Bram down" }) as HTMLButtonElement).disabled
  ).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Move Bram up" }));
  await waitFor(() =>
    expect(
      screen.getAllByRole("button", { name: /^(Ada|Bram)$/ }).map((b) => b.textContent?.trim())
    ).toEqual(["Bram", "Ada"])
  );
  expect(status()).toContain("Moved “Bram” up, to position 1 of 2.");
});

it("moves a reference linked to the scene and saves the scene's order", async () => {
  currentProject.setCurrentScene(scene("a", "Gate", 0));
  ui.referencesPanelCollapsed = false;
  const states: SceneReferenceState[] = references.map((r, position) => ({
    scene_id: "a",
    reference_type: "characters",
    reference_id: r.id,
    position,
    expanded: false,
  }));
  responses = { get_references: references, get_scene_reference_state: states };
  render(ReferencesPanel);
  await fireEvent.click(await screen.findByRole("button", { name: "Ada" }));
  await fireEvent.click(screen.getByRole("button", { name: "Move Ada down" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("save_scene_reference_state", {
      sceneId: "a",
      referenceType: "characters",
      states: [
        { reference_id: "bram", position: 0, expanded: false },
        { reference_id: "ada", position: 1, expanded: true },
      ],
    })
  );
  await waitFor(() => expect(status()).toContain("Moved “Ada” down, to position 2 of 2."));
});
