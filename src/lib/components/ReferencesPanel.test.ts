import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
import type { FieldDefinition, FieldValue, Project, ReferenceItem, Scene } from "../types";
import ReferencesPanel from "./ReferencesPanel.svelte";
import ReferenceEditDialog from "./ReferenceEditDialog.svelte";
import FieldRenderer from "./FieldRenderer.svelte";
import Onboarding from "./Onboarding.svelte";
import sampleSource from "../../../src-tauri/src/commands/sample_project.rs?raw";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    clear: () => values.clear(),
  });
});

const project: Project = {
  id: "project",
  name: "The Letter",
  source_type: "Blank",
  source_path: null,
  created_at: "",
  modified_at: "",
  author_pen_name: "E. M. Hale",
  genre: "Gothic Mystery",
  description: null,
  word_target: 90000,
  reference_types: ["characters"],
  project_type: "novel",
  target_page_count: null,
};
const reference: ReferenceItem = {
  id: "eleanor",
  project_id: project.id,
  name: "Eleanor Blackwood",
  reference_type: "characters",
  description: null,
  attributes: { Role: "Legacy role", Age: "17", Keepsake: "Pocket knife" },
  source_id: null,
};
const definitions: FieldDefinition[] = [
  {
    id: "role",
    project_id: project.id,
    entity_type: "character",
    name: "Role",
    field_type: "text",
    options: null,
    default_value: null,
    position: 0,
    required: false,
    visible: true,
    created_at: "",
  },
  {
    id: "age",
    project_id: project.id,
    entity_type: "character",
    name: "Age",
    field_type: "number",
    options: null,
    default_value: null,
    position: 1,
    required: false,
    visible: true,
    created_at: "",
  },
];
const values: FieldValue[] = [
  { id: "v1", field_definition_id: "role", entity_id: reference.id, value: "protagonist" },
  { id: "v2", field_definition_id: "age", entity_id: reference.id, value: "18" },
];
function mockFields(defs = definitions, fieldValues = values) {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_references") return [reference];
    if (command === "get_field_definitions") return defs;
    if (command === "get_field_values_bulk" || command === "get_field_values") return fieldValues;
    return [];
  });
}
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(project);
  currentProject.setCurrentScene(null);
  ui.referencesPanelCollapsed = false;
  mockFields();
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  ui.referencesPanelCollapsed = false;
});

async function expandReference() {
  await fireEvent.click(await screen.findByRole("button", { name: reference.name }));
}

describe("reference fixture rendering", () => {
  it("renders overlapping labels once, preferring typed values and preserving other legacy values", async () => {
    render(ReferencesPanel);
    await expandReference();
    await screen.findByText("protagonist");
    expect(screen.getAllByText("Role:")).toHaveLength(1);
    expect(screen.getAllByText("Age:")).toHaveLength(1);
    expect(screen.queryByText("Legacy role")).toBeNull();
    expect(screen.getByText("18")).toBeTruthy();
    expect(screen.getByText("Keepsake:")).toBeTruthy();
  });
  it("preserves legacy-only projects without field definitions", async () => {
    mockFields([], []);
    render(ReferencesPanel);
    await expandReference();
    expect(await screen.findByText("Legacy role")).toBeTruthy();
    expect(screen.getAllByText("Role:")).toHaveLength(1);
    expect(screen.getByText("17")).toBeTruthy();
  });
  it("keeps a legacy value when the matching typed field is empty", async () => {
    mockFields(definitions, []);
    render(ReferencesPanel);
    await expandReference();
    expect(await screen.findByText("Legacy role")).toBeTruthy();
    expect(screen.getAllByText("Role:")).toHaveLength(1);
  });
  it("returns the collapsed panel's minimum width and restores its pixel width on expansion", async () => {
    const { container } = render(ReferencesPanel);
    const aside = container.querySelector("aside")!;
    expect(aside.style.width).toBe(`${ui.referencesPanelWidth}px`);
    expect(aside.classList.contains("min-w-0")).toBe(false);
    ui.referencesPanelCollapsed = true;
    await tick();
    expect(aside.classList.contains("w-0")).toBe(true);
    expect(aside.classList.contains("min-w-0")).toBe(true);
    expect(aside.getAttribute("style") ?? "").not.toContain("width:");
    ui.referencesPanelCollapsed = false;
    await tick();
    expect(aside.style.width).toBe(`${ui.referencesPanelWidth}px`);
    expect(aside.classList.contains("min-w-0")).toBe(false);
  });
  it("keeps typed and legacy values editable in distinctly labelled sections", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    render(ReferenceEditDialog, {
      referenceType: REFERENCE_TYPE_OPTIONS.find((t) => t.id === "characters")!,
      reference,
      projectId: project.id,
      onSave,
      onClose: vi.fn(),
    });
    const typedHeading = await screen.findByText("Custom Fields");
    const legacyHeading = screen.getByText("Legacy Attributes");
    expect(typedHeading.parentElement?.querySelector("input")?.value).toBe("protagonist");
    expect(legacyHeading.parentElement?.parentElement?.querySelector("input")?.value).toBe("Role");
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(onSave).toHaveBeenCalledWith(
        expect.objectContaining({
          attributes: reference.attributes,
          fieldValues: { role: "protagonist", age: "18" },
        })
      )
    );
  });
  it.each(["multi_select", "multiselect"] as const)(
    "edits %s Traits as a multi-select",
    async (field_type) => {
      const onChange = vi.fn();
      render(FieldRenderer, {
        definition: {
          ...definitions[0],
          name: "Traits",
          field_type,
          options: '["observant","loyal"]',
        },
        value: '["observant"]',
        onChange,
      });
      await fireEvent.click(screen.getByText("loyal"));
      expect(onChange).toHaveBeenCalledWith('["observant","loyal"]');
    }
  );
  it("aligns the onboarding sidebar with a seeded chapter", () => {
    ui.startOnboarding();
    ui.goToStep("tour-sidebar");
    render(Onboarding);
    expect(screen.queryByText("Chapter 1: The Beginning")).toBeNull();
    expect(screen.getByText("The Letter")).toBeTruthy();
    expect(sampleSource).toMatch(/Chapter::new\(project.id, "The Letter"/);
  });
});

