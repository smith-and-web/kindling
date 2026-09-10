import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import SettingsDialog from "./SettingsDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockProject, mockAppSettings, mockScenes } from "../../dev/mock-data";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});

const second = {
  ...mockProject,
  id: "second-project",
  name: "Second manuscript",
  project_type: "screenplay" as const,
  target_page_count: 120,
  genre: "Mystery",
};
let projects = [mockProject, second];

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  projects = structuredClone([mockProject, second]);
  currentProject.setProject(mockProject);
  currentProject.setCurrentScene(mockScenes[0]);
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const data = args as { projectId?: string; settings?: Record<string, unknown> };
    if (command === "get_all_projects") return projects;
    if (command === "get_app_settings") return { ...mockAppSettings };
    if (command === "get_writing_stats")
      return { daily_goal: data.projectId === second.id ? 900 : 500 };
    if (command === "update_app_settings") return data.settings;
    if (command === "update_project_settings") {
      const updated = { ...projects.find((p) => p.id === data.projectId)!, ...data.settings };
      projects = projects.map((p) => (p.id === updated.id ? updated : p));
      return updated;
    }
    return [];
  });
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  vi.restoreAllMocks();
});

async function openProjects() {
  await fireEvent.click(screen.getByRole("button", { name: "Project Details" }));
  await screen.findByRole("combobox", { name: "Project" });
}
async function choose(id: string) {
  await fireEvent.change(screen.getByRole("combobox", { name: "Project" }), {
    target: { value: id },
  });
}

