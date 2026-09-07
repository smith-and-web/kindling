import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import type { Editor } from "@tiptap/core";
import { invoke } from "@tauri-apps/api/core";
import RevisionsPanel from "./RevisionsPanel.svelte";
import type { SceneReview } from "../utils/revisions";
let persisted: SceneReview;
let failSave = false;
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLElement.prototype.scrollIntoView = vi.fn();
  persisted = {
    scene_id: "scene",
    version: 0,
    mode: "page",
    documents: [{ id: "scene", label: "Scene page", html: "<p>The red fox.</p>" }],
    data: { status: "first_draft", drafts: [], annotations: [] },
  };
  failSave = false;
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "get_scene_review") return structuredClone(persisted);
    if (cmd === "get_revision_overview")
      return [
        {
          scene_id: "scene",
          title: "Arrival",
          chapter: "Chapter 1",
          status: "first_draft",
          drafts: 0,
        },
      ];
    if (cmd === "save_scene_review") {
      if (failSave) throw new Error("database is locked");
      const { data, next } = args as {
        data: SceneReview["data"];
        next: { documents: SceneReview["documents"]; mode: SceneReview["mode"] } | null;
      };
      persisted = {
        ...persisted,
        version: persisted.version + 1,
        data: structuredClone(data),
        ...(next ? { documents: next.documents, mode: next.mode } : {}),
      };
      return structuredClone(persisted);
    }
  });
});
afterEach(cleanup);
async function setup(locked = false) {
  const onApplied = vi.fn(),
    onClose = vi.fn();
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked,
    onApplied,
    onClose,
  });
  await screen.findByText("No comments or suggestions yet.");
  return { onApplied, onClose };
}
async function select(from = 5, to = 8) {
  const element = document.querySelector(".tiptap") as HTMLElement & { editor: Editor };
  element.editor.commands.setTextSelection({ from, to });
  await waitFor(() => expect(screen.queryByText("Select text in the prose above.")).toBeNull());
}
it("creates numbered named drafts, compares and restores with a preserved current draft", async () => {
  const { onApplied } = await setup();
  await fireEvent.click(screen.getByText("Draft history"));
  await fireEvent.input(screen.getByLabelText("Draft name"), { target: { value: "First pass" } });
  await fireEvent.click(screen.getByText("Save named draft"));
  await screen.findAllByRole("option", { name: "Draft 1 · First pass" });
  await fireEvent.click(screen.getByText("Restore selected draft"));
  await fireEvent.click(screen.getByText("Restore and preserve current prose"));
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(2));
  expect(persisted.data.drafts[1].name).toBe("Before restoring First pass");
  expect(onApplied).toHaveBeenCalledOnce();
  await fireEvent.click(screen.getByText("All scenes"));
  expect(screen.getByText("Chapter 1")).toBeTruthy();
});
it("anchors inline comments, persists replies and resolves/reopens the thread", async () => {
  await setup();
  await select();
  await fireEvent.input(screen.getByLabelText("Comment"), { target: { value: "Stronger image?" } });
  await fireEvent.click(screen.getByText("Add comment"));
  await screen.findByText("Stronger image?");
  expect(document.querySelector(".review-comment")?.textContent).toBe("red");
  await fireEvent.input(screen.getByLabelText("Reply"), { target: { value: "Agreed." } });
  await fireEvent.click(screen.getByText("Reply to thread"));
  await screen.findByText("Agreed.");
  await fireEvent.click(screen.getByText("Resolve thread"));
  await screen.findByText("Reopen thread");
  expect(persisted.data.annotations[0].state).toBe("resolved");
  await fireEvent.click(screen.getByText("Reopen thread"));
  await screen.findByText("Resolve thread");
});
it("previews and accepts suggestions only after successful persistence; rejection leaves prose intact", async () => {
  const { onApplied } = await setup();
  await select();
  await fireEvent.change(screen.getByLabelText("Annotation"), { target: { value: "suggestion" } });
  await fireEvent.input(screen.getByLabelText("Suggested text (leave empty to delete)"), {
    target: { value: "blue" },
  });
  await fireEvent.click(screen.getByText("Add suggestion"));
  await screen.findByText("Accept change");
  expect(document.querySelector(".review-deletion")?.textContent).toBe("red");
  expect(document.querySelector(".review-insertion")?.textContent).toBe("blue");
  failSave = true;
  await fireEvent.click(screen.getByText("Accept change"));
  await screen.findByRole("alert");
  expect(onApplied).not.toHaveBeenCalled();
  expect(persisted.documents[0].html).toBe("<p>The red fox.</p>");
  failSave = false;
  await fireEvent.click(screen.getByText("Accept all"));
  await waitFor(() => expect(onApplied).toHaveBeenCalledOnce());
  expect(persisted.documents[0].html).toBe("<p>The blue fox.</p>");
  expect(persisted.data.drafts[0].documents[0].html).toBe("<p>The red fox.</p>");
  await select(1, 4);
  await fireEvent.input(screen.getByLabelText("Suggested text (leave empty to delete)"), {
    target: { value: "A" },
  });
  await fireEvent.click(screen.getByText("Add suggestion"));
  await screen.findByText("Reject change");
  await fireEvent.click(screen.getByText("Reject all"));
  await waitFor(() => expect(persisted.data.annotations[1].state).toBe("rejected"));
  expect(persisted.documents[0].html).toBe("<p>The blue fox.</p>");
});
it("allows reading locked scenes while disabling writes", async () => {
  await setup(true);
  expect(
    (screen.getByLabelText("Revision status") as HTMLSelectElement).closest("fieldset")?.disabled
  ).toBe(true);
  await fireEvent.click(screen.getByText("Draft history"));
  expect(
    (screen.getByLabelText("Draft name") as HTMLInputElement).closest("fieldset")?.disabled
  ).toBe(true);
});

