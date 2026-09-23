import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import SnapshotsPanel from "./SnapshotsPanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockChapters, mockProject } from "../../dev/mock-data";
import type { SnapshotMetadata } from "../types";

// Restore/delete-trigger/prepareRestore ordering is covered in SearchSaving.test.ts;
// these tests cover the inline create, delete and restore flows around it.

function snapshot(overrides: Partial<SnapshotMetadata> = {}): SnapshotMetadata {
  return {
    id: "snap-1",
    project_id: mockProject.id,
    name: "Before rewrite",
    description: "Act two as drafted",
    trigger_type: "manual",
    created_at: "2026-01-01T00:00:00Z",
    file_path: "/snapshots/snap-1.db",
    file_size: 2048,
    chapter_count: 3,
    scene_count: 1,
    beat_count: 0,
    schema_version: 1,
    ...overrides,
  };
}

type Handler = (args: Record<string, unknown>) => unknown;

function mockInvoke(handlers: Record<string, Handler> = {}) {
  const all: Record<string, Handler> = {
    list_snapshots: () => [snapshot()],
    ...handlers,
  };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const handler = all[command];
    if (!handler) throw new Error(`Unexpected invoke: ${command}`);
    return handler((args ?? {}) as Record<string, unknown>);
  });
}

function commands() {
  return vi.mocked(invoke).mock.calls.map(([command]) => command);
}

function panel() {
  return screen.getByRole("dialog", { name: "Snapshots" });
}

async function renderLoaded(overrides: { onClose?: () => void } = {}) {
  const props = {
    onClose: overrides.onClose ?? vi.fn(),
    prepareRestore: vi.fn().mockResolvedValue(undefined),
  };
  render(SnapshotsPanel, props);
  await screen.findByText("Before rewrite");
  return props;
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
});

afterEach(() => {
  cleanup();
  currentProject.setProject(null);
});

