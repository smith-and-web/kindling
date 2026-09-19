import { invoke } from "./mock-tauri";
import { describe, it, expect } from "vitest";
import type {
  Project,
  ReferenceCopyPreview,
  ReferenceCopyResult,
  ReferenceItem,
  ReferenceCopyRequest,
} from "../lib/types";

describe("reference copy development backend", () => {
  it("copies between distinct projects and rejects replays", async () => {
    const source = await invoke<Project>("create_blank_project", { name: "One" });
    const dest = await invoke<Project>("create_blank_project", { name: "Two" });
    const id = await invoke<string>("create_reference", {
      projectId: source.id,
      referenceType: "characters",
      reference: { name: "Mara", attributes: { notes: "Captain" } },
    });
    const request: ReferenceCopyRequest = {
      source_project_id: source.id,
      destination_project_id: dest.id,
      selection: null,
      keep_both: [],
    };
    const preview = await invoke<ReferenceCopyPreview>("preview_reference_copy", { request });
    expect(preview.copied).toBe(1);
    request.selection = [{ id, reference_type: "characters" }];
    const result = await invoke<ReferenceCopyResult>("copy_references_between_projects", {
      request,
      expectedRevision: preview.revision,
    });
    expect(result.copied).toBe(1);
    const copies = await invoke<ReferenceItem[]>("get_references", {
      projectId: dest.id,
      referenceType: "characters",
    });
    expect(copies[0].id).not.toBe(id);
    expect(copies[0].attributes.notes).toBe("Captain");
    await expect(
      invoke("copy_references_between_projects", { request, expectedRevision: preview.revision })
    ).rejects.toMatchObject({ code: "stale_preview" });
    const repeated = await invoke<ReferenceCopyPreview>("preview_reference_copy", { request });
    expect(repeated.copied).toBe(0);
    expect(repeated.skipped).toBe(1);
    request.keep_both = request.selection;
    const keep = await invoke<ReferenceCopyPreview>("preview_reference_copy", { request });
    expect(keep.references[0].destination_name).toBe("Mara (copy)");
  });
});

it("copies same-name source rows, preserves plural tags, and ignores project metadata saves", async () => {
  const source = await invoke<Project>("create_blank_project", { name: "Review source" });
  const dest = await invoke<Project>("create_blank_project", { name: "Review destination" });
  const ids = [];
  for (let i = 0; i < 2; i++)
    ids.push(
      await invoke<string>("create_reference", {
        projectId: source.id,
        referenceType: "items",
        reference: { name: "Compass" },
      })
    );
  const tag = await invoke<{ id: string }>("create_tag", { projectId: source.id, name: "Journey" });
  await invoke("tag_entity", { tagId: tag.id, entityType: "items", entityId: ids[0] });
  const request: ReferenceCopyRequest = {
    source_project_id: source.id,
    destination_project_id: dest.id,
    selection: ids.map((id) => ({ id, reference_type: "items" })),
    keep_both: [],
  };
  const preview = await invoke<ReferenceCopyPreview>("preview_reference_copy", { request });
  expect(preview.copied).toBe(2);
  expect(preview.skipped).toBe(0);
  expect(preview.references.every((r) => !r.conflict)).toBe(true);
  expect(preview.references.map((r) => r.destination_name).sort()).toEqual([
    "Compass",
    "Compass (copy)",
  ]);
  await invoke("update_project_settings", {
    projectId: dest.id,
    settings: { description: "Changed while reviewing" },
  });
  expect((await invoke<ReferenceCopyPreview>("preview_reference_copy", { request })).revision).toBe(
    preview.revision
  );
  const result = await invoke<ReferenceCopyResult>("copy_references_between_projects", {
    request,
    expectedRevision: preview.revision,
  });
  expect(result.copied).toBe(2);
  expect(result.project.description).toBe("Changed while reviewing");
  const copied = await invoke<ReferenceItem[]>("get_references", {
    projectId: dest.id,
    referenceType: "items",
  });
  const assigned = await Promise.all(
    copied.map((r) => invoke<unknown[]>("get_entity_tags", { entityType: "items", entityId: r.id }))
  );
  expect(assigned.flat()).toHaveLength(1);
});