it("scrolls to changes on fresh-open and cross-beat navigation without native selection", async () => {
  persisted.mode = "beat";
  persisted.documents.push(
    { id: "beat-1", label: "Beat 1", html: "<p>First beat.</p>" },
    { id: "beat-2", label: "Beat 2", html: "<p>Second beat.</p>" }
  );
  persisted.data.annotations = [1, 2].map((i) => ({
    id: "change-" + i,
    document_id: "beat-" + i,
    anchor_html: persisted.documents[i].html,
    from: 1,
    to: 1,
    quote: "",
    replacement: "New ",
    state: "open",
    messages: [],
  }));
  const onApplied = vi.fn(),
    onClose = vi.fn();
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied,
    onClose,
  });
  await screen.findByText("2 pending changes");
  window.getSelection()?.removeAllRanges();
  await fireEvent.click(screen.getByText("Next change"));
  await waitFor(() => expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled());
  vi.mocked(HTMLElement.prototype.scrollIntoView).mockClear();
  await fireEvent.click(screen.getByText("Next change"));
  await waitFor(() =>
    expect((screen.getByLabelText("Prose source") as HTMLSelectElement).value).toBe("beat-2")
  );
  expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();
  expect(document.querySelector('.review-insertion[data-selected="true"]')).toBeTruthy();
});

it("requires a fresh nonempty range when re-anchoring a replacement", async () => {
  persisted.data.annotations = [
    {
      id: "old",
      document_id: "scene",
      anchor_html: "<p>The old fox.</p>",
      from: 5,
      to: 8,
      quote: "old",
      replacement: "blue",
      state: "open",
      messages: [],
    },
  ];
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  await fireEvent.click(await screen.findByText("Suggestion · open · old"));
  const button = screen.getByText("Re-anchor to selection") as HTMLButtonElement;
  expect(button.disabled).toBe(true);
  await select(2, 2);
  expect(button.disabled).toBe(true);
  await select();
  expect(button.disabled).toBe(false);
  await fireEvent.click(button);
  await screen.findByText("Accept change");
  expect(persisted.data.annotations[0].quote).toBe("red");
  expect(persisted.data.annotations[0].from).toBe(5);
  expect(persisted.data.annotations[0].to).toBe(8);
});

