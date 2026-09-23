import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ReferenceEditDialog from "./ReferenceEditDialog.svelte";
import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
import type { FieldDefinition } from "../types";

const characters = REFERENCE_TYPE_OPTIONS[0];

function field(id: string, name: string, field_type: string): FieldDefinition {
  return {
    id,
    project_id: "p1",
    entity_type: "character",
    name,
    field_type,
    options: field_type === "multiselect" ? JSON.stringify(["Brave", "Wary"]) : null,
    default_value: null,
    position: 0,
    required: false,
    visible: true,
  } as FieldDefinition;
}

afterEach(() => {
  cleanup();
  vi.mocked(invoke).mockReset();
});

describe("ReferenceEditDialog", () => {
  it("keeps Save enabled and explains an empty name instead of disabling", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const onSave = vi.fn();
    render(ReferenceEditDialog, {
      referenceType: characters,
      projectId: "p1",
      onSave,
      onClose: vi.fn(),
    });

    const save = screen.getByTestId("reference-save") as HTMLButtonElement;
    expect(save.disabled).toBe(false);
    await fireEvent.click(save);

    const name = screen.getByLabelText("Name");
    expect(onSave).not.toHaveBeenCalled();
    expect(screen.getByText("Enter a name to continue.")).toBeTruthy();
    expect(name.getAttribute("aria-invalid")).toBe("true");
    expect(name.getAttribute("aria-describedby")).toBe("ref-name-error");
    expect(document.activeElement).toBe(name);

    await fireEvent.input(name, { target: { value: "Eleanor" } });
    expect(screen.queryByText("Enter a name to continue.")).toBeNull();
    expect(name.getAttribute("aria-invalid")).toBeNull();
  });

  it("saves a named reference and closes", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(ReferenceEditDialog, {
      referenceType: characters,
      projectId: "p1",
      onSave,
      onClose,
    });

    await fireEvent.input(screen.getByLabelText("Name"), { target: { value: " Eleanor " } });
    await fireEvent.click(screen.getByTestId("reference-save"));

    await vi.waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ name: "Eleanor" }));
  });

  it("lays custom fields out in a paired grid", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) =>
      cmd === "get_field_definitions"
        ? [field("f1", "Role", "text"), field("f2", "Age", "number")]
        : []
    );
    render(ReferenceEditDialog, {
      referenceType: characters,
      projectId: "p1",
      onSave: vi.fn(),
      onClose: vi.fn(),
    });

    const heading = await screen.findByRole("heading", { name: "Custom fields" });
    const grid = heading.parentElement?.querySelector(".reference-fields");
    expect(grid?.querySelectorAll(".field-renderer")).toHaveLength(2);
  });
});
