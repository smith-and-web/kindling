import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import type { Project, ReferenceCopyPreview, ReferenceCopyRequest } from "../types";
import CopyReferencesDialog from "./CopyReferencesDialog.svelte";
import { copyErrorMessage } from "../referenceCopy";

const destination = {
  id: "two",
  name: "Book Two",
  reference_types: [],
  project_type: "novel",
  created_at: "2026-09-06",
} as unknown as Project;
const source = { ...destination, id: "one", name: "Book One" };
const base: ReferenceCopyPreview = {
  references: [
    {
      id: "mara",
      reference_type: "characters",
      name: "Mara",
      description: "Captain",
      selected: true,
      conflict: false,
      action: "copy",
      destination_name: "Mara",
    },
    {
      id: "town",
      reference_type: "locations",
      name: "Town",
      description: null,
      selected: true,
      conflict: false,
      action: "copy",
      destination_name: "Town",
    },
  ],
  changes: [
    {
      kind: "field",
      entity_type: "character",
      source_name: "Age",
      destination_name: "Age (from Book One)",
    },
  ],
  enabled_types: ["characters", "locations"],
  copied: 2,
  skipped: 0,
  revision: "revision",
};
let onComplete = vi.fn<(result: unknown) => Promise<void>>();
let onClose = vi.fn<() => void>();
const mock = vi.mocked(invoke);
function makePreview(request: ReferenceCopyRequest) {
  const references = base.references.map((r) => {
    const selected = request.selection === null || request.selection.some((s) => s.id === r.id);
    return { ...r, selected, action: selected ? "copy" : "unselected" };
  });
  return { ...base, references, copied: references.filter((r) => r.selected).length };
}
function mount() {
  return render(CopyReferencesDialog, { destination, onClose, onComplete });
}
async function selectSource() {
  await waitFor(() => expect(screen.getByRole("option", { name: /Book One/ })).toBeTruthy());
  await fireEvent.change(screen.getByLabelText("Source project"), { target: { value: "one" } });
  await waitFor(() =>
    expect(
      (screen.getByRole("button", { name: "Copy 2 references" }) as HTMLButtonElement).disabled
    ).toBe(false)
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
  onComplete = vi.fn().mockResolvedValue(undefined);
  onClose = vi.fn();
  mock.mockImplementation(async (command, args) => {
    if (command === "get_all_projects") return [destination, source];
    if (command === "preview_reference_copy")
      return makePreview((args as { request: ReferenceCopyRequest }).request);
    if (command === "copy_references_between_projects")
      return {
        project: destination,
        created_reference_ids: ["new-mara", "new-town"],
        copied: 2,
        skipped: 0,
      };
    throw new Error(command);
  });
});
afterEach(cleanup);

