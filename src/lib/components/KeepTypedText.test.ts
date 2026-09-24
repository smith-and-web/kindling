import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import BeatView from "./BeatView.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import type { Beat, Chapter } from "../types";
import { mockProject } from "../../dev/mock-data";

// M18: a failed save must not throw away what the writer typed.

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
  planning_status: "flexible",
};
const beat: Beat = {
  id: "beat",
  scene_id: "scene",
  content: "She knocks",
  prose: null,
  position: 0,
};

let failing: string | null = null;
beforeEach(() => {
  HTMLElement.prototype.scrollIntoView = vi.fn();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === failing) throw "Disk is read-only";
    return command === "get_chapters" ? [chapter] : [];
  });
  vi.spyOn(console, "error").mockImplementation(() => {});
  ui.clearToast();
  currentProject.setProject(mockProject);
});
afterEach(() => {
  failing = null;
  cleanup();
  currentProject.setProject(null);
  ui.clearToast();
  vi.restoreAllMocks();
});

it("keeps a chapter synopsis open with its text when the save fails, then closes once saved", async () => {
  failing = "update_chapter_synopsis";
  render(Sidebar);
  await fireEvent.click(await screen.findByRole("button", { name: "Add synopsis…" }));
  const field = screen.getByLabelText("Chapter synopsis") as HTMLTextAreaElement;
  await fireEvent.input(field, { target: { value: "The house remembers her." } });
  await fireEvent.blur(field);
  await waitFor(() =>
    expect(ui.toast?.message).toBe("Failed to save chapter synopsis: Disk is read-only")
  );
  expect((screen.getByLabelText("Chapter synopsis") as HTMLTextAreaElement).value).toBe(
    "The house remembers her."
  );
  expect(currentProject.chapters[0].synopsis).toBeNull();

  failing = null;
  await fireEvent.blur(screen.getByLabelText("Chapter synopsis"));
  await waitFor(() => expect(screen.queryByLabelText("Chapter synopsis")).toBeNull());
  expect(currentProject.chapters[0].synopsis).toBe("The house remembers her.");
  expect(screen.getByRole("button", { name: "The house remembers her." })).toBeTruthy();
});

it("keeps a beat title being renamed when the save fails, then applies it once saved", async () => {
  failing = "rename_beat";
  currentProject.setBeats([beat]);
  render(BeatView, { beats: [beat] });
  await fireEvent.contextMenu(screen.getByTestId("beat-header"));
  await fireEvent.click(screen.getByRole("menuitem", { name: "Rename" }));
  const input = screen.getByLabelText("Beat title") as HTMLInputElement;
  await fireEvent.input(input, { target: { value: "She hesitates" } });
  await fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(ui.toast?.message).toBe("Failed to rename beat: Disk is read-only"));
  expect((screen.getByLabelText("Beat title") as HTMLInputElement).value).toBe("She hesitates");
  expect(currentProject.beats[0].content).toBe("She knocks");

  failing = null;
  await fireEvent.keyDown(screen.getByLabelText("Beat title"), { key: "Enter" });
  await waitFor(() => expect(screen.queryByLabelText("Beat title")).toBeNull());
  expect(invoke).toHaveBeenLastCalledWith("rename_beat", {
    beatId: beat.id,
    content: "She hesitates",
  });
  expect(currentProject.beats[0].content).toBe("She hesitates");
});
