/// <reference lib="es2022.intl" />
import { invoke } from "@tauri-apps/api/core";
import type { Beat, Chapter, Scene } from "../types";
import { proseSaves } from "./proseSaves";
import { synopsisSaves } from "../stores/synopsisSaves.svelte";

/** Keep only the closing three sentences, with HTML decoded into inert text. */
export function closingProse(html: string): string {
  // Parse in an inert template: imported images must not trigger requests just
  // because a writer opens the next scene. Never mount or return this markup.
  const template = document.createElement("template");
  template.innerHTML = html;
  const root = template.content;
  for (const node of root.querySelectorAll("script, style")) node.remove();
  for (const node of root.querySelectorAll(
    "p, div, blockquote, h1, h2, h3, h4, h5, h6, li, br, hr, pre"
  )) {
    node.before(" ");
    node.after(" ");
  }
  const text = (root.textContent ?? "").replace(/\s+/gu, " ").trim();
  if (!text) return "";
  const sentences =
    typeof Intl.Segmenter === "function"
      ? Array.from(
          new Intl.Segmenter(undefined, { granularity: "sentence" }).segment(text),
          (part) => part.segment
        )
      : (text.match(/[^.!?。！？]+(?:[.!?。！？]+["'”’)]*|$)\s*/gu) ?? [text]);
  return sentences.slice(-3).join("").trim();
}

/** Walk manuscript order backwards, including across empty chapters. */
export async function loadPreviousScene(projectId: string, scene: Scene, chapters: Chapter[]) {
  const synopsisDrafts = synopsisSaves.snapshot().filter((draft) => draft.projectId === projectId);
  const drafts = new Map(
    proseSaves.draftsForRecovery(projectId).map((draft) => [draft.id, draft.prose])
  );
  const ordered = chapters
    .filter((chapter) => !chapter.archived)
    .sort((a, b) => a.position - b.position);
  const chapterIndex = ordered.findIndex((chapter) => chapter.id === scene.chapter_id);
  for (let index = chapterIndex; index >= 0; index--) {
    const scenes = (await invoke<Scene[]>("get_scenes", { chapterId: ordered[index].id }))
      .filter((item) => !item.archived)
      .sort((a, b) => a.position - b.position);
    const sceneIndex =
      index === chapterIndex ? scenes.findIndex((item) => item.id === scene.id) : scenes.length;
    // A deleted/moved selection must not accidentally point at another chapter's ending.
    if (index === chapterIndex && sceneIndex < 0) return null;
    const previous = scenes[sceneIndex - 1];
    if (!previous) continue;
    // Navigation submits editor saves before this read. Capture drafts on both sides
    // of the IPC read so an in-flight save cannot make the excerpt stale.
    let documents: { id: string; prose: string | null }[] = [previous];
    if (previous.editor_mode !== "page") {
      const beats = await invoke<Beat[]>("get_beats", { sceneId: previous.id });
      if (beats.length) documents = [...beats].sort((a, b) => a.position - b.position);
    }
    for (const draft of proseSaves.draftsForRecovery(projectId)) drafts.set(draft.id, draft.prose);
    const html = documents.map((doc) => drafts.get(doc.id) ?? doc.prose ?? "").join("\n");
    // Retain a draft even if its save completed during the read. The old chapter
    // may no longer be in the project store when the save queue publishes it.
    const synopsisDraft =
      synopsisSaves.getState(projectId, previous.id).draft ??
      synopsisDrafts.find((draft) => draft.sceneId === previous.id);
    return {
      scene: synopsisDraft ? { ...previous, synopsis: synopsisDraft.synopsis } : previous,
      excerpt: closingProse(html),
    };
  }
  return null;
}
