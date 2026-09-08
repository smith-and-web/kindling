import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { EditorialSaves } from "./editorialSaves";
import type { EditorialRound, EditorialSession } from "./editorial";

const round: EditorialRound = {
  id: "round",
  project_id: "project",
  name: "Pass",
  title: "Letter",
  brief: "",
  created_at: "",
  sources: [],
};
const draft: EditorialSession = {
  reviewer_id: "editor",
  name: "Rowan",
  generation: 0,
  document: { type: "doc" },
  changes: [],
  position: 1,
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  const data = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => data.set(key, value),
    removeItem: (key: string) => data.delete(key),
  });
});

describe("editorial save journal", () => {
  it("exposes only the acknowledged payload while another draft is pending", async () => {
    const queue = new EditorialSaves(round, null);
    vi.mocked(invoke).mockResolvedValue(undefined);
    queue.stage(draft);
    await queue.flush();
    queue.stage({ ...draft, name: "Still typing" });
    const saved = queue.savedSession()!;
    expect(saved.name).toBe("Rowan");
    expect(saved.generation).toBe(1);
    saved.name = "External mutation";
    expect(queue.savedSession()!.name).toBe("Rowan");
  });
  it("journals before debounce, coalesces edits, and advances acknowledged generations", async () => {
    const queue = new EditorialSaves(round, null);
    queue.stage(draft);
    queue.stage({ ...draft, name: "Newest" });
    expect(JSON.parse(localStorage.getItem(queue.key)!).session.name).toBe("Newest");
    vi.mocked(invoke).mockResolvedValue(undefined);
    expect(await queue.flush()).toBe(1);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith(
      "save_editorial_session",
      expect.objectContaining({
        expectedGeneration: null,
        session: expect.objectContaining({ generation: 1, name: "Newest" }),
      })
    );
    expect(localStorage.getItem(queue.key)).toBeNull();
    queue.stage(draft);
    expect(await queue.flush()).toBe(2);
    expect(await queue.flush()).toBe(2);
  });

  it("resumes a portable recovery on a fresh installation without rolling generations backward", async () => {
    const recovered = { ...draft, generation: 8 };
    const queue = new EditorialSaves(round, recovered, null);
    queue.stage(recovered);
    expect(queue.recoveryGeneration()).toBeGreaterThan(8);
    vi.mocked(invoke).mockResolvedValue(undefined);
    expect(await queue.flush()).toBe(9);
    expect(invoke).toHaveBeenCalledWith(
      "save_editorial_session",
      expect.objectContaining({
        expectedGeneration: null,
        session: expect.objectContaining({ generation: 9 }),
      })
    );
  });
  it("serializes in-flight saves without dropping a newer draft", async () => {
    let finish!: () => void;
    vi.mocked(invoke)
      .mockImplementationOnce(
        () =>
          new Promise<void>((resolve) => {
            finish = resolve;
          })
      )
      .mockResolvedValue(undefined);
    const queue = new EditorialSaves(round, null);
    queue.stage(draft);
    const first = queue.flush();
    queue.stage({ ...draft, name: "Newer" });
    const second = queue.flush();
    finish();
    await Promise.all([first, second]);
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(vi.mocked(invoke).mock.calls[1][1]).toMatchObject({
      expectedGeneration: 1,
      session: { generation: 2, name: "Newer" },
    });
  });

  it("retains work on failure, supports retry, and restores interrupted work", async () => {
    const queue = new EditorialSaves(round, { ...draft, generation: 4 });
    expect(queue.recover({ ...draft, generation: 4 })?.generation).toBe(4);
    queue.stage({ ...draft, name: "Recovery" });
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));
    await expect(queue.flush()).rejects.toThrow("disk full");
    const resumed = new EditorialSaves(round, { ...draft, generation: 4 });
    expect(resumed.recover({ ...draft, generation: 4 })?.name).toBe("Recovery");
    vi.mocked(invoke).mockResolvedValue(undefined);
    expect(await resumed.flush()).toBe(5);
    expect(localStorage.getItem(queue.key)).toBeNull();
  });

  it("keeps newly received discussions alongside interrupted edits and honors writer version order", async () => {
    const note = {
      id: "note",
      revision: 1,
      kind: "comment" as const,
      from: 1,
      to: 1,
      before: null,
      after: null,
      state: "open" as const,
      messages: [],
      writer_decision: "open",
    };
    const queue = new EditorialSaves(round, { ...draft, generation: 4 });
    queue.stage({
      ...draft,
      writer_version: 3,
      changes: [note],
      document: { type: "doc", content: [] },
    });
    const message = {
      id: "reply",
      author: "Writer",
      text: "Clarified the motivation.",
      created_at: "today",
    };
    const received = {
      ...draft,
      generation: 4,
      writer_version: 2,
      changes: [
        { ...note, writer_decision: "resolved", messages: [message] },
        { ...note, id: "discussion-old-1", messages: [message] },
        { ...note, id: "withdrawn", kind: "suggestion" as const, messages: [message] },
      ],
    };
    const recovered = queue.recover(received)!;
    expect(recovered.document).toEqual({ type: "doc", content: [] });
    expect(recovered.changes[0].writer_decision).toBe("open");
    expect(recovered.writer_version).toBe(3);
    expect(recovered.changes.map((c) => c.id)).toEqual([
      "note",
      "discussion-old-1",
      "discussion-withdrawn",
    ]);
    expect(recovered.changes[0].messages).toEqual([message]);
    queue.discard();
    expect(localStorage.getItem(queue.key)).toBeNull();
    expect(await queue.flush()).toBe(4);
    expect(invoke).not.toHaveBeenCalled();
  });
  it("keeps newer journal discussion metadata when a recovery has newer prose", () => {
    const change = {
      id: "note",
      revision: 1,
      kind: "comment" as const,
      from: 1,
      to: 1,
      before: null,
      after: null,
      state: "open" as const,
      messages: [
        { id: "new-reply", author: "Writer", text: "Please reopen this.", created_at: "today" },
      ],
      writer_decision: "open",
    };
    const saved = {
      ...draft,
      generation: 7,
      writer_version: 2,
      document: { type: "doc", content: [] },
      changes: [{ ...change, messages: [], writer_decision: "resolved" }],
    };
    const queue = new EditorialSaves(round, saved, 5);
    localStorage.setItem(
      queue.key,
      JSON.stringify({
        round,
        session: { ...draft, generation: 6, writer_version: 3, changes: [change] },
      })
    );
    const recovered = queue.recover(saved)!;
    expect(recovered.document).toEqual(saved.document);
    expect(recovered.generation).toBe(7);
    expect(recovered.writer_version).toBe(3);
    expect(recovered.changes[0].writer_decision).toBe("open");
    expect(recovered.changes[0].messages).toEqual(change.messages);
  });
  it("never substitutes a different manuscript, reviewer, or older generation", () => {
    const queue = new EditorialSaves(round, null);
    queue.stage(draft);
    const newer = { ...draft, generation: 10 };
    expect(queue.recover(newer)).toBe(newer);
    const other = { ...draft, reviewer_id: "other" };
    expect(queue.recover(other)).toBe(other);
    localStorage.setItem(
      queue.key,
      JSON.stringify({ round: { ...round, brief: "different" }, session: draft })
    );
    expect(() => queue.recover(draft)).toThrow("different manuscript");
  });
});
