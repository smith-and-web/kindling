import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ProseSaveQueue } from "./proseSaves";
const save = { projectId: "project", kind: "beat" as const, id: "beat", prose: "Latest draft" };
beforeEach(() => {
  vi.mocked(invoke).mockReset();
});
it("retains failed saves for retry, isolates projects, and recovers after failure", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(queue.save(save)).rejects.toBe("Disk full");
  await queue.flush("other");
  expect(invoke).toHaveBeenCalledTimes(1);
  await expect(queue.flush("project")).rejects.toThrow("Could not save");
  vi.mocked(invoke).mockResolvedValue(undefined);
  await queue.flush("project");
  expect(invoke).toHaveBeenLastCalledWith("save_beat_prose", {
    beatId: "beat",
    prose: "Latest draft",
  });
  const count = vi.mocked(invoke).mock.calls.length;
  await queue.flush("project");
  expect(invoke).toHaveBeenCalledTimes(count);
});
it("flushes drafts across projects on exit and retains terminal failures for confirmation", async () => {
  const queue = new ProseSaveQueue();
  const page = { ...save, projectId: "other", kind: "page" as const, id: "scene" };
  const locked = { ...save, projectId: "third", id: "locked" };
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(queue.save(save)).rejects.toBe("Disk full");
  await expect(queue.save(page)).rejects.toBe("Disk full");
  vi.mocked(invoke).mockRejectedValue("Cannot edit a locked scene");
  await expect(queue.save(locked)).rejects.toBe("Cannot edit a locked scene");
  expect(queue.draftsForRecovery()).toEqual(expect.arrayContaining([save, page, locked]));
  vi.mocked(invoke).mockResolvedValue(undefined);
  expect(await queue.flush()).toEqual([save, page]);
  expect(queue.pendingFor()).toEqual([]);
  expect(queue.draftsForRecovery()).toEqual([locked]);
});
it("serializes writes and retains the newest failed draft when an earlier write succeeds", async () => {
  const queue = new ProseSaveQueue();
  let finish!: () => void;
  vi.mocked(invoke)
    .mockImplementationOnce(() => new Promise<void>((resolve) => (finish = resolve)))
    .mockRejectedValue("Disk full");
  const first = queue.save(save);
  const second = queue.save({ ...save, kind: "page", prose: "Newer draft" });
  await Promise.resolve();
  expect(invoke).toHaveBeenCalledTimes(1);
  finish();
  await first;
  await expect(second).rejects.toBe("Disk full");
  vi.mocked(invoke).mockResolvedValue(undefined);
  await queue.flush("project");
  expect(invoke).toHaveBeenLastCalledWith("save_scene_page_prose", {
    sceneId: "beat",
    prose: "Newer draft",
  });
});

it("allows explicit discard of permanent failures without affecting other projects", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue("Beat not found");
  await expect(queue.save(save)).rejects.toBe("Beat not found");
  await expect(queue.save({ ...save, projectId: "other", id: "other-beat" })).rejects.toBe(
    "Beat not found"
  );
  await queue.discard(queue.draftsForRecovery("project"));
  await queue.flush("project");
  expect(queue.pendingFor("other")).toEqual([]);
  expect(queue.draftsForRecovery("other")).toHaveLength(1);
});

it("does not discard a newer draft using an outdated recovery confirmation", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue("Cannot edit a locked scene");
  await expect(queue.save(save)).rejects.toBe("Cannot edit a locked scene");
  const snapshot = queue.draftsForRecovery("project");
  await expect(queue.save({ ...save, prose: "Newer text" })).rejects.toBe(
    "Cannot edit a locked scene"
  );
  await expect(queue.discard(snapshot)).rejects.toThrow("Unsaved drafts changed");
  expect(queue.draftsForRecovery("project")[0].prose).toBe("Newer text");
});

it("cleans editor copies only after validating the discard snapshot", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue("Beat not found");
  await expect(queue.save(save)).rejects.toBe("Beat not found");
  const cleanup = vi.fn(() => expect(queue.draftsForRecovery("project")).toEqual([]));
  await expect(queue.discard([{ ...save }], cleanup)).rejects.toThrow("Unsaved drafts changed");
  expect(cleanup).not.toHaveBeenCalled();
  await queue.discard([save], cleanup);
  expect(cleanup).toHaveBeenCalledOnce();
});

