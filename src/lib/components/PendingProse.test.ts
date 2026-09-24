// Export and sync read prose from the database, so pending edits must be saved
// first, and a draft that cannot be saved must stop them rather than being
// silently left out of the file or the comparison.
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { mockProject } from "../../dev/mock-data";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { proseSaves } from "../utils/proseSaves";
import { registerProseFlush, saveProseBefore } from "../utils/proseFlush";
import ClassicExportDialog from "./ClassicExportDialog.svelte";
import ExportWorkspace from "./ExportWorkspace.svelte";
import Sidebar from "./Sidebar.svelte";
import SyncDialog from "./SyncDialog.svelte";

const values = vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    clear: () => values.clear(),
  });
  return values;
});
const document = (text: string) => [
  {
    id: "a",
    title: "Chapter a",
    part: null,
    scenes: [
      {
        id: "scene-a",
        title: "Scene a",
        synopsis: null,
        blocks: [{ heading: null, html: `<p>${text}</p>` }],
      },
    ],
  },
];
let manuscript = document("Text when the workspace opened.");

beforeEach(() => {
  values.clear();
  values.set("kindling:lastExportPath", "/tmp/exports");
  window.structuredClone = structuredClone;
  currentProject.setProject({ ...mockProject, source_type: "NovelWriter", source_path: "/nw" });
  manuscript = document("Text when the workspace opened.");
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command: string) => {
    if (command === "get_project_word_count") return 1200;
    if (command === "get_export_prototype_document") return manuscript;
    if (command === "get_sync_preview") return { additions: [], changes: [] };
    if (command.startsWith("export_to")) return { files_created: 1, output_path: "/tmp/exports" };
    return [];
  });
});
afterEach(async () => {
  cleanup();
  registerProseFlush(async () => {})();
  vi.mocked(invoke).mockResolvedValue(undefined);
  await proseSaves.discard(proseSaves.draftsForRecovery());
  currentProject.setProject(null);
});

/** Leaves a beat draft that the database refused, as a locked scene would. */
async function unsavedDraft() {
  const fallback = vi.mocked(invoke).getMockImplementation();
  vi.mocked(invoke).mockRejectedValueOnce("Cannot edit a locked scene");
  await proseSaves
    .save({ projectId: mockProject.id, kind: "beat", id: "beat-1", prose: "<p>Unsaved.</p>" })
    .catch(() => {});
  vi.mocked(invoke).mockImplementation(fallback!);
}
const called = (command: string) =>
  vi.mocked(invoke).mock.calls.some(([name]) => String(name).startsWith(command));

it("flushes the editors, then refuses while a draft is unsaved", async () => {
  const flush = vi.fn(async () => {});
  registerProseFlush(flush);
  await saveProseBefore(mockProject.id, "exporting");
  expect(flush).toHaveBeenCalledTimes(1);
  await unsavedDraft();
  await expect(saveProseBefore(mockProject.id, "exporting")).rejects.toThrow(
    "Save or recover unsaved prose before exporting"
  );
});

it("an old writing surface unregistering does not remove its replacement's flush", async () => {
  const old = vi.fn(async () => {});
  const replacement = vi.fn(async () => {});
  const unregisterOld = registerProseFlush(old);
  registerProseFlush(replacement);
  unregisterOld();
  await saveProseBefore(mockProject.id, "exporting");
  expect(replacement).toHaveBeenCalledTimes(1);
  expect(old).not.toHaveBeenCalled();
});

it("classic export saves pending prose first and will not export over an unsaved draft", async () => {
  const flush = vi.fn(async () => {});
  registerProseFlush(flush);
  await unsavedDraft();
  render(ClassicExportDialog, {
    scope: "project",
    scopeId: null,
    scopeTitle: mockProject.name,
    onClose: vi.fn(),
    onSuccess: vi.fn(),
    onCustomize: vi.fn(),
  });
  await fireEvent.click(screen.getByTestId("export-format-markdown"));
  const confirm = screen.getByTestId("export-confirm") as HTMLButtonElement;
  await waitFor(() => expect(confirm.disabled).toBe(false));
  await fireEvent.click(confirm);
  expect(await screen.findByText(/Save or recover unsaved prose before exporting/)).toBeTruthy();
  expect(flush).toHaveBeenCalled();
  expect(called("export_to")).toBe(false);
});

it("export workspace exports the prose saved at export time, not when it opened", async () => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
  render(ExportWorkspace, {
    scope: "project",
    scopeId: null,
    onClose: vi.fn(),
    onClassic: vi.fn(),
  });
  await waitFor(() => expect(called("get_export_prototype_document")).toBe(true));
  await tick();
  manuscript = document("Text saved just before exporting.");
  await fireEvent.change(screen.getByLabelText("Output format"), { target: { value: "epub" } });
  vi.mocked(save).mockResolvedValueOnce("/tmp/book.epub");
  await fireEvent.click(screen.getByRole("button", { name: "Export EPUB" }));
  await waitFor(() => expect(called("export_workspace_document")).toBe(true));
  const call = vi.mocked(invoke).mock.calls.find(([name]) => name === "export_workspace_document");
  expect(JSON.stringify(call)).toContain("Text saved just before exporting.");
  expect(JSON.stringify(call)).not.toContain("Text when the workspace opened.");

  vi.mocked(invoke).mockClear();
  await unsavedDraft();
  vi.mocked(save).mockResolvedValueOnce("/tmp/book.epub");
  await fireEvent.click(screen.getByRole("button", { name: "Export EPUB" }));
  expect(await screen.findByText(/Save or recover unsaved prose before exporting/)).toBeTruthy();
  expect(called("export_workspace_document")).toBe(false);
});

it("sync neither previews nor applies over an unsaved draft", async () => {
  const flush = vi.fn(async () => {});
  registerProseFlush(flush);
  await unsavedDraft();
  render(Sidebar);
  window.dispatchEvent(new CustomEvent("kindling:sync"));
  await waitFor(() =>
    expect(ui.toast?.message).toMatch(/Save or recover unsaved prose before syncing/)
  );
  expect(flush).toHaveBeenCalled();
  expect(called("get_sync_preview")).toBe(false);
  cleanup();

  render(SyncDialog, {
    projectId: mockProject.id,
    syncPreview: {
      additions: [{ id: "scene-x", item_type: "scene", title: "New", parent_title: null }],
      changes: [],
    },
    onClose: vi.fn(),
    onSyncComplete: vi.fn(),
  });
  await fireEvent.click(screen.getByTestId("sync-confirm"));
  expect(await screen.findByText(/Save or recover unsaved prose before syncing/)).toBeTruthy();
  expect(called("apply_sync")).toBe(false);
});
