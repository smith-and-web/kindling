import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { proseSaves } from "../utils/proseSaves";
import { invoke } from "@tauri-apps/api/core";
import FindReplaceDialog from "./FindReplaceDialog.svelte";
import * as proseSearch from "../utils/proseSearch";
import type { ProseDocument } from "../utils/proseSearch";

const docs: ProseDocument[] = [
  {
    id: "beat",
    scene_id: "scene",
    chapter_id: "chapter",
    chapter_title: "Chapter One",
    scene_title: "Arrival",
    beat_title: "Greeting",
    prose: "<p>Al<strong>ice</strong> met Alice.</p>",
    locked: false,
  },
  {
    id: "page",
    scene_id: "other",
    chapter_id: "chapter",
    chapter_title: "Chapter One",
    scene_title: "Departure",
    beat_title: null,
    prose: "<p>alice leaves.</p>",
    locked: false,
  },
  {
    id: "locked",
    scene_id: "locked",
    chapter_id: "chapter",
    chapter_title: "Chapter One",
    scene_title: "Locked scene",
    beat_title: null,
    prose: "<p>Alice waits.</p>",
    locked: true,
  },
];
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents" ? structuredClone(docs) : undefined
  );
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});
async function open(props: Partial<Parameters<typeof FindReplaceDialog>[1]> = {}) {
  const prepare = vi.fn().mockResolvedValue(undefined);
  const onApplied = vi.fn();
  const onClose = vi.fn();
  render(FindReplaceDialog, {
    projectId: "project",
    sceneId: "scene",
    showReplace: true,
    prepare,
    onApplied,
    onClose,
    ...props,
  });
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("get_search_documents", { projectId: "project" })
  );
  await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Alice" } });
  return { prepare, onApplied, onClose };
}

