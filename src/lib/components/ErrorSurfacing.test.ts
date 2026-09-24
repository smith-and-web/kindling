import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import BeatView from "./BeatView.svelte";
import ScenePanel from "./ScenePanel.svelte";
import TagSelector from "./TagSelector.svelte";
import TemplateBrowser from "./TemplateBrowser.svelte";
import ReferencesPanel from "./ReferencesPanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import type { Beat, Chapter, Scene, Tag } from "../types";
import { mockProject } from "../../dev/mock-data";

// A failed action must tell the writer; before M17 these only reached console.error,
// so the button simply appeared to do nothing.

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

const chapter: Chapter = {
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
};
const scene: Scene = {
  id: "scene",
  chapter_id: chapter.id,
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
const beat: Beat = {
  id: "beat",
  scene_id: scene.id,
  content: "She knocks",
  prose: null,
  position: 0,
};

/** Resolve every command with `data`, except `failing`, which rejects with the backend's string. */
function mockInvoke(failing: string, data: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === failing) throw "Disk is read-only";
    return command in data ? data[command] : [];
  });
}

beforeEach(() => {
  HTMLElement.prototype.scrollIntoView = vi.fn();
  vi.mocked(invoke).mockReset();
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.clearToast();
  currentProject.setProject(mockProject);
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  ui.clearToast();
  vi.restoreAllMocks();
});

async function toast(message: string) {
  await waitFor(() => expect(ui.toast?.message).toBe(message));
}

async function openChapterMenu() {
  await fireEvent.contextMenu(await screen.findByRole("button", { name: chapter.title }));
}

it.each([
  ["duplicate_chapter", "Duplicate", "Failed to duplicate"],
  ["lock_chapter", "Lock", "Failed to change lock"],
  ["get_chapter_content_counts", "Delete", "Failed to prepare delete"],
])("sidebar shows a %s failure", async (command, item, message) => {
  mockInvoke(command, { get_chapters: [chapter], get_scenes: [scene] });
  render(Sidebar);
  await openChapterMenu();
  await fireEvent.click(screen.getByRole("menuitem", { name: item }));
  await toast(`${message}: Disk is read-only`);
});

it("sidebar shows a failed scene load, create and sync preview", async () => {
  mockInvoke("get_scenes", { get_chapters: [chapter] });
  render(Sidebar);
  await toast("Failed to load scenes: Disk is read-only");

  ui.clearToast();
  mockInvoke("create_scene", { get_chapters: [chapter], get_scenes: [scene] });
  currentProject.setScenes([scene]);
  await fireEvent.click(await screen.findByTestId("new-scene-button"));
  const input = screen.getByTestId("title-input") as HTMLInputElement;
  await fireEvent.input(input, { target: { value: "The Well" } });
  await fireEvent.keyDown(input, { key: "Enter" });
  await toast("Failed to create scene: Disk is read-only");
  // The typed title stays so the writer can retry.
  expect((screen.getByTestId("title-input") as HTMLInputElement).value).toBe("The Well");

  ui.clearToast();
  mockInvoke("get_sync_preview", { get_chapters: [chapter], get_scenes: [scene] });
  window.dispatchEvent(new Event("kindling:sync"));
  await toast("Failed to check for sync changes: Disk is read-only");
});

it.each([
  ["create_beat", "Failed to create beat"],
  ["delete_beat", "Failed to delete beat"],
])("beat view shows a %s failure", async (command, message) => {
  mockInvoke(command);
  currentProject.setCurrentScene(scene);
  currentProject.setBeats([beat]);
  render(BeatView, { beats: [beat] });
  if (command === "create_beat") {
    await fireEvent.click(screen.getAllByRole("button", { name: "Add beat" })[0]);
    const input = screen.getByLabelText("New beat") as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "The door opens" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await toast(`${message}: Disk is read-only`);
    expect((screen.getByLabelText("New beat") as HTMLInputElement).value).toBe("The door opens");
  } else {
    await fireEvent.contextMenu(screen.getByTestId("beat-header"));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
    await fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await toast(`${message}: Disk is read-only`);
  }
});

it("scene panel shows a failed discovery note and keeps the draft", async () => {
  mockInvoke("create_discovery_note", { get_scene_reference_items: {} });
  currentProject.setCurrentChapter(chapter);
  currentProject.setCurrentScene(scene);
  render(ScenePanel);
  await fireEvent.click(await screen.findByRole("button", { name: /Discovery notes/ }));
  await fireEvent.click(await screen.findByRole("button", { name: /Add (a )?note/i }));
  const textarea = screen.getByLabelText("New discovery note") as HTMLTextAreaElement;
  await fireEvent.input(textarea, { target: { value: "The gate was never locked" } });
  await fireEvent.click(screen.getByRole("button", { name: "Add note" }));
  await toast("Failed to add discovery note: Disk is read-only");
  expect((screen.getByLabelText("New discovery note") as HTMLTextAreaElement).value).toBe(
    "The gate was never locked"
  );
});

it("tag selector shows a failed removal", async () => {
  mockInvoke("untag_entity");
  const tag: Tag = {
    id: "tag",
    project_id: mockProject.id,
    name: "Clue",
    color: null,
    parent_id: null,
    position: 0,
    created_at: "",
  };
  render(TagSelector, {
    projectId: mockProject.id,
    entityType: "scene",
    entityId: scene.id,
    allTags: [tag],
    entityTagIds: [tag.id],
  });
  await fireEvent.click(screen.getByRole("button", { name: "Remove tag Clue" }));
  await toast("Failed to remove tag: Disk is read-only");
});

it("template browser shows a failed load instead of an empty list alone", async () => {
  mockInvoke("get_bundled_templates");
  render(TemplateBrowser, { onSelect: vi.fn(), onClose: vi.fn() });
  await toast("Failed to load templates: Disk is read-only");
});

it("references panel shows a failed detect-all request", async () => {
  mockInvoke("detect_all_references");
  render(ReferencesPanel);
  window.dispatchEvent(new Event("kindling:detectAllReferences"));
  await toast("Failed to detect references: Disk is read-only");
});