describe("SnapshotsPanel", () => {
  it("lists snapshots with their metadata", async () => {
    mockInvoke();
    await renderLoaded();

    const list = screen.getByRole("list", { name: "Snapshots" });
    const row = within(list).getByRole("listitem");
    expect(row.textContent).toContain("Manual · 2.0 KB");
    expect(row.textContent).toContain("3 chapters · 1 scene · 0 beats");
    expect(within(row).getByText("Act two as drafted")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("list_snapshots", { projectId: mockProject.id });
  });

  it("shows the empty state when there are no snapshots", async () => {
    mockInvoke({ list_snapshots: () => [] });
    render(SnapshotsPanel, { onClose: vi.fn(), prepareRestore: vi.fn() });

    expect(await screen.findByRole("heading", { name: "No snapshots yet" })).toBeTruthy();
  });

  it("shows a load failure", async () => {
    mockInvoke({
      list_snapshots: () => {
        throw new Error("snapshot folder missing");
      },
    });
    render(SnapshotsPanel, { onClose: vi.fn(), prepareRestore: vi.fn() });

    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("Could not load snapshots")).toBeTruthy();
    expect(within(alert).getByText("snapshot folder missing")).toBeTruthy();
  });

  describe("inline create", () => {
    it("opens a form, submits it and shows the success notice", async () => {
      const created = snapshot({ id: "snap-2", name: "Draft two done", description: null });
      mockInvoke({ create_snapshot: () => created });
      await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));

      const form = screen.getByRole("region", { name: "New snapshot" });
      const name = within(form).getByLabelText("Name") as HTMLInputElement;
      expect(document.activeElement).toBe(name);
      expect(name.value).toMatch(/^Snapshot /);
      expect(screen.queryByTestId("snapshot-create-button")).toBeNull();

      await fireEvent.input(name, { target: { value: "  Draft two done  " } });
      await fireEvent.input(within(form).getByLabelText(/Description/), {
        target: { value: "Finished the middle" },
      });
      await fireEvent.click(screen.getByTestId("snapshot-confirm-create"));

      const notice = await screen.findByRole("status");
      expect(within(notice).getByText("Snapshot created")).toBeTruthy();
      expect(notice.textContent).toContain("“Draft two done” is now the newest restore point.");
      expect(invoke).toHaveBeenCalledWith("create_snapshot", {
        projectId: mockProject.id,
        options: {
          name: "Draft two done",
          description: "Finished the middle",
          trigger_type: "manual",
        },
      });

      expect(screen.queryByRole("region", { name: "New snapshot" })).toBeNull();
      const rows = within(screen.getByRole("list", { name: "Snapshots" })).getAllByRole("listitem");
      expect(rows.map((row) => row.querySelector("p")?.textContent?.trim())).toEqual([
        "Draft two done",
        "Before rewrite",
      ]);
    });

    it("submits on Enter and omits an empty description", async () => {
      mockInvoke({ create_snapshot: () => snapshot({ id: "snap-2", name: "Quick" }) });
      await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));
      const name = screen.getByLabelText("Name");
      await fireEvent.input(name, { target: { value: "Quick" } });
      await fireEvent.keyDown(name, { key: "Enter" });

      expect(await screen.findByText("Snapshot created")).toBeTruthy();
      expect(invoke).toHaveBeenCalledWith("create_snapshot", {
        projectId: mockProject.id,
        options: { name: "Quick", description: undefined, trigger_type: "manual" },
      });
    });

    it("disables create for a blank name", async () => {
      mockInvoke();
      await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));
      const name = screen.getByLabelText("Name");
      await fireEvent.input(name, { target: { value: "   " } });

      const confirm = screen.getByTestId("snapshot-confirm-create") as HTMLButtonElement;
      expect(confirm.disabled).toBe(true);
      await fireEvent.keyDown(name, { key: "Enter" });
      expect(commands()).not.toContain("create_snapshot");
    });

    it("shows a create failure in the form and keeps it open", async () => {
      mockInvoke({
        create_snapshot: () => {
          throw new Error("Disk full");
        },
      });
      await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));
      await fireEvent.click(screen.getByTestId("snapshot-confirm-create"));

      const form = screen.getByRole("region", { name: "New snapshot" });
      expect((await within(form).findByRole("alert")).textContent).toBe("Disk full");
      expect(screen.queryByText("Snapshot created")).toBeNull();
      expect((screen.getByTestId("snapshot-confirm-create") as HTMLButtonElement).disabled).toBe(
        false
      );
      expect(
        within(screen.getByRole("list", { name: "Snapshots" })).getAllByRole("listitem")
      ).toHaveLength(1);
    });

    it("cancels the form and refocuses Create snapshot", async () => {
      mockInvoke();
      await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));
      await fireEvent.click(
        within(screen.getByRole("region", { name: "New snapshot" })).getByRole("button", {
          name: "Cancel",
        })
      );
      await tick();

      expect(screen.queryByRole("region", { name: "New snapshot" })).toBeNull();
      expect(document.activeElement).toBe(screen.getByTestId("snapshot-create-button"));
      expect(commands()).not.toContain("create_snapshot");
    });

    it("closes only the create step on Escape, not the whole panel", async () => {
      mockInvoke();
      const { onClose } = await renderLoaded();

      await fireEvent.click(screen.getByTestId("snapshot-create-button"));
      await fireEvent.keyDown(screen.getByLabelText("Name"), { key: "Escape" });
      await tick();

      expect(onClose).not.toHaveBeenCalled();
      expect(panel()).toBeTruthy();
      expect(screen.queryByRole("region", { name: "New snapshot" })).toBeNull();
      expect(screen.getByText("Before rewrite")).toBeTruthy();
      expect(document.activeElement).toBe(screen.getByTestId("snapshot-create-button"));

      // With the create step gone, the next Escape closes the panel once.
      await fireEvent.keyDown(panel(), { key: "Escape" });
      expect(onClose).toHaveBeenCalledTimes(1);
    });
  });

  describe("inline delete", () => {
    function deleteTrigger() {
      return screen.getByRole("button", { name: /^Delete snapshot from / });
    }

    it("confirms inline, then deletes the snapshot", async () => {
      mockInvoke({ delete_snapshot: () => undefined });
      await renderLoaded();

      await fireEvent.click(deleteTrigger());
      const confirm = screen.getByRole("alertdialog", { name: "Delete this snapshot?" });
      expect(confirm.textContent).toContain("Your project is not changed.");
      expect(document.activeElement).toBe(within(confirm).getByRole("button", { name: "Cancel" }));
      expect(commands()).not.toContain("delete_snapshot");

      await fireEvent.click(within(confirm).getByRole("button", { name: "Delete" }));

      expect(await screen.findByRole("heading", { name: "No snapshots yet" })).toBeTruthy();
      expect(invoke).toHaveBeenCalledWith("delete_snapshot", { snapshotId: "snap-1" });
      expect(screen.queryByRole("alertdialog")).toBeNull();
    });

    it("cancels without invoking and refocuses the delete trigger", async () => {
      mockInvoke();
      await renderLoaded();

      const trigger = deleteTrigger();
      await fireEvent.click(trigger);
      await fireEvent.click(
        within(screen.getByRole("alertdialog")).getByRole("button", { name: "Cancel" })
      );
      await tick();

      expect(screen.queryByRole("alertdialog")).toBeNull();
      expect(commands()).not.toContain("delete_snapshot");
      expect(document.activeElement).toBe(trigger);
      expect(screen.getByText("Before rewrite")).toBeTruthy();
    });

    it("shows a delete failure inside the confirmation and keeps the snapshot", async () => {
      mockInvoke({
        delete_snapshot: () => {
          throw new Error("Snapshot file is in use");
        },
      });
      await renderLoaded();

      await fireEvent.click(deleteTrigger());
      const confirm = screen.getByRole("alertdialog");
      await fireEvent.click(within(confirm).getByRole("button", { name: "Delete" }));

      expect((await within(confirm).findByRole("alert")).textContent).toBe(
        "Snapshot file is in use"
      );
      expect(screen.getByText("Before rewrite")).toBeTruthy();
      expect(
        (within(confirm).getByRole("button", { name: "Delete" }) as HTMLButtonElement).disabled
      ).toBe(false);
    });
  });

  describe("inline restore", () => {
    it("restores into a new project when that mode is chosen", async () => {
      const restoredProject = { ...mockProject, id: "proj-copy", name: "Ember & Ash (Restored)" };
      mockInvoke({
        restore_snapshot: () => restoredProject,
        get_chapters: () => mockChapters,
        get_characters: () => [],
        get_locations: () => [],
      });
      const { onClose, prepareRestore } = await renderLoaded();

      await fireEvent.click(screen.getByRole("button", { name: "Restore snapshot" }));
      const confirm = screen.getByRole("alertdialog", { name: "Restore this snapshot?" });
      expect(document.activeElement).toBe(
        within(confirm).getByRole("radio", { name: /Replace current project/ })
      );

      await fireEvent.click(within(confirm).getByRole("radio", { name: /Create new project/ }));
      const nameInput = within(confirm).getByLabelText("New project name") as HTMLInputElement;
      expect(nameInput.value).toBe("Ember & Ash (Restored)");

      await fireEvent.click(within(confirm).getByRole("button", { name: "Restore" }));

      await vi.waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
      expect(prepareRestore).toHaveBeenCalledTimes(1);
      expect(invoke).toHaveBeenCalledWith("restore_snapshot", {
        snapshotId: "snap-1",
        options: { mode: "create_new", new_project_name: "Ember & Ash (Restored)" },
      });
      expect(invoke).toHaveBeenCalledWith("get_chapters", { projectId: "proj-copy" });
      expect(currentProject.value).toEqual(restoredProject);
      expect(currentProject.chapters).toEqual(mockChapters);
      expect(currentProject.currentChapter).toBeNull();
    });

    it("requires a name before restoring into a new project", async () => {
      mockInvoke();
      await renderLoaded();

      await fireEvent.click(screen.getByRole("button", { name: "Restore snapshot" }));
      const confirm = screen.getByRole("alertdialog");
      await fireEvent.click(within(confirm).getByRole("radio", { name: /Create new project/ }));
      await fireEvent.input(within(confirm).getByLabelText("New project name"), {
        target: { value: "  " },
      });

      expect(within(confirm).getByText("Enter a name for the new project.")).toBeTruthy();
      expect(
        (within(confirm).getByRole("button", { name: "Restore" }) as HTMLButtonElement).disabled
      ).toBe(true);
    });

    it("cancels and returns focus to the Restore snapshot opener", async () => {
      mockInvoke();
      const { onClose, prepareRestore } = await renderLoaded();

      const opener = screen.getByRole("button", { name: "Restore snapshot" });
      await fireEvent.click(opener);
      expect(opener.getAttribute("aria-expanded")).toBe("true");
      await fireEvent.click(
        within(screen.getByRole("alertdialog")).getByRole("button", { name: "Cancel" })
      );
      await tick();

      expect(screen.queryByRole("alertdialog")).toBeNull();
      expect(document.activeElement).toBe(opener);
      expect(opener.getAttribute("aria-expanded")).toBe("false");
      expect(prepareRestore).not.toHaveBeenCalled();
      expect(commands()).not.toContain("restore_snapshot");
      expect(onClose).not.toHaveBeenCalled();
    });

    it("dismisses the restore confirmation on Escape and refocuses the opener", async () => {
      mockInvoke();
      const { onClose } = await renderLoaded();

      const opener = screen.getByRole("button", { name: "Restore snapshot" });
      await fireEvent.click(opener);
      await fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
      await tick();

      expect(onClose).not.toHaveBeenCalled();
      expect(screen.queryByRole("alertdialog")).toBeNull();
      expect(document.activeElement).toBe(opener);
    });

    it("shows a restore error in the row without hiding the list", async () => {
      mockInvoke({
        list_snapshots: () => [snapshot(), snapshot({ id: "snap-0", name: "Older" })],
        restore_snapshot: () => {
          throw new Error("Snapshot is corrupt");
        },
      });
      const { onClose } = await renderLoaded();

      const openers = screen.getAllByRole("button", { name: "Restore snapshot" });
      await fireEvent.click(openers[0]);
      const confirm = screen.getByRole("alertdialog");
      await fireEvent.click(within(confirm).getByRole("button", { name: "Restore" }));

      expect((await within(confirm).findByRole("alert")).textContent).toBe("Snapshot is corrupt");
      expect(onClose).not.toHaveBeenCalled();
      const list = screen.getByRole("list", { name: "Snapshots" });
      expect(within(list).getByText("Before rewrite")).toBeTruthy();
      expect(within(list).getByText("Older")).toBeTruthy();
      expect(commands()).not.toContain("get_chapters");
      expect(currentProject.value).toEqual(mockProject);
      // The writer can retry once the restore settles.
      expect(
        (within(confirm).getByRole("button", { name: "Restore" }) as HTMLButtonElement).disabled
      ).toBe(false);
    });
  });
});