describe("Find and Replace", () => {
  it("flushes before loading, scopes to current scene, navigates and resets results", async () => {
    let finish!: () => void;
    const prepare = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        })
    );
    render(FindReplaceDialog, {
      projectId: "project",
      sceneId: "scene",
      prepare,
      onApplied: vi.fn(),
      onClose: vi.fn(),
    });
    expect(prepare).toHaveBeenCalled();
    expect(invoke).not.toHaveBeenCalled();
    finish();
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Alice" } });
    expect(screen.getByText("2 matches")).toBeTruthy();
    expect(screen.getByText(/1 of 2/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Previous" }));
    expect(screen.getByText(/2 of 2/)).toBeTruthy();
    await fireEvent.keyDown(screen.getByLabelText("Find"), { key: "Enter" });
    expect(screen.getByText(/1 of 2/)).toBeTruthy();
    await fireEvent.change(screen.getByLabelText("Search in"), { target: { value: "project" } });
    expect(screen.getByText("4 matches")).toBeTruthy();
    await fireEvent.click(screen.getByLabelText("Match case"));
    expect(screen.getByText("3 matches")).toBeTruthy();
    await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Ali" } });
    await fireEvent.click(screen.getByLabelText("Whole words"));
    expect(screen.getByText("No matches found.")).toBeTruthy();
  });
  it("focuses Find after asynchronous loading enables the fieldset", async () => {
    let finish!: (docs: ProseDocument[]) => void;
    vi.mocked(invoke).mockImplementation(
      () => new Promise<ProseDocument[]>((resolve) => (finish = resolve))
    );
    render(FindReplaceDialog, {
      projectId: "project",
      sceneId: "scene",
      prepare: async () => {},
      onApplied: vi.fn(),
      onClose: vi.fn(),
    });
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    expect(screen.getByLabelText("Find").closest("fieldset")?.disabled).toBe(true);
    finish(docs);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  });

  it("parses documents once, reuses text while typing, and reparses only replaced documents", async () => {
    const parse = vi.spyOn(proseSearch, "proseText");
    await open({ initialScope: "project" });
    expect(parse).toHaveBeenCalledTimes(docs.length);
    for (const value of ["A", "Al", "Alice"]) {
      await fireEvent.input(screen.getByLabelText("Find"), { target: { value } });
    }
    await fireEvent.click(screen.getByLabelText("Match case"));
    expect(parse).toHaveBeenCalledTimes(docs.length);
    await fireEvent.click(screen.getByText("Replace match"));
    await screen.findByText("Replacement saved.");
    expect(parse).toHaveBeenCalledTimes(docs.length + 1);
    expect(screen.getByText("2 matches")).toBeTruthy();
  });

  it("clears an earlier success message when a later replacement fails", async () => {
    await open();
    await fireEvent.click(screen.getByText("Replace match"));
    await screen.findByText("Replacement saved.");
    vi.mocked(invoke).mockRejectedValue("Disk full");
    await fireEvent.click(screen.getByText("Undo replacement"));
    await screen.findByRole("alert");
    expect(screen.queryByText("Replacement saved.")).toBeNull();
  });

  it("replaces one literal match, updates the editor, and undoes with expected prose", async () => {
    const { onApplied } = await open();
    await fireEvent.input(screen.getByLabelText("Replace with"), { target: { value: "<$&>" } });
    await fireEvent.click(screen.getByText("Replace match"));
    await screen.findByText("Replacement saved.");
    const changed = "<p>&lt;$&amp;&gt;<strong></strong> met Alice.</p>";
    expect(invoke).toHaveBeenLastCalledWith("replace_prose_batch", {
      projectId: "project",
      changes: [{ id: "beat", expected_prose: docs[0].prose, prose: changed }],
    });
    expect(onApplied).toHaveBeenCalled();
    expect(screen.getByText("1 match")).toBeTruthy();
    await fireEvent.click(screen.getByText("Undo replacement"));
    await screen.findByText("Replacement undone.");
    expect(invoke).toHaveBeenLastCalledWith("replace_prose_batch", {
      projectId: "project",
      changes: [{ id: "beat", expected_prose: changed, prose: docs[0].prose }],
    });
    expect(screen.getByText("2 matches")).toBeTruthy();
  });
  it("confirms project replacement, skips locked prose, and supports deletion", async () => {
    await open({ initialScope: "project" });
    await fireEvent.click(screen.getByText("Replace all"));
    expect(screen.getByText(/Replace 3 matches in the entire project/)).toBeTruthy();
    expect(invoke).toHaveBeenCalledTimes(1);
    await fireEvent.click(screen.getByText("Cancel"));
    await fireEvent.click(screen.getByText("Replace all"));
    await fireEvent.click(screen.getByText("Confirm replace all"));
    await screen.findByText("Replacement saved.");
    expect(invoke).toHaveBeenLastCalledWith("replace_prose_batch", {
      projectId: "project",
      changes: [
        { id: "beat", expected_prose: docs[0].prose, prose: "<p><strong></strong> met .</p>" },
        { id: "page", expected_prose: docs[1].prose, prose: "<p> leaves.</p>" },
      ],
    });
    expect((screen.getByText("Replace match") as HTMLButtonElement).disabled).toBe(true);
  });
  it("protects recovered documents while allowing other replacements and explicit recovery", async () => {
    const draft = {
      projectId: "project",
      kind: "beat" as const,
      id: "beat",
      prose: "<p>Unsaved Alice</p>",
    };
    vi.mocked(invoke).mockRejectedValueOnce("Cannot edit beats in a locked scene");
    await expect(proseSaves.save(draft)).rejects.toBe("Cannot edit beats in a locked scene");
    const { onApplied } = await open({ initialScope: "project" });
    expect((screen.getByText("Replace match") as HTMLButtonElement).disabled).toBe(true);
    await fireEvent.click(screen.getByText("Replace all"));
    expect(screen.getByText(/Replace 1 matches in the entire project/)).toBeTruthy();
    await fireEvent.click(screen.getByText("Confirm replace all"));
    await screen.findByText("Replacement saved.");
    expect(invoke).toHaveBeenLastCalledWith("replace_prose_batch", {
      projectId: "project",
      changes: [{ id: "page", expected_prose: docs[1].prose, prose: "<p> leaves.</p>" }],
    });
    await fireEvent.click(screen.getByText("Retry saving drafts"));
    await waitFor(() => expect(onApplied).toHaveBeenCalledWith([draft]));
    expect(proseSaves.draftsForRecovery("project")).toEqual([]);
    await waitFor(() =>
      expect((screen.getByText("Replace match") as HTMLButtonElement).disabled).toBe(false)
    );
  });

  it("opens the selected scene and reports navigation errors without closing", async () => {
    const onOpenScene = vi
      .fn()
      .mockRejectedValueOnce("Scene unavailable")
      .mockResolvedValue(undefined);
    const { onClose } = await open({ onOpenScene });
    await fireEvent.click(screen.getByText("Open scene"));
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", "Scene unavailable");
    expect(onClose).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByText("Open scene"));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(onOpenScene).toHaveBeenLastCalledWith(docs[0]);
  });

  it("keeps results and undo intact on failed writes", async () => {
    const { onApplied } = await open();
    vi.mocked(invoke).mockRejectedValue("Prose changed since searching.");
    await fireEvent.click(screen.getByText("Replace match"));
    expect(await screen.findByRole("alert")).toHaveProperty(
      "textContent",
      "Prose changed since searching."
    );
    expect(onApplied).not.toHaveBeenCalled();
    expect(screen.getByText("2 matches")).toBeTruthy();
    expect((screen.getByText("Undo replacement") as HTMLButtonElement).disabled).toBe(true);
  });
  it("shows save/load failures and allows closing without discarding prose", async () => {
    const onClose = vi.fn();
    render(FindReplaceDialog, {
      projectId: "project",
      sceneId: null,
      prepare: vi.fn().mockRejectedValue("Save failed"),
      onApplied: vi.fn(),
      onClose,
    });
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", "Save failed");
    expect(invoke).not.toHaveBeenCalled();
    await fireEvent(screen.getByRole("dialog"), new Event("cancel", { cancelable: true }));
    expect(onClose).toHaveBeenCalled();
  });
  it("uses project scope with no scene, handles no-op replacement, and closes", async () => {
    const { onClose } = await open({ sceneId: null });
    expect(screen.getByText("4 matches")).toBeTruthy();
    await fireEvent.input(screen.getByLabelText("Replace with"), { target: { value: "Alice" } });
    // The first match spans markup, so use the next, already identical text node.
    await fireEvent.click(screen.getByText("Next"));
    await fireEvent.click(screen.getByText("Replace match"));
    expect(await screen.findByText("No changes needed.")).toBeTruthy();
    await fireEvent.click(screen.getByLabelText("Close Find and Replace"));
    expect(onClose).toHaveBeenCalled();
  });
});

