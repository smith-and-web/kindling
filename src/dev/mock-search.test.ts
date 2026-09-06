import { beforeEach, expect, it, vi } from "vitest";
import type { Beat, Chapter, Scene, Project } from "../lib/types";
import type { ProseDocument } from "../lib/utils/proseSearch";
let invoke: typeof import("./mock-tauri").invoke;
beforeEach(async () => {
  vi.resetModules();
  ({ invoke } = await import("./mock-tauri"));
});
async function fixture() {
  const project = (await invoke<Project[]>("get_all_projects"))[0];
  const chapter = await invoke<Chapter>("create_chapter", {
    projectId: project.id,
    title: "Mock search chapter",
  });
  const beatScene = await invoke<Scene>("create_scene", { chapterId: chapter.id });
  const beat = await invoke<Beat>("create_beat", { sceneId: beatScene.id });
  await invoke("save_beat_prose", { beatId: beat.id, prose: "<p>Alice</p>" });
  const page = await invoke<Scene>("create_scene", { chapterId: chapter.id });
  await invoke("switch_scene_editor_mode", { sceneId: page.id, mode: "page" });
  await invoke("save_scene_page_prose", { sceneId: page.id, prose: "<p>Alice again</p>" });
  return { projectId: project.id, chapter, beatScene, beat, page };
}
it("searches and replaces active beat/page prose with optimistic undo", async () => {
  const { projectId, beat, page } = await fixture();
  const docs = await invoke<ProseDocument[]>("get_search_documents", { projectId });
  expect(docs.find((d) => d.id === beat.id)?.prose).toBe("<p>Alice</p>");
  expect(docs.find((d) => d.id === page.id)?.beat_title).toBeNull();
  const changes = [
    { id: beat.id, expected_prose: "<p>Alice</p>", prose: "<p>Bob</p>" },
    { id: page.id, expected_prose: "<p>Alice again</p>", prose: "<p>Bob again</p>" },
  ];
  await invoke("replace_prose_batch", { projectId, changes });
  const updated = await invoke<ProseDocument[]>("get_search_documents", { projectId });
  expect(updated.find((d) => d.id === beat.id)?.prose).toBe("<p>Bob</p>");
  await invoke("replace_prose_batch", {
    projectId,
    changes: changes.map((c) => ({ id: c.id, expected_prose: c.prose, prose: c.expected_prose })),
  });
  expect(await invoke("get_search_documents", { projectId })).toEqual(docs);
});
it.each(["flexible", "undefined"] as const)(
  "excludes %s prose and rejects changes after demotion",
  async (status) => {
    const { projectId, beatScene, beat, page } = await fixture();
    const changes = [
      { id: beat.id, expected_prose: "<p>Alice</p>", prose: "Changed" },
      { id: page.id, expected_prose: "<p>Alice again</p>", prose: "Changed" },
    ];
    // The dev mock returns its stored scene objects.
    page.planning_status = status;
    await expect(invoke("replace_prose_batch", { projectId, changes })).rejects.toThrow(
      "no longer available"
    );
    beatScene.planning_status = status;
    const hidden = await invoke<ProseDocument[]>("get_search_documents", { projectId });
    expect(hidden.some((doc) => doc.id === beat.id || doc.id === page.id)).toBe(false);
    beatScene.planning_status = "fixed";
    page.planning_status = "fixed";
    const restored = await invoke<ProseDocument[]>("get_search_documents", { projectId });
    expect(restored.find((doc) => doc.id === beat.id)?.prose).toBe("<p>Alice</p>");
    expect(restored.find((doc) => doc.id === page.id)?.prose).toBe("<p>Alice again</p>");
  }
);
it("rejects stale, duplicate, foreign, locked and archived targets without partial writes", async () => {
  const { projectId, beat, page } = await fixture();
  const change = { id: beat.id, expected_prose: "<p>Alice</p>", prose: "Changed" };
  for (const bad of [
    { id: page.id, expected_prose: "stale", prose: "Changed" },
    change,
    { ...change, id: "missing" },
  ]) {
    await expect(
      invoke("replace_prose_batch", { projectId, changes: [change, bad] })
    ).rejects.toThrow();
    expect(
      (await invoke<ProseDocument[]>("get_search_documents", { projectId })).find(
        (d) => d.id === beat.id
      )?.prose
    ).toBe("<p>Alice</p>");
  }
  await expect(
    invoke("replace_prose_batch", { projectId: "foreign", changes: [change] })
  ).rejects.toThrow();
  await invoke("lock_scene", { sceneId: page.id });
  const pageChange = { id: page.id, expected_prose: "<p>Alice again</p>", prose: "Changed" };
  await expect(
    invoke("replace_prose_batch", { projectId, changes: [change, pageChange] })
  ).rejects.toThrow("locked");
  await invoke("archive_scene", { sceneId: page.id });
  expect(
    (await invoke<ProseDocument[]>("get_search_documents", { projectId })).some(
      (d) => d.id === page.id
    )
  ).toBe(false);
  await expect(invoke("replace_prose_batch", { projectId, changes: [pageChange] })).rejects.toThrow(
    "no longer available"
  );
});
