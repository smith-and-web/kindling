import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import WritingProgress from "./WritingProgress.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockProject } from "../../dev/mock-data";
import type { WritingStats } from "../types";

const stats: WritingStats = {
  project_id: mockProject.id,
  project_words: 2500,
  chapter_words: {},
  scene_words: {},
  daily_goal: 500,
  today_words: 300,
  session_words: 25,
  streak: 3,
};
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(stats);
  currentProject.setProject(mockProject);
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  vi.useRealTimers();
});
it("shows project, daily, session and streak counts and refreshes across midnight", async () => {
  render(WritingProgress);
  await vi.advanceTimersByTimeAsync(0);
  expect(screen.getByText("2,500 project words")).toBeTruthy();
  expect(screen.getByText("Session: 25 words")).toBeTruthy();
  expect(screen.getByText("3 days writing streak")).toBeTruthy();
  expect((screen.getByLabelText("Daily writing goal") as HTMLProgressElement).value).toBe(300);
  vi.mocked(invoke).mockResolvedValue({ ...stats, today_words: 0 });
  await vi.advanceTimersByTimeAsync(30000);
  expect((screen.getByLabelText("Daily writing goal") as HTMLProgressElement).value).toBe(0);
});
it("finishes editor debounce saves before resetting the session", async () => {
  let finish!: () => void;
  const prepareReset = vi.fn(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      })
  );
  render(WritingProgress, { prepareReset });
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByLabelText("Reset writing session"));
  expect(prepareReset).toHaveBeenCalledOnce();
  expect(invoke).not.toHaveBeenCalledWith("reset_writing_session", expect.anything());
  expect((screen.getByLabelText("Reset writing session") as HTMLButtonElement).disabled).toBe(true);
  finish();
  await vi.advanceTimersByTimeAsync(0);
  expect(invoke).toHaveBeenCalledWith("reset_writing_session", { projectId: mockProject.id });
});
it("retains the session when saving pending editor prose fails", async () => {
  render(WritingProgress, {
    prepareReset: async () => {
      throw new Error("disk full");
    },
  });
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByLabelText("Reset writing session"));
  expect(screen.getByRole("alert").textContent).toContain("disk full");
  expect(invoke).not.toHaveBeenCalledWith("reset_writing_session", expect.anything());
});
it("shows an off goal without a progress bar and a load error without stale totals", async () => {
  vi.mocked(invoke).mockResolvedValue({ ...stats, daily_goal: 0 });
  render(WritingProgress);
  await vi.advanceTimersByTimeAsync(0);
  expect(screen.queryByLabelText("Daily writing goal")).toBeNull();
  expect(screen.getByText(/goal off/)).toBeTruthy();
  cleanup();
  vi.mocked(invoke).mockRejectedValue(new Error("database unavailable"));
  render(WritingProgress);
  await vi.advanceTimersByTimeAsync(0);
  expect(screen.getByRole("alert").textContent).toContain("database unavailable");
  expect(screen.queryByText("2,500 project words")).toBeNull();
});
