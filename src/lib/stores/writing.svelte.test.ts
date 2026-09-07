import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { WritingStore } from "./writing.svelte";
import type { WritingStats } from "../types";

const stats: WritingStats = {
  project_id: "p",
  project_words: 9,
  chapter_words: { c: 9 },
  scene_words: { s: 9 },
  daily_goal: 500,
  today_words: 3,
  session_words: 2,
  streak: 1,
};
const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));
beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("writing statistics", () => {
  it("loads saved totals, refreshes after changes, and clears on close", async () => {
    vi.mocked(invoke).mockResolvedValue(stats);
    const store = new WritingStore();
    store.open("p");
    await settle();
    expect(store.value).toEqual(stats);
    await store.refresh();
    expect(invoke).toHaveBeenLastCalledWith("get_writing_stats", { projectId: "p" });
    store.open(null);
    await store.refresh();
    expect(store.value).toBeNull();
    expect(invoke).toHaveBeenCalledTimes(2);
  });
  it("ignores stale responses and saves from a previous project", async () => {
    let resolve!: (value: WritingStats) => void;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((r) => {
          resolve = r as typeof resolve;
        })
    );
    const store = new WritingStore();
    store.open("p");
    vi.mocked(invoke).mockResolvedValue({ ...stats, project_id: "q" });
    store.open("q");
    await settle();
    resolve(stats);
    await settle();
    await store.refresh("p");
    expect(store.value?.project_id).toBe("q");
    expect(invoke).toHaveBeenCalledTimes(2);
  });
  it("keeps the latest refresh when responses finish out of order", async () => {
    let reject!: (reason: unknown) => void;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((_, r) => {
          reject = r;
        })
    );
    const store = new WritingStore();
    store.open("p");
    vi.mocked(invoke).mockResolvedValue(stats);
    await store.refresh();
    reject(new Error("old error"));
    await settle();
    expect(store.error).toBeNull();
    expect(store.value).toEqual(stats);
  });
  it("surfaces errors and recovers on the next successful read", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("disk error"));
    const store = new WritingStore();
    store.open("p");
    await settle();
    expect(store.error).toContain("disk error");
    vi.mocked(invoke).mockResolvedValue(stats);
    await store.refresh();
    expect(store.error).toBeNull();
  });
  it("resets the session and refreshes without clearing daily counts", async () => {
    vi.mocked(invoke).mockResolvedValue(stats);
    const store = new WritingStore();
    store.open("p");
    await settle();
    vi.mocked(invoke).mockResolvedValue({ ...stats, session_words: 0 });
    await store.reset("p");
    expect(invoke).toHaveBeenCalledWith("reset_writing_session", { projectId: "p" });
    expect(store.value?.session_words).toBe(0);
    expect(store.value?.today_words).toBe(3);
  });
  it("reports reset failures only for the current project", async () => {
    vi.mocked(invoke).mockResolvedValue(stats);
    const store = new WritingStore();
    store.open("p");
    await settle();
    vi.mocked(invoke).mockRejectedValue(new Error("locked"));
    await store.reset("q");
    expect(store.error).toBeNull();
    await store.reset("p");
    expect(store.error).toContain("Could not reset session");
  });
});