it("advances past replacement text containing the query and stops before wrapping", async () => {
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents"
      ? [{ ...docs[0], prose: "<p>in the room in the hall</p>" }]
      : undefined
  );
  await open();
  await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "in" } });
  await fireEvent.input(screen.getByLabelText("Replace with"), { target: { value: "into" } });
  await fireEvent.click(screen.getByText("Replace match"));
  await screen.findByText("Replacement saved.");
  expect(screen.getByText(/2 of 2/)).toBeTruthy();
  await fireEvent.click(screen.getByText("Replace match"));
  await waitFor(() =>
    expect((screen.getByText("Replace match") as HTMLButtonElement).disabled).toBe(true)
  );
  const writes = vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "replace_prose_batch");
  expect(writes[1][1]).toEqual({
    projectId: "project",
    changes: [
      {
        id: "beat",
        expected_prose: "<p>into the room in the hall</p>",
        prose: "<p>into the room into the hall</p>",
      },
    ],
  });
  await fireEvent.click(screen.getByText("Previous"));
  expect(screen.getByText(/2 of 2/)).toBeTruthy();
  await fireEvent.click(screen.getByText("Next"));
  expect(screen.getByText(/1 of 2/)).toBeTruthy();
});

it("advances to the next document after replacing its last match", async () => {
  await open({ initialScope: "project" });
  await fireEvent.input(screen.getByLabelText("Replace with"), { target: { value: "Alice!" } });
  await fireEvent.click(screen.getByText("Next"));
  await fireEvent.click(screen.getByText("Replace match"));
  await screen.findByText("Replacement saved.");
  expect(screen.getByText(/Departure/)).toBeTruthy();
});

