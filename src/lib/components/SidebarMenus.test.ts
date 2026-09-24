import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import { currentProject } from "../stores/project.svelte";
import type { Chapter, Scene } from "../types";
import { mockProject } from "../../dev/mock-data";

// M27: the sidebar's menus work from the keyboard, and every scene row has a menu button.

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

function chapter(overrides: Partial<Chapter> = {}): Chapter {
  return {
    id: "chapter",
    project_id: mockProject.id,
    title: "Arrival",
    position: 0,
    source_id: null,
    archived: false,
    locked: false,
    is_part: false,
    synopsis: null,
    planning_status: "fixed",
    ...overrides,
  };
}
const scene: Scene = {
  id: "scene",
  chapter_id: "chapter",
  title: "The Gate",
  synopsis: null,
  prose: null,
  position: 0,
  source_id: null,
  archived: false,
  locked: false,
  scene_type: "normal",
  scene_status: "draft",
  planning_status: "fixed",
  editor_mode: "beat",
};

function mockOutline(chapters: Chapter[]) {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_chapters") return chapters;
    if (command === "get_scenes") return [scene];
    return [];
  });
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
});

const focused = () => document.activeElement?.textContent?.trim();

it.each([
  ["More actions", "Export", ["Snapshots", "Archive"]],
  ["More options", "New chapter", ["New part", "New part"]],
])(
  "the %s menu takes focus, moves with the arrow keys and closes on Escape",
  async (name, first, next) => {
    mockOutline([chapter()]);
    render(Sidebar);
    const trigger = await screen.findByRole("button", { name });
    await fireEvent.click(trigger);
    await tick();
    expect(focused()).toBe(first);
    await fireEvent.keyDown(document.activeElement!, { key: "ArrowDown" });
    expect(focused()).toBe(next[0]);
    await fireEvent.keyDown(document.activeElement!, { key: "End" });
    expect(focused()).toBe(next[1]);
    await fireEvent.keyDown(document.activeElement!, { key: "Home" });
    expect(focused()).toBe(first);
    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
    await tick();
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("false");

    // Tab also closes the menu rather than leaving it open behind the focus.
    await fireEvent.click(trigger);
    await tick();
    await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
    await tick();
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  }
);

it("the scene filter popover takes focus and closes on Escape", async () => {
  mockOutline([chapter()]);
  render(Sidebar);
  const trigger = await screen.findByRole("button", { name: "Filter by type & status" });
  await fireEvent.click(trigger);
  await tick();
  const popover = screen.getByRole("dialog", { name: "Scene filters" });
  expect(popover.contains(document.activeElement)).toBe(true);
  await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
  await tick();
  expect(screen.queryByRole("dialog", { name: "Scene filters" })).toBeNull();
  expect(document.activeElement).toBe(trigger);
});

it("scenes in a flexible chapter have a menu button", async () => {
  mockOutline([chapter({ planning_status: "flexible" })]);
  render(Sidebar);
  await screen.findByRole("button", { name: scene.title });
  await fireEvent.click(screen.getByRole("button", { name: "Scene menu" }));
  expect(await screen.findByRole("menuitem", { name: "Rename" })).toBeTruthy();
});

it("disables edits for an unlocked scene inside a locked chapter but still offers Lock", async () => {
  mockOutline([chapter({ locked: true })]);
  render(Sidebar);
  await waitFor(() =>
    expect(screen.getAllByRole("button", { name: "Scene menu" })).toHaveLength(1)
  );
  await fireEvent.click(screen.getByRole("button", { name: "Scene menu" }));
  const item = (label: string) =>
    screen.getByRole("menuitem", { name: label }) as HTMLButtonElement;
  for (const label of ["Rename", "Archive", "Delete"]) expect(item(label).disabled).toBe(true);
  expect(item("Planning").getAttribute("aria-disabled")).toBe("true");
  // The toggle reflects the scene's own lock, which the writer can still set.
  expect(item("Lock").disabled).toBe(false);
  expect(item("Duplicate").disabled).toBe(false);
});
