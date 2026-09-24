import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { mockProject } from "../../dev/mock-data";
import { currentProject } from "../stores/project.svelte";
import ClassicExportDialog from "./ClassicExportDialog.svelte";

const refusal =
  "“Novel.scriv” already exists, so it was left untouched. Choose a new name to create a " +
  "Scrivener project, or choose Update Existing to write your prose into that one.";

beforeEach(() => {
  vi.stubGlobal("localStorage", {
    getItem: () => null,
    setItem: vi.fn(),
    removeItem: vi.fn(),
  });
  currentProject.setProject(mockProject);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "export_to_scrivener") throw refusal;
    return 0;
  });
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  vi.unstubAllGlobals();
});

it("shows the refusal in the dialog when Create New targets an existing .scriv", async () => {
  const onSuccess = vi.fn();
  render(ClassicExportDialog, {
    scope: "project",
    scopeId: null,
    scopeTitle: mockProject.name,
    onClose: vi.fn(),
    onSuccess,
    onCustomize: vi.fn(),
  });
  await fireEvent.click(screen.getByTestId("export-format-scrivener"));
  vi.mocked(save).mockResolvedValueOnce("/Users/writer/Novel.scriv");
  await fireEvent.click(screen.getByRole("button", { name: "Choose save location" }));
  await waitFor(() =>
    expect((screen.getByLabelText("Save Location") as HTMLInputElement).value).toBe(
      "/Users/writer/Novel.scriv"
    )
  );
  await fireEvent.click(screen.getByTestId("export-confirm"));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain(refusal);
  expect(invoke).toHaveBeenCalledWith(
    "export_to_scrivener",
    expect.objectContaining({
      options: expect.objectContaining({
        mode: "create_new",
        output_path: "/Users/writer/Novel.scriv",
      }),
    })
  );
  expect(onSuccess).not.toHaveBeenCalled();
});
