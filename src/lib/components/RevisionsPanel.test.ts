import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import RevisionsPanel from "./RevisionsPanel.svelte";
import type { SceneReview } from "../utils/revisions";
let persisted: SceneReview;
let failSave = false;
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLElement.prototype.scrollIntoView = vi.fn();
  persisted = {
    scene_id: "scene",
    version: 0,
    mode: "page",
    documents: [{ id: "scene", label: "Scene page", html: "<p>The red fox.</p>" }],
    data: { status: "first_draft", drafts: [], annotations: [] },
  };
  failSave = false;
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "get_scene_review") return structuredClone(persisted);
    if (cmd === "get_revision_overview")
      return [
        {
          scene_id: "scene",
          title: "Arrival",
          chapter: "Chapter 1",
          status: "first_draft",
          drafts: 0,
        },
      ];
    if (cmd === "save_scene_review") {
      if (failSave) throw new Error("database is locked");
      const { data, next } = args as {
        data: SceneReview["data"];
        next: { documents: SceneReview["documents"]; mode: SceneReview["mode"] } | null;
      };
      persisted = {
        ...persisted,
        version: persisted.version + 1,
        data: structuredClone(data),
        ...(next ? { documents: next.documents, mode: next.mode } : {}),
      };
      return structuredClone(persisted);
    }
  });
});
afterEach(cleanup);
async function setup(locked = false) {
  const onApplied = vi.fn(),
    onClose = vi.fn();
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked,
    onApplied,
    onClose,
  });
  await screen.findByText(
    "No saved drafts yet. Save a named draft to keep this scene’s current prose."
  );
  return { onApplied, onClose };
}
it("creates numbered named drafts, compares and restores with a preserved current draft", async () => {
  const { onApplied } = await setup();
  await fireEvent.click(screen.getByText("Draft history"));
  await fireEvent.input(screen.getByLabelText("Draft name"), { target: { value: "First pass" } });
  await fireEvent.click(screen.getByText("Save named draft"));
  await screen.findAllByRole("option", { name: "Draft 1 · First pass" });
  await fireEvent.click(screen.getByText("Restore selected draft"));
  await fireEvent.click(screen.getByText("Restore and preserve current prose"));
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(2));
  expect(persisted.data.drafts[1].name).toBe("Before restoring First pass");
  expect(onApplied).toHaveBeenCalledOnce();
  await fireEvent.click(screen.getByText("All scenes"));
  expect(screen.getByText("Chapter 1")).toBeTruthy();
});
it("allows reading locked scenes while disabling writes", async () => {
  await setup(true);
  expect(
    (screen.getByLabelText("Revision status") as HTMLSelectElement).closest("fieldset")?.disabled
  ).toBe(true);
  await fireEvent.click(screen.getByText("Draft history"));
  expect(
    (screen.getByLabelText("Draft name") as HTMLInputElement).closest("fieldset")?.disabled
  ).toBe(true);
});
