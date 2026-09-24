/**
 * Every div-based modal dialog shares modalFocus: it takes focus on open, keeps Tab
 * inside, closes on Escape without the key reaching window listeners behind it
 * (ScenePanel discards a discovery-note draft on a window Escape), and returns
 * focus to where it was when it closes.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";

import { invoke } from "@tauri-apps/api/core";
import { mockProject } from "../../dev/mock-data";
import { currentProject } from "../stores/project.svelte";
import { tabbableElements } from "../utils/modalFocus";
import ArchivePanel from "./ArchivePanel.svelte";
import ConfirmDialog from "./ConfirmDialog.svelte";
import ExportSuccessDialog from "./ExportSuccessDialog.svelte";
import ImportLongformDialog from "./ImportLongformDialog.svelte";
import NewProjectDialog from "./NewProjectDialog.svelte";
import PartDeleteDialog from "./PartDeleteDialog.svelte";
import ReferenceClassificationDialog from "./ReferenceClassificationDialog.svelte";
import RenameDialog from "./RenameDialog.svelte";
import SnapshotsPanel from "./SnapshotsPanel.svelte";
import SyncDialog from "./SyncDialog.svelte";
import SyncSummaryDialog from "./SyncSummaryDialog.svelte";
import TemplateBrowser from "./TemplateBrowser.svelte";

const summary = {
  chapters_added: 1,
  chapters_updated: 0,
  scenes_added: 2,
  scenes_updated: 0,
  beats_added: 0,
  beats_updated: 0,
  prose_preserved: 0,
  prose_updated: 0,
};

type DialogComponent = Parameters<typeof render>[0];

interface Case {
  name: string;
  component: DialogComponent;
  closeProp: string;
  props: () => Record<string, unknown>;
  setup?: () => void;
}

const cases: Case[] = [
  {
    name: "Archive",
    component: ArchivePanel as DialogComponent,
    closeProp: "onClose",
    props: () => ({}),
    setup: () =>
      vi
        .mocked(invoke)
        .mockImplementation(async (cmd) =>
          cmd === "get_archived_items" ? { chapters: [], scenes: [] } : []
        ),
  },
  {
    name: "Confirm",
    component: ConfirmDialog as DialogComponent,
    closeProp: "onCancel",
    props: () => ({ title: "Delete scene", message: "Delete it?", onConfirm: vi.fn() }),
  },
  {
    name: "Export success",
    component: ExportSuccessDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({
      result: {
        output_path: "/tmp/out",
        files_created: 1,
        chapters_exported: 1,
        scenes_exported: 1,
      },
    }),
  },
  {
    name: "Longform import",
    component: ImportLongformDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({ onSelectIndex: vi.fn(), onSelectVault: vi.fn() }),
  },
  {
    name: "New project",
    component: NewProjectDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({}),
  },
  {
    name: "Part delete",
    component: PartDeleteDialog as DialogComponent,
    closeProp: "onCancel",
    props: () => ({
      partTitle: "Act One",
      childChapterCount: 2,
      onDeletePartOnly: vi.fn(),
      onDeletePartAndChapters: vi.fn(),
    }),
  },
  {
    name: "Reference classification",
    component: ReferenceClassificationDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({ projectId: mockProject.id, onComplete: vi.fn() }),
    // With nothing to classify the dialog closes itself, so give it one character.
    setup: () =>
      vi
        .mocked(invoke)
        .mockImplementation(async (cmd) =>
          cmd === "get_characters" ? [{ id: "c1", name: "Alice", description: null }] : []
        ),
  },
  {
    name: "Rename",
    component: RenameDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({ title: "Rename scene", currentName: "Opening", onSave: vi.fn() }),
  },
  {
    name: "Snapshots",
    component: SnapshotsPanel as DialogComponent,
    closeProp: "onClose",
    props: () => ({ prepareRestore: vi.fn() }),
  },
  {
    name: "Sync",
    component: SyncDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({
      projectId: mockProject.id,
      syncPreview: { additions: [], changes: [] },
      onSyncComplete: vi.fn(),
    }),
  },
  {
    name: "Sync summary",
    component: SyncSummaryDialog as DialogComponent,
    closeProp: "onClose",
    props: () => ({ summary }),
  },
  {
    name: "Template browser",
    component: TemplateBrowser as DialogComponent,
    closeProp: "onClose",
    props: () => ({ onSelect: vi.fn() }),
  },
];

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});

beforeEach(() => {
  currentProject.setProject(mockProject);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue([]);
});

afterEach(() => {
  currentProject.setProject(null);
  document.querySelectorAll("[data-opener]").forEach((el) => el.remove());
});

function open(testCase: Case) {
  testCase.setup?.();
  const opener = document.createElement("button");
  opener.dataset.opener = "";
  opener.textContent = "Opener";
  document.body.append(opener);
  opener.focus();
  const view = { unmount: () => {} };
  const close = vi.fn(() => view.unmount());
  view.unmount = render(testCase.component, {
    props: { ...testCase.props(), [testCase.closeProp]: close } as DialogComponent,
  }).unmount;
  return { opener, close };
}

function dialogRoot(): HTMLElement {
  return document.activeElement!.closest<HTMLElement>(".dialog-scrim")!;
}

async function focusedInside() {
  await waitFor(() =>
    expect(
      document.activeElement?.closest(".dialog-scrim [aria-modal], .dialog-scrim")
    ).toBeTruthy()
  );
}

describe.each(cases)("$name dialog", (testCase) => {
  it("moves focus inside on open and wraps Tab at the edges", async () => {
    open(testCase);
    await focusedInside();
    const items = tabbableElements(dialogRoot());
    const last = items[items.length - 1];
    last.focus();
    await fireEvent.keyDown(last, { key: "Tab" });
    expect(document.activeElement).toBe(items[0]);
    await fireEvent.keyDown(items[0], { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("closes once on Escape, keeps the key from the window, and restores focus", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    try {
      const { opener, close } = open(testCase);
      await focusedInside();
      await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
      expect(close).toHaveBeenCalledTimes(1);
      expect(behind).not.toHaveBeenCalled();
      expect(document.activeElement).toBe(opener);
    } finally {
      window.removeEventListener("keydown", behind);
    }
  });
});

describe("destructive confirmations", () => {
  it.each(["Confirm", "Part delete"])("%s starts on Cancel", async (name) => {
    open(cases.find((c) => c.name === name)!);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("dialog-cancel")));
  });
});

describe("Enter", () => {
  it("does not save a rename when Enter is pressed on Cancel", async () => {
    const onSave = vi.fn();
    render(RenameDialog, {
      props: { title: "Rename", currentName: "Opening", onSave, onClose: vi.fn() },
    });
    const cancel = screen.getByRole("button", { name: "Cancel" });
    cancel.focus();
    await fireEvent.keyDown(cancel, { key: "Enter" });
    expect(onSave).not.toHaveBeenCalled();
    await fireEvent.keyDown(screen.getByLabelText("Name"), { key: "Enter" });
    expect(onSave).toHaveBeenCalledWith("Opening");
  });

  it("does not close Export complete when Enter activates Open folder", async () => {
    const onClose = vi.fn();
    render(ExportSuccessDialog, { props: { ...cases[2].props(), onClose } as never });
    const openFolder = screen.getByRole("button", { name: "Open folder" });
    await fireEvent.keyDown(openFolder, { key: "Enter" });
    expect(onClose).not.toHaveBeenCalled();
    await fireEvent.keyDown(document.querySelector(".dialog-scrim")!, { key: "Enter" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

describe("nested dialogs", () => {
  it("Escape in Template browser closes only it, not New project (#334)", async () => {
    const onClose = vi.fn();
    render(NewProjectDialog, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: /template/i }));
    const browser = await screen.findByRole("dialog", { name: "Story structure templates" });
    await waitFor(() => expect(browser.contains(document.activeElement)).toBe(true));
    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Story structure templates" })).toBeNull();
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "New project" })).toBeTruthy();
  });
});

describe("embedded confirmation", () => {
  it("leaves keys and focus to the host dialog", async () => {
    const onCancel = vi.fn();
    render(ConfirmDialog, {
      props: { embedded: true, title: "Quit?", message: "Quit?", onConfirm: vi.fn(), onCancel },
    });
    await fireEvent.keyDown(screen.getByTestId("dialog-cancel"), { key: "Escape" });
    expect(onCancel).not.toHaveBeenCalled();
  });
});
