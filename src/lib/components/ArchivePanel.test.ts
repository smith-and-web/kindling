import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import ArchivePanel from "./ArchivePanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockChapters, mockProject, mockScenes } from "../../dev/mock-data";
import type { ArchivedItems, Chapter, Scene } from "../types";

const archivedChapter: Chapter = {
  ...mockChapters[2],
  id: "ch-archived",
  title: "Lost Chapter",
  archived: true,
};
const archivedScene: Scene = {
  ...mockScenes[0],
  id: "sc-archived",
  chapter_id: "ch-2",
  title: "Cut Scene",
  archived: true,
};

type Handler = (args: Record<string, unknown>) => unknown;

function mockInvoke(handlers: Record<string, Handler> = {}) {
  const all: Record<string, Handler> = {
    get_archived_items: () =>
      ({ chapters: [archivedChapter], scenes: [archivedScene] }) satisfies ArchivedItems,
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

function rowFor(title: string) {
  const row = screen.getByText(title).closest("li");
  if (!row) throw new Error(`No row for ${title}`);
  return row as HTMLElement;
}

async function renderLoaded(onClose = vi.fn()) {
  render(ArchivePanel, { onClose });
  await screen.findByText("Lost Chapter");
  return onClose;
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
  currentProject.setChapters(mockChapters);
});

afterEach(() => {
  cleanup();
  currentProject.setProject(null);
});

describe("ArchivePanel", () => {
  it("shows a loading state, then lists archived chapters and scenes", async () => {
    let resolve!: (items: ArchivedItems) => void;
    vi.mocked(invoke).mockReturnValue(new Promise((r) => (resolve = r)));
    render(ArchivePanel, { onClose: vi.fn() });

    const dialog = screen.getByRole("dialog", { name: "Archive" });
    expect(within(dialog).getByText(mockProject.name)).toBeTruthy();
    expect(within(dialog).getByRole("status").textContent).toContain("Loading archived items…");
    expect(invoke).toHaveBeenCalledWith("get_archived_items", { projectId: mockProject.id });

    resolve({ chapters: [archivedChapter], scenes: [archivedScene] });
    expect(await screen.findByText("Lost Chapter")).toBeTruthy();

    expect(screen.queryByRole("status")).toBeNull();
    expect(screen.getByRole("heading", { name: "Archived chapters (1)" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Archived scenes (1)" })).toBeTruthy();
    expect(within(rowFor("Lost Chapter")).getByText("Chapter")).toBeTruthy();
    // Scenes name their parent chapter from the project's chapter list.
    expect(within(rowFor("Cut Scene")).getByText("Scene in The Awakening")).toBeTruthy();
  });

  it("falls back to Unknown Chapter when a scene's chapter is not loaded", async () => {
    mockInvoke({
      get_archived_items: () => ({
        chapters: [],
        scenes: [{ ...archivedScene, chapter_id: "gone" }],
      }),
    });
    render(ArchivePanel, { onClose: vi.fn() });

    expect(await screen.findByText("Scene in Unknown Chapter")).toBeTruthy();
    expect(screen.queryByRole("heading", { name: /Archived chapters/ })).toBeNull();
  });

  it("shows the empty state when nothing is archived", async () => {
    mockInvoke({ get_archived_items: () => ({ chapters: [], scenes: [] }) });
    render(ArchivePanel, { onClose: vi.fn() });

    expect(await screen.findByRole("heading", { name: "No archived items" })).toBeTruthy();
    expect(screen.getByText("Archived chapters and scenes will appear here.")).toBeTruthy();
    expect(screen.queryByTestId("archive-restore")).toBeNull();
  });

  it("shows a load failure", async () => {
    mockInvoke({
      get_archived_items: () => {
        throw new Error("database is locked");
      },
    });
    render(ArchivePanel, { onClose: vi.fn() });

    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("Could not load the archive")).toBeTruthy();
    expect(within(alert).getByText("database is locked")).toBeTruthy();
  });

  it("restores a chapter, removes it from the list and adds it to the project", async () => {
    const restored = { ...archivedChapter, archived: false };
    mockInvoke({ restore_chapter: () => restored });
    await renderLoaded();

    await fireEvent.click(within(rowFor("Lost Chapter")).getByTestId("archive-restore"));

    await vi.waitFor(() => expect(screen.queryByText("Lost Chapter")).toBeNull());
    expect(invoke).toHaveBeenCalledWith("restore_chapter", { chapterId: "ch-archived" });
    expect(currentProject.chapters[currentProject.chapters.length - 1]).toEqual(restored);
    expect(screen.getByText("Cut Scene")).toBeTruthy();
  });

  it("restores a scene into the open chapter", async () => {
    const restored = { ...archivedScene, archived: false };
    mockInvoke({ restore_scene: () => restored });
    currentProject.setCurrentChapter(mockChapters[1]);
    await renderLoaded();

    await fireEvent.click(within(rowFor("Cut Scene")).getByTestId("archive-restore"));

    await vi.waitFor(() => expect(screen.queryByText("Cut Scene")).toBeNull());
    expect(invoke).toHaveBeenCalledWith("restore_scene", { sceneId: "sc-archived" });
    expect(currentProject.scenes).toEqual([restored]);
  });

  it("does not add a restored scene to a different open chapter", async () => {
    mockInvoke({ restore_scene: () => ({ ...archivedScene, archived: false }) });
    currentProject.setCurrentChapter(mockChapters[2]);
    await renderLoaded();

    await fireEvent.click(within(rowFor("Cut Scene")).getByTestId("archive-restore"));

    await vi.waitFor(() => expect(screen.queryByText("Cut Scene")).toBeNull());
    expect(currentProject.scenes).toEqual([]);
  });

  it("shows a restore failure in the affected row and keeps the item", async () => {
    mockInvoke({
      restore_chapter: () => {
        throw new Error("Parent part is missing");
      },
    });
    await renderLoaded();

    await fireEvent.click(within(rowFor("Lost Chapter")).getByTestId("archive-restore"));

    const row = rowFor("Lost Chapter");
    expect((await within(row).findByRole("alert")).textContent).toBe("Parent part is missing");
    expect(within(rowFor("Cut Scene")).queryByRole("alert")).toBeNull();
    expect((within(row).getByTestId("archive-restore") as HTMLButtonElement).disabled).toBe(false);
  });

  it("uses a fallback message when a scene restore rejects without detail", async () => {
    mockInvoke({
      restore_scene: () => {
        throw "";
      },
    });
    await renderLoaded();

    await fireEvent.click(within(rowFor("Cut Scene")).getByTestId("archive-restore"));

    expect((await within(rowFor("Cut Scene")).findByRole("alert")).textContent).toBe(
      "Could not restore this scene."
    );
  });

  it("confirms inline before permanently deleting a chapter", async () => {
    mockInvoke({ delete_chapter: () => undefined });
    await renderLoaded();

    const trigger = within(rowFor("Lost Chapter")).getByRole("button", { name: "Delete" });
    await fireEvent.click(trigger);

    const confirm = screen.getByRole("alertdialog", { name: "Permanently delete “Lost Chapter”?" });
    expect(confirm.textContent).toContain("This permanently deletes the chapter.");
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    expect(document.activeElement).toBe(within(confirm).getByRole("button", { name: "Cancel" }));
    expect(commands()).not.toContain("delete_chapter");

    await fireEvent.click(within(confirm).getByRole("button", { name: "Delete permanently" }));

    await vi.waitFor(() => expect(screen.queryByText("Lost Chapter")).toBeNull());
    expect(invoke).toHaveBeenCalledWith("delete_chapter", { chapterId: "ch-archived" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("permanently deletes a scene with its chapter id", async () => {
    mockInvoke({ delete_scene: () => undefined });
    await renderLoaded();

    await fireEvent.click(within(rowFor("Cut Scene")).getByRole("button", { name: "Delete" }));
    const confirm = screen.getByRole("alertdialog", { name: "Permanently delete “Cut Scene”?" });
    expect(confirm.textContent).toContain("This permanently deletes the scene.");
    await fireEvent.click(within(confirm).getByRole("button", { name: "Delete permanently" }));

    await vi.waitFor(() => expect(screen.queryByText("Cut Scene")).toBeNull());
    expect(invoke).toHaveBeenCalledWith("delete_scene", {
      sceneId: "sc-archived",
      chapterId: "ch-2",
    });
  });

  it("cancels the inline delete without invoking anything and refocuses the trigger", async () => {
    mockInvoke();
    await renderLoaded();
    const callsBefore = vi.mocked(invoke).mock.calls.length;

    const trigger = within(rowFor("Lost Chapter")).getByRole("button", { name: "Delete" });
    await fireEvent.click(trigger);
    await fireEvent.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "Cancel" })
    );
    await tick();

    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(vi.mocked(invoke).mock.calls.length).toBe(callsBefore);
    expect(screen.getByText("Lost Chapter")).toBeTruthy();
    expect(document.activeElement).toBe(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
  });

  it("toggles the confirmation closed when Delete is pressed again", async () => {
    mockInvoke();
    await renderLoaded();

    const trigger = within(rowFor("Lost Chapter")).getByRole("button", { name: "Delete" });
    await fireEvent.click(trigger);
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    await fireEvent.click(trigger);
    await tick();

    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(commands()).not.toContain("delete_chapter");
  });

  it("shows a delete failure inside the confirmation and keeps the item", async () => {
    mockInvoke({
      delete_chapter: () => {
        throw new Error("Chapter is locked");
      },
    });
    await renderLoaded();

    await fireEvent.click(within(rowFor("Lost Chapter")).getByRole("button", { name: "Delete" }));
    const confirm = screen.getByRole("alertdialog");
    await fireEvent.click(within(confirm).getByRole("button", { name: "Delete permanently" }));

    expect((await within(confirm).findByRole("alert")).textContent).toBe("Chapter is locked");
    expect(screen.getByText("Lost Chapter")).toBeTruthy();
    expect(
      (within(confirm).getByRole("button", { name: "Delete permanently" }) as HTMLButtonElement)
        .disabled
    ).toBe(false);
  });

  it("closes exactly once on Escape", async () => {
    mockInvoke();
    const onClose = await renderLoaded();

    // Dispatched inside the scrim, so both the scrim and window listeners see it.
    await fireEvent.keyDown(screen.getByRole("dialog", { name: "Archive" }), { key: "Escape" });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("closes once on an Escape that reaches only the window", async () => {
    mockInvoke();
    const onClose = await renderLoaded();

    await fireEvent.keyDown(window, { key: "Escape" });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("dismisses an open delete confirmation on Escape instead of closing", async () => {
    mockInvoke();
    const onClose = await renderLoaded();

    const trigger = within(rowFor("Lost Chapter")).getByRole("button", { name: "Delete" });
    await fireEvent.click(trigger);
    await fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
    await tick();

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it("closes from the archive-close header button", async () => {
    mockInvoke();
    const onClose = await renderLoaded();

    const close = screen.getByTestId("archive-close");
    expect(close.getAttribute("aria-label")).toBe("Close Archive");
    await fireEvent.click(close);

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("renders one archive-restore button per archived item", async () => {
    mockInvoke();
    await renderLoaded();

    const restores = screen.getAllByTestId("archive-restore");
    expect(restores).toHaveLength(2);
    expect(restores.every((button) => button.textContent?.trim() === "Restore")).toBe(true);
  });

  it("closes on a backdrop click but not a click inside the surface", async () => {
    mockInvoke();
    const onClose = await renderLoaded();

    await fireEvent.click(screen.getByText("Lost Chapter"));
    expect(onClose).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("dialog", { name: "Archive" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