it("copies into a project with no enabled categories while preserving the current scene", async () => {
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
  const empty = { ...project, reference_types: [] };
  currentProject.setProject(empty);
  const scene = { id: "scene-copy", chapter_id: "chapter-copy", title: "Scene" } as Scene;
  currentProject.setCurrentScene(scene);
  const source = { ...project, id: "source", name: "Book One" };
  const row = {
    id: "source-character",
    reference_type: "characters",
    name: reference.name,
    description: null,
    selected: true,
    conflict: false,
    action: "copy",
    destination_name: reference.name,
  };
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_all_projects") return [empty, source];
    if (command === "preview_reference_copy")
      return {
        references: [row],
        changes: [],
        enabled_types: ["characters"],
        copied: 1,
        skipped: 0,
        revision: "preview",
      };
    if (command === "copy_references_between_projects")
      return { project, created_reference_ids: [reference.id], copied: 1, skipped: 0 };
    if (command === "get_references") return [reference];
    return [];
  });
  render(ReferencesPanel);
  const button = screen.getByRole("button", { name: "Copy references from project…" });
  expect((button as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(button);
  await screen.findByRole("option", { name: /Book One/ });
  await fireEvent.change(screen.getByLabelText("Source project"), { target: { value: source.id } });
  await waitFor(() =>
    expect(
      (screen.getByRole("button", { name: "Copy 1 references" }) as HTMLButtonElement).disabled
    ).toBe(false)
  );
  await fireEvent.click(screen.getByRole("button", { name: "Copy 1 references" }));
  await waitFor(() => expect(currentProject.value?.reference_types).toEqual(["characters"]));
  await fireEvent.click(await screen.findByRole("button", { name: "Done" }));
  await screen.findByRole("button", { name: reference.name });
  expect(currentProject.currentScene?.id).toBe(scene.id);
  expect(currentProject.characters[0].id).toBe(reference.id);
});

it.each([
  ["items", "item"],
  ["objectives", "objective"],
  ["organizations", "organization"],
  ["timelines", "timeline"],
] as const)("writes and reloads %s tags using the same entity type", async (type, entityType) => {
  currentProject.setProject({ ...project, reference_types: [type] });
  const tag = {
    id: "tag",
    project_id: project.id,
    name: "Important",
    color: null,
    parent_id: null,
    position: 0,
    created_at: "",
  };
  let assigned = false;
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    const input = args as Record<string, unknown>;
    if (cmd === "get_references") return [{ ...reference, reference_type: type }];
    if (cmd === "get_tags") return [tag];
    if (cmd === "tag_entity" && input.entityType === entityType) assigned = true;
    if (cmd === "untag_entity" && input.entityType === entityType) assigned = false;
    if (cmd === "get_entity_tags") return assigned && input.entityType === entityType ? [tag] : [];
    return [];
  });
  render(ReferencesPanel);
  await expandReference();
  await fireEvent.click(screen.getByRole("button", { name: "Add tag" }));
  await fireEvent.click(screen.getByRole("button", { name: "Important" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("tag_entity", {
      tagId: tag.id,
      entityId: reference.id,
      entityType,
    })
  );
  await fireEvent.click(await screen.findByRole("button", { name: "Remove tag Important" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("untag_entity", {
      tagId: tag.id,
      entityId: reference.id,
      entityType,
    })
  );
});
