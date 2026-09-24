import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import RevisionsPanel from "./RevisionsPanel.svelte";
import revisionsSource from "./RevisionsPanel.svelte?raw";
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
  await screen.findByText(
    "No saved drafts yet. Save a named draft to keep this scene’s current prose."
  );
  return { onApplied, onClose };
}
it("creates numbered named drafts, compares and restores with a preserved current draft", async () => {
  const { onApplied } = await setup();
  await fireEvent.click(screen.getByRole("button", { name: "Draft history" }));
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
it("allows reading locked scenes while disabling writes", async () => {
  await setup(true);
  expect(
    (screen.getByLabelText("Revision status") as HTMLSelectElement).closest("fieldset")?.disabled
  ).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Draft history" }));
  expect(
    (screen.getByLabelText("Draft name") as HTMLInputElement).closest("fieldset")?.disabled
  ).toBe(true);
});
it("compares active prose in separate versions and selects the newest saved draft", async () => {
  persisted.documents.push({
    id: "hidden",
    label: "Inactive beat",
    html: "<p>Hidden current prose.</p>",
  });
  persisted.data.drafts = [
    {
      name: "First pass",
      created_at: "2026-01-01",
      mode: "page",
      documents: [{ id: "scene", label: "Page", html: "<p>An earlier fox.</p>" }],
    },
    {
      name: "Second pass",
      created_at: "2026-01-02",
      mode: "page",
      documents: [
        { id: "scene", label: "Page", html: "<p>The blue fox.</p>" },
        { id: "hidden", label: "Inactive beat", html: "<p>Hidden old prose.</p>" },
      ],
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
  const left = await screen.findByRole("region", { name: "Saved draft" });
  const right = screen.getByRole("region", { name: "Comparison version" });
  expect(within(left).getByRole("heading", { name: "Second pass" })).toBeTruthy();
  expect(left.querySelector("del")?.textContent).toContain("blue");
  expect(right.querySelector("ins")?.textContent).toContain("red");
  expect(left.textContent).not.toContain("red fox");
  expect(right.textContent).not.toContain("blue fox");
  expect(screen.queryByText(/Hidden current prose|Hidden old prose/)).toBeNull();
  await fireEvent.change(screen.getByLabelText("Compare with"), { target: { value: "1" } });
  expect(screen.getByText(/No prose text changes/)).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: /Draft 1.*First pass/ }));
  expect(within(left).getByRole("heading", { name: "First pass" })).toBeTruthy();
});

it("compares prose on manuscript paper with changed words in manuscript ink", async () => {
  persisted.data.drafts = [
    {
      name: "First pass",
      created_at: "2026-01-01",
      mode: "page",
      documents: [{ id: "scene", label: "Page", html: "<p>The blue fox.</p>" }],
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
  const left = await screen.findByRole("region", { name: "Saved draft" });
  const right = screen.getByRole("region", { name: "Comparison version" });
  // Both versions sit on the always-light paper sheet, not the dark chrome.
  for (const version of [left, right]) {
    const prose = version.querySelector(".diff-prose")!;
    expect(prose.parentElement?.classList.contains("app-prose-sheet")).toBe(true);
  }
  expect(left.querySelector(".app-prose-sheet del")?.textContent).toContain("blue");
  expect(right.querySelector(".app-prose-sheet ins")?.textContent).toContain("red");
  // jsdom doesn't apply component CSS, so check the rules themselves: prose and
  // changed words read in manuscript ink, never the chrome's text or status colours.
  const css = revisionsSource.slice(revisionsSource.indexOf("<style>"));
  const rule = (selector: string) =>
    css.match(new RegExp(`(?:\\}|\\*/)\\n  ${selector} \\{([^}]*)\\}`))?.[1] ?? "";
  expect(rule("\\.diff-prose")).toContain("color: var(--color-prose-text)");
  expect(rule("del,\\n  ins")).toContain("color: var(--color-prose-text)");
  for (const selector of ["del", "ins"]) {
    expect(rule(selector)).not.toMatch(/(^|[^-])color:/);
    expect(rule(selector)).toContain("-wash)");
  }
});

it("marks the drafts it keeps automatically and stays on the chosen draft when old ones are pruned", async () => {
  persisted.data.drafts = [
    {
      name: "Before accepting changes",
      created_at: "2026-01-01",
      mode: "page",
      documents: [{ id: "scene", label: "Page", html: "<p>An earlier fox.</p>" }],
      automatic: true,
    },
    {
      name: "First pass",
      created_at: "2026-01-02",
      mode: "page",
      documents: [{ id: "scene", label: "Page", html: "<p>The blue fox.</p>" }],
    },
  ];
  const original = vi.mocked(invoke).getMockImplementation()!;
  const sent: SceneReview["data"][] = [];
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd !== "save_scene_review") return original(cmd, args);
    const { data } = args as { data: SceneReview["data"] };
    sent.push(structuredClone(data));
    // As the backend does past its limit (here one), drop the oldest automatic draft.
    const automatic = data.drafts.filter((d) => d.automatic);
    const pruned = automatic.length > 1 ? automatic[0] : null;
    return original(cmd, {
      ...args,
      data: { ...data, drafts: data.drafts.filter((d) => d !== pruned) },
    });
  });
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  const left = await screen.findByRole("region", { name: "Saved draft" });
  expect(within(left).getByRole("heading", { name: "First pass" })).toBeTruthy();
  await fireEvent.click(screen.getByText("Restore selected draft"));
  await fireEvent.click(screen.getByText("Restore and preserve current prose"));
  await waitFor(() => expect(sent).toHaveLength(1));
  expect(sent[0].drafts[sent[0].drafts.length - 1]).toMatchObject({
    name: "Before restoring First pass",
    automatic: true,
  });
  await waitFor(() =>
    expect(persisted.data.drafts.map((d) => d.name)).toEqual([
      "First pass",
      "Before restoring First pass",
    ])
  );
  // The comparison still shows the draft the writer chose, not whatever moved into its slot.
  expect(within(left).getByRole("heading", { name: "First pass" })).toBeTruthy();

  await fireEvent.input(screen.getByLabelText("Draft name"), { target: { value: "Keeper" } });
  await fireEvent.click(screen.getByText("Save named draft"));
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(3));
  expect(persisted.data.drafts[2]).not.toHaveProperty("automatic");
  await waitFor(() => expect(within(left).getByRole("heading", { name: "Keeper" })).toBeTruthy());
});

