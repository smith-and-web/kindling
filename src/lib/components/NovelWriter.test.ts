import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import ExportDialog from "./ExportDialog.svelte";
import SyncDialog from "./SyncDialog.svelte";
import Sidebar from "./Sidebar.svelte";
import Onboarding from "./Onboarding.svelte";
import StartScreen from "./StartScreen.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { runImport } from "../utils/import";
import { COMMAND_DEFS } from "../commands";
import type { Project, SyncPreview } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

const project: Project = {
  id: "project-1",
  name: "A story",
  source_type: "NovelWriter",
  source_path: "/source",
  created_at: "2026-09-05",
  modified_at: "2026-09-05",
  author_pen_name: null,
  genre: null,
  description: null,
  word_target: null,
  reference_types: ["characters", "locations"],
  project_type: "novel",
  target_page_count: null,
};
const result = {
  output_path: "/export",
  files_created: 4,
  chapters_exported: 1,
  scenes_exported: 2,
};
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(open).mockReset();
  localStorage.clear();
  currentProject.setProject(null);
  currentProject.setProject({ ...project });
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_project_word_count" ? 2000 : cmd === "export_to_novelwriter" ? result : []
  );
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  currentProject.setProject(null);
});

describe("novelWriter export", () => {
  function dialog() {
    const onClose = vi.fn();
    const onSuccess = vi.fn();
    render(ExportDialog, {
      scope: "project",
      scopeId: null,
      scopeTitle: "Project",
      onClose,
      onSuccess,
    });
    return { onClose, onSuccess };
  }
  it("defaults beat comments and notes on, requires a folder, and sends options", async () => {
    const { onSuccess } = dialog();
    await fireEvent.click(screen.getByRole("radio", { name: /novelWriter/ }));
    expect((screen.getByLabelText("Include beat comments") as HTMLInputElement).checked).toBe(true);
    expect(
      (screen.getByLabelText("Include characters, locations and notes") as HTMLInputElement).checked
    ).toBe(true);
    expect(screen.getByText(/Turning off beat comments disables beat-level sync/)).toBeTruthy();
    const exportButton = screen.getByTestId("export-confirm") as HTMLButtonElement;
    expect(exportButton.disabled).toBe(true);
    vi.mocked(open).mockResolvedValue("/export");
    await fireEvent.click(
      screen.getByRole("button", { name: "Choose novelWriter destination folder" })
    );
    expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: true }));
    await fireEvent.click(screen.getByLabelText("Include characters, locations and notes"));
    await fireEvent.click(exportButton);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("export_to_novelwriter", {
        projectId: project.id,
        outputPath: "/export",
        options: { include_beat_comments: true, include_notes: false, create_snapshot: false },
      })
    );
    expect(onSuccess).toHaveBeenCalledWith(result);
  });
  it("keeps the dialog open when export fails", async () => {
    const { onClose, onSuccess } = dialog();
    await fireEvent.click(screen.getByRole("radio", { name: /novelWriter/ }));
    vi.mocked(open).mockResolvedValue("/occupied");
    await fireEvent.click(
      screen.getByRole("button", { name: "Choose novelWriter destination folder" })
    );
    vi.mocked(invoke).mockRejectedValue("Choose an empty destination folder");
    await fireEvent.click(screen.getByTestId("export-confirm"));
    expect(await screen.findByText("Choose an empty destination folder")).toBeTruthy();
    expect(onClose).not.toHaveBeenCalled();
    expect(onSuccess).not.toHaveBeenCalled();
  });
  it("hides novelWriter for screenplay projects", () => {
    currentProject.setProject({ ...project, project_type: "screenplay" });
    dialog();
    expect(screen.queryByRole("radio", { name: /novelWriter/ })).toBeNull();
  });
});

