import { afterAll, afterEach, beforeAll, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, waitFor } from "@testing-library/svelte";
import type { Editor } from "@tiptap/core";
import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import Sidebar from "./Sidebar.svelte";
import ScenePanel from "./ScenePanel.svelte";
import NovelEditor from "./NovelEditor.svelte";
import { currentProject } from "../stores/project.svelte";
import { session } from "../stores/session.svelte";
import { ui } from "../stores/ui.svelte";
import { mockProject, mockChapters, mockScenes } from "../../dev/mock-data";
import type { SessionState } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});

const chapter = { ...mockChapters[1], id: "later-chapter", position: 2 };
const scene = {
  ...mockScenes[0],
  id: "later-scene",
  chapter_id: chapter.id,
  prose: "<p>A long enough draft for the cursor.</p>",
};
const beat = {
  id: "last-beat",
  scene_id: scene.id,
  content: "Last beat",
  prose: scene.prose,
  position: 1,
};
const disk = new Map<string, SessionState>();
const saved: SessionState = {
  project_id: mockProject.id,
  current_chapter_id: chapter.id,
  current_scene_id: scene.id,
  current_beat_id: beat.id,
  cursor_position: 14,
  scroll_position: 330,
  editor_scroll_position: 85,
  last_opened_at: "2026-09-06",
};
let mode: "page" | "beat";

// jsdom has no layout implementation for DOM ranges used by ProseMirror.
beforeAll(() => {
  Object.defineProperty(Range.prototype, "getClientRects", { configurable: true, value: () => [] });
});
afterAll(() => {
  Reflect.deleteProperty(Range.prototype, "getClientRects");
});

beforeEach(async () => {
  await session.flush();
  disk.clear();
  mode = "beat";
  ui.setExpandedBeat(null);
  currentProject.setProject(null);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    const input = args as Record<string, unknown>;
    if (cmd === "save_session_state") {
      const state = input.session as SessionState;
      disk.set(state.project_id, structuredClone(state));
      return;
    }
    if (cmd === "get_session_state") return disk.get(input.projectId as string) ?? null;
    if (cmd === "get_chapters") return [mockChapters[1], chapter];
    if (cmd === "get_scenes")
      return input.chapterId === chapter.id ? [{ ...scene, editor_mode: mode }] : mockScenes;
    if (cmd === "get_beats") return [beat];
    if (cmd === "get_scene_reference_items") return {};
    return [];
  });
});

afterEach(async () => {
  cleanup();
  await session.flush();
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.restoreAllMocks();
});

function activeEditor() {
  return (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
}

it.each(["beat", "page"] as const)(
  "reopens the last scene in %s mode with cursor and both scroll positions",
  async (editorMode) => {
    mode = editorMode;
    disk.set(mockProject.id, { ...saved, current_beat_id: mode === "page" ? null : beat.id });
    currentProject.setProject(mockProject);
    render(Sidebar);
    render(ScenePanel);
    await waitFor(() => expect(session.restoring).toBe(false));
    await waitFor(() => expect(document.querySelector(".tiptap")).not.toBeNull());
    await waitFor(() => expect(activeEditor().state.selection.head).toBe(14));
    expect(currentProject.currentChapter?.id).toBe(chapter.id);
    expect(currentProject.currentScene?.id).toBe(scene.id);
    expect(ui.expandedBeatId).toBe(mode === "page" ? null : beat.id);
    const outer = document.querySelector('[data-testid="scene-panel"] > div') as HTMLElement;
    const inner = document.querySelector(".novel-pages-container") as HTMLElement;
    await waitFor(() => expect(outer.scrollTop).toBe(330));
    await waitFor(() => expect(inner.scrollTop).toBe(85));

    activeEditor().view.dom.focus();
    activeEditor().commands.setTextSelection(23);
    outer.scrollTop = 460;
    await fireEvent.scroll(outer);
    inner.scrollTop = 120;
    await fireEvent.scroll(inner);
    await session.flush();
    expect(disk.get(mockProject.id)).toMatchObject({
      cursor_position: 23,
      scroll_position: 460,
      editor_scroll_position: 120,
    });

    cleanup();
    currentProject.setProject(null);
    currentProject.setProject(mockProject);
    render(Sidebar);
    render(ScenePanel);
    await waitFor(() => expect(document.querySelector(".tiptap")).not.toBeNull());
    await waitFor(() => expect(activeEditor().state.selection.head).toBe(23));
    await waitFor(() =>
      expect(
        (document.querySelector('[data-testid="scene-panel"] > div') as HTMLElement).scrollTop
      ).toBe(460)
    );
  }
);

it("keeps the default chapter behavior for a project without a saved scene", async () => {
  currentProject.setProject(mockProject);
  render(Sidebar);
  await waitFor(() => expect(currentProject.currentChapter?.id).toBe(mockChapters[1].id));
  expect(currentProject.currentScene).toBeNull();
  expect(disk.size).toBe(0);
});

it("clamps an outdated cursor and resumes at the start of a beat when no cursor was saved", async () => {
  for (const cursor of [10000, null]) {
    currentProject.setProject(null);
    currentProject.setProject(mockProject);
    disk.set(mockProject.id, { ...saved, cursor_position: cursor });
    await session.load(mockProject.id);
    currentProject.setCurrentScene(scene);
    render(NovelEditor, {
      content: "<p>Short</p>",
      projectId: mockProject.id,
      sceneId: scene.id,
      beatId: beat.id,
    });
    await waitFor(() => expect(activeEditor().state.selection.head).toBe(cursor === null ? 1 : 6));
    cleanup();
    await tick();
  }
});

it("ignores editor callbacks belonging to a previous project", async () => {
  currentProject.setProject(mockProject);
  currentProject.setCurrentScene(scene);
  const { unmount } = render(NovelEditor, {
    content: scene.prose,
    projectId: mockProject.id,
    sceneId: scene.id,
    beatId: beat.id,
  });
  const old = activeEditor();
  await new Promise((resolve) => requestAnimationFrame(resolve));
  currentProject.setProject({ ...mockProject, id: "other-project" });
  currentProject.setCurrentScene({ ...scene, id: "other-scene" });
  old.view.dom.focus();
  old.commands.setTextSelection(20);
  await unmount();
  await session.flush();
  expect(disk.get("other-project")).toMatchObject({
    current_scene_id: "other-scene",
    cursor_position: null,
  });
});
