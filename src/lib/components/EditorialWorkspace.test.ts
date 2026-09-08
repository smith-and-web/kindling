import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, fireEvent, waitFor, cleanup } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type { Editor } from "@tiptap/core";
import { Transform } from "@tiptap/pm/transform";
import {
  manuscript,
  editorialSchema,
  trackChanges,
  type EditorialPackage,
  type EditorialFeedback,
} from "../utils/editorial";
import EditorialWorkspace from "./EditorialWorkspace.svelte";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

const source = {
  id: "a",
  scene_id: "s",
  chapter_id: "c",
  chapter: "Chapter One",
  scene: "The Letter",
  mode: "page" as const,
  html: "<p>Eleanor opened the letter.</p>",
  locked: false,
};
const round = {
  id: "round",
  project_id: "project",
  title: "The Letter",
  name: "First review",
  brief: "Check motivation",
  created_at: "2026-01-01",
  sources: [source],
};
const packageData: EditorialPackage = {
  format: "kindling-editorial",
  version: 1,
  kind: "review",
  round,
  session: null,
};
const prepareWriting = vi.fn().mockResolvedValue(undefined),
  onManuscriptChanged = vi.fn().mockResolvedValue(undefined);
let returned: EditorialFeedback;
function editor() {
  return (document.querySelector(".editorial-prose") as HTMLElement & { editor: Editor }).editor;
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  const storage = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => storage.get(k) ?? null,
    setItem: (k: string, v: string) => storage.set(k, v),
    removeItem: (k: string) => storage.delete(k),
  });
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
  HTMLElement.prototype.scrollIntoView = vi.fn();
  document.elementFromPoint = vi.fn().mockReturnValue(null);
  Range.prototype.getClientRects = vi.fn().mockReturnValue([]);
  Range.prototype.getBoundingClientRect = vi
    .fn()
    .mockReturnValue({ left: 0, right: 0, top: 0, bottom: 0 });
  returned = { round, sources: [source], entries: [], version: 1 };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "take_editorial_open_files") return [];
    if (command === "open_editorial_package") return structuredClone(packageData);
    if (command === "get_editorial_feedback" || command === "import_editorial_feedback")
      return structuredClone(returned);
    if (command === "editorial_sources") return [source];
    if (command === "list_editorial_rounds") return [round];
    if (command === "export_editorial_review") return round;
    if (command === "decide_editorial_feedback") {
      const decision = (args as { decision: string }).decision;
      return {
        ...returned,
        version: 2,
        entries: returned.entries.map((e) => ({ ...e, decision })),
      };
    }
    return undefined;
  });
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("editorial workspace", () => {
  it("shows an actionable error when a package fails before the workspace opens", async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "open_editorial_package") throw new Error("Unsupported review package version");
      return original(cmd, args);
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/future.kindling-review");
    expect((view.getByRole("dialog") as HTMLDialogElement).open).toBe(true);
    expect(view.getByRole("alert").textContent).toContain("Unsupported review package version");
    expect(view.getByText("Open review or feedback file…")).toBeTruthy();
    await fireEvent.click(view.getByText("Back to Kindling"));
    await waitFor(() => expect(view.component.isOpen()).toBe(false));
  });
  it("resumes separate saved suggestions in a substantial scene without changing their identities", async () => {
    const bookSource = {
      ...source,
      html: `<p>${"The tide reaches the lighthouse. ".repeat(3000)}</p>`,
    };
    const bookRound = { ...round, sources: [bookSource] };
    const base = manuscript(bookRound.sources);
    let doc = base;
    let changes: ReturnType<typeof trackChanges> = [];
    for (let i = 0; i < 20; i++) {
      doc = new Transform(doc).insert(
        1 + i * 4100,
        editorialSchema.text("Quietly. ".repeat(20))
      ).doc;
      changes = trackChanges(base, doc, changes);
    }
    expect(changes).toHaveLength(20);
    const saved = {
      reviewer_id: "rowan",
      name: "Rowan",
      generation: 8,
      document: doc.toJSON(),
      changes,
      position: 1,
    };
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) =>
      cmd === "open_editorial_package"
        ? { ...packageData, round: bookRound, session: saved, saved_generation: 8 }
        : original(cmd, args)
    );
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/saved.kindling-review");
    editor().commands.setTextSelection(editor().state.doc.content.size - 1);
    editor().commands.insertContent(" End.");
    await tick();
    await view.component.flush();
    const updated = (
      vi
        .mocked(invoke)
        .mock.calls.filter(([cmd]) => cmd === "save_editorial_session")
        .slice(-1)[0]![1] as {
        session: { changes: typeof changes };
      }
    ).session.changes;
    expect(updated).toHaveLength(21);
    for (const previous of changes) {
      expect(updated.find((c) => c.id === previous.id)?.revision).toBe(previous.revision);
    }
  });
  it("opens directly, types suggestions, undoes, saves and exports a resumable review", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("At dusk, ");
    await tick();
    expect(view.getByText("Insertion · open")).toBeTruthy();
    editor().commands.undo();
    await tick();
    expect(view.queryByText("Insertion · open")).toBeNull();
    editor().commands.redo();
    await tick();
    await view.component.flush();
    expect(invoke).toHaveBeenCalledWith(
      "save_editorial_session",
      expect.objectContaining({
        session: expect.objectContaining({ name: "Rowan", changes: expect.any(Array) }),
      })
    );
    vi.mocked(save).mockResolvedValue("/returned.kindling-feedback");
    await fireEvent.click(view.getByRole("button", { name: "Export feedback" }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "export_editorial_feedback",
        expect.objectContaining({ path: "/returned.kindling-feedback" })
      )
    );
    await fireEvent.click(view.getByText("Back to Kindling"));
    await waitFor(() => expect(view.component.isOpen()).toBe(false));
  });

  it("keeps focus in search while selecting a match and focuses comment composition", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    const search = view.getByLabelText("Find in manuscript");
    search.focus();
    await fireEvent.input(search, { target: { value: "letter" } });
    expect(document.activeElement).toBe(search);
    expect(
      editor().state.doc.textBetween(editor().state.selection.from, editor().state.selection.to)
    ).toBe("letter");
    await fireEvent.click(view.getByRole("button", { name: "Add comment" }));
    await tick();
    expect(document.activeElement).toBe(view.getByLabelText("Comment"));
    await view.component.flush();
  });
  it("exports a failed save as a reopenable review with a newer generation", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await view.component.flush();
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    await expect(view.component.flush()).rejects.toThrow("disk full");
    vi.mocked(save).mockResolvedValue("/recovery.kindling-review");
    await fireEvent.click(view.getByText("Export recovery copy"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "export_editorial_recovery",
        expect.objectContaining({
          path: "/recovery.kindling-review",
          session: expect.objectContaining({ generation: 3 }),
        })
      )
    );
    await view.component.flush();
  });
  it("adds contextual comments, replies, resolves and reopens them", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    editor().commands.setTextSelection({ from: 1, to: 8 });
    await fireEvent.click(view.getByRole("button", { name: "Add comment" }));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "What is Eleanor afraid of?" },
    });
    await fireEvent.click(view.getByText("Save comment"));
    expect(view.getByText("What is Eleanor afraid of?")).toBeTruthy();
    await fireEvent.input(view.getByLabelText("Reply"), {
      target: { value: "The letter should make this clearer." },
    });
    await fireEvent.click(view.getByRole("button", { name: "Reply" }));
    expect(view.getByText("The letter should make this clearer.")).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Resolve" }));
    await fireEvent.change(view.getByLabelText("Show", { exact: true }), {
      target: { value: "all" },
    });
    await fireEvent.click(view.getByRole("button", { name: "Reopen" }));
    expect(view.getByText("Comment · open")).toBeTruthy();
    await view.component.flush();
  });

  it("previews returned feedback before import and applies an explicit writer decision", async () => {
    const base = manuscript([source]);
    const proposed = manuscript([{ ...source, html: "<p>Eleanor burned the letter.</p>" }]);
    const change = trackChanges(base, proposed, [])[0];
    returned.entries = [{ key: "entry", reviewer: "Rowan", change, decision: "open" }];
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "take_editorial_open_files") return [];
      if (command === "open_editorial_package")
        return {
          ...packageData,
          kind: "feedback",
          session: {
            name: "Rowan",
            changes: [change],
            document: proposed.toJSON(),
            reviewer_id: "rowan",
            generation: 1,
            position: 1,
          },
        };
      if (command === "import_editorial_feedback") return returned;
      if (command === "decide_editorial_feedback")
        return {
          ...returned,
          version: 2,
          entries: [{ ...returned.entries[0], decision: (args as { decision: string }).decision }],
        };
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/feedback.kindling-feedback");
    expect(invoke).not.toHaveBeenCalledWith("import_editorial_feedback", expect.anything());
    await fireEvent.click(view.getByText("Import and review feedback"));
    await waitFor(() => expect(view.getByText("Rowan · suggestion · open")).toBeTruthy());
    await fireEvent.click(view.getByText("Rowan · suggestion · open"));
    await fireEvent.click(view.getByRole("button", { name: "Accept" }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "decide_editorial_feedback",
        expect.objectContaining({
          keys: ["entry"],
          decision: "accepted",
          replacements: [
            { id: "a", expected: source.html, html: "<p>Eleanor burned the letter.</p>" },
          ],
        })
      )
    );
  });

  it("withdraws a suggestion without moving unrelated comments, including undo and redo", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    editor().commands.setTextSelection({ from: 20, to: 26 });
    await fireEvent.click(view.getByText("Add comment"));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "Keep the letter tangible." },
    });
    await fireEvent.click(view.getByText("Save comment"));
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("At dusk, ");
    await tick();
    await fireEvent.click(view.getByText("Insertion · open"));
    await fireEvent.click(view.getByText("Withdraw suggestion"));
    await tick();
    await view.component.flush();
    const saved = () =>
      vi
        .mocked(invoke)
        .mock.calls.filter(([cmd]) => cmd === "save_editorial_session")
        .slice(-1)[0]![1] as { session: { changes: { kind: string; from: number; to: number }[] } };
    expect(saved().session.changes.find((c) => c.kind === "comment")).toMatchObject({
      from: 20,
      to: 26,
    });
    editor().commands.undo();
    await tick();
    await view.component.flush();
    expect(saved().session.changes.find((c) => c.kind === "comment")).toMatchObject({
      from: 20,
      to: 26,
    });
    editor().commands.redo();
    await tick();
    await view.component.flush();
    expect(saved().session.changes.find((c) => c.kind === "comment")).toMatchObject({
      from: 20,
      to: 26,
    });
  });
  it("keeps intervening comments anchored when withdrawing after a multi-paragraph paste", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("<p>First note.</p><p>Middle note.</p><p>Last note.</p>");
    expect(editor().state.doc.child(1).attrs.source).toBeNull();
    const originalStart = editor().state.doc.content.size - 27;
    editor().commands.setTextSelection({ from: originalStart + 19, to: originalStart + 25 });
    await fireEvent.click(view.getByText("Add comment"));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "Keep the letter tangible." },
    });
    await fireEvent.click(view.getByText("Save comment"));
    editor().commands.setTextSelection(editor().state.doc.content.size - 1);
    editor().commands.insertContent(" Later.");
    await tick();
    await fireEvent.click(view.getAllByText("Insertion · open").slice(-1)[0]!);
    await fireEvent.click(view.getByText("Withdraw suggestion"));
    await tick();
    await view.component.flush();
    const saved = () =>
      vi
        .mocked(invoke)
        .mock.calls.filter(([cmd]) => cmd === "save_editorial_session")
        .slice(-1)[0]![1] as {
        session: { changes: { kind: string; from: number; to: number }[] };
      };
    const comment = () => saved().session.changes.find((c) => c.kind === "comment");
    expect(comment()).toMatchObject({ from: 20, to: 26 });
    expect(editor().state.doc.textContent).toContain("Middle note.");
    expect(editor().state.doc.textContent).not.toContain(" Later.");
    editor().commands.undo();
    await tick();
    await view.component.flush();
    expect(comment()).toMatchObject({ from: 20, to: 26 });
    editor().commands.redo();
    await tick();
    await view.component.flush();
    expect(comment()).toMatchObject({ from: 20, to: 26 });
  });
  it("keeps writer reading context on decisions and exports replies for the selected editor", async () => {
    const base = manuscript([source]);
    const proposed = manuscript([{ ...source, html: "<p>Eleanor burned the letter.</p>" }]);
    returned.entries = [
      {
        key: "rowan/change/1",
        reviewer: "Rowan",
        change: trackChanges(base, proposed, [])[0],
        decision: "open",
      },
    ];
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    await fireEvent.click(view.getByText("First review"));
    await waitFor(() => expect(view.getByText("Rowan · suggestion · open")).toBeTruthy());
    await fireEvent.click(view.getByText("Rowan · suggestion · open"));
    const before = editor();
    const selection = before.state.selection;
    await fireEvent.click(view.getByRole("button", { name: "Reject" }));
    await waitFor(() => expect(view.getByText("Review decision saved.")).toBeTruthy());
    expect(editor()).toBe(before);
    expect(editor().state.selection.eq(selection)).toBe(true);
    vi.mocked(save).mockResolvedValue("/reply.kindling-review");
    await fireEvent.click(view.getByText("Export replies and decisions"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("export_editorial_reply", {
        roundId: "round",
        reviewerId: "rowan",
        path: "/reply.kindling-review",
      })
    );
  });

  it("remaps writer selection when acceptance changes prose above the reading position", async () => {
    const base = manuscript([source]);
    const nextSource = { ...source, html: "<p>At dusk, Eleanor opened the letter.</p>" };
    const next = manuscript([nextSource]);
    returned.entries = [
      {
        key: "rowan/change/1",
        reviewer: "Rowan",
        change: trackChanges(base, next, [])[0],
        decision: "open",
      },
    ];
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) =>
      cmd === "decide_editorial_feedback"
        ? {
            ...returned,
            sources: [nextSource],
            entries: [{ ...returned.entries[0], decision: "accepted" }],
            version: 2,
          }
        : original(cmd, args)
    );
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    await fireEvent.click(view.getByText("First review"));
    await waitFor(() => expect(view.getByText("Rowan · suggestion · open")).toBeTruthy());
    await fireEvent.click(view.getByText("Rowan · suggestion · open"));
    editor().commands.setTextSelection({ from: 20, to: 26 });
    await fireEvent.click(view.getByRole("button", { name: "Accept" }));
    await waitFor(() => expect(editor().state.doc.textContent).toContain("At dusk"));
    expect(editor().state.selection.from).toBe(29);
    expect(editor().state.selection.to).toBe(35);
  });
  it("requires an intentional selection before applying a conflicting suggestion", async () => {
    const base = manuscript([source]);
    const proposed = manuscript([{ ...source, html: "<p>Eleanor burned the letter.</p>" }]);
    returned.entries = [
      {
        key: "rowan/change/1",
        reviewer: "Rowan",
        change: trackChanges(base, proposed, [])[0],
        decision: "open",
      },
    ];
    returned.sources = [{ ...source, html: "<p>Eleanor hid the letter.</p>" }];
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    await fireEvent.click(view.getByText("First review"));
    await waitFor(() => expect(view.getByText("Rowan · suggestion · open")).toBeTruthy());
    await fireEvent.click(view.getByText("Rowan · suggestion · open"));
    const apply = view.getByText("Apply to selected passage") as HTMLButtonElement;
    expect(apply.disabled).toBe(true);
    editor().commands.setTextSelection({ from: 9, to: 12 });
    await fireEvent.mouseUp(document.querySelector(".editorial-prose")!);
    await tick();
    expect(apply.disabled).toBe(false);
    await fireEvent.click(apply);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "decide_editorial_feedback",
        expect.objectContaining({ decision: "accepted" })
      )
    );
  });

  it("records scroll-based reading position independently from the cursor", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    vi.spyOn(editor().view, "posAtCoords").mockReturnValue({ pos: 15, inside: 0 });
    await fireEvent.scroll(document.querySelector(".manuscript-column")!);
    await new Promise((resolve) => setTimeout(resolve, 250));
    await view.component.flush();
    expect(invoke).toHaveBeenCalledWith(
      "save_editorial_session",
      expect.objectContaining({
        session: expect.objectContaining({ reading_position: 15, position: 1 }),
      })
    );
  });
  it("keeps the workspace open with recoverable work when saving fails", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await view.component.flush();
    vi.mocked(invoke).mockRejectedValue(new Error("disk full"));
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    await fireEvent.click(view.getByText("Back to Kindling"));
    await waitFor(() => expect(view.getByRole("alert").textContent).toContain("disk full"));
    expect(view.component.isOpen()).toBe(true);
    expect(localStorage.getItem("kindling.editorial.recovery.round")).toContain("Rowan");
  });

  it("exports scoped review rounds and opens their saved feedback", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    vi.mocked(save).mockResolvedValue("/round.kindling-review");
    await fireEvent.click(view.getByLabelText("Chapter One"));
    await fireEvent.click(view.getByText("Export review package"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "export_editorial_review",
        expect.objectContaining({ projectId: "project", chapterIds: ["c"] })
      )
    );
    await fireEvent.click(view.getAllByText("First review")[0]);
    await waitFor(() => expect(view.getByText("Returned feedback")).toBeTruthy());
  });
});