it("moves to the nearest older draft and says so when the compared draft is pruned", async () => {
  const draft = (name: string, day: number, automatic = false) => ({
    name,
    created_at: `2026-01-0${day}`,
    mode: "page" as const,
    documents: [{ id: "scene", label: "Page", html: `<p>${name} fox.</p>` }],
    ...(automatic && { automatic }),
  });
  persisted.data.drafts = [
    draft("Opening", 1),
    draft("Middle", 2),
    draft("Before accepting A", 3, true),
    draft("Before accepting B", 4, true),
  ];
  const original = vi.mocked(invoke).getMockImplementation()!;
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd !== "save_scene_review") return original(cmd, args);
    // As the backend does past its limit (here one), drop the oldest automatic draft.
    const { data } = args as { data: SceneReview["data"] };
    const automatic = data.drafts.filter((d) => d.automatic);
    const pruned = automatic.length > 1 ? automatic[0] : null;
    return original(cmd, {
      ...args,
      data: { ...data, drafts: data.drafts.filter((d) => d !== pruned) },
    });
  });
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
  const left = await screen.findByRole("region", { name: "Saved draft" });
  await fireEvent.click(screen.getByRole("button", { name: /Draft 3.*Before accepting A/ }));
  expect(within(left).getByRole("heading", { name: "Before accepting A" })).toBeTruthy();
  expect(screen.queryByText(/was removed/)).toBeNull();

  await fireEvent.change(screen.getByLabelText("Revision status"), {
    target: { value: "editor_review" },
  });
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(3));
  // Not the first draft: the nearest surviving draft older than the one that went.
  await waitFor(() => expect(within(left).getByRole("heading", { name: "Middle" })).toBeTruthy());
  expect(screen.getByRole("status").textContent).toContain(
    "“Before accepting A” was removed because only the newest automatic drafts are kept. Now comparing “Middle” with the current prose."
  );
});