describe("independent reference copies", () => {
  it("uses all projects, excludes destination, and commits the reviewed selection", async () => {
    mount();
    await selectSource();
    expect(screen.queryByRole("option", { name: /Book Two/ })).toBeNull();
    expect(screen.getByText(/independent copies/)).toBeTruthy();
    expect(screen.getByText(/Enable categories: Characters, Locations/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Copy 2 references" }));
    await waitFor(() => expect(onComplete).toHaveBeenCalledOnce());
    expect(mock).toHaveBeenCalledWith("copy_references_between_projects", {
      request: {
        source_project_id: "one",
        destination_project_id: "two",
        selection: base.references.map(({ id, reference_type }) => ({ id, reference_type })),
        keep_both: [],
      },
      expectedRevision: "revision",
    });
    expect(screen.getByText(/Copied 2 references from Book One to Book Two/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Copy 2 references" })).toBeNull();
  });
  it("keeps hidden search selections and supports clearing and category selection", async () => {
    mount();
    await selectSource();
    await fireEvent.input(screen.getByLabelText("Search references"), {
      target: { value: "Mara" },
    });
    expect(screen.queryByRole("checkbox", { name: "Town" })).toBeNull();
    expect(screen.getByText("2 selected across all categories")).toBeTruthy();
    await fireEvent.click(screen.getByRole("checkbox", { name: "Mara" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Copy 1 references" })).toBeTruthy()
    );
    await fireEvent.click(screen.getByRole("button", { name: "Clear selection" }));
    await waitFor(() => expect(screen.getByText("0 selected across all categories")).toBeTruthy());
    await fireEvent.click(screen.getByRole("checkbox", { name: "Select visible characters" }));
    await waitFor(() => expect(screen.getByText("1 selected across all categories")).toBeTruthy());
    await fireEvent.click(screen.getByRole("button", { name: "Select all references" }));
    await waitFor(() => expect(screen.getByText("2 selected across all categories")).toBeTruthy());
  });
  it("requires a fresh preview after a failed commit and preserves selections", async () => {
    mount();
    await selectSource();
    mock.mockRejectedValueOnce({
      code: "stale_preview",
      message: "References changed. Refresh the preview.",
    });
    await fireEvent.click(screen.getByRole("button", { name: "Copy 2 references" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("References changed")
    );
    expect(onComplete).not.toHaveBeenCalled();
    expect(
      (screen.getByRole("button", { name: "Copy 0 references" }) as HTMLButtonElement).disabled
    ).toBe(true);
    await fireEvent.click(screen.getByRole("button", { name: "Refresh preview" }));
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "Copy 2 references" }) as HTMLButtonElement).disabled
      ).toBe(false)
    );
  });
  it("does not offer another copy when a successful commit cannot refresh", async () => {
    onComplete.mockRejectedValueOnce(new Error("Refresh unavailable"));
    mount();
    await selectSource();
    await fireEvent.click(screen.getByRole("button", { name: "Copy 2 references" }));
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("copy completed"));
    await fireEvent.click(screen.getByRole("button", { name: "Refresh references" }));
    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(2));
    expect(
      mock.mock.calls.filter(([cmd]) => cmd === "copy_references_between_projects")
    ).toHaveLength(1);
  });
  it("prevents duplicate submission and Escape during a commit, then restores focus", async () => {
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    const rendered = mount();
    await selectSource();
    let resolve!: (v: unknown) => void;
    mock.mockImplementationOnce(
      () =>
        new Promise((r) => {
          resolve = r;
        })
    );
    await fireEvent.click(screen.getByRole("button", { name: "Copy 2 references" }));
    expect((screen.getByRole("button", { name: "Copying…" }) as HTMLButtonElement).disabled).toBe(
      true
    );
    await fireEvent(screen.getByRole("dialog"), new Event("cancel", { cancelable: true }));
    expect(onClose).not.toHaveBeenCalled();
    resolve({ project: destination, created_reference_ids: [], copied: 2, skipped: 0 });
    await waitFor(() => expect(screen.getByRole("button", { name: "Done" })).toBeTruthy());
    await fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(onClose).toHaveBeenCalledOnce();
    rendered.unmount();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });
  it("shows an empty state and a retry for project loading errors", async () => {
    mock.mockRejectedValueOnce("Cannot read projects");
    mount();
    await waitFor(() => expect(screen.getByRole("alert").textContent).toBe("Cannot read projects"));
    mock.mockResolvedValueOnce([destination]);
    await fireEvent.click(screen.getByRole("button", { name: "Retry loading projects" }));
    await waitFor(() => expect(screen.getByText(/Create another project first/)).toBeTruthy());
  });
  it("discards late source previews", async () => {
    mount();
    await selectSource();
    let resolve!: (v: unknown) => void;
    mock.mockImplementationOnce(
      () =>
        new Promise((r) => {
          resolve = r;
        })
    );
    await fireEvent.click(screen.getByRole("checkbox", { name: "Mara" }));
    await fireEvent.change(screen.getByLabelText("Source project"), { target: { value: "" } });
    resolve(base);
    await waitFor(() => expect(screen.queryByRole("checkbox", { name: "Mara" })).toBeNull());
    expect(
      (screen.getByRole("button", { name: "Copy 0 references" }) as HTMLButtonElement).disabled
    ).toBe(true);
  });
  it("previews an explicit keep-both choice", async () => {
    mount();
    await selectSource();
    const conflict = {
      ...base,
      copied: 1,
      skipped: 1,
      references: base.references.map((r) =>
        r.id === "mara" ? { ...r, conflict: true, action: "skip" } : r
      ),
    };
    mock.mockResolvedValueOnce(conflict);
    await fireEvent.click(screen.getByRole("button", { name: "Select all references" }));
    await waitFor(() => expect(screen.getByLabelText("Duplicate choice for Mara")).toBeTruthy());
    await fireEvent.change(screen.getByLabelText("Duplicate choice for Mara"), {
      target: { value: "keep" },
    });
    await waitFor(() =>
      expect(mock).toHaveBeenLastCalledWith("preview_reference_copy", {
        request: expect.objectContaining({
          keep_both: [{ id: "mara", reference_type: "characters" }],
        }),
      })
    );
  });
  it("provides useful messages for IPC errors", () => {
    expect(copyErrorMessage(new Error("x"))).toBe("x");
    expect(copyErrorMessage("y")).toBe("y");
    expect(copyErrorMessage(null)).toContain("Could not copy");
    expect(copyErrorMessage({ message: 42 })).toContain("Could not copy");
  });
});

it("shows an allocated source-duplicate name without requiring Keep both", async () => {
  mount();
  await selectSource();
  mock.mockResolvedValueOnce({
    ...base,
    references: [
      base.references[0],
      { ...base.references[0], id: "other-mara", destination_name: "Mara (copy)" },
    ],
  });
  await fireEvent.click(screen.getByRole("button", { name: "Select all references" }));
  await screen.findByText("Copy as: Mara (copy)");
  expect(screen.queryByLabelText("Duplicate choice for Mara")).toBeNull();
  expect(
    (screen.getByRole("button", { name: "Copy 2 references" }) as HTMLButtonElement).disabled
  ).toBe(false);
});