it("keeps inactive-mode suggestions out of navigation and bulk acceptance", async () => {
  persisted.documents.push({ id: "beat", label: "Beat 1", html: "<p>Beat.</p>" });
  persisted.data.annotations = [
    {
      id: "page",
      document_id: "scene",
      anchor_html: persisted.documents[0].html,
      from: 5,
      to: 8,
      quote: "red",
      replacement: "blue",
      state: "open",
      messages: [],
    },
    {
      id: "beat",
      document_id: "beat",
      anchor_html: persisted.documents[1].html,
      from: 1,
      to: 5,
      quote: "Beat",
      replacement: "Changed",
      state: "open",
      messages: [],
    },
  ];
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  await screen.findByText("1 pending changes");
  expect(screen.queryByText("Suggestion · open · Beat")).toBeNull();
  await fireEvent.click(screen.getByText("Accept all"));
  await waitFor(() => expect(persisted.data.annotations[0].state).toBe("accepted"));
  expect(persisted.data.annotations[1].state).toBe("open");
  expect(persisted.documents[1].html).toBe("<p>Beat.</p>");
});

it("does not replace the prose document or reset selection when replying to a thread", async () => {
  await setup();
  await select();
  await fireEvent.input(screen.getByLabelText("Comment"), { target: { value: "Comment here" } });
  await fireEvent.click(screen.getByText("Add comment"));
  await screen.findByText("Resolve thread");
  const editor = (document.querySelector(".tiptap") as HTMLElement & { editor: Editor }).editor;
  await select(9, 12);
  const doc = editor.state.doc;
  await fireEvent.input(screen.getByLabelText("Reply"), { target: { value: "Reply" } });
  await fireEvent.click(screen.getByText("Reply to thread"));
  await waitFor(() => expect(persisted.data.annotations[0].messages.length).toBe(2));
  expect(editor.state.doc).toBe(doc);
  expect(editor.state.selection.from).toBe(9);
  expect(editor.state.selection.to).toBe(12);
});

it("never treats a remounted editor's initial cursor as an explicit re-anchor", async () => {
  persisted.mode = "beat";
  persisted.documents.push(
    { id: "b1", label: "Beat 1", html: "<p>First.</p>" },
    { id: "b2", label: "Beat 2", html: "<p>Second.</p>" }
  );
  persisted.data.annotations = [
    {
      id: "outdated",
      document_id: "b2",
      anchor_html: "<p>Older.</p>",
      from: 3,
      to: 3,
      quote: "",
      replacement: "New",
      state: "open",
      messages: [],
    },
  ];
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  await fireEvent.click(await screen.findByText("Suggestion · open · Insertion"));
  const button = () => screen.getByText("Re-anchor to selection") as HTMLButtonElement;
  expect(button().disabled).toBe(true);
  await fireEvent.click(screen.getByText("Draft history"));
  await fireEvent.click(screen.getByText("Editorial review"));
  expect(button().disabled).toBe(true);
  // Explicitly click the same initial position: native selection need not have
  // changed the ProseMirror selection for this user action to count.
  const prose = document.querySelector(".review-prose")!;
  const text = prose.querySelector("p")!.firstChild!;
  window.getSelection()!.setBaseAndExtent(text, 0, text, 0);
  await fireEvent.mouseUp(prose);
  expect(button().disabled).toBe(false);
  await fireEvent.click(button());
  await waitFor(() => expect(persisted.data.annotations[0].anchor_html).toBe("<p>Second.</p>"));
  expect(persisted.data.annotations[0].from).toBe(1);
});

it("requires an explicit prose selection to create a suggestion and re-scrolls a repeated navigation", async () => {
  await setup();
  await fireEvent.change(screen.getByLabelText("Annotation"), { target: { value: "suggestion" } });
  await fireEvent.input(screen.getByLabelText("Suggested text (leave empty to delete)"), {
    target: { value: "New " },
  });
  expect((screen.getByText("Add suggestion") as HTMLButtonElement).disabled).toBe(true);
  const prose = document.querySelector(".review-prose")!;
  const text = prose.querySelector("p")!.firstChild!;
  window.getSelection()!.setBaseAndExtent(text, 0, text, 0);
  await fireEvent.mouseUp(prose);
  await fireEvent.click(screen.getByText("Add suggestion"));
  await screen.findByText("Accept change");
  vi.mocked(HTMLElement.prototype.scrollIntoView).mockClear();
  await fireEvent.click(screen.getByText("Next change"));
  expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();
});
