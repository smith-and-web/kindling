import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import type { Editor } from "@tiptap/core";
import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import ScenePanel from "./ScenePanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { mockProject, mockChapters, mockScenes } from "../../dev/mock-data";
import type { Scene } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    clear: () => values.clear(),
  });
});

function editor() {
  return (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
}

function pageScene(id: string, prose: string): Scene {
  return { ...mockScenes[0], id, prose, editor_mode: "page", planning_status: "fixed" };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue([]);
  currentProject.setProject(mockProject);
  currentProject.setCurrentChapter(mockChapters[0]);
  currentProject.setBeats([]);
});

afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.useRealTimers();
  vi.restoreAllMocks();
});

it.each([
  { mode: "page", button: "view-page" },
  { mode: "beat", button: "view-beats" },
] as const)(
  "re-selecting the active $mode view leaves the scene alone",
  async ({ mode, button }) => {
    currentProject.setCurrentScene({ ...pageScene("scene", "<p>Draft</p>"), editor_mode: mode });
    render(ScenePanel);
    await vi.advanceTimersByTimeAsync(0);
    await fireEvent.click(screen.getByTestId(button));
    await vi.advanceTimersByTimeAsync(0);
    expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "switch_scene_editor_mode")).toBe(
      false
    );
    expect(screen.queryByRole("button", { name: "Switch" })).toBeNull();
    expect(currentProject.currentScene?.editor_mode).toBe(mode);
  }
);

it("undo after switching Page View scenes never restores the previous scene's text", async () => {
  const sceneA = pageScene("scene-a", "<p>Scene A text</p>");
  const sceneB = pageScene("scene-b", "<p>Scene B text</p>");
  currentProject.setScenes([sceneA, sceneB]);
  currentProject.setCurrentScene(sceneA);
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  const first = editor();
  expect(first.getText()).toBe("Scene A text");

  currentProject.setCurrentScene(sceneB);
  await tick();
  await vi.advanceTimersByTimeAsync(0);
  expect(editor()).not.toBe(first);
  expect(editor().getText()).toBe("Scene B text");

  editor().commands.insertContentAt(editor().state.doc.content.size - 1, " more");
  for (let i = 0; i < 5; i++) editor().commands.undo();
  expect(editor().getText()).toBe("Scene B text");
  await vi.advanceTimersByTimeAsync(600);
  const saves = vi
    .mocked(invoke)
    .mock.calls.filter(([cmd]) => cmd === "save_scene_page_prose")
    .map(([, args]) => args as { sceneId: string; prose: string });
  expect(saves.length).toBeGreaterThan(0);
  expect(saves.every(({ sceneId }) => sceneId === sceneB.id)).toBe(true);
  expect(saves.some(({ prose }) => prose.includes("Scene A text"))).toBe(false);
});

it("mounts a page scene reached from a beat scene with its own prose", async () => {
  const sceneA = pageScene("scene-a", "<p>Scene A text</p>");
  const beatScene = { ...pageScene("scene-c", "<p>Beat scene</p>"), editor_mode: "beat" as const };
  const sceneB = pageScene("scene-b", "<p>Scene B text</p>");
  currentProject.setCurrentScene(sceneA);
  render(ScenePanel);
  await vi.advanceTimersByTimeAsync(0);
  currentProject.setCurrentScene(beatScene);
  await tick();
  await vi.advanceTimersByTimeAsync(0);
  const mounted = new Set<Element>();
  const observer = new MutationObserver((records) => {
    for (const record of records)
      for (const node of record.addedNodes)
        if (node instanceof HTMLElement)
          for (const el of [node, ...node.querySelectorAll(".tiptap")])
            if (el.matches(".tiptap")) mounted.add(el);
  });
  observer.observe(document.body, { childList: true, subtree: true });
  currentProject.setCurrentScene(sceneB);
  await tick();
  await vi.advanceTimersByTimeAsync(0);
  observer.disconnect();
  expect(mounted.size).toBe(1);
  expect(editor().getText()).toBe("Scene B text");
  editor().commands.undo();
  expect(editor().getText()).toBe("Scene B text");
});
