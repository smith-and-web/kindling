import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { SynopsisSaveQueue } from "./synopsisSaves.svelte";

const draft = { projectId: "project", sceneId: "scene", synopsis: "Draft" };
let saves: SynopsisSaveQueue;
beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(undefined);
  saves = new SynopsisSaveQueue();
});
afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

it.each(["database is locked", "disk full", "Cannot edit a locked scene"])(
  "retains %s failures and retries without further typing",
  async (error) => {
    vi.mocked(invoke).mockRejectedValue(error);
    saves.stage(draft);
    await vi.advanceTimersByTimeAsync(1000);
    expect(saves.getState(draft.projectId, draft.sceneId)).toEqual({ draft, error, saving: false });
    await expect(saves.flush()).rejects.toThrow(error);
    expect(saves.failedCount).toBe(1);
    vi.mocked(invoke).mockResolvedValue(undefined);
    await saves.flush();
    expect(saves.getState(draft.projectId, draft.sceneId).draft).toBeUndefined();
    expect(saves.failedCount).toBe(0);
  }
);

it.each([false, true])(
  "drains newer edits made during a save (older save fails: %s)",
  async (rejectOld) => {
    let finish!: () => void;
    vi.mocked(invoke).mockImplementationOnce(async () => {
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
      if (rejectOld) throw "disk full";
    });
    saves.stage(draft);
    const writing = saves.flush();
    await vi.advanceTimersByTimeAsync(0);
    saves.stage({ ...draft, synopsis: "Newer draft" });
    const closing = saves.flush();
    finish();
    await Promise.all([writing, closing]);
    expect(invoke).toHaveBeenLastCalledWith("save_scene_synopsis", {
      sceneId: draft.sceneId,
      synopsis: "Newer draft",
    });
    expect(saves.getState(draft.projectId, draft.sceneId).draft).toBeUndefined();
    expect(saves.failedCount).toBe(0);
    expect(invoke).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(1000);
    expect(invoke).toHaveBeenCalledTimes(2);
  }
);

it("retains an old project's failure without blocking another project's save", async () => {
  const other = { ...draft, projectId: "other-project", sceneId: "other-scene" };
  vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
    if ((args as { sceneId: string }).sceneId === draft.sceneId) throw "locked scene";
  });
  saves.stage(draft);
  saves.stage(other);
  await expect(saves.flush()).rejects.toThrow("locked scene");
  expect(saves.getState(other.projectId, other.sceneId).draft).toBeUndefined();
  expect(saves.getState(draft.projectId, draft.sceneId).draft).toEqual(draft);
  await expect(saves.flush(other.projectId, other.sceneId)).resolves.toBeUndefined();
});

it("keeps clearing a synopsis as a recoverable null draft", async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error("database is locked"));
  saves.stage({ ...draft, synopsis: null });
  await expect(saves.flush()).rejects.toThrow("database is locked");
  expect(saves.getState(draft.projectId, draft.sceneId).draft?.synopsis).toBeNull();
  await saves.flush();
  expect(invoke).toHaveBeenLastCalledWith("save_scene_synopsis", {
    sceneId: draft.sceneId,
    synopsis: null,
  });
});
