import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import WritingStatusBar from "./WritingStatusBar.svelte";
import WritingProgress from "./WritingProgress.svelte";
import { currentProject } from "../stores/project.svelte";
import { writing } from "../stores/writing.svelte";
import { mockChapters, mockProject, mockScenes } from "../../dev/mock-data";
import type { WritingStats } from "../types";

const chapter = mockChapters[1];
const emptyChapter = mockChapters[2];
const scene = { ...mockScenes[0], chapter_id: chapter.id };
const stats: WritingStats = {
  project_id: mockProject.id,
  project_words: 1001,
  chapter_words: { [chapter.id]: 1001, [emptyChapter.id]: 0 },
  scene_words: { [scene.id]: 801, another: 200, empty: 0 },
  daily_goal: 500,
  today_words: 20,
  session_words: -5,
  streak: 0,
};

beforeEach(async () => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(stats);
  currentProject.setProject(mockProject);
  currentProject.setChapters([chapter, emptyChapter]);
  currentProject.setCurrentChapter(chapter);
  currentProject.setCurrentScene(scene);
  writing.open(mockProject.id);
  await writing.refresh();
});

afterEach(() => {
  cleanup();
  writing.open(null);
  currentProject.setProject(null);
});

it("shows scene, chapter, project and signed session totals and follows scene selection", async () => {
  render(WritingStatusBar);
  expect(screen.getByText("Scene: 801 words")).toBeTruthy();
  expect(screen.getByText("Chapter: 1,001 words")).toBeTruthy();
  expect(screen.getByText("Project: 1,001 words")).toBeTruthy();
  expect(screen.getByText("Session: -5 words")).toBeTruthy();

  currentProject.setCurrentScene({ ...scene, id: "empty", chapter_id: emptyChapter.id });
  await tick();
  expect(screen.getByText("Scene: 0 words")).toBeTruthy();
  expect(screen.getByText("Chapter: 0 words")).toBeTruthy();
});

it("opens a chapter breakdown and includes empty scenes in the average", async () => {
  render(WritingStatusBar);
  expect(screen.queryByRole("region", { name: "Writing statistics" })).toBeNull();
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  const panel = within(screen.getByRole("region", { name: "Writing statistics" }));
  expect(panel.getByText("Total words").nextElementSibling?.textContent?.trim()).toBe("1,001");
  expect(panel.getByText("Scenes with prose").nextElementSibling?.textContent?.trim()).toBe("2");
  expect(panel.getByText("Empty scenes").nextElementSibling?.textContent?.trim()).toBe("1");
  expect(panel.getByText("Average words per scene").nextElementSibling?.textContent?.trim()).toBe(
    "333.7"
  );
  const rows = within(panel.getByRole("table", { name: "Words per chapter" })).getAllByRole("row");
  expect(rows).toHaveLength(3);
  expect(rows[1].textContent).toContain(chapter.title);
  expect(within(rows[1]).getByRole("cell").textContent?.trim()).toBe("1,001");
  expect(rows[2].textContent).toContain(emptyChapter.title);
  expect(within(rows[2]).getByRole("cell").textContent?.trim()).toBe("0");
  expect(
    screen.getByRole("button", { name: "Hide statistics" }).getAttribute("aria-expanded")
  ).toBe("true");
  await fireEvent.click(screen.getByRole("button", { name: "Hide statistics" }));
  expect(screen.queryByRole("region", { name: "Writing statistics" })).toBeNull();
});

it("refreshes saved statistics on opening and updates the expanded panel after saves", async () => {
  render(WritingStatusBar);
  vi.mocked(invoke).mockClear();
  vi.mocked(invoke).mockResolvedValue({
    ...stats,
    project_words: 1500,
    scene_words: { ...stats.scene_words, empty: 499 },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  expect(invoke).toHaveBeenCalledWith("get_writing_stats", { projectId: mockProject.id });
  expect(screen.getByText("Project: 1,500 words")).toBeTruthy();
  expect(screen.getByText("Scenes with prose").nextElementSibling?.textContent?.trim()).toBe("3");
  expect(screen.getByText("Average words per scene").nextElementSibling?.textContent?.trim()).toBe(
    "500"
  );

  vi.mocked(invoke).mockResolvedValue(stats);
  await writing.refresh();
  await tick();
  expect(screen.getByText("Project: 1,001 words")).toBeTruthy();
  expect(screen.getByText("Empty scenes").nextElementSibling?.textContent?.trim()).toBe("1");
});

it("keeps project totals available with no scene selected and handles a project with no scenes", async () => {
  currentProject.setCurrentChapter(null);
  currentProject.setChapters([]);
  vi.mocked(invoke).mockResolvedValue({
    ...stats,
    project_words: 0,
    scene_words: {},
    chapter_words: {},
  });
  await writing.refresh();
  render(WritingStatusBar);
  expect(screen.getByText("Scene: — words")).toBeTruthy();
  expect(screen.getByText("Chapter: — words")).toBeTruthy();
  expect(screen.getByText("Project: 0 words")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  expect(screen.getByText("No chapters yet.")).toBeTruthy();
  expect(screen.getByText("Average words per scene").nextElementSibling?.textContent?.trim()).toBe(
    "0"
  );
});

it("omits archived chapters and never shows another project's statistics", async () => {
  currentProject.setChapters([chapter, { ...emptyChapter, archived: true }]);
  render(WritingStatusBar);
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  expect(screen.queryByText(emptyChapter.title)).toBeNull();
  currentProject.setProject({ ...mockProject, id: "other-project" });
  await tick();
  expect(screen.queryByText("Project: 1,001 words")).toBeNull();
  expect(screen.queryByRole("region", { name: "Writing statistics" })).toBeNull();
});

it("excludes Part headings from the chapter breakdown, including a project with only Parts", async () => {
  const part = mockChapters[0];
  currentProject.setChapters([part, chapter, emptyChapter]);
  vi.mocked(invoke).mockResolvedValue({
    ...stats,
    chapter_words: { ...stats.chapter_words, [part.id]: 0 },
  });
  await writing.refresh();
  render(WritingStatusBar);
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  expect(screen.queryByText(part.title)).toBeNull();
  expect(screen.getAllByRole("row")).toHaveLength(3);

  currentProject.setChapters([part]);
  await tick();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByText("No chapters yet.")).toBeTruthy();
});

it("announces a refresh error once with the sidebar mounted and keeps it visible when statistics close", async () => {
  render(WritingProgress);
  render(WritingStatusBar);
  await writing.refresh();
  await tick();
  vi.mocked(invoke).mockRejectedValue(new Error("database unavailable"));
  await fireEvent.click(screen.getByRole("button", { name: "Writing statistics" }));
  expect(screen.getByRole("alert").textContent).toContain("database unavailable");
  expect(screen.getByText("Project: 1,001 words")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Hide statistics" }));
  expect(screen.getByRole("alert").textContent).toContain("database unavailable");
  vi.mocked(invoke).mockResolvedValue(stats);
  await writing.refresh();
  await tick();
  expect(screen.queryByRole("alert")).toBeNull();
});
