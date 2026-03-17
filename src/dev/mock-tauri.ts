/**
 * Mock Tauri invoke() for browser-only dev (npm run dev without Tauri).
 * Provides an in-memory backend so Cursor can drive the full UI via browser tools.
 */

import type {
  Project,
  Chapter,
  Scene,
  Beat,
  Character,
  Location,
  ReferenceItem,
  AppSettings,
  SnapshotMetadata,
  SceneReferenceState,
  FieldDefinition,
  FieldValue,
  Tag,
  EntityTag,
  SavedFilter,
} from "../lib/types";

import {
  mockProject,
  mockChapters,
  mockScenes,
  mockBeats,
  mockCharacters,
  mockLocations,
  mockReferenceItems,
  mockAppSettings,
} from "./mock-data";

// Mutable in-memory store (cloned from seed so we can mutate)
let projects: Project[] = [{ ...mockProject }];
let chapters: Chapter[] = mockChapters.map((c) => ({ ...c }));
let scenes: Scene[] = mockScenes.map((s) => ({ ...s }));
let beats: Beat[] = mockBeats.map((b) => ({ ...b }));
let characters: Character[] = mockCharacters.map((c) => ({ ...c }));
let locations: Location[] = mockLocations.map((l) => ({ ...l }));
let referenceItems: ReferenceItem[] = mockReferenceItems.map((r) => ({ ...r }));
let appSettings: AppSettings = { ...mockAppSettings };
let sceneReferenceStates: SceneReferenceState[] = [];
let snapshots: SnapshotMetadata[] = [];
let fieldDefinitions: FieldDefinition[] = [];
let fieldValues: FieldValue[] = [];
let tags: Tag[] = [];
let entityTags: EntityTag[] = [];
let savedFilters: SavedFilter[] = [];

let idCounter = 100;

function nextId(prefix: string): string {
  return `${prefix}-${idCounter++}-${Math.random().toString(36).slice(2, 9)}`;
}

function getArg<T>(args: Record<string, unknown>, ...keys: string[]): T | undefined {
  for (const k of keys) {
    const v = args[k];
    if (v !== undefined && v !== null) return v as T;
  }
  return undefined;
}

