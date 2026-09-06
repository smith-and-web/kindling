import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import type { Editor } from "@tiptap/core";
import { invoke } from "@tauri-apps/api/core";
import ScenePanel from "./ScenePanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { mockProject, mockScenes, mockChapters } from "../../dev/mock-data";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    clear: () => values.clear(),
  });
});

const scene = { ...mockScenes[0], planning_status: "fixed" as const, editor_mode: "beat" as const };
const destination = { ...scene, id: "destination", synopsis: "Destination synopsis" };
const placeholder = "Write a brief synopsis for this scene...";

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue([]);
  currentProject.setProject(mockProject);
  currentProject.setCurrentChapter(mockChapters[0]);
  currentProject.setScenes([scene, destination]);
  currentProject.setCurrentScene(scene);
});

afterEach(async () => {
  cleanup();
  await vi.advanceTimersByTimeAsync(2000);
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.useRealTimers();
  vi.restoreAllMocks();
});

async function editSynopsis(text = "Updated synopsis") {
  await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
  const textarea = screen.getByPlaceholderText(placeholder) as HTMLTextAreaElement;
  textarea.focus();
  await fireEvent.input(textarea, { target: { value: text } });
  return textarea;
}

// Discord report, Quizzical, July 13–14, 2026: autosave interrupts synopsis/beat editing.
it("keeps the synopsis open and focused through repeated autosaves", async () => {
  render(ScenePanel);
  const textarea = await editSynopsis();
  textarea.setSelectionRange(4, 4);
  await vi.advanceTimersByTimeAsync(1000);
  expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
    sceneId: scene.id,
    synopsis: "Updated synopsis",
  });
  expect(screen.queryByPlaceholderText(placeholder)).toBe(textarea);
  expect(document.activeElement).toBe(textarea);
  expect(textarea.selectionStart).toBe(4);
  await fireEvent.input(textarea, { target: { value: "Updated synopsis continued" } });
  await vi.advanceTimersByTimeAsync(1000);
  expect(screen.queryByPlaceholderText(placeholder)).toBe(textarea);
  expect(currentProject.currentScene?.synopsis).toBe("Updated synopsis continued");
});

it.each([0, 1000])("saves to the original scene when navigating after %i ms", async (delay) => {
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  let finish!: () => void;
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis")
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
    return [];
  });
  await editSynopsis();
  await vi.advanceTimersByTimeAsync(delay);
  currentProject.setCurrentScene(destination);
  await tick();
  await vi.advanceTimersByTimeAsync(1000);
  expect(screen.queryByPlaceholderText(placeholder)).toBeNull();
  expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
    sceneId: scene.id,
    synopsis: "Updated synopsis",
  });
  finish();
  await vi.advanceTimersByTimeAsync(0);
  expect(currentProject.currentScene?.synopsis).toBe(destination.synopsis);
  expect(currentProject.scenes.find((item) => item.id === scene.id)?.synopsis).toBe(
    "Updated synopsis"
  );
});

it("flushes the synopsis on Escape so reopening keeps the latest text", async () => {
  render(ScenePanel);
  await editSynopsis();
  await fireEvent.keyDown(window, { key: "Escape" });
  await vi.advanceTimersByTimeAsync(0);
  expect(screen.queryByPlaceholderText(placeholder)).toBeNull();
  expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
    sceneId: scene.id,
    synopsis: "Updated synopsis",
  });
  await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
  expect((screen.getByPlaceholderText(placeholder) as HTMLTextAreaElement).value).toBe(
    "Updated synopsis"
  );
});

it("flushes a pending synopsis when the scene panel unmounts", async () => {
  const view = render(ScenePanel);
  await editSynopsis();
  view.unmount();
  await vi.advanceTimersByTimeAsync(0);
  expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
    sceneId: scene.id,
    synopsis: "Updated synopsis",
  });
});

it("serializes synopsis saves and reopens the newest draft while an older save is pending", async () => {
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  let finish!: () => void;
  let persisted = scene.synopsis;
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "save_scene_synopsis") {
      if (!finish)
        await new Promise<void>((resolve) => {
          finish = resolve;
        });
      persisted = (args as { synopsis: string }).synopsis;
    }
    return [];
  });
  const textarea = await editSynopsis("First draft");
  await vi.advanceTimersByTimeAsync(1000);
  await fireEvent.input(textarea, { target: { value: "Latest draft" } });
  await fireEvent.keyDown(window, { key: "Escape" });
  await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
  const reopened = screen.getByPlaceholderText(placeholder) as HTMLTextAreaElement;
  expect(reopened.value).toBe("Latest draft");
  await vi.advanceTimersByTimeAsync(1000);
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_synopsis")
  ).toHaveLength(1);
  finish();
  await vi.advanceTimersByTimeAsync(0);
  expect(persisted).toBe("Latest draft");
  expect(currentProject.currentScene?.synopsis).toBe("Latest draft");
  expect(screen.queryByPlaceholderText(placeholder)).toBe(reopened);
});

it("keeps synopsis editing usable after a save failure", async () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  vi.mocked(invoke).mockRejectedValueOnce("Disk full");
  const textarea = await editSynopsis();
  await vi.advanceTimersByTimeAsync(1000);
  expect(screen.queryByPlaceholderText(placeholder)).toBe(textarea);
  expect(document.activeElement).toBe(textarea);
  await fireEvent.input(textarea, { target: { value: "Continued after failure" } });
  await vi.advanceTimersByTimeAsync(1000);
  expect(invoke).toHaveBeenLastCalledWith("save_scene_synopsis", {
    sceneId: scene.id,
    synopsis: "Continued after failure",
  });
  expect(screen.queryByText("Saving...")).toBeNull();
});

it.each([false, true])(
  "preserves the beat cursor when autosave finishes (newer edit: %s)",
  async (newerEdit) => {
    const beat = {
      id: "beat",
      scene_id: scene.id,
      content: "Greeting",
      prose: "<p>First paragraph</p><p>Last paragraph</p>",
      position: 0,
    };
    currentProject.setBeats([beat]);
    const { component } = render(ScenePanel);
    await tick();
    ui.setExpandedBeat(beat.id);
    await vi.advanceTimersByTimeAsync(0);
    const element = document.querySelector(".tiptap") as HTMLElement & { editor: Editor };
    const editor = element.editor;
    editor.view.focus();
    editor.commands.setTextSelection(4);
    editor.commands.insertContent("X");
    let finish!: () => void;
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_beat_prose" && !finish)
        await new Promise<void>((resolve) => {
          finish = resolve;
        });
      return [];
    });
    await vi.advanceTimersByTimeAsync(500);
    if (newerEdit) editor.commands.insertContent("Y");
    const expectedText = editor.getHTML();
    const expectedCursor = editor.state.selection.head;
    finish();
    await vi.advanceTimersByTimeAsync(0);
    expect(editor.state.selection.head).toBe(expectedCursor);
    expect(editor.getHTML()).toBe(expectedText);
    expect(document.activeElement).toBe(element);
    await component.prepareForSearch();
    await vi.advanceTimersByTimeAsync(1000);
    expect(editor.state.selection.head).toBe(expectedCursor);
    expect(editor.getHTML()).toBe(expectedText);
    expect(currentProject.beats[0].prose).toBe(expectedText);
  }
);
