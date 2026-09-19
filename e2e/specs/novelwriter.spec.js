/** Human-gated native smoke: run on a supported tauri-driver platform. */
import { mkdtempSync, readFileSync, writeFileSync, readdirSync, existsSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  waitForAppReady,
  importProject,
  skipOnboardingIfPresent,
  selectChapter,
  selectScene,
  expandBeat,
} from "./helpers.js";

async function invoke(command, args = {}) {
  const result = await browser.executeAsync(
    async (command, args, done) => {
      try {
        done({ value: await window.__KINDLING_TEST__.invoke(command, args) });
      } catch (e) {
        done({ error: String(e) });
      }
    },
    command,
    args
  );
  if (result.error) throw new Error(result.error);
  return result.value;
}

describe("novelWriter round trip and prose sync", () => {
  let folder;
  const createdProjects = [];
  before(async () => {
    await waitForAppReady();
    await skipOnboardingIfPresent();
  });
  after(async () => {
    for (const projectId of createdProjects) await invoke("delete_project", { projectId });
    if (folder) rmSync(folder, { recursive: true, force: true });
  });
  it("exports a sample, imports it, and applies an on-disk prose edit in the editor", async () => {
    folder = mkdtempSync(join(tmpdir(), "kindling-novelwriter-"));
    const original = await invoke("create_sample_project");
    createdProjects.push(original.id);
    const originalChapters = await invoke("get_chapters", { projectId: original.id });
    const originalScenes = (
      await Promise.all(originalChapters.map((c) => invoke("get_scenes", { chapterId: c.id })))
    ).flat();
    await invoke("export_to_novelwriter", {
      projectId: original.id,
      outputPath: folder,
      options: { include_beat_comments: true, include_notes: true, create_snapshot: false },
    });
    expect(existsSync(join(folder, "nwProject.nwx"))).toBe(true);
    expect(readdirSync(join(folder, "content")).every((name) => name.endsWith(".md"))).toBe(true);
    const imported = await importProject(folder, "novelwriter");
    const projectId = imported.id;
    createdProjects.push(projectId);
    const chapters = await invoke("get_chapters", { projectId });
    const scenes = (
      await Promise.all(chapters.map((c) => invoke("get_scenes", { chapterId: c.id })))
    ).flat();
    expect(chapters.length).toBe(originalChapters.length);
    expect(scenes.length).toBe(originalScenes.length);
    expect((await invoke("get_sync_preview", { projectId })).changes).toHaveLength(0);
    const scene = scenes[0];
    const chapter = chapters.find((c) => c.id === scene.chapter_id);
    const file = join(folder, "content", `${scene.source_id}.md`);
    const sentence = "The visitor returned with an unexpected letter.";
    writeFileSync(file, `${readFileSync(file, "utf8")}\n${sentence}\n`);
    await $('[data-testid="sync-button"]').click();
    const diff = await $('[data-testid="sync-prose-diff"]');
    await diff.waitForDisplayed();
    expect(await diff.getText()).toContain(sentence);
    await diff.$("..").$("..").$('input[type="checkbox"]').click();
    await $('[data-testid="sync-confirm"]').click();
    await $('[data-testid="sync-summary-dialog"]').waitForDisplayed();
    await $('[data-testid="dialog-close"]').click();
    await selectChapter(chapter.title);
    await selectScene(scene.title);
    const beats = await invoke("get_beats", { sceneId: scene.id });
    if (scene.editor_mode === "beat") await expandBeat(beats.length - 1);
    await browser.waitUntil(
      async () => (await $('[contenteditable="true"]').getText()).includes(sentence),
      {
        timeout: 5000,
        timeoutMsg: "Accepted prose did not appear in editor",
      }
    );
  });
});