it.each([
  "Beat not found",
  "Scene not found",
  "Cannot edit beats in a locked scene",
  "Cannot edit a locked scene",
  "Query returned no rows",
  "Unexpected backend rejection",
])("retires %s from pending and does not requeue the same draft", async (error) => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue(error);
  await expect(queue.save(save)).rejects.toBe(error);
  expect(queue.pendingFor("project")).toEqual([]);
  await expect(queue.flush("project")).resolves.toEqual([]);
  await expect(queue.save({ ...save })).rejects.toBe(error);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(queue.draftsForRecovery("project")).toEqual([save]);
});

it.each([
  "database is busy",
  "database table is locked",
  "database or disk is full",
  "disk I/O error",
  "unable to open database file",
])("keeps known storage error %s retryable", async (error) => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue(error);
  await expect(queue.save(save)).rejects.toBe(error);
  expect(queue.pendingFor("project")).toEqual([save]);
  await expect(queue.flush("project")).rejects.toThrow("Could not save");
});

it("keeps database lock failures retryable and reports only successfully flushed writes", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue(new Error("database is locked"));
  await expect(queue.save(save)).rejects.toThrow("database is locked");
  expect(queue.pendingFor("project")).toEqual([save]);
  vi.mocked(invoke).mockRejectedValue(new Error("Beat not found"));
  expect(await queue.flush("project")).toEqual([]);
  expect(queue.pendingFor("project")).toEqual([]);
  expect(queue.draftsForRecovery("project")).toEqual([save]);
});

it("allows explicit retry after unlocking and returns the recovered write for refreshing the editor", async () => {
  const queue = new ProseSaveQueue();
  vi.mocked(invoke).mockRejectedValue("Cannot edit a locked scene");
  await expect(queue.save(save)).rejects.toBe("Cannot edit a locked scene");
  await expect(queue.save({ ...save, projectId: "other", id: "other" })).rejects.toBe(
    "Cannot edit a locked scene"
  );
  await queue.retryRecovered("project");
  expect(queue.pendingFor("project")).toEqual([]);
  vi.mocked(invoke).mockResolvedValue(undefined);
  expect(await queue.retryRecovered("project")).toEqual([save]);
  expect(queue.draftsForRecovery("project")).toEqual([]);
  expect(queue.draftsForRecovery("other")).toHaveLength(1);
});

it("does not let a late terminal failure retire a newer queued write", async () => {
  const queue = new ProseSaveQueue();
  let reject!: (reason: string) => void;
  vi.mocked(invoke)
    .mockImplementationOnce(() => new Promise((_resolve, fail) => (reject = fail)))
    .mockResolvedValue(undefined);
  const first = queue.save(save);
  const second = queue.save({ ...save, prose: "Newer" });
  await Promise.resolve();
  reject("Beat not found");
  await expect(first).rejects.toBe("Beat not found");
  await second;
  expect(queue.pendingFor("project")).toEqual([]);
  expect(queue.draftsForRecovery("project")).toEqual([]);
});

it("publishes successful retries even when a later draft keeps the flush blocked", async () => {
  const queue = new ProseSaveQueue();
  const second = { ...save, id: "second" };
  vi.mocked(invoke).mockRejectedValue("Disk full");
  await expect(queue.save(save)).rejects.toBe("Disk full");
  await expect(queue.save(second)).rejects.toBe("Disk full");
  vi.mocked(invoke).mockResolvedValueOnce(undefined);
  const onSaved = vi.fn();
  await expect(queue.flush("project", onSaved)).rejects.toThrow("Could not save");
  expect(onSaved).toHaveBeenCalledExactlyOnceWith(save);
  expect(queue.pendingFor("project")).toEqual([second]);
  vi.mocked(invoke).mockResolvedValue(undefined);
  await queue.flush("project", onSaved);
  expect(onSaved.mock.calls).toEqual([[save], [second]]);
});
