// TypeScript types matching the Rust models

export type SourceType = "Scrivener" | "Plottr" | "Markdown";

export interface Project {
  id: string;
  name: string;
  source_type: SourceType;
  source_path: string | null;
  created_at: string;
  modified_at: string;
}

export interface Chapter {
  id: string;
  project_id: string;
  title: string;
  position: number;
  source_id?: string | null;
  archived: boolean;
  locked: boolean;
}

export interface Scene {
  id: string;
  chapter_id: string;
  title: string;
  synopsis: string | null;
  prose: string | null;
  position: number;
  source_id?: string | null;
  archived: boolean;
  locked: boolean;
}

export interface ArchivedItems {
  chapters: Chapter[];
  scenes: Scene[];
}

export interface Beat {
  id: string;
  scene_id: string;
  content: string;
  prose: string | null;
  position: number;
}

export interface Character {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  attributes: Record<string, string>;
  source_id: string | null;
}

export interface Location {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  attributes: Record<string, string>;
  source_id: string | null;
}

export interface SessionState {
  project_id: string;
  current_scene_id: string | null;
  cursor_position: number | null;
  scroll_position: number | null;
  last_opened_at: string | null;
}

// Sync preview types
export interface SyncAddition {
  id: string;
  item_type: "chapter" | "scene" | "beat";
  title: string;
  parent_title: string | null;
}

export interface SyncChange {
  id: string;
  item_type: "chapter" | "scene" | "beat";
  field: "title" | "synopsis" | "content";
  item_title: string;
  current_value: string;
  new_value: string;
  db_id: string;
}

export interface SyncPreview {
  additions: SyncAddition[];
  changes: SyncChange[];
}

export interface ReimportSummary {
  chapters_added: number;
  chapters_updated: number;
  scenes_added: number;
  scenes_updated: number;
  beats_added: number;
  beats_updated: number;
  prose_preserved: number;
}
