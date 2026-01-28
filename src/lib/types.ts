/**
 * TypeScript types matching the Rust models in src-tauri/src/models/
 *
 * These types mirror the Rust structs and are used for type-safe communication
 * between the frontend and backend via Tauri's invoke() API.
 *
 * When adding new fields, update both this file AND the corresponding Rust model.
 */

/** Supported outline import formats */
export type SourceType = "Plottr" | "Markdown" | "YWriter" | "Scrivener" | "Longform";

/** Top-level container for a writing project */
export interface Project {
  id: string;
  name: string;
  source_type: SourceType;
  /** Path to the original imported file (for re-import sync) */
  source_path: string | null;
  created_at: string;
  modified_at: string;
  // Project-specific metadata for export title page
  /** Pen name for this project (overrides app-level author name if set) */
  author_pen_name: string | null;
  /** Genre of the work (displayed on title page) */
  genre: string | null;
  /** Project description or summary */
  description: string | null;
  /** Optional word count target for the project */
  word_target: number | null;
  /** Enabled reference types for this project */
  reference_types: ReferenceTypeId[];
}

/** App-wide settings (stored in JSON file, not database) */
export interface AppSettings {
  /** Author's name (used in contact info on title pages) */
  author_name: string | null;
  /** First line of contact address (e.g., street address) */
  contact_address_line1: string | null;
  /** Second line of contact address (e.g., city, country, postal code) */
  contact_address_line2: string | null;
  /** Phone number */
  contact_phone: string | null;
  /** Email address */
  contact_email: string | null;
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
  /** True if this chapter is a Part header (section heading). Part chapters group subsequent chapters until the next Part. */
  is_part: boolean;
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
  scene_type: SceneType;
  scene_status: SceneStatus;
}

export type SceneType = "normal" | "notes" | "todo" | "unused";

export type SceneStatus = "draft" | "revised" | "final";

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

/** Supported reference types for the References panel */
export type ReferenceTypeId = "characters" | "locations" | "items" | "objectives" | "organizations";

/** Generic reference card used for extended reference types */
export interface ReferenceItem {
  id: string;
  project_id: string;
  reference_type: ReferenceTypeId;
  name: string;
  description: string | null;
  attributes: Record<string, string>;
  source_id: string | null;
}

/** Persisted per-scene reference workspace state */
export interface SceneReferenceState {
  scene_id: string;
  reference_type: ReferenceTypeId;
  reference_id: string;
  position: number;
  expanded: boolean;
}

export interface SceneReferenceStateUpdate {
  reference_id: string;
  position: number;
  expanded: boolean;
}

export interface ReferenceReclassification {
  reference_id: string;
  new_type: ReferenceTypeId;
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
  /** Delete existing export folder if it exists */
  delete_existing: boolean;
  /** Custom name for the export folder (defaults to project name) */
  export_name?: string;
  /** Create a snapshot before exporting */
  create_snapshot?: boolean;
}

/** Options for Longform export */
export interface LongformExportOptions {
  /** What to export (project, chapter, or scene) */
  scope: ExportScope;
  /** Output directory path */
  output_path: string;
  /** Custom name for the export folder (defaults to project name) */
  export_name?: string;
  /** Delete existing export folder if it exists */
  delete_existing?: boolean;
  /** Create a snapshot before exporting */
  create_snapshot?: boolean;
}

/** Chapter heading style for DOCX export */
export type ChapterHeadingStyle =
  | "number_only"
  | "number_and_title"
  | "title_only"
  | "number_arabic"
  | "number_arabic_and_title";

/** Scene break marker style for DOCX export */
export type SceneBreakStyle = "hash" | "asterisks" | "asterism" | "blank_line";

/** Font family for DOCX export */
export type FontFamily = "courier_new" | "times_new_roman";

/** Line spacing option for DOCX export */
export type LineSpacingOption = "single" | "one_and_half" | "double";

/** Options for DOCX export */
export interface DocxExportOptions {
  /** What to export (project, chapter, or scene) */
  scope: ExportScope;
  /** Include beat markers as Heading 3 in output */
  include_beat_markers: boolean;
  /** Include scene synopsis as italicized paragraph */
  include_synopsis: boolean;
  /** Output file path (full path including filename) */
  output_path: string;
  /** Create a snapshot before exporting */
  create_snapshot?: boolean;
  /** Add page breaks between chapters */
  page_breaks_between_chapters?: boolean;
  /** Include a Standard Manuscript Format title page */
  include_title_page?: boolean;
  /** Chapter heading style */
  chapter_heading_style?: ChapterHeadingStyle;
  /** Scene break marker style */
  scene_break_style?: SceneBreakStyle;
  /** Font family for body text */
  font_family?: FontFamily;
  /** Line spacing for body text */
  line_spacing?: LineSpacingOption;
}

/** Styling themes for EPUB export */
export type EpubTheme = "classic" | "modern" | "minimal";

/** EPUB metadata fields */
export interface EpubMetadata {
  title: string;
  author: string;
  description?: string;
  language: string;
}

/** Options for EPUB export */
export interface EpubExportOptions {
  /** What to export (project, chapter, or scene) */
  scope: ExportScope;
  /** Include beat markers as headings */
  include_beat_markers: boolean;
  /** Include scene synopsis as italicized paragraph */
  include_synopsis: boolean;
  /** Output file path */
  output_path: string;
  /** Create a snapshot before exporting */
  create_snapshot?: boolean;
  /** Metadata fields for EPUB */
  metadata: EpubMetadata;
  /** Theme selection for EPUB styling */
  theme: EpubTheme;
  /** Include cover image */
  include_cover_image: boolean;
  /** Cover image file path */
  cover_image_path?: string;
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

// =============================================================================
// Snapshot Types
// Used for creating and restoring project snapshots (versioning)
// =============================================================================

/** Trigger type for snapshot creation */
export type SnapshotTrigger = "manual" | "export" | "auto";

/** Restore mode for snapshots */
export type RestoreMode = "replace_current" | "create_new";

/** Metadata about a snapshot stored in the database */
export interface SnapshotMetadata {
  id: string;
  project_id: string;
  name: string;
  description?: string | null;
  trigger_type: SnapshotTrigger;
  created_at: string;
  file_path: string;
  file_size: number;
  uncompressed_size?: number | null;
  chapter_count: number;
  scene_count: number;
  beat_count: number;
  word_count?: number | null;
  schema_version: number;
}

/** Options for creating a snapshot */
export interface CreateSnapshotOptions {
  name: string;
  description?: string;
  trigger_type: SnapshotTrigger;
}

/** Options for restoring a snapshot */
export interface RestoreSnapshotOptions {
  mode: RestoreMode;
  new_project_name?: string;
}

/** Preview of a snapshot (lightweight metadata) */
export interface SnapshotPreview {
  metadata: SnapshotMetadata;
  project_name: string;
}