/** Stand in for the backend's automatic-draft limit (here `limit`) on every save. */
function pruneOnSave(limit: number) {
  const original = vi.mocked(invoke).getMockImplementation()!;
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd !== "save_scene_review") return original(cmd, args);
    const { data } = args as { data: SceneReview["data"] };
    const automatic = data.drafts.filter((d) => d.automatic);
    const pruned = new Set(automatic.slice(0, Math.max(0, automatic.length - limit)));
    return original(cmd, {
      ...args,
      data: { ...data, drafts: data.drafts.filter((d) => !pruned.has(d)) },
    });
  });
}
const pruneDraft = (name: string, day: number, automatic = false) => ({
  name,
  created_at: `2026-01-0${day}`,
  mode: "page" as const,
  documents: [{ id: "scene", label: "Page", html: `<p>${name} fox.</p>` }],
  ...(automatic && { automatic }),
});
function renderPanel() {
  render(RevisionsPanel, {
    sceneId: "scene",
    projectId: "project",
    title: "Arrival",
    locked: false,
    onApplied: vi.fn(),
    onClose: vi.fn(),
  });
}

it("never collapses both sides of a comparison onto one draft when one is pruned", async () => {
  persisted.data.drafts = [
    pruneDraft("Opening", 1),
    pruneDraft("Middle", 2),
    pruneDraft("Before accepting A", 3, true),
    pruneDraft("Before accepting B", 4, true),
  ];
  pruneOnSave(1);
  renderPanel();
  const left = await screen.findByRole("region", { name: "Saved draft" });
  const right = screen.getByRole("region", { name: "Comparison version" });
  await fireEvent.click(screen.getByRole("button", { name: /Draft 2.*Middle/ }));
  await fireEvent.change(screen.getByLabelText("Compare with"), { target: { value: "2" } });
  expect(within(right).getByRole("heading", { name: "Before accepting A" })).toBeTruthy();

  await fireEvent.change(screen.getByLabelText("Revision status"), {
    target: { value: "editor_review" },
  });
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(3));
  // The comparison side moves to the nearest newer draft, never onto the saved side's draft.
  await waitFor(() =>
    expect(within(right).getByRole("heading", { name: "Before accepting B" })).toBeTruthy()
  );
  expect(within(left).getByRole("heading", { name: "Middle" })).toBeTruthy();
  // Two different drafts, so there is a real difference to show.
  expect(right.textContent).toContain("Before accepting B fox.");
  expect(right.querySelector("ins")).not.toBeNull();
  expect(screen.getByRole("status").textContent).toContain(
    "“Before accepting A” was removed because only the newest automatic drafts are kept. Now comparing “Middle” with “Before accepting B”."
  );
});

it("never names a missing draft when nothing is left to compare", async () => {
  persisted.data.drafts = [
    pruneDraft("Before accepting A", 1, true),
    pruneDraft("Before accepting B", 2, true),
  ];
  pruneOnSave(0);
  renderPanel();
  await screen.findByRole("region", { name: "Saved draft" });
  await fireEvent.change(screen.getByLabelText("Revision status"), {
    target: { value: "editor_review" },
  });
  await waitFor(() => expect(persisted.data.drafts).toHaveLength(0));
  // The newest draft is selected, so it is the one the writer was comparing.
  const notice = await screen.findByText(/was removed/);
  expect(notice.textContent).toBe(
    "“Before accepting B” was removed because only the newest automatic drafts are kept."
  );
  expect(document.body.textContent).not.toContain("undefined");
});