export async function invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const projectId = getArg<string>(args, "projectId", "project_id");
  const chapterId = getArg<string>(args, "chapterId", "chapter_id");
  const sceneId = getArg<string>(args, "sceneId", "scene_id");
  const beatId = getArg<string>(args, "beatId", "beat_id");
  const referenceType = getArg<string>(args, "referenceType", "reference_type");
  const referenceId = getArg<string>(args, "referenceId", "reference_id");
  const snapshotId = getArg<string>(args, "snapshotId", "snapshot_id");

  switch (cmd) {
    // Import - return the seed project
    case "import_plottr":
    case "import_ywriter":
    case "import_markdown":
    case "import_longform": {
      const path = getArg<string>(args, "path") ?? "/mock/path/story.pltr";
      const project = projects[0]!;
      return { ...project, source_path: path, modified_at: new Date().toISOString() } as T;
    }

    case "preview_import": {
      void getArg<string>(args, "path");
      void getArg<string>(args, "format");
      return {
        project_name: "Sample Project",
        chapter_count: 3,
        scene_count: 8,
        beat_count: 24,
        character_count: 5,
        location_count: 2,
      } as T;
    }

    case "create_sample_project": {
      // Mock: return first project as "sample" (real Tauri creates full sample)
      const project = projects[0]!;
      return {
        ...project,
        name: "Sample Project",
        source_path: null,
        source_type: "Markdown",
        modified_at: new Date().toISOString(),
      } as T;
    }

    case "get_project": {
      const id = getArg<string>(args, "id") ?? projectId;
      const p = projects.find((x) => x.id === id);
      if (!p) throw new Error(`Project not found: ${id}`);
      return p as T;
    }

    case "get_recent_projects":
    case "get_all_projects":
      return projects as T;

    case "update_project_settings": {
      const settings = getArg<Partial<Project>>(args, "settings");
      if (!projectId || !settings) throw new Error("Missing projectId or settings");
      const idx = projects.findIndex((p) => p.id === projectId);
      if (idx < 0) throw new Error(`Project not found: ${projectId}`);
      projects[idx] = { ...projects[idx]!, ...settings };
      return projects[idx] as T;
    }

    case "delete_project": {
      if (!projectId) throw new Error("Missing projectId");
      projects = projects.filter((p) => p.id !== projectId);
      chapters = chapters.filter((c) => c.project_id !== projectId);
      scenes = scenes.filter((s) => {
        const ch = chapters.find((c) => c.id === s.chapter_id);
        return ch?.project_id !== projectId;
      });
      beats = beats.filter((b) => {
        const sc = scenes.find((s) => s.id === b.scene_id);
        return !!sc;
      });
      characters = characters.filter((c) => c.project_id !== projectId);
      locations = locations.filter((l) => l.project_id !== projectId);
      referenceItems = referenceItems.filter((r) => r.project_id !== projectId);
      return undefined as T;
    }

    case "get_chapters": {
      const list = chapters.filter((c) => c.project_id === projectId && !c.archived);
      list.sort((a, b) => a.position - b.position);
      return list as T;
    }

    case "create_chapter": {
      const title = getArg<string>(args, "title") ?? "New Chapter";
      const isPart = getArg<boolean>(args, "isPart", "is_part") ?? false;
      const afterId = getArg<string>(args, "afterId", "after_id");
      if (!projectId) throw new Error("Missing projectId");
      const projectChapters = chapters.filter((c) => c.project_id === projectId && !c.archived);
      const maxPos = projectChapters.length
        ? Math.max(...projectChapters.map((c) => c.position))
        : -1;
      let position = maxPos + 1;
      if (afterId) {
        const after = projectChapters.find((c) => c.id === afterId);
        if (after) position = after.position + 1;
      }
      const ch: Chapter = {
        id: nextId("ch"),
        project_id: projectId,
        title,
        position,
        source_id: null,
        archived: false,
        locked: false,
        is_part: isPart,
        synopsis: null,
        planning_status: "fixed",
      };
      chapters.push(ch);
      return ch as T;
    }

    case "get_scenes": {
      const list = scenes.filter((s) => s.chapter_id === chapterId && !s.archived);
      list.sort((a, b) => a.position - b.position);
      return list as T;
    }

    case "create_scene": {
      const title = getArg<string>(args, "title") ?? "New Scene";
      if (!chapterId) throw new Error("Missing chapterId");
      const chapterScenes = scenes.filter((s) => s.chapter_id === chapterId && !s.archived);
      const maxPos = chapterScenes.length ? Math.max(...chapterScenes.map((s) => s.position)) : -1;
      const sc: Scene = {
        id: nextId("sc"),
        chapter_id: chapterId,
        title,
        synopsis: null,
        prose: null,
        position: maxPos + 1,
        source_id: null,
        archived: false,
        locked: false,
        scene_type: "normal",
        scene_status: "draft",
        planning_status: "fixed",
        editor_mode: "beat",
      };
      scenes.push(sc);
      return sc as T;
    }

    case "get_beats": {
      const list = beats.filter((b) => b.scene_id === sceneId);
      list.sort((a, b) => a.position - b.position);
      return list as T;
    }

    case "create_beat": {
      const content = getArg<string>(args, "content") ?? "New beat";
      if (!sceneId) throw new Error("Missing sceneId");
      const sceneBeats = beats.filter((b) => b.scene_id === sceneId);
      const maxPos = sceneBeats.length ? Math.max(...sceneBeats.map((b) => b.position)) : -1;
      const b: Beat = {
        id: nextId("beat"),
        scene_id: sceneId,
        content,
        prose: null,
        position: maxPos + 1,
      };
      beats.push(b);
      return b as T;
    }

    case "delete_beat": {
      if (!beatId) throw new Error("Missing beatId");
      const beat = beats.find((b) => b.id === beatId);
      if (!beat) throw new Error(`Beat not found: ${beatId}`);
      const sceneIdForRebase = beat.scene_id;
      beats = beats.filter((b) => b.id !== beatId);
      // Rebase positions for remaining beats in the scene
      const sceneBeats = beats.filter((b) => b.scene_id === sceneIdForRebase);
      sceneBeats.sort((a, b) => a.position - b.position);
      sceneBeats.forEach((b, i) => {
        b.position = i;
      });
      return undefined as T;
    }

    case "reorder_beats": {
      const beatIds = getArg<string[]>(args, "beatIds", "beat_ids");
      if (!sceneId || !beatIds) throw new Error("Missing sceneId or beatIds");
      const sceneBeats = beats.filter((b) => b.scene_id === sceneId);
      if (beatIds.length !== sceneBeats.length) {
        throw new Error("Beat order must include all beats in the scene");
      }
      beatIds.forEach((id, position) => {
        const b = beats.find((x) => x.id === id);
        if (b && b.scene_id === sceneId) b.position = position;
      });
      return undefined as T;
    }

    case "split_beat": {
      const beatId = getArg<string>(args, "beatId", "beat_id");
      const splitBeforeParagraph = getArg<number>(
        args,
        "splitBeforeParagraph",
        "split_before_paragraph"
      );
      if (!beatId || splitBeforeParagraph == null)
        throw new Error("Missing beatId or splitBeforeParagraph");
      const beat = beats.find((b) => b.id === beatId);
      if (!beat || !beat.prose) throw new Error("Beat not found or has no prose");
      const paraMatch = beat.prose.match(/<p[^>]*>/g);
      const paraCount = paraMatch?.length ?? 1;
      if (splitBeforeParagraph >= paraCount) throw new Error("Paragraph not found for split");
      const parts = beat.prose.split(/(<p[^>]*>)/g);
      let idx = 0;
      let before = "";
      let after = "";
      for (let i = 0; i < parts.length; i++) {
        if (parts[i].match(/^<p[^>]*>$/)) {
          if (idx === splitBeforeParagraph) {
            before = parts.slice(0, i).join("");
            after = parts.slice(i).join("");
            break;
          }
          idx++;
        }
      }
      if (!after) {
        before = beat.prose.slice(0, Math.floor(beat.prose.length / 2));
        after = beat.prose.slice(Math.floor(beat.prose.length / 2));
      }
      beat.prose = before.trim();
      const newBeat: Beat = {
        id: crypto.randomUUID(),
        scene_id: beat.scene_id,
        content: "",
        prose: after.trim(),
        position: beat.position + 1,
      };
      beats
        .filter((b) => b.scene_id === beat.scene_id && b.position >= newBeat.position)
        .forEach((b) => b.position++);
      beats.push(newBeat);
      return newBeat as T;
    }

    case "merge_beats": {
      const firstBeatId = getArg<string>(args, "firstBeatId", "first_beat_id");
      const secondBeatId = getArg<string>(args, "secondBeatId", "second_beat_id");
      if (!firstBeatId || !secondBeatId) throw new Error("Missing beat ids");
      const first = beats.find((b) => b.id === firstBeatId);
      const second = beats.find((b) => b.id === secondBeatId);
      if (
        !first ||
        !second ||
        first.scene_id !== second.scene_id ||
        first.position + 1 !== second.position
      ) {
        throw new Error("Beats must be adjacent in same scene");
      }
      first.content = first.content ? `${first.content} / ${second.content}` : second.content;
      first.prose = first.prose
        ? `${first.prose}<p></p>${second.prose ?? ""}`
        : (second.prose ?? "");
      beats = beats.filter((b) => b.id !== secondBeatId);
      if (first.prose === "") first.prose = null;
      return first as T;
    }

    case "get_characters":
      return characters.filter((c) => c.project_id === projectId) as T;

    case "get_locations":
      return locations.filter((l) => l.project_id === projectId) as T;

    case "get_references": {
      const list = referenceItems.filter(
        (r) => r.project_id === projectId && r.reference_type === referenceType
      );
      return list as T;
    }

    case "get_scene_reference_items": {
      const list = referenceItems.filter((r) => r.reference_type === referenceType);
      return list as T;
    }

    case "get_scene_reference_state":
      return sceneReferenceStates.filter((s) => s.scene_id === sceneId) as T;

    case "save_scene_reference_state": {
      const states = getArg<Array<{ reference_id: string; position: number; expanded: boolean }>>(
        args,
        "states"
      );
      if (!sceneId || !referenceType || !states) return undefined as T;
      sceneReferenceStates = sceneReferenceStates.filter(
        (s) => !(s.scene_id === sceneId && s.reference_type === referenceType)
      );
      states.forEach((st) => {
        sceneReferenceStates.push({
          scene_id: sceneId,
          reference_type: referenceType as ReferenceItem["reference_type"],
          reference_id: st.reference_id,
          position: st.position,
          expanded: st.expanded,
        });
      });
      return undefined as T;
    }

    case "create_reference": {
      const reference = getArg<{
        name: string;
        description?: string;
        attributes?: Record<string, string>;
      }>(args, "reference");
      if (!projectId || !referenceType || !reference)
        throw new Error("Missing args for create_reference");
      const newRef: ReferenceItem = {
        id: nextId("ref"),
        project_id: projectId,
        reference_type: referenceType as ReferenceItem["reference_type"],
        name: reference.name,
        description: reference.description ?? null,
        attributes: reference.attributes ?? {},
        source_id: null,
      };
      referenceItems.push(newRef);
      return undefined as T;
    }

    case "update_reference": {
      const reference = getArg<{
        name: string;
        description?: string;
        attributes?: Record<string, string>;
      }>(args, "reference");
      if (!referenceId || !reference) return undefined as T;
      const ref = referenceItems.find((r) => r.id === referenceId);
      if (ref) {
        ref.name = reference.name;
        ref.description = reference.description ?? ref.description;
        ref.attributes = reference.attributes ?? ref.attributes;
      }
      return undefined as T;
    }

    case "delete_reference": {
      if (!referenceId) return undefined as T;
      referenceItems = referenceItems.filter((r) => r.id !== referenceId);
      return undefined as T;
    }

    case "reclassify_references": {
      const changes = getArg<Array<{ reference_id: string; new_type: string }>>(args, "changes");
      if (!projectId || !changes) throw new Error("Missing projectId or changes");
      changes.forEach((ch) => {
        const ref = referenceItems.find((r) => r.id === ch.reference_id);
        if (ref) ref.reference_type = ch.new_type as ReferenceItem["reference_type"];
      });
      const proj = projects.find((p) => p.id === projectId);
      if (!proj) throw new Error(`Project not found: ${projectId}`);
      return proj as T;
    }

    case "save_beat_prose": {
      const prose = getArg<string>(args, "prose");
      if (!beatId) throw new Error("Missing beatId");
      const b = beats.find((x) => x.id === beatId);
      if (b) b.prose = prose ?? null;
      return undefined as T;
    }

    case "save_scene_synopsis": {
      const synopsis = getArg<string | null>(args, "synopsis");
      if (!sceneId) throw new Error("Missing sceneId");
      const s = scenes.find((x) => x.id === sceneId);
      if (s) s.synopsis = synopsis ?? null;
      return undefined as T;
    }

    case "update_scene_metadata": {
      const metadata = getArg<{ scene_type?: string; scene_status?: string }>(args, "metadata");
      if (!sceneId || !metadata) return undefined as T;
      const s = scenes.find((x) => x.id === sceneId);
      if (s) {
        if (metadata.scene_type) s.scene_type = metadata.scene_type as Scene["scene_type"];
        if (metadata.scene_status) s.scene_status = metadata.scene_status as Scene["scene_status"];
      }
      return undefined as T;
    }

    case "save_scene_prose":
      return undefined as T;

    case "reorder_chapters": {
      const chapterIds = getArg<string[]>(args, "chapterIds", "chapter_ids");
      if (!projectId || !chapterIds) throw new Error("Missing projectId or chapterIds");
      chapterIds.forEach((id, i) => {
        const ch = chapters.find((c) => c.id === id);
        if (ch) ch.position = i;
      });
      return undefined as T;
    }

    case "reorder_scenes": {
      const sceneIds = getArg<string[]>(args, "sceneIds", "scene_ids");
      if (!chapterId || !sceneIds) throw new Error("Missing chapterId or sceneIds");
      sceneIds.forEach((id, i) => {
        const sc = scenes.find((s) => s.id === id);
        if (sc) sc.position = i;
      });
      return undefined as T;
    }

    case "move_scene_to_chapter":
      return undefined as T;

    case "get_chapter_content_counts": {
      if (!chapterId) throw new Error("Missing chapterId");
      const chScenes = scenes.filter((s) => s.chapter_id === chapterId && !s.archived);
      let totalBeats = 0;
      let totalWords = 0;
      for (const sc of chScenes) {
        const scBeats = beats.filter((b) => b.scene_id === sc.id);
        totalBeats += scBeats.length;
        totalWords += scBeats.reduce((sum, b) => sum + (b.prose?.split(/\s+/).length ?? 0), 0);
      }
      return {
        scene_count: chScenes.length,
        beat_count: totalBeats,
        word_count: totalWords,
      } as T;
    }

    case "get_scene_beat_count": {
      if (!sceneId) throw new Error("Missing sceneId");
      const count = beats.filter((b) => b.scene_id === sceneId).length;
      return count as T;
    }

    case "delete_chapter": {
      if (!chapterId) throw new Error("Missing chapterId");
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.archived = true;
      return undefined as T;
    }

    case "delete_scene": {
      if (!sceneId) throw new Error("Missing sceneId");
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.archived = true;
      return undefined as T;
    }

    case "switch_scene_editor_mode": {
      if (!sceneId) throw new Error("Missing sceneId");
      const mode = getArg<string>(args, "mode") ?? "beat";
      const sc = scenes.find((s) => s.id === sceneId);
      if (!sc) throw new Error(`Scene not found: ${sceneId}`);
      sc.editor_mode = mode as "beat" | "page";
      if (mode === "page") {
        const sceneBeats = beats.filter((b) => b.scene_id === sceneId);
        sceneBeats.sort((a, b) => a.position - b.position);
        sc.prose = sceneBeats.map((b) => b.prose ?? "").join("");
      }
      return sc as T;
    }

    case "save_scene_page_prose": {
      if (!sceneId) throw new Error("Missing sceneId");
      const prose = getArg<string>(args, "prose") ?? "";
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.prose = prose;
      return undefined as T;
    }

    case "reimport_project": {
      const proj = projects.find((p) => p.id === projectId);
      if (!proj) throw new Error(`Project not found: ${projectId}`);
      return {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
      } as T;
    }

    case "get_sync_preview":
      return { additions: [], changes: [] } as T;

    case "apply_sync": {
      const proj = projects.find((p) => p.id === projectId);
      if (!proj) throw new Error(`Project not found: ${projectId}`);
      return {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
      } as T;
    }

    case "rename_chapter": {
      const title = getArg<string>(args, "title");
      if (!chapterId || !title) return undefined as T;
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.title = title;
      return undefined as T;
    }

    case "rename_scene": {
      const title = getArg<string>(args, "title");
      if (!sceneId || !title) return undefined as T;
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.title = title;
      return undefined as T;
    }

    case "duplicate_chapter": {
      const src = chapters.find((c) => c.id === chapterId);
      if (!src || !projectId) throw new Error("Chapter not found");
      const srcScenes = scenes.filter((s) => s.chapter_id === chapterId && !s.archived);
      const newCh: Chapter = {
        ...src,
        id: nextId("ch"),
        title: `${src.title} (copy)`,
        position: src.position + 1,
      };
      chapters.push(newCh);
      for (const sc of srcScenes) {
        const newSc: Scene = {
          ...sc,
          id: nextId("sc"),
          chapter_id: newCh.id,
          position: sc.position,
        };
        scenes.push(newSc);
        const scBeats = beats.filter((b) => b.scene_id === sc.id);
        for (const b of scBeats) {
          beats.push({
            ...b,
            id: nextId("beat"),
            scene_id: newSc.id,
            position: b.position,
          });
        }
      }
      return newCh as T;
    }

    case "duplicate_scene": {
      const src = scenes.find((s) => s.id === sceneId);
      if (!src) throw new Error("Scene not found");
      const targetChapterId = src.chapter_id;
      const newSc: Scene = {
        ...src,
        id: nextId("sc"),
        chapter_id: targetChapterId,
        title: `${src.title} (copy)`,
        position: src.position + 1,
      };
      scenes.push(newSc);
      const srcBeats = beats.filter((b) => b.scene_id === sceneId);
      for (const b of srcBeats) {
        beats.push({
          ...b,
          id: nextId("beat"),
          scene_id: newSc.id,
          position: b.position,
        });
      }
      return newSc as T;
    }

    case "archive_chapter": {
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.archived = true;
      return undefined as T;
    }

    case "archive_scene": {
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.archived = true;
      return undefined as T;
    }

    case "restore_chapter": {
      const ch = chapters.find((c) => c.id === chapterId);
      if (!ch) throw new Error("Chapter not found");
      ch.archived = false;
      return ch as T;
    }

    case "restore_scene": {
      const sc = scenes.find((s) => s.id === sceneId);
      if (!sc) throw new Error("Scene not found");
      sc.archived = false;
      return sc as T;
    }

    case "get_archived_items": {
      const archCh = chapters.filter((c) => c.project_id === projectId && c.archived);
      const archSc = scenes.filter((s) => {
        const ch = chapters.find((c) => c.id === s.chapter_id);
        return ch?.project_id === projectId && s.archived;
      });
      return { chapters: archCh, scenes: archSc } as T;
    }

    case "lock_chapter": {
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.locked = true;
      return undefined as T;
    }

    case "unlock_chapter": {
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.locked = false;
      return undefined as T;
    }

    case "lock_scene": {
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.locked = true;
      return undefined as T;
    }

    case "unlock_scene": {
      const sc = scenes.find((s) => s.id === sceneId);
      if (sc) sc.locked = false;
      return undefined as T;
    }

    case "set_chapter_is_part": {
      const isPart = getArg<boolean>(args, "isPart", "is_part") ?? false;
      const ch = chapters.find((c) => c.id === chapterId);
      if (ch) ch.is_part = isPart;
      return undefined as T;
    }

    case "export_to_markdown":
    case "export_to_longform":
    case "export_to_docx":
    case "export_to_epub": {
      const options = getArg<{ output_path?: string }>(args, "options");
      const outputPath = options?.output_path ?? "/mock/path/export";
      return {
        output_path: outputPath,
        files_created: 1,
        chapters_exported: chapters.filter((c) => c.project_id === projectId && !c.archived).length,
        scenes_exported: scenes.filter((s) => {
          const ch = chapters.find((c) => c.id === s.chapter_id);
          return ch?.project_id === projectId && !s.archived;
        }).length,
      } as T;
    }

    case "get_project_word_count": {
      let total = 0;
      const projChapters = chapters.filter((c) => c.project_id === projectId && !c.archived);
      for (const ch of projChapters) {
        const chScenes = scenes.filter((s) => s.chapter_id === ch.id && !s.archived);
        for (const sc of chScenes) {
          const scBeats = beats.filter((b) => b.scene_id === sc.id);
          total += scBeats.reduce((sum, b) => sum + (b.prose?.split(/\s+/).length ?? 0), 0);
        }
      }
      return total as T;
    }

    case "create_snapshot": {
      if (!projectId) throw new Error("Missing projectId");
      const options = getArg<{ name: string; description?: string; trigger_type?: string }>(
        args,
        "options"
      );
      const name = options?.name ?? "Snapshot";
      const proj = projects.find((p) => p.id === projectId);
      if (!proj) throw new Error(`Project not found: ${projectId}`);
      const chCount = chapters.filter((c) => c.project_id === projectId && !c.archived).length;
      const scCount = scenes.filter((s) => {
        const ch = chapters.find((c) => c.id === s.chapter_id);
        return ch?.project_id === projectId && !s.archived;
      }).length;
      const beatCount = beats.filter((b) => {
        const sc = scenes.find((s) => s.id === b.scene_id);
        return sc && !sc.archived;
      }).length;
      const meta: SnapshotMetadata = {
        id: nextId("snap"),
        project_id: projectId,
        name,
        description: options?.description ?? null,
        trigger_type: (options?.trigger_type as SnapshotMetadata["trigger_type"]) ?? "manual",
        created_at: new Date().toISOString(),
        file_path: `/mock/snapshots/${name}.snap`,
        file_size: 1024,
        uncompressed_size: 2048,
        chapter_count: chCount,
        scene_count: scCount,
        beat_count: beatCount,
        word_count: 0,
        schema_version: 1,
      };
      snapshots.push(meta);
      return meta as T;
    }

    case "list_snapshots":
      return snapshots.filter((s) => s.project_id === projectId) as T;

    case "delete_snapshot": {
      snapshots = snapshots.filter((s) => s.id !== snapshotId);
      return undefined as T;
    }

    case "restore_snapshot": {
      const proj = projects.find((p) => p.id === projectId);
      if (!proj) throw new Error("Project not found");
      return proj as T;
    }

    case "preview_snapshot": {
      const snap = snapshots.find((s) => s.id === snapshotId);
      if (!snap) throw new Error("Snapshot not found");
      const proj = projects.find((p) => p.id === snap.project_id);
      return {
        metadata: snap,
        project_name: proj?.name ?? "Unknown",
      } as T;
    }

    case "get_app_settings":
      return appSettings as T;

    case "update_app_settings": {
      const settings = getArg<Partial<AppSettings>>(args, "settings");
      if (settings) appSettings = { ...appSettings, ...settings };
      return appSettings as T;
    }

    case "get_field_definitions": {
      const entityType = getArg<string>(args, "entityType", "entity_type") ?? "";
      return fieldDefinitions
        .filter((d) => d.project_id === projectId && d.entity_type === entityType)
        .sort((a, b) => a.position - b.position) as T;
    }

    case "get_all_field_definitions": {
      return fieldDefinitions
        .filter((d) => d.project_id === projectId)
        .sort((a, b) => a.position - b.position) as T;
    }

    case "create_field_definition": {
      const def = getArg<Partial<FieldDefinition>>(args, "definition");
      if (!def || !projectId) throw new Error("Missing definition or projectId");
      const existing = fieldDefinitions.filter(
        (d) => d.project_id === projectId && d.entity_type === def.entity_type
      );
      const newDef: FieldDefinition = {
        id: nextId("fd"),
        project_id: projectId,
        entity_type: (def.entity_type ?? "character") as FieldDefinition["entity_type"],
        name: def.name ?? "Untitled",
        field_type: (def.field_type ?? "text") as FieldDefinition["field_type"],
        options: def.options ?? null,
        default_value: def.default_value ?? null,
        position: existing.length,
        required: def.required ?? false,
        visible: def.visible ?? true,
        created_at: new Date().toISOString(),
      };
      fieldDefinitions.push(newDef);
      return newDef as T;
    }

    case "update_field_definition": {
      const defId = getArg<string>(args, "definitionId", "definition_id") ?? "";
      const update = getArg<Partial<FieldDefinition>>(args, "definition");
      const existing = fieldDefinitions.find((d) => d.id === defId);
      if (existing && update) {
        if (update.name !== undefined) existing.name = update.name;
        if (update.field_type !== undefined)
          existing.field_type = update.field_type as FieldDefinition["field_type"];
        if (update.options !== undefined) existing.options = update.options;
        if (update.default_value !== undefined) existing.default_value = update.default_value;
        if (update.required !== undefined) existing.required = update.required;
        if (update.visible !== undefined) existing.visible = update.visible;
      }
      return undefined as T;
    }

    case "delete_field_definition": {
      const defId = getArg<string>(args, "definitionId", "definition_id") ?? "";
      fieldDefinitions = fieldDefinitions.filter((d) => d.id !== defId);
      fieldValues = fieldValues.filter((v) => v.field_definition_id !== defId);
      return undefined as T;
    }

    case "reorder_field_definitions": {
      const ids = getArg<string[]>(args, "definitionIds", "definition_ids") ?? [];
      ids.forEach((id, i) => {
        const d = fieldDefinitions.find((fd) => fd.id === id);
        if (d) d.position = i;
      });
      return undefined as T;
    }

    case "get_field_values": {
      const entityId = getArg<string>(args, "entityId", "entity_id") ?? "";
      return fieldValues.filter((v) => v.entity_id === entityId) as T;
    }

    case "get_field_values_bulk": {
      const entityIds = getArg<string[]>(args, "entityIds", "entity_ids") ?? [];
      const idSet = new Set(entityIds);
      return fieldValues.filter((v) => idSet.has(v.entity_id)) as T;
    }

    case "set_field_value": {
      const fieldDefId = getArg<string>(args, "fieldDefinitionId", "field_definition_id") ?? "";
      const entityId = getArg<string>(args, "entityId", "entity_id") ?? "";
      const value = getArg<string | null>(args, "value") ?? null;
      const existing = fieldValues.find(
        (v) => v.field_definition_id === fieldDefId && v.entity_id === entityId
      );
      if (existing) {
        existing.value = value;
      } else {
        fieldValues.push({
          id: nextId("fv"),
          field_definition_id: fieldDefId,
          entity_id: entityId,
          value,
        });
      }
      return undefined as T;
    }

    case "clear_field_value": {
      const fieldDefId = getArg<string>(args, "fieldDefinitionId", "field_definition_id") ?? "";
      const entityId = getArg<string>(args, "entityId", "entity_id") ?? "";
      fieldValues = fieldValues.filter(
        (v) => !(v.field_definition_id === fieldDefId && v.entity_id === entityId)
      );
      return undefined as T;
    }

    // Tag commands
    case "get_tags": {
      return tags
        .filter((t) => t.project_id === projectId)
        .sort((a, b) => a.position - b.position) as T;
    }

    case "create_tag": {
      const name = getArg<string>(args, "name") ?? "";
      const color = getArg<string | null>(args, "color") ?? null;
      const parentId = getArg<string | null>(args, "parentId", "parent_id") ?? null;
      const newTag: Tag = {
        id: nextId("tag"),
        project_id: projectId ?? "",
        name,
        color,
        parent_id: parentId,
        position: tags.filter((t) => t.project_id === projectId).length,
        created_at: new Date().toISOString(),
      };
      tags.push(newTag);
      return newTag as T;
    }

    case "update_tag": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      const update = getArg<Partial<{ name: string; color: string | null }>>(args, "update");
      const existing = tags.find((t) => t.id === tagId);
      if (existing && update) {
        if (update.name !== undefined) existing.name = update.name;
        if (update.color !== undefined) existing.color = update.color;
      }
      return undefined as T;
    }

    case "delete_tag": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      tags = tags.filter((t) => t.id !== tagId);
      entityTags = entityTags.filter((et) => et.tag_id !== tagId);
      return undefined as T;
    }

    case "reorder_tags": {
      const ids = getArg<string[]>(args, "tagIds", "tag_ids") ?? [];
      ids.forEach((id, i) => {
        const t = tags.find((tag) => tag.id === id);
        if (t) t.position = i;
      });
      return undefined as T;
    }

    case "tag_entity": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const entId = getArg<string>(args, "entityId", "entity_id") ?? "";
      if (
        !entityTags.some(
          (et) => et.tag_id === tagId && et.entity_type === entType && et.entity_id === entId
        )
      ) {
        entityTags.push({ tag_id: tagId, entity_type: entType, entity_id: entId });
      }
      return undefined as T;
    }

    case "untag_entity": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const entId = getArg<string>(args, "entityId", "entity_id") ?? "";
      entityTags = entityTags.filter(
        (et) => !(et.tag_id === tagId && et.entity_type === entType && et.entity_id === entId)
      );
      return undefined as T;
    }

    case "get_entity_tags": {
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const entId = getArg<string>(args, "entityId", "entity_id") ?? "";
      const tagIds = entityTags
        .filter((et) => et.entity_type === entType && et.entity_id === entId)
        .map((et) => et.tag_id);
      return tags.filter((t) => tagIds.includes(t.id)) as T;
    }

    case "bulk_tag": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const entIds = getArg<string[]>(args, "entityIds", "entity_ids") ?? [];
      for (const entId of entIds) {
        if (
          !entityTags.some(
            (et) => et.tag_id === tagId && et.entity_type === entType && et.entity_id === entId
          )
        ) {
          entityTags.push({ tag_id: tagId, entity_type: entType, entity_id: entId });
        }
      }
      return undefined as T;
    }

    case "bulk_untag": {
      const tagId = getArg<string>(args, "tagId", "tag_id") ?? "";
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const entIds = getArg<string[]>(args, "entityIds", "entity_ids") ?? [];
      const idSet = new Set(entIds);
      entityTags = entityTags.filter(
        (et) => !(et.tag_id === tagId && et.entity_type === entType && idSet.has(et.entity_id))
      );
      return undefined as T;
    }

    case "get_all_entity_tags": {
      return entityTags.filter((et) => {
        const tag = tags.find((t) => t.id === et.tag_id);
        return tag?.project_id === projectId;
      }) as T;
    }

    case "filter_entities": {
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const filterJson = getArg<string>(args, "filterJson", "filter_json") ?? "{}";
      try {
        const filter = JSON.parse(filterJson) as { tags?: string[]; operator?: string };
        const filterTagIds = filter.tags ?? [];
        if (filterTagIds.length === 0) return [] as T;
        const matchingEts = entityTags.filter(
          (et) => et.entity_type === entType && filterTagIds.includes(et.tag_id)
        );
        const entityIds = [...new Set(matchingEts.map((et) => et.entity_id))];
        return entityIds as T;
      } catch {
        return [] as T;
      }
    }

    case "save_filter": {
      const name = getArg<string>(args, "name") ?? "";
      const entType = getArg<string>(args, "entityType", "entity_type") ?? "";
      const filterJson = getArg<string>(args, "filterJson", "filter_json") ?? "{}";
      const newFilter: SavedFilter = {
        id: nextId("sf"),
        project_id: projectId ?? "",
        name,
        entity_type: entType,
        filter_json: filterJson,
        position: savedFilters.filter((f) => f.project_id === projectId).length,
      };
      savedFilters.push(newFilter);
      return newFilter as T;
    }

    case "get_saved_filters": {
      return savedFilters.filter((f) => f.project_id === projectId) as T;
    }

    case "delete_saved_filter": {
      const filterId = getArg<string>(args, "filterId", "filter_id") ?? "";
      savedFilters = savedFilters.filter((f) => f.id !== filterId);
      return undefined as T;
    }

    // Detection commands (stubs in dev mode)
    case "detect_scene_references":
      return [] as T;

    case "detect_all_references":
      return {} as T;

    case "dismiss_suggestion":
      return undefined as T;

    default:
      throw new Error(`Mock invoke: unknown command "${cmd}"`);
  }
}