describe("novelWriter import", () => {
  it("uses the directory picker and surfaces failures", async () => {
    vi.mocked(open).mockResolvedValue("/source");
    vi.mocked(invoke).mockResolvedValue(project);
    expect(await runImport("novelwriter")).toEqual(project);
    expect(open).toHaveBeenCalledWith({ multiple: false, directory: true });
    expect(invoke).toHaveBeenCalledWith("import_novelwriter", { path: "/source" });
    const showError = vi.spyOn(ui, "showError");
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValue("nwProject.nwx missing");
    expect(await runImport("novelwriter")).toBeNull();
    expect(showError).toHaveBeenCalledWith(expect.stringContaining("nwProject.nwx missing"));
    vi.mocked(open).mockResolvedValue(null);
    expect(await runImport("novelwriter")).toBeNull();
  });
  it("offers guided preview/import and a menu command", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue("/source");
    vi.mocked(invoke).mockImplementation(async (cmd) =>
      cmd === "preview_import"
        ? {
            project_name: "Guided novel",
            chapter_count: 1,
            scene_count: 2,
            beat_count: 3,
            character_count: 1,
            location_count: 1,
          }
        : project
    );
    render(Onboarding);
    await fireEvent.click(screen.getByRole("button", { name: /novelWriter/ }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("preview_import", {
        path: "/source",
        format: "novelwriter",
      })
    );
    expect(await screen.findByText(/Guided novel/)).toBeTruthy();
    await fireEvent.click(screen.getByTestId("guided-import-confirm"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("import_novelwriter", { path: "/source" })
    );
    expect(COMMAND_DEFS.some((c) => c.id === "import_novelwriter")).toBe(true);
  });
});

describe("prose sync UI", () => {
  it("shows full before/after prose and selects it independently", async () => {
    const syncPreview: SyncPreview = {
      additions: [],
      changes: [
        {
          id: "prose-1",
          item_type: "scene",
          field: "prose",
          item_title: "Arrival",
          current_value: "Before\n\n" + "Long paragraph. ".repeat(40),
          new_value: "After\n\nA new paragraph.",
          db_id: "scene-1",
        },
        {
          id: "title-1",
          item_type: "scene",
          field: "title",
          item_title: "Arrival",
          current_value: "Arrival",
          new_value: "Departure",
          db_id: "scene-1",
        },
      ],
    };
    render(SyncDialog, {
      projectId: project.id,
      syncPreview,
      onClose: vi.fn(),
      onSyncComplete: vi.fn(),
    });
    expect(screen.getByTestId("sync-prose-diff").textContent).toContain(
      syncPreview.changes[0].current_value
    );
    await fireEvent.click(screen.getAllByRole("checkbox")[0]);
    await fireEvent.click(screen.getByTestId("sync-confirm"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("apply_sync", {
        projectId: project.id,
        acceptedChangeIds: ["prose-1"],
        acceptedAdditionIds: [],
      })
    );
  });
  it.each([
    "Scrivener",
    "Blank",
    "NovelWriter",
    "Plottr",
    "YWriter",
    "Markdown",
    "Longform",
  ] as const)("gates sidebar sync for %s", async (source_type) => {
    currentProject.setProject({ ...project, source_type });
    render(Sidebar);
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    expect(screen.queryByTestId("sync-button") !== null).toBe(
      !["Scrivener", "Blank"].includes(source_type)
    );
  });
});

// Reference categories introduced by novelWriter must work throughout the UI.
import ProjectSettings from "./ProjectSettings.svelte";
import ReferenceEditDialog from "./ReferenceEditDialog.svelte";
import ReferenceClassificationDialog from "./ReferenceClassificationDialog.svelte";
import ScenePanel from "./ScenePanel.svelte";
import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
import { mockScenes, mockChapters } from "../../dev/mock-data";

describe("timeline and custom references", () => {
  it("loads both categories in classification and submits a character reclassification", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "get_characters") return [{ id: "jane", name: "Jane", description: null }];
      if (cmd === "get_references") {
        const type = (args as { referenceType: string }).referenceType;
        if (["timelines", "custom"].includes(type))
          return [{ id: type, name: `Loaded ${type}`, description: null }];
      }
      return cmd === "reclassify_references" ? project : [];
    });
    render(ReferenceClassificationDialog, {
      projectId: project.id,
      onClose: vi.fn(),
      onComplete: vi.fn(),
    });
    await screen.findByText("Loaded timelines");
    expect(screen.getByText("Loaded custom")).toBeTruthy();
    const select = screen.getByText("Jane").closest("tr")!.querySelector("select")!;
    await fireEvent.change(select, { target: { value: "timelines" } });
    await fireEvent.click(screen.getByText("Apply changes"));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("reclassify_references", {
        projectId: project.id,
        changes: [{ reference_id: "jane", new_type: "timelines" }],
      })
    );
  });

  it("offers custom field definitions for both types in project settings", async () => {
    currentProject.setProject({ ...project, reference_types: ["timelines", "custom"] });
    render(ProjectSettings, { project: currentProject.value!, section: "fields", onSave: vi.fn() });
    await waitFor(() => {
      for (const entityType of ["timeline", "custom"])
        expect(invoke).toHaveBeenCalledWith("get_field_definitions", {
          projectId: project.id,
          entityType,
        });
    });
  });

  it.each(["timelines", "custom"])("loads %s fields when editing a reference", async (type) => {
    const option = REFERENCE_TYPE_OPTIONS.find((option) => option.id === type)!;
    render(ReferenceEditDialog, {
      projectId: project.id,
      referenceType: option,
      onClose: vi.fn(),
      onSave: vi.fn(),
    });
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_field_definitions", {
        projectId: project.id,
        entityType: type === "timelines" ? "timeline" : "custom",
      })
    );
  });

  it("shows linked timeline and custom notes in the scene panel", async () => {
    currentProject.setCurrentChapter(mockChapters[0]);
    currentProject.setCurrentScene({
      ...mockScenes[0],
      planning_status: "fixed",
      editor_mode: "beat",
    });
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      const type = (args as { referenceType?: string } | undefined)?.referenceType;
      if (cmd === "get_scene_reference_items" && ["timelines", "custom"].includes(type ?? ""))
        return [{ id: type, name: `Linked ${type}` }];
      return [];
    });
    render(ScenePanel);
    await screen.findByText("Linked timelines");
    expect(screen.getByText("Linked custom")).toBeTruthy();
  });
});

describe("novelWriter home screen import", () => {
  it("picks a project folder, opens the import and notifies reference classification", async () => {
    currentProject.setProject(null);
    ui.setView("start");
    vi.mocked(open).mockResolvedValue("/source");
    vi.mocked(invoke).mockResolvedValue(project);
    const onImportComplete = vi.fn();
    render(StartScreen, { recentProjects: [], onImportComplete });

    await fireEvent.click(screen.getByRole("button", { name: "novelWriter Project folder" }));

    await waitFor(() => expect(onImportComplete).toHaveBeenCalledWith(project, "novelwriter"));
    expect(open).toHaveBeenCalledWith({ multiple: false, directory: true });
    expect(invoke).toHaveBeenCalledWith("import_novelwriter", { path: "/source" });
    expect(currentProject.value?.id).toBe(project.id);
    expect(ui.currentView).toBe("editor");
  });
});
