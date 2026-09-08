import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, fireEvent, waitFor, cleanup } from "@testing-library/svelte";
import { tick, flushSync } from "svelte";
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
import type { SceneReview } from "../utils/revisions";
import EditorialWorkspace from "./EditorialWorkspace.svelte";
import EditorialManuscript from "./EditorialManuscript.svelte";

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
let localReview: SceneReview;
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
  localReview = {
    scene_id: "s",
    version: 0,
    mode: "page",
    documents: [{ id: "s", label: "Page", html: source.html }],
    data: { status: "first_draft", drafts: [], annotations: [] },
  };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "take_editorial_open_files") return [];
    if (command === "open_local_editorial_review") return structuredClone(packageData);
    if (command === "get_project_scene_reviews") return [structuredClone(localReview)];
    if (command === "get_scene_review") return structuredClone(localReview);
    if (command === "save_scene_review") {
      const update = args as {
        data: SceneReview["data"];
        next: { documents: SceneReview["documents"] } | null;
      };
      localReview.data = update.data;
      if (update.next) localReview.documents = update.next.documents;
      localReview.version++;
      return structuredClone(localReview);
    }
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
  it("keeps restored reading position ahead of a queued cursor reveal", async () => {
    const view = render(EditorialManuscript, {
      sources: [source],
      initial: manuscript([source]).toJSON(),
      onChange: vi.fn(),
      onSelection: vi.fn(),
      onComment: vi.fn(),
      onError: vi.fn(),
      onAnnotation: vi.fn(),
      onReadingPosition: vi.fn(),
    });
    const pane = document.querySelector<HTMLElement>(".editorial-sheet")!.parentElement!;
    vi.spyOn(pane, "getBoundingClientRect").mockReturnValue({ top: 0, bottom: 400 } as ReturnType<
      HTMLElement["getBoundingClientRect"]
    >);
    vi.spyOn(editor().view, "coordsAtPos").mockImplementation((position) => ({
      top: position === 15 ? 1200 : -500,
      bottom: position === 15 ? 1220 : -480,
      left: 0,
      right: 10,
    }));
    view.component.select(1, 1, false);
    view.component.restoreReadingPosition(15);
    const restored = pane.scrollTop;
    await tick();
    expect(restored).toBeGreaterThan(1000);
    expect(pane.scrollTop).toBe(restored);
  });
  it("scrolls the read-only manuscript to feedback selected in the sidebar", async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation((cmd, args) =>
      cmd === "editorial_sources" ? Promise.resolve([{ ...source, id: "s" }]) : original(cmd, args)
    );
    localReview.data.annotations.push({
      id: "late-note",
      document_id: "s",
      anchor_html: source.html,
      from: 1,
      to: 8,
      quote: "Eleanor",
      replacement: null,
      state: "open",
      messages: [{ author: "Editor", text: "Check this passage", created_at: "2026-01-01" }],
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    const pane = document.querySelector<HTMLElement>(".manuscript-column")!;
    vi.spyOn(pane, "getBoundingClientRect").mockReturnValue({ top: 0, bottom: 400 } as ReturnType<
      HTMLElement["getBoundingClientRect"]
    >);
    vi.spyOn(editor().view, "coordsAtPos").mockReturnValue({
      top: 1200,
      bottom: 1220,
      left: 0,
      right: 10,
    });
    expect(editor().isEditable).toBe(false);
    await fireEvent.click(view.getByRole("button", { name: /Check this passage/ }));
    await waitFor(() => expect(pane.scrollTop).toBeGreaterThan(1000));
    expect(
      editor().state.doc.textBetween(editor().state.selection.from, editor().state.selection.to)
    ).toBe("Eleanor");
  });
  it("uses the shared manuscript menu and dismisses on outside pointer presses and actions", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    const trigger = view.getByLabelText("Manuscript actions");
    // Native event dispatch can flush Svelte's mount before the opening click
    // reaches window. Reproduce that timing instead of relying on batched events.
    const mountBeforeWindow = () => flushSync();
    document.addEventListener("click", mountBeforeWindow);
    try {
      await fireEvent.click(trigger.querySelector("svg")!);
    } finally {
      document.removeEventListener("click", mountBeforeWindow);
    }
    expect(view.getByRole("menu")).toBeTruthy();
    await fireEvent.click(trigger);
    expect(view.queryByRole("menu")).toBeNull();
    await fireEvent.click(trigger);
    const manuscript = document.querySelector<HTMLElement>(".editorial-prose")!;
    manuscript.addEventListener("pointerdown", (e) => e.stopPropagation(), { once: true });
    await fireEvent.pointerDown(manuscript);
    expect(view.queryByRole("menu")).toBeNull();
    await fireEvent.click(trigger);
    await fireEvent.keyDown(view.getByRole("menuitem", { name: "Draft history" }), {
      key: "Escape",
    });
    expect(view.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    await fireEvent.click(trigger);
    await fireEvent.click(view.getByRole("menuitem", { name: "Draft history" }));
    await waitFor(() => expect(view.getByRole("dialog")).toBeTruthy());
    expect(view.queryByRole("menu")).toBeNull();
  });
  it("keeps review options open while editing identity and dismisses on Escape, outside click, or an action", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    const trigger = view.getByLabelText("Review options");
    const menu = trigger.closest("details")!;
    await fireEvent.click(trigger);
    expect(menu.open).toBe(true);
    expect(view.queryByRole("group", { name: "Current review round" })).toBeNull();
    await fireEvent.input(view.getByLabelText("Your name"), { target: { value: "Rowan" } });
    await fireEvent.click(view.getByLabelText("Your name"));
    expect(menu.open).toBe(true);
    await fireEvent.keyDown(view.getByLabelText("Your name"), { key: "Escape" });
    expect(menu.open).toBe(false);
    expect(document.activeElement).toBe(trigger);
    await fireEvent.click(trigger);
    await fireEvent.click(view.getByLabelText("Show feedback"));
    expect(menu.open).toBe(false);
    await fireEvent.click(trigger);
    await fireEvent.click(view.getByRole("button", { name: "Refresh manuscript" }));
    expect(menu.open).toBe(false);
    await fireEvent.click(trigger);
    const manuscriptActions = view.getByLabelText("Manuscript actions");
    // Keyboard activation sends click without a preceding pointerdown.
    await fireEvent.click(manuscriptActions);
    expect(menu.open).toBe(false);
    expect(view.getByRole("menu")).toBeTruthy();
    await fireEvent.click(manuscriptActions);
    await fireEvent.click(trigger);
    await fireEvent.pointerDown(manuscriptActions);
    expect(menu.open).toBe(false);
  });
  it("leaves Escape available after a native project close with review options open", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    await fireEvent.click(view.getByLabelText("Review options"));
    await view.component.closeWorkspace();
    const escape = new KeyboardEvent("keydown", {
      key: "Escape",
      bubbles: true,
      cancelable: true,
    });
    document.body.dispatchEvent(escape);
    expect(escape.defaultPrevented).toBe(false);
  });
  it("keeps inactive prose feedback discoverable with its original discussion", async () => {
    localReview.documents.push({ id: "hidden-beat", label: "Beat 1", html: "<p>Old prose</p>" });
    localReview.data.annotations.push({
      id: "hidden-note",
      document_id: "hidden-beat",
      anchor_html: "<p>Old prose</p>",
      from: 1,
      to: 4,
      quote: "Old",
      replacement: null,
      state: "open",
      messages: [{ author: "Rowan", text: "Keep this perspective.", created_at: "today" }],
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    await fireEvent.click(view.getByRole("button", { name: /Keep this perspective/ }));
    expect(view.getByText(/Saved on inactive beat prose/)).toBeTruthy();
    expect(view.getByText("Keep this perspective.")).toBeTruthy();
    expect(view.queryByRole("button", { name: "Reply" })).toBeNull();
    expect(document.querySelectorAll(".editorial-margin-marker")).toHaveLength(0);
  });
  it.each(["accepted", "rejected", "locked", "failed"])(
    "groups active scene suggestions atomically for %s bulk decisions",
    async (decision) => {
      const sources = ["s", "t"].map((id) => ({
        ...source,
        id,
        scene_id: id,
        locked: decision === "locked" && id === "t",
      }));
      const reviews: SceneReview[] = sources.map((src) => ({
        ...structuredClone(localReview),
        scene_id: src.id,
        documents: [{ id: src.id, label: "Page", html: src.html }],
        data: {
          status: "editor_review",
          drafts: [],
          annotations: [
            {
              id: `note-${src.id}`,
              document_id: src.id,
              anchor_html: src.html,
              from: 1,
              to: 8,
              quote: "Eleanor",
              replacement: "Rowan",
              state: "open",
              messages: [],
            },
          ],
        },
      }));
      const original = vi.mocked(invoke).getMockImplementation()!;
      vi.mocked(invoke).mockImplementation(async (cmd, args) => {
        if (cmd === "editorial_sources") return structuredClone(sources);
        if (cmd === "get_project_scene_reviews") return structuredClone(reviews);
        if (cmd === "save_scene_review_batch" && decision === "failed")
          throw new Error("Review changed in another window");
        return original(cmd, args);
      });
      const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
      await view.component.openLocal("project", "s");
      await fireEvent.click(view.getByLabelText("Review options"));
      await fireEvent.change(view.getByLabelText("Show feedback"), {
        target: { value: "accepted" },
      });
      expect(view.queryByRole("group", { name: "Active scene prose" })).toBeNull();
      await fireEvent.change(view.getByLabelText("Show feedback"), {
        target: { value: "open" },
      });
      await fireEvent.click(
        view.getByRole("button", {
          name: `${decision === "rejected" ? "Reject" : "Accept"} visible suggestions on active scene prose`,
        })
      );
      if (decision === "failed") {
        await waitFor(() =>
          expect(view.getByRole("alert").textContent).toContain("another window")
        );
        expect(reviews.every((r) => r.data.annotations[0].state === "open")).toBe(true);
      } else if (decision === "locked") {
        await waitFor(() => expect(view.getByRole("alert").textContent).toContain("Unlock"));
        expect(
          vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "save_scene_review_batch")
        ).toBe(false);
      } else {
        await waitFor(() =>
          expect(
            vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "save_scene_review_batch")
          ).toBe(true)
        );
        const call = vi
          .mocked(invoke)
          .mock.calls.find(([cmd]) => cmd === "save_scene_review_batch")![1] as {
          updates: {
            expected: SceneReview;
            data: SceneReview["data"];
            next: { documents: SceneReview["documents"] } | null;
          }[];
        };
        expect(call.updates.map((u) => u.expected.scene_id)).toEqual(["s", "t"]);
        for (const update of call.updates) {
          expect(update.data.annotations[0].state).toBe(decision);
          expect(update.expected.data.annotations[0].state).toBe("open");
          if (decision === "accepted") {
            expect(update.next!.documents[0].id).toBe(update.expected.scene_id);
            expect(update.next!.documents[0].html).toContain("Rowan");
            expect(update.data.drafts[0].documents).toEqual(update.expected.documents);
          } else expect(update.next).toBeNull();
        }
      }
    }
  );
  it("reanchors and accepts a saved scene suggestion using the selected current passage", async () => {
    const current = { ...source, id: "s", html: "<p>Young Eleanor opened the letter.</p>" };
    localReview.documents[0].html = current.html;
    localReview.data.annotations.push({
      id: "old-edit",
      document_id: "s",
      anchor_html: source.html,
      from: 1,
      to: 8,
      quote: "Eleanor",
      replacement: "Rowan",
      state: "open",
      messages: [],
    });
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) =>
      cmd === "editorial_sources" ? [current] : original(cmd, args)
    );
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    await fireEvent.click(view.getByRole("button", { name: /Suggested edit/ }));
    expect((view.getByRole("button", { name: "Accept" }) as HTMLButtonElement).disabled).toBe(true);
    editor().commands.setTextSelection({ from: 7, to: 14 });
    await tick();
    await fireEvent.click(view.getByRole("button", { name: "Apply to selected passage" }));
    await waitFor(() => expect(localReview.data.annotations[0].state).toBe("accepted"));
    expect(localReview.documents[0].html).toContain("Young Rowan opened");
    expect(localReview.data.drafts[0].documents[0].html).toBe(current.html);
  });
  it("propagates failed workspace closing so its project cannot be discarded", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    await fireEvent.change(view.getByLabelText("Editor mode"), { target: { value: "review" } });
    await waitFor(() => expect(editor().isEditable).toBe(true));
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "import_editorial_feedback") throw new Error("disk full");
      return original(cmd, args);
    });
    await expect(view.component.closeWorkspace()).rejects.toThrow("disk full");
    expect(view.component.isOpen()).toBe(true);
  });
  it("keeps first-time attribution mounted through native-style character input", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    const input = view.getByLabelText("Name shown with feedback");
    for (const value of ["R", "Ro", "Rowan"]) {
      await fireEvent.input(input, { target: { value } });
      expect(view.getByLabelText("Name shown with feedback")).toBe(input);
    }
    await fireEvent.click(view.getByRole("button", { name: "Done" }));
    expect(view.queryByLabelText("Name shown with feedback")).toBeNull();
    expect(localStorage.getItem("kindling.editorial.name")).toBe("Rowan");
  });
  it("activates inline feedback without expanding the edit selection", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("Quietly, ");
    await fireEvent.change(view.getByLabelText("Markup view"), { target: { value: "all" } });
    const marker = document.querySelector(".editorial-margin-marker");
    editor().commands.setTextSelection(4);
    await tick();
    expect(document.querySelector(".editorial-margin-marker")).toBe(marker);
    const mark = document.querySelector(".editorial-insertion")!;
    await fireEvent.click(mark);
    expect(editor().state.selection.from).toBe(4);
    expect(editor().state.selection.to).toBe(4);
    editor().commands.insertContent("X");
    expect(editor().getText()).toContain("QuiXetly, ");
  });
  it("uses current scene locks when resuming and refreshing an existing local pass", async () => {
    let locked = true;
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) =>
      cmd === "editorial_sources" ? [{ ...source, locked }] : original(cmd, args)
    );
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    await fireEvent.change(view.getByLabelText("Editor mode"), { target: { value: "review" } });
    await waitFor(() => expect(editor().isEditable).toBe(true));
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("Blocked ");
    expect(editor().getText()).not.toContain("Blocked");
    locked = false;
    await view.component.refreshLocalContext();
    editor().commands.insertContent("Allowed ");
    expect(editor().getText()).toContain("Allowed");
  });
  it("does not offer unsupported new comments in returned portable feedback", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    await fireEvent.click(view.getByRole("button", { name: "First review" }));
    await waitFor(() => expect(editor().isEditable).toBe(false));
    expect(view.queryByRole("button", { name: "Comment" })).toBeNull();
  });
  it("opens local revisions in the editor, suggests with normal gestures and publishes only acknowledged saves", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s", { sourceId: "a", from: 9, to: 15 });
    expect(view.queryByRole("dialog")).toBeNull();
    expect(view.component.isLocal()).toBe(true);
    expect(editor().state.selection.from).toBe(9);
    await fireEvent.change(view.getByLabelText("Editor mode"), { target: { value: "review" } });
    await waitFor(() => expect(editor().isEditable).toBe(true));
    expect(editor().state.selection.from).toBe(9);
    editor().commands.insertContent("read");
    await tick();
    expect((view.getByLabelText("Markup view") as HTMLSelectElement).value).toBe("simple");
    expect(document.querySelectorAll(".editorial-margin-marker")).toHaveLength(1);
    expect(document.querySelector(".editorial-deletion")).toBeNull();
    await fireEvent.click(view.getByRole("button", { name: /Suggested edit/ }));
    expect(document.querySelector(".editorial-deletion")).toBeTruthy();
    await view.component.flush();
    const calls = vi.mocked(invoke).mock.calls;
    const saved = calls.filter(([cmd]) => cmd === "save_editorial_session").slice(-1)[0]![1] as {
      session: unknown;
    };
    const published = calls
      .filter(([cmd]) => cmd === "import_editorial_feedback")
      .slice(-1)[0]![1] as {
      package: { session: unknown };
    };
    expect(published.package.session).toEqual(saved.session);
    await fireEvent.change(view.getByLabelText("Editor mode"), { target: { value: "feedback" } });
    await waitFor(() => expect(editor().isEditable).toBe(false));
    await fireEvent.click(view.getByRole("button", { name: "Return to writing" }));
    expect(view.component.isOpen()).toBe(false);
  });
  it("presents saved scene comments and preserves a failed reply for retry", async () => {
    // A page source uses the scene id; this fixture exercises existing scene annotations.
    const localSource = { ...source, id: "s" };
    localReview.data.annotations = [
      {
        id: "old",
        document_id: "s",
        anchor_html: source.html,
        from: 1,
        to: 8,
        quote: "Eleanor",
        replacement: null,
        state: "open",
        messages: [{ author: "Rowan", text: "Whose letter?", created_at: "2026-01-01" }],
      },
    ];
    const original = vi.mocked(invoke).getMockImplementation()!;
    let fail = true;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "editorial_sources") return [localSource];
      if (cmd === "save_scene_review" && fail)
        throw new Error("Another window changed this review");
      return original(cmd, args);
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openLocal("project", "s");
    fail = false;
    await fireEvent.click(view.getByLabelText("Manuscript actions"));
    await fireEvent.click(view.getByRole("menuitem", { name: "Draft history" }));
    await fireEvent.input(await view.findByLabelText("Draft name"), {
      target: { value: "Before review" },
    });
    await fireEvent.click(view.getByRole("button", { name: "Save named draft" }));
    await waitFor(() => expect(localReview.version).toBe(1));
    await fireEvent.click(view.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(view.queryByRole("dialog")).toBeNull());
    fail = true;
    await fireEvent.click(view.getByRole("button", { name: /Rowan.*Whose letter/ }));
    await fireEvent.input(view.getByLabelText("Reply"), {
      target: { value: "Her guardian sent it." },
    });
    await fireEvent.click(view.getByRole("button", { name: "Reply" }));
    await waitFor(() => expect(view.getByRole("alert").textContent).toContain("Another window"));
    expect((view.getByLabelText("Reply") as HTMLTextAreaElement).value).toBe(
      "Her guardian sent it."
    );
    fail = false;
    await fireEvent.click(view.getByRole("button", { name: "Reply" }));
    await waitFor(() =>
      expect((view.getByLabelText("Reply") as HTMLTextAreaElement).value).toBe("")
    );
    expect(localReview.data.annotations[0].messages.slice(-1)[0]?.text).toBe(
      "Her guardian sent it."
    );
    await fireEvent.click(view.getByRole("button", { name: "Resolve thread" }));
    await waitFor(() => expect(localReview.data.annotations[0].state).toBe("resolved"));
  });
  it("keeps the current workspace usable when opening local feedback fails", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    editor().commands.insertContent("A note. ");
    const text = editor().getText();
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "get_project_scene_reviews") throw new Error("Scene unavailable");
      return original(cmd, args);
    });
    await view.component.openLocal("project", "s");
    expect(view.component.isLocal()).toBe(false);
    expect(editor().getText()).toBe(text);
    expect(view.getByRole("alert").textContent).toContain("Scene unavailable");
  });
  it("shows an actionable error when a package fails before the workspace opens", async () => {
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "open_editorial_package") throw new Error("Unsupported review package version");
      return original(cmd, args);
    });
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/future.kindling-review");
    expect(view.component.isOpen()).toBe(true);
    expect(view.queryByRole("dialog")).toBeNull();
    expect(view.getByRole("alert").textContent).toContain("Unsupported review package version");
    expect(view.getByText("Open review or feedback file…")).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Close review" }));
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
    expect(view.getByRole("button", { name: /Suggested edit/ })).toBeTruthy();
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
    await fireEvent.click(view.getByLabelText("Manuscript actions"));
    await fireEvent.click(view.getByRole("menuitem", { name: "Export feedback" }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "export_editorial_feedback",
        expect.objectContaining({ path: "/returned.kindling-feedback" })
      )
    );
    await fireEvent.click(view.getByRole("button", { name: "Close review" }));
    await waitFor(() => expect(view.component.isOpen()).toBe(false));
  });

  it("keeps focus in search while selecting a match and focuses comment composition", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openFile("/review.kindling-review");
    await fireEvent.click(view.getByRole("button", { name: "Find in manuscript" }));
    const search = view.getByRole("searchbox");
    const pane = document.querySelector<HTMLElement>(".manuscript-column")!;
    // The search controls remain outside the prose's scroll container.
    expect(pane.contains(search)).toBe(false);
    expect(pane.parentElement).toBe(search.closest(".manuscript-region"));
    vi.spyOn(pane, "getBoundingClientRect").mockReturnValue({ top: 0, bottom: 400 } as ReturnType<
      HTMLElement["getBoundingClientRect"]
    >);
    vi.spyOn(editor().view, "coordsAtPos").mockReturnValue({
      top: 1200,
      bottom: 1220,
      left: 0,
      right: 10,
    });
    search.focus();
    await fireEvent.input(search, { target: { value: "letter" } });
    expect(document.activeElement).toBe(search);
    expect(
      editor().state.doc.textBetween(editor().state.selection.from, editor().state.selection.to)
    ).toBe("letter");
    await waitFor(() => expect(pane.scrollTop).toBeGreaterThan(1000));
    expect(document.querySelector(".editorial-search-match")?.textContent).toBe("letter");
    editor().chain().setTextSelection(1).insertContent("Before this, ").run();
    await tick();
    expect(document.querySelector(".editorial-search-match")?.textContent).toBe("letter");
    editor().chain().setTextSelection(1).insertContent("Another letter. ").run();
    await tick();
    expect(view.getByText("1 of 2")).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Next" }));
    expect(view.getByText("2 of 2")).toBeTruthy();
    await fireEvent.keyDown(search, { key: "Escape" });
    expect(document.querySelector(".editorial-search-match")).toBeNull();
    await fireEvent.click(view.getByRole("button", { name: "Find in manuscript" }));
    await tick();
    expect((view.getByRole("searchbox") as HTMLInputElement).value).toBe("letter");
    expect(view.getByText("2 of 2")).toBeTruthy();
    expect(document.querySelector(".editorial-search-match")?.textContent).toBe("letter");
    await fireEvent.click(view.getByRole("button", { name: "Comment" }));
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
    await fireEvent.click(view.getAllByText("Export recovery copy")[0]);
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
    await fireEvent.click(view.getByRole("button", { name: "Comment" }));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "What is Eleanor afraid of?" },
    });
    await fireEvent.click(view.getByText("Save comment"));
    expect(view.getAllByText("What is Eleanor afraid of?")).toHaveLength(1);
    await fireEvent.input(view.getByLabelText("Reply"), {
      target: { value: "The letter should make this clearer." },
    });
    await fireEvent.click(view.getByRole("button", { name: "Reply" }));
    expect(view.getByText("The letter should make this clearer.")).toBeTruthy();
    await fireEvent.click(view.getByRole("button", { name: "Resolve thread" }));
    await fireEvent.change(view.getByLabelText("Show feedback"), {
      target: { value: "all" },
    });
    await fireEvent.click(view.getByRole("button", { name: "Reopen thread" }));
    expect(view.getByRole("button", { name: /Rowan.*Comment/ })).toBeTruthy();
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
    await waitFor(() =>
      expect(view.getByRole("button", { name: /Rowan.*Suggested edit/ })).toBeTruthy()
    );
    await fireEvent.click(view.getByRole("button", { name: /Rowan.*Suggested edit/ }));
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
    await fireEvent.click(view.getByRole("button", { name: "Comment" }));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "Keep the letter tangible." },
    });
    await fireEvent.click(view.getByText("Save comment"));
    editor().commands.setTextSelection(1);
    editor().commands.insertContent("At dusk, ");
    await tick();
    await fireEvent.click(view.getByRole("button", { name: /Suggested edit/ }));
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
    await fireEvent.click(view.getByRole("button", { name: "Comment" }));
    await fireEvent.input(view.getByLabelText("Comment"), {
      target: { value: "Keep the letter tangible." },
    });
    await fireEvent.click(view.getByText("Save comment"));
    editor().commands.setTextSelection(editor().state.doc.content.size - 1);
    editor().commands.insertContent(" Later.");
    await tick();
    await fireEvent.click(view.getAllByRole("button", { name: /Suggested edit/ }).slice(-1)[0]!);
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
    await waitFor(() =>
      expect(view.getByRole("button", { name: /Rowan.*Suggested edit/ })).toBeTruthy()
    );
    await fireEvent.click(view.getByRole("button", { name: /Rowan.*Suggested edit/ }));
    const before = editor();
    const selection = before.state.selection;
    await fireEvent.click(view.getByRole("button", { name: "Reject" }));
    await waitFor(() => expect(view.getByText("Review decision saved.")).toBeTruthy());
    expect(editor()).toBe(before);
    expect(editor().state.selection.eq(selection)).toBe(true);
    vi.mocked(save).mockResolvedValue("/reply.kindling-review");
    await fireEvent.click(view.getByLabelText("Review options"));
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
    await waitFor(() =>
      expect(view.getByRole("button", { name: /Rowan.*Suggested edit/ })).toBeTruthy()
    );
    await fireEvent.click(view.getByRole("button", { name: /Rowan.*Suggested edit/ }));
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
    await waitFor(() =>
      expect(view.getByRole("button", { name: /Rowan.*Suggested edit/ })).toBeTruthy()
    );
    await fireEvent.click(view.getByRole("button", { name: /Rowan.*Suggested edit/ }));
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
    await fireEvent.click(view.getByRole("button", { name: "Close review" }));
    await waitFor(() => expect(view.getByRole("alert").textContent).toContain("disk full"));
    expect(view.component.isOpen()).toBe(true);
    expect(localStorage.getItem("kindling.editorial.recovery.round")).toContain("Rowan");
  });

  it.each(["feedback", "review"] as const)(
    "returns from packages to the same %s mode and manuscript position",
    async (mode) => {
      const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
      await view.component.openLocal("project", "s");
      if (mode === "review") {
        await fireEvent.change(view.getByLabelText("Editor mode"), { target: { value: mode } });
        await waitFor(() => expect(editor().isEditable).toBe(true));
        editor().commands.insertContent("Revised ");
      }
      await fireEvent.change(view.getByLabelText("Show feedback"), { target: { value: "all" } });
      editor().commands.setTextSelection({ from: 9, to: 15 });
      const text = editor().getText();
      document.querySelector<HTMLElement>(".manuscript-column")!.scrollTop = 123;
      await fireEvent.click(view.getByLabelText("Manuscript actions"));
      await fireEvent.click(view.getByRole("menuitem", { name: "Review packages and rounds…" }));
      await waitFor(() =>
        expect(view.getByRole("button", { name: "Return to revisions" })).toBeTruthy()
      );
      expect(document.querySelector(".editorial-prose")).toBeNull();
      // Reopening package setup through the application menu preserves its origin too.
      await view.component.openProject("project");
      if (mode === "review") {
        const original = vi.mocked(invoke).getMockImplementation()!;
        vi.mocked(invoke).mockImplementation(async (cmd, args) => {
          if (cmd === "get_editorial_feedback") throw new Error("Cannot open this round");
          return original(cmd, args);
        });
        await fireEvent.click(view.getByRole("button", { name: "First review" }));
        await waitFor(() =>
          expect(view.getByRole("alert").textContent).toContain("Cannot open this round")
        );
      } else {
        vi.mocked(save).mockResolvedValue("/round.kindling-review");
        await fireEvent.click(view.getByRole("button", { name: "Export review package" }));
        await waitFor(() => expect(view.getByText(/Review package saved to/)).toBeTruthy());
      }
      await fireEvent.click(view.getByRole("button", { name: "Return to revisions" }));
      await waitFor(() => expect(view.getByLabelText("Editor mode")).toBeTruthy());
      expect(view.queryByRole("alert")).toBeNull();
      expect(view.queryByText(/Review package saved to/)).toBeNull();
      expect(view.component.isLocal()).toBe(true);
      expect((view.getByLabelText("Editor mode") as HTMLSelectElement).value).toBe(mode);
      expect((view.getByLabelText("Show feedback") as HTMLSelectElement).value).toBe("all");
      expect(editor().getText()).toBe(text);
      expect(editor().state.selection.from).toBe(9);
      expect(editor().state.selection.to).toBe(15);
      expect(document.querySelector<HTMLElement>(".manuscript-column")!.scrollTop).toBe(123);
      await view.component.closeWorkspace();
      await view.component.openProject("project");
      expect(view.queryByRole("button", { name: "Return to revisions" })).toBeNull();
      expect(view.component.isLocal()).toBe(true);
      await fireEvent.click(view.getByRole("button", { name: "Return to writing" }));
      await waitFor(() => expect(view.component.isOpen()).toBe(false));
    }
  );

  it("shows local date and time including seconds for review rounds", async () => {
    const dates = ["2026-09-08T12:30:45Z", "2026-09-08T12:30:15Z"];
    const original = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (cmd, args) =>
      cmd === "list_editorial_rounds"
        ? dates.map((created_at, i) => ({ ...round, id: `round-${i}`, created_at }))
        : original(cmd, args)
    );
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    const times = Array.from(document.querySelectorAll(".round-link time"));
    expect(times.map((e) => e.getAttribute("datetime"))).toEqual(dates);
    expect(times.map((e) => e.textContent)).toEqual(
      dates.map((date) =>
        new Date(date).toLocaleString(undefined, {
          month: "short",
          day: "numeric",
          year: "numeric",
          hour: "numeric",
          minute: "2-digit",
          second: "2-digit",
        })
      )
    );
    expect(times[0].textContent).not.toBe(times[1].textContent);
  });

  it("exports scoped review rounds and opens their saved feedback", async () => {
    const view = render(EditorialWorkspace, { prepareWriting, onManuscriptChanged });
    await view.component.openProject("project");
    vi.mocked(save).mockResolvedValue("/round.kindling-review");
    await fireEvent.click(view.getByLabelText("Selected chapters"));
    expect((view.getByLabelText("Selected chapters") as HTMLInputElement).name).toBe(
      (view.getByLabelText("Entire manuscript") as HTMLInputElement).name
    );
    expect((view.getByLabelText("Selected chapters") as HTMLInputElement).name).not.toBe("");
    expect((view.getByText("Export review package") as HTMLButtonElement).disabled).toBe(true);
    await fireEvent.click(view.getByLabelText("Chapter One"));
    await fireEvent.click(view.getByText("Export review package"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "export_editorial_review",
        expect.objectContaining({ projectId: "project", chapterIds: ["c"] })
      )
    );
    await fireEvent.click(view.getAllByText("First review")[0]);
    await waitFor(() => expect(view.getByRole("tab", { name: /Review/ })).toBeTruthy());
  });
});
