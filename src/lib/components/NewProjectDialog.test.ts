import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import NewProjectDialog from "./NewProjectDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { mockProject } from "../../dev/mock-data";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(mockProject);
});

afterEach(() => {
  cleanup();
  currentProject.setProject(null);
});

describe("NewProjectDialog", () => {
  it("sends the selected screenplay length as the camelCase targetLength argument (#300)", async () => {
    render(NewProjectDialog, { props: { onClose: vi.fn() } });

    await fireEvent.click(screen.getByLabelText("Screenplay"));
    await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Short film" } });
    await fireEvent.change(screen.getByLabelText("Target length"), {
      target: { value: "short" },
    });
    await fireEvent.click(screen.getByRole("button", { name: /^create/i }));

    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("create_screenplay_project", {
        name: "Short film",
        targetLength: "short",
      })
    );
  });
});
