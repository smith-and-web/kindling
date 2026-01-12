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
}

export interface Scene {
  id: string;
  chapter_id: string;
  title: string;
  synopsis: string | null;
  prose: string | null;
  position: number;
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