it.each(["prepare", "documents"])(
  "can unmount while %s is pending without a late focus or rejection",
  async (stage) => {
    let finish!: () => void;
    const pending = new Promise<void>((resolve) => (finish = resolve));
    const prepare = stage === "prepare" ? () => pending : async () => {};
    if (stage === "documents")
      vi.mocked(invoke).mockImplementation(async () => {
        await pending;
        return docs;
      });
    const { unmount } = render(FindReplaceDialog, {
      projectId: "project",
      sceneId: "scene",
      prepare,
      onApplied: vi.fn(),
      onClose: vi.fn(),
    });
    if (stage === "documents") await waitFor(() => expect(invoke).toHaveBeenCalled());
    const focus = vi.spyOn(screen.getByLabelText("Find"), "focus");
    unmount();
    finish();
    await pending;
    await tick();
    expect(focus).not.toHaveBeenCalled();
    if (stage === "prepare") expect(invoke).not.toHaveBeenCalled();
  }
);

it("keeps terminal drafts recoverable without blocking search or automatically retrying them", async () => {
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_beat_prose") throw "Beat not found";
    return docs;
  });
  await expect(
    proseSaves.save({
      projectId: "recovery",
      kind: "beat",
      id: "removed",
      prose: "<p>Keep this draft</p>",
    })
  ).rejects.toBe("Beat not found");
  const onDiscardDrafts = vi.fn(async (drafts) => proseSaves.discard(drafts));
  const props = {
    projectId: "recovery",
    sceneId: null,
    prepare: async () => {
      await proseSaves.flush("recovery");
    },
    onApplied: vi.fn(),
    onClose: vi.fn(),
    onDiscardDrafts,
  };
  const first = render(FindReplaceDialog, props);
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  expect(screen.queryByRole("alert")).toBeNull();
  first.unmount();
  render(FindReplaceDialog, props);
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_beat_prose")).toHaveLength(1);
  expect((screen.getByLabelText("Draft text (select to copy)") as HTMLTextAreaElement).value).toBe(
    "Keep this draft\n"
  );
  await fireEvent.input(screen.getByLabelText("Find"), { target: { value: "Alice" } });
  expect(screen.getByText("4 matches")).toBeTruthy();
  await fireEvent.click(screen.getByText("Retry saving drafts"));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_beat_prose")).toHaveLength(2);
  expect(proseSaves.pendingFor("recovery")).toEqual([]);
  await fireEvent.click(screen.getByText("Discard unsaved drafts…"));
  await fireEvent.click(screen.getByText("Keep drafts"));
  expect(proseSaves.draftsForRecovery("recovery")).toHaveLength(1);
  await fireEvent.click(screen.getByText("Discard unsaved drafts…"));
  await fireEvent.click(screen.getByText("Confirm discard drafts"));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  expect(proseSaves.draftsForRecovery("recovery")).toEqual([]);
});

it("retries loading successfully after a transient failure without reopening", async () => {
  const prepare = vi.fn().mockRejectedValueOnce("Save failed").mockResolvedValue(undefined);
  render(FindReplaceDialog, {
    projectId: "project",
    sceneId: "scene",
    prepare,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  await screen.findByRole("alert");
  await fireEvent.click(screen.getByText("Retry loading"));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Find")));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(prepare).toHaveBeenCalledTimes(2);
});