it("edits another project's metadata and goal without changing the editor, and reloads saved values", async () => {
  render(SettingsDialog, { onClose: vi.fn() });
  await openProjects();
  await choose(second.id);
  await waitFor(() =>
    expect((screen.getByLabelText("Daily writing goal") as HTMLInputElement).value).toBe("900")
  );
  await fireEvent.input(screen.getByLabelText(/Genre/), { target: { value: "Fantasy" } });
  await fireEvent.input(screen.getByLabelText(/Word Target/), { target: { value: "80000" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save project changes" }));
  await screen.findByText("Project changes saved.");
  expect(invoke).toHaveBeenCalledWith("update_project_settings", {
    projectId: second.id,
    settings: expect.objectContaining({
      genre: "Fantasy",
      word_target: 80000,
      daily_writing_goal: 900,
      target_page_count: 120,
    }),
  });
  expect(currentProject.value?.id).toBe(mockProject.id);
  expect(currentProject.currentScene?.id).toBe(mockScenes[0].id);
  await choose(mockProject.id);
  await choose(second.id);
  expect((screen.getByLabelText(/Genre/) as HTMLInputElement).value).toBe("Fantasy");
});

it("preserves drafts across sections and guards switching and closing", async () => {
  const onClose = vi.fn();
  render(SettingsDialog, { onClose });
  await openProjects();
  await fireEvent.input(screen.getByLabelText(/Genre/), { target: { value: "Unsaved" } });
  await fireEvent.click(screen.getByRole("button", { name: "Appearance & Guidance" }));
  await openProjects();
  expect((screen.getByLabelText(/Genre/) as HTMLInputElement).value).toBe("Unsaved");
  await choose(second.id);
  await screen.findByText(/Discard unsaved changes for this project/);
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe(mockProject.id);
  await fireEvent.click(screen.getByRole("button", { name: "Close settings" }));
  expect(onClose).not.toHaveBeenCalled();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  await choose(second.id);
  await fireEvent.click(screen.getByRole("button", { name: "Discard changes" }));
  expect((screen.getByLabelText(/Genre/) as HTMLInputElement).value).toBe("Mystery");
  expect(invoke).not.toHaveBeenCalledWith("update_project_settings", expect.anything());
});

it("saves reference types for the selected project and refreshes only the open project", async () => {
  render(SettingsDialog, { onClose: vi.fn() });
  await openProjects();
  await fireEvent.click(screen.getByRole("button", { name: "Reference Types" }));
  const checkbox = screen.getByRole("checkbox", { name: "Characters" }) as HTMLInputElement;
  const wasChecked = checkbox.checked;
  await fireEvent.click(checkbox);
  await fireEvent.click(screen.getByRole("button", { name: "Save project changes" }));
  await screen.findByText("Project changes saved.");
  expect(currentProject.value?.reference_types.includes("characters")).toBe(!wasChecked);
  expect(currentProject.currentScene?.id).toBe(mockScenes[0].id);
});

it("retains failed saves and prevents navigation or closing during a save", async () => {
  const original = vi.mocked(invoke).getMockImplementation()!;
  let rejectSave: (error: Error) => void = () => {};
  vi.mocked(invoke).mockImplementation((command, args) =>
    command === "update_project_settings"
      ? new Promise((_, reject) => {
          rejectSave = reject;
        })
      : original(command, args)
  );
  const onClose = vi.fn();
  render(SettingsDialog, { onClose });
  await openProjects();
  await fireEvent.input(screen.getByLabelText(/Genre/), { target: { value: "Keep me" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save project changes" }));
  expect((screen.getByRole("combobox") as HTMLSelectElement).disabled).toBe(true);
  expect(
    (screen.getByRole("button", { name: "Close settings" }) as HTMLButtonElement).disabled
  ).toBe(true);
  rejectSave(new Error("disk full"));
  await screen.findByRole("alert");
  expect(screen.getByText("disk full")).toBeTruthy();
  expect((screen.getByLabelText(/Genre/) as HTMLInputElement).value).toBe("Keep me");
  expect(onClose).not.toHaveBeenCalled();
});

it("keeps app settings available without projects and retries a failed project list", async () => {
  currentProject.setProject(null);
  const original = vi.mocked(invoke).getMockImplementation()!;
  let failed = true;
  vi.mocked(invoke).mockImplementation((command, args) =>
    command === "get_all_projects"
      ? failed
        ? Promise.reject(new Error("unavailable"))
        : Promise.resolve([])
      : original(command, args)
  );
  render(SettingsDialog, { onClose: vi.fn() });
  expect(screen.getByRole("radio", { name: "Light" })).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Project Details" }));
  await screen.findByText(/Could not load projects/);
  failed = false;
  await fireEvent.click(screen.getByRole("button", { name: "Retry loading projects" }));
  await screen.findByText(/No projects yet/);
  await fireEvent.click(screen.getByRole("button", { name: "Author & Contact" }));
  expect(screen.getByLabelText("Author Name")).toBeTruthy();
});

it("does not expose blank author fields after a load failure and retries before saving", async () => {
  const original = vi.mocked(invoke).getMockImplementation()!;
  let failed = true;
  vi.mocked(invoke).mockImplementation((command, args) =>
    command === "get_app_settings" && failed
      ? Promise.reject(new Error("contact file unavailable"))
      : original(command, args)
  );
  render(SettingsDialog, { onClose: vi.fn() });
  await fireEvent.click(screen.getByRole("button", { name: "Author & Contact" }));
  await screen.findByText("contact file unavailable");
  expect(screen.queryByRole("button", { name: "Save author details" })).toBeNull();
  failed = false;
  await fireEvent.click(screen.getByRole("button", { name: "Retry loading author details" }));
  await screen.findByLabelText("Author Name");
  await fireEvent.input(screen.getByLabelText("Author Name"), { target: { value: "New Author" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save author details" }));
  await screen.findByText("Author details saved.");
  expect(invoke).toHaveBeenCalledWith("update_app_settings", {
    settings: expect.objectContaining({ author_name: "New Author" }),
  });
});

it("preserves tag drafts across areas and guards switching to another project", async () => {
  render(SettingsDialog, { onClose: vi.fn() });
  await openProjects();
  await fireEvent.click(screen.getByRole("button", { name: "Tags" }));
  await fireEvent.click(await screen.findByRole("button", { name: "New tag" }));
  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Draft tag" } });
  await openProjects();
  await fireEvent.click(screen.getByRole("button", { name: "Tags" }));
  expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe("Draft tag");
  await choose(second.id);
  await screen.findByText(/Discard unsaved changes for this project/);
});

it("protects custom field drafts when their reference type is disabled", async () => {
  render(SettingsDialog, { onClose: vi.fn() });
  await openProjects();
  await fireEvent.click(screen.getByRole("button", { name: "Custom Fields" }));
  const buttons = await screen.findAllByRole("button", { name: "Add field" });
  await fireEvent.click(buttons[0]);
  await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Draft field" } });
  await fireEvent.click(screen.getByRole("button", { name: "Reference Types" }));
  await fireEvent.click(screen.getByRole("checkbox", { name: "Characters" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save project changes" }));
  await screen.findByText(/Save or cancel the Characters custom field draft/);
  expect(invoke).not.toHaveBeenCalledWith("update_project_settings", expect.anything());
  await fireEvent.click(screen.getByRole("button", { name: "Custom Fields" }));
  expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe("Draft field");
  await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await fireEvent.click(screen.getByRole("button", { name: "Reference Types" }));
  await fireEvent.click(screen.getByRole("button", { name: "Save project changes" }));
  await screen.findByText("Project changes saved.");
});

it("groups navigation by scope and keeps the project selector in the sidebar across areas", async () => {
  render(SettingsDialog, { onClose: vi.fn() });
  const nav = screen.getByRole("navigation", { name: "Settings areas" });
  const shared = within(nav).getByRole("region", { name: "Kindling" });
  const projectGroup = within(nav).getByRole("region", { name: "Projects" });
  expect(within(shared).getByRole("heading", { name: "Preferences" })).toBeTruthy();
  expect(within(projectGroup).getByRole("heading", { name: "Manuscript" })).toBeTruthy();
  expect(within(projectGroup).getByRole("heading", { name: "Reference Library" })).toBeTruthy();
  const selector = await within(projectGroup).findByRole("combobox", { name: "Project" });
  expect(screen.getAllByRole("combobox", { name: "Project" })).toHaveLength(1);
  expect(screen.getByTestId("settings-location").textContent).toContain("Kindling / Preferences");
  await fireEvent.change(selector, { target: { value: second.id } });
  expect(screen.getByRole("button", { name: "Project Details" }).getAttribute("aria-current")).toBe(
    "page"
  );
  expect(screen.getByTestId("settings-location").textContent?.trim()).toBe(
    "Projects / Second manuscript / Manuscript"
  );
  await fireEvent.click(screen.getByRole("button", { name: "Custom Fields" }));
  expect(screen.getByTestId("settings-location").textContent).toContain("Reference Library");
  expect(within(projectGroup).getByRole("combobox", { name: "Project" })).toBe(selector);
  expect(currentProject.value?.id).toBe(mockProject.id);
});
