import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { session } from "./session.svelte";
import { mockScenes } from "../../dev/mock-data";
import type { SessionState } from "../types";

const scene = mockScenes[0];
const saved: SessionState = {
  project_id: "project",
  current_scene_id: scene.id,
  current_chapter_id: scene.chapter_id,
  current_beat_id: "beat",
  cursor_position: 17,
  scroll_position: 230,
  editor_scroll_position: 80,
  last_opened_at: "2026-09-06",
};

beforeEach(async () => {
  await session.flush();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(null);
  session.open("project");
});
afterEach(async () => {
  await session.flush();
  session.open(null);
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("writing session persistence", () => {
  it("debounces cursor and scroll bursts and does not write unchanged positions", async () => {
    vi.useFakeTimers();
    session.selectScene(scene);
    for (let i = 1; i <= 20; i++) {
      session.update("project", scene.id, { cursor_position: i, scroll_position: i * 10 });
      await vi.advanceTimersByTimeAsync(50);
    }
    expect(invoke).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(500);
    expect(invoke).toHaveBeenCalledExactlyOnceWith("save_session_state", {
      session: expect.objectContaining({ cursor_position: 20, scroll_position: 200 }),
    });
    session.update("project", scene.id, { cursor_position: 20, scroll_position: 200 });
    await vi.advanceTimersByTimeAsync(600);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("flushes before the debounce expires and cancels the redundant timer", async () => {
    vi.useFakeTimers();
    session.selectScene(scene);
    session.update("project", scene.id, { cursor_position: 25 });
    await session.flush();
    expect(invoke).toHaveBeenCalledExactlyOnceWith("save_session_state", {
      session: expect.objectContaining({ cursor_position: 25 }),
    });
    await vi.advanceTimersByTimeAsync(600);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("releases a rejected write and retries the newest pending snapshot", async () => {
    session.selectScene(scene);
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));
    await expect(session.flush()).rejects.toThrow("disk full");
    session.update("project", scene.id, { cursor_position: 29 });
    await expect(session.flush()).resolves.toBeUndefined();
    expect(invoke).toHaveBeenLastCalledWith("save_session_state", {
      session: expect.objectContaining({ cursor_position: 29 }),
    });
  });

  it("reports a debounced failure without an unhandled rejection and allows later saves", async () => {
    vi.useFakeTimers();
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    session.selectScene(scene);
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));
    await vi.advanceTimersByTimeAsync(600);
    expect(error).toHaveBeenCalledWith("Failed to save writing position:", expect.any(Error));
    await session.flush();
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("only publishes viewports for the active scene and clears them on navigation", () => {
    session.selectScene(scene);
    session.restoreViewport("other", scene.id, 100);
    expect(session.viewport).toBeNull();
    session.restoreViewport("project", scene.id, 1800);
    expect(session.viewport).toEqual({ projectId: "project", sceneId: scene.id, position: 1800 });
    session.selectScene({ ...scene, id: "new-scene" });
    expect(session.viewport).toBeNull();
  });
  it("loads a saved position without overwriting it when selecting the restored scene", async () => {
    vi.mocked(invoke).mockResolvedValue(saved);
    expect(await session.load("project")).toEqual(saved);
    session.selectScene(scene);
    expect(session.value).toEqual(saved);
    expect(session.viewport).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("save_session_state", expect.anything());
  });

  it("resets cursor and both scroll positions for a different scene", async () => {
    vi.mocked(invoke).mockResolvedValue(saved);
    await session.load("project");
    session.selectScene({ ...scene, id: "new-scene" });
    await session.flush();
    expect(session.value).toMatchObject({
      current_scene_id: "new-scene",
      current_beat_id: null,
      cursor_position: null,
      scroll_position: 0,
      editor_scroll_position: 0,
    });
    expect(session.viewport).toBeNull();
  });

  it("serializes writes and flushes the newest state for each project across a switch", async () => {
    let release!: () => void;
    const writes: SessionState[] = [];
    vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
      const state = (args as { session: SessionState }).session;
      writes.push(state);
      if (writes.length === 1)
        await new Promise<void>((resolve) => {
          release = resolve;
        });
    });
    session.selectScene(scene);
    const firstWrite = session.flush();
    session.update("project", scene.id, { cursor_position: 4 });
    session.update("project", scene.id, { cursor_position: 19, scroll_position: 90 });
    session.open("other");
    session.selectScene({ ...scene, id: "other-scene" });
    session.update("project", scene.id, { cursor_position: 999 });
    const flushed = session.flush();
    expect(writes).toHaveLength(1);
    release();
    await Promise.all([firstWrite, flushed]);
    expect(writes).toHaveLength(3);
    expect(writes[1]).toMatchObject({
      project_id: "project",
      cursor_position: 19,
      scroll_position: 90,
    });
    expect(writes[2]).toMatchObject({
      project_id: "other",
      current_scene_id: "other-scene",
      cursor_position: null,
    });
  });

  it("waits for pending saves before immediately reopening the same project", async () => {
    let release!: () => void;
    let disk: SessionState | null = null;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "get_session_state") return disk;
      await new Promise<void>((resolve) => {
        release = resolve;
      });
      disk = (args as { session: SessionState }).session;
    });
    session.selectScene(scene);
    session.open(null);
    session.open("project");
    const loading = session.load("project");
    expect(invoke).not.toHaveBeenCalledWith("get_session_state", expect.anything());
    release();
    await loading;
    expect(session.value?.current_scene_id).toBe(scene.id);
  });

  it.each(["switch", "close-reopen", "navigate"])(
    "ignores a stale load after %s",
    async (action) => {
      let release!: (state: SessionState) => void;
      vi.mocked(invoke).mockImplementation(async (cmd) => {
        if (cmd === "get_session_state")
          return new Promise<SessionState>((resolve) => {
            release = resolve;
          });
      });
      const loading = session.load("project");
      await vi.waitFor(() => expect(release).toBeTypeOf("function"));
      if (action === "navigate") session.selectScene({ ...scene, id: "new" });
      else {
        session.open(null);
        session.open(action === "switch" ? "other" : "project");
      }
      release(saved);
      expect(await loading).toBeNull();
      expect(session.value?.current_scene_id).toBe(action === "navigate" ? "new" : undefined);
    }
  );

  it("handles missing state and storage failures without blocking navigation or later saves", async () => {
    expect(await session.load("project")).toBeNull();
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValue(new Error("disk unavailable"));
    expect(await session.load("project")).toBeNull();
    session.selectScene(scene);
    await expect(session.flush()).rejects.toThrow("disk unavailable");
    expect(error).toHaveBeenCalledTimes(1);
    vi.mocked(invoke).mockResolvedValue(null);
    session.update("project", scene.id, { cursor_position: 5 });
    await session.flush();
    expect(invoke).toHaveBeenLastCalledWith("save_session_state", {
      session: expect.objectContaining({ cursor_position: 5 }),
    });
    session.open(null);
    session.selectScene(scene);
    expect(session.value).toBeNull();
  });
});
