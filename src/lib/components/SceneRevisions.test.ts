import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import type { Editor } from "@tiptap/core";
import ScenePanel from "./ScenePanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { session } from "../stores/session.svelte";
import { proseSaves } from "../utils/proseSaves";
import { mockProject, mockChapters, mockScenes } from "../../dev/mock-data";
import type { SceneReview } from "../utils/revisions";
vi.hoisted(() => {
  const data = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => data.set(k, v),
    removeItem: (k: string) => data.delete(k),
  });
});
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLElement.prototype.scrollIntoView = vi.fn();
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
  currentProject.setCurrentChapter(mockChapters[0]);
});
afterEach(async () => {
  cleanup();
  await session.flush();
  await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  currentProject.setProject(null);
  ui.setExpandedBeat(null);
  vi.restoreAllMocks();
});
it.each(["page", "beat"] as const)(
  "flushes pending %s prose before review and publishes accepted prose back to its editor",
  async (mode) => {
    const scene = {
      ...mockScenes[0],
      chapter_id: mockChapters[0].id,
      prose: "<p>Old prose.</p>",
      editor_mode: mode,
      planning_status: "fixed" as const,
    };
    const beat = {
      id: "review-beat",
      scene_id: scene.id,
      content: "Beat",
      prose: "<p>Old prose.</p>",
      position: 0,
    };
    currentProject.setCurrentScene(scene);
    currentProject.setBeats([beat]);
    if (mode === "beat") ui.setExpandedBeat(beat.id);
    let releaseSave!: () => void;
    let html = "<p>Old prose.</p>";
    let saved: SceneReview;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "save_scene_page_prose" || cmd === "save_beat_prose") {
        await new Promise<void>((r) => (releaseSave = r));
        html = (args as { prose: string }).prose;
      }
      if (cmd === "get_scene_reference_items") return {};
      if (cmd === "get_scene_review") {
        const id = mode === "page" ? scene.id : beat.id;
        saved = {
          scene_id: scene.id,
          version: 0,
          mode,
          documents: [
            { id: scene.id, label: "Scene page", html: mode === "page" ? html : scene.prose },
            { id: beat.id, label: "Beat 1", html: mode === "beat" ? html : beat.prose },
          ],
          data: {
            status: "editor_review",
            drafts: [],
            annotations: [
              {
                id: "change",
                document_id: id,
                anchor_html: html,
                from: 1,
                to: 7,
                quote: "Newest",
                replacement: "Reviewed",
                state: "open",
                messages: [],
              },
            ],
          },
        };
        return structuredClone(saved);
      }
      if (cmd === "save_scene_review") {
        const input = args as { next: SceneReview; data: SceneReview["data"] };
        saved = {
          ...saved!,
          documents: input.next.documents,
          mode: input.next.mode,
          data: input.data,
          version: 1,
        };
        return structuredClone(saved);
      }
      return [];
    });
    const onOpenEditorial = vi.fn().mockResolvedValue(undefined);
    render(ScenePanel, { onOpenEditorial });
    await waitFor(() => expect(document.querySelector(".tiptap")).not.toBeNull());
    await tick();
    // NovelEditor enables change notifications on its first timer turn.
    await new Promise((resolve) => setTimeout(resolve, 0));
    const editor = (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
    editor.commands.setContent("<p>Newest prose.</p>");
    await fireEvent.click(screen.getByText("Revisions"));
    await waitFor(() => expect(releaseSave).toBeTypeOf("function"));
    expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "get_scene_review")).toBe(false);
    releaseSave();
    await waitFor(() =>
      expect(onOpenEditorial).toHaveBeenCalledWith(mockProject.id, scene.id, expect.anything())
    );
    expect(html).toBe("<p>Newest prose.</p>");
  }
);
