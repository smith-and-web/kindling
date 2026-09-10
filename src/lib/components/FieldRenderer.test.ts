import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import FieldRenderer from "./FieldRenderer.svelte";
import NewProjectDialog from "./NewProjectDialog.svelte";
import TagManager from "./TagManager.svelte";
import type { FieldDefinition, FieldType } from "../types";

vi.hoisted(() => {
  vi.stubGlobal("localStorage", { getItem: () => null, setItem: vi.fn(), removeItem: vi.fn() });
});
afterEach(cleanup);
const definition: FieldDefinition = {
  id: "field",
  project_id: "project",
  entity_type: "character",
  name: "Motivation",
  field_type: "text",
  options: '["First", "Second"]',
  default_value: null,
  position: 0,
  required: true,
  visible: true,
  created_at: "2026-09-10",
};

it.each(["text", "number", "date", "url", "select", "checkbox", "unknown"])(
  "associates the %s control with its field name and required state",
  (type) => {
    render(FieldRenderer, {
      definition: { ...definition, field_type: type as FieldType },
      value: null,
      onChange: vi.fn(),
    });
    const control = screen.getByLabelText(definition.name);
    expect(control.id).toBeTruthy();
    expect(control.getAttribute("aria-required")).toBe("true");
    expect(document.querySelector("label")?.htmlFor).toBe(control.id);
  }
);

it("assigns unique labels when the same definition appears twice", () => {
  for (let i = 0; i < 2; i++) render(FieldRenderer, { definition, value: null, onChange: vi.fn() });
  const controls = screen.getAllByLabelText(definition.name);
  expect(controls).toHaveLength(2);
  expect(controls[0].id).not.toBe(controls[1].id);
});

it.each(["multiselect", "multi_select"])(
  "names the %s group and describes its required constraint",
  async (type) => {
    const onChange = vi.fn();
    render(FieldRenderer, {
      definition: { ...definition, field_type: type as FieldType },
      value: null,
      onChange,
    });
    const group = screen.getByRole("group", { name: definition.name });
    expect(document.getElementById(group.getAttribute("aria-describedby")!)?.textContent).toContain(
      "required"
    );
    await fireEvent.click(within(group).getByRole("checkbox", { name: "First" }));
    expect(onChange).toHaveBeenCalledWith('["First"]');
    expect(within(group).getByRole("checkbox", { name: "Second" }).hasAttribute("required")).toBe(
      false
    );
  }
);

it("preserves optional and disabled control semantics", async () => {
  const onChange = vi.fn();
  render(FieldRenderer, {
    definition: { ...definition, required: false },
    value: "Text",
    disabled: true,
    onChange,
  });
  const control = screen.getByLabelText(definition.name) as HTMLInputElement;
  expect(control.disabled).toBe(true);
  expect(control.getAttribute("aria-required")).toBe("false");
  expect(control.value).toBe("Text");
});

it("names the project type and template groups", () => {
  render(NewProjectDialog, { onClose: vi.fn() });
  expect(
    within(screen.getByRole("group", { name: "Project type" })).getByRole("button", {
      name: "Novel",
    })
  ).toBeTruthy();
  expect(
    within(screen.getByRole("group", { name: "Structure template" })).getByRole("button", {
      name: "Browse templates...",
    })
  ).toBeTruthy();
});

it("names the tag color group", async () => {
  vi.mocked(invoke).mockResolvedValue([]);
  render(TagManager, { projectId: "project" });
  await fireEvent.click(await screen.findByRole("button", { name: /New tag/i }));
  expect(
    within(screen.getByRole("group", { name: "Color" })).getByRole("button", { name: "No color" })
  ).toBeTruthy();
});
