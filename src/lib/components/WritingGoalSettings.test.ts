import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ProjectSettingsDialog from "./ProjectSettingsDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockProject } from "../../dev/mock-data";

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_writing_stats") return { daily_goal: 500 };
    if (command === "update_project_settings") return mockProject;
    return [];
  });
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
});

it("saves the loaded daily goal with metadata in one command", async () => {
  const onSave = vi.fn();
  render(ProjectSettingsDialog, { onClose: vi.fn(), onSave });
  await waitFor(() =>
    expect((screen.getByLabelText("Daily writing goal") as HTMLInputElement).value).toBe("500")
  );
  await fireEvent.input(screen.getByLabelText("Daily writing goal"), { target: { value: "750" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(invoke).toHaveBeenCalledWith("update_project_settings", {
    projectId: mockProject.id,
    settings: expect.objectContaining({ daily_writing_goal: 750 }),
  });
  expect(invoke).not.toHaveBeenCalledWith("set_daily_writing_goal", expect.anything());
  expect(onSave).toHaveBeenCalledWith(mockProject);
});

it.each(["pending", "failed"])(
  "keeps metadata editable while the goal load is %s",
  async (state) => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_writing_stats") {
        if (state === "failed") throw new Error("stats unavailable");
        return new Promise(() => {});
      }
      if (command === "update_project_settings") return mockProject;
      return [];
    });
    const onSave = vi.fn();
    render(ProjectSettingsDialog, { onClose: vi.fn(), onSave });
    if (state === "failed") await screen.findByText(/Could not load daily goal/);
    expect((screen.getByLabelText("Daily writing goal") as HTMLInputElement).disabled).toBe(true);
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(onSave).toHaveBeenCalledWith(mockProject);
    const save = vi
      .mocked(invoke)
      .mock.calls.find(([command]) => command === "update_project_settings");
    expect(save?.[1]).toEqual({
      projectId: mockProject.id,
      settings: expect.not.objectContaining({ daily_writing_goal: expect.anything() }),
    });
  }
);

it("rejects a fractional goal after loading without saving metadata", async () => {
  render(ProjectSettingsDialog, { onClose: vi.fn(), onSave: vi.fn() });
  await waitFor(() =>
    expect((screen.getByLabelText("Daily writing goal") as HTMLInputElement).disabled).toBe(false)
  );
  await fireEvent.input(screen.getByLabelText("Daily writing goal"), { target: { value: "1.5" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(screen.getByText(/Daily goal must be a whole number/)).toBeTruthy();
  expect(invoke).not.toHaveBeenCalledWith("update_project_settings", expect.anything());
});
