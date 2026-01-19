/**
 * TypeScript types matching the Rust models in src-tauri/src/models/
 *
 * These types mirror the Rust structs and are used for type-safe communication
 * between the frontend and backend via Tauri's invoke() API.
 *
 * When adding new fields, update both this file AND the corresponding Rust model.
 */

/** Supported outline import formats */
export type SourceType = "Scrivener" | "Plottr" | "Markdown";

/** Top-level container for a writing project */
export interface Project {
  id: string;
  name: string;
  source_type: SourceType;
  /** Path to the original imported file (for re-import sync) */
  source_path: string | null;
  created_at: string;
  modified_at: string;
}

/** A chapter groups related scenes together */
export interface Chapter {
  id: string;
  project_id: string;
  title: string;
  /** Order within the project (0-indexed) */
  position: number;
  /** ID from the original source file (for re-import sync) */
  source_id?: string | null;
  /** Soft-deleted chapters are hidden but recoverable */
  archived: boolean;
  /** Locked chapters cannot be edited */
  locked: boolean;
}

/** A scene is the primary unit of writing, containing beats and prose */
export interface Scene {
  id: string;
  chapter_id: string;
  title: string;
  /** Brief description of the scene's purpose */
  synopsis: string | null;
  /** Scene-level prose (separate from beat prose) */
  prose: string | null;
  /** Order within the chapter (0-indexed) */
  position: number;
  /** ID from the original source file (for re-import sync) */
  source_id?: string | null;
  archived: boolean;
  locked: boolean;
}

/** Container for archived (soft-deleted) items */
export interface ArchivedItems {
  chapters: Chapter[];
  scenes: Scene[];
}

/**
 * A beat is a story point within a scene.
 * Writers expand beats to write prose beneath them.
 */
export interface Beat {
  id: string;
  scene_id: string;
  /** The beat's outline content (what needs to happen) */
  content: string;
  /** The prose written for this beat */
  prose: string | null;
  /** Order within the scene (0-indexed) */
  position: number;
}

/** A character reference card shown in the References panel */
export interface Character {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  /** Flexible key-value attributes (e.g., "Age": "32", "Role": "Protagonist") */
  attributes: Record<string, string>;
  source_id: string | null;
}

/** A location reference card shown in the References panel */
export interface Location {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  /** Flexible key-value attributes (e.g., "Type": "City", "Climate": "Tropical") */
  attributes: Record<string, string>;
  source_id: string | null;
}

/** Persisted state for resuming where the user left off */
export interface SessionState {
  project_id: string;
  current_scene_id: string | null;
  cursor_position: number | null;
  scroll_position: number | null;
  last_opened_at: string | null;
}

// =============================================================================
// Sync/Reimport Types
// Used when re-importing from a source file to sync changes while preserving prose
// =============================================================================

/** A new item found in the source file that doesn't exist in the database */
export interface SyncAddition {
  id: string;
  item_type: "chapter" | "scene" | "beat";
  title: string;
  parent_title: string | null;
}

/** A change detected between the source file and the database */
export interface SyncChange {
  id: string;
  item_type: "chapter" | "scene" | "beat";
  field: "title" | "synopsis" | "content";
  item_title: string;
  current_value: string;
  new_value: string;
  /** The database ID of the item to update */
  db_id: string;
}

/** Preview of changes that would occur during a sync operation */
export interface SyncPreview {
  additions: SyncAddition[];
  changes: SyncChange[];
}

/** Summary statistics after a reimport operation completes */
export interface ReimportSummary {
  chapters_added: number;
  chapters_updated: number;
  scenes_added: number;
  scenes_updated: number;
  beats_added: number;
  beats_updated: number;
  /** Count of prose blocks that were preserved (not overwritten) */
  prose_preserved: number;
}

// =============================================================================
// Export Types
// Used for exporting projects to various formats (Markdown, DOCX)
// =============================================================================

/** Export scope - what portion of the project to export */
export type ExportScope = "project" | { chapter: string } | { scene: string };

/** Options for Markdown export */
export interface MarkdownExportOptions {
  /** What to export (project, chapter, or scene) */
  scope: ExportScope;
  /** Include beat markers (## Beat: content) in output */
  include_beat_markers: boolean;
  /** Output directory path */
  output_path: string;
}

/** Result of an export operation */
export interface ExportResult {
  /** Path where export was saved */
  output_path: string;
  /** Number of files created */
  files_created: number;
  /** Total chapters exported */
  chapters_exported: number;
  /** Total scenes exported */
  scenes_exported: number;
}
