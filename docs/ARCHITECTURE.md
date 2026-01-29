# Kindling Architecture

This document provides an overview of the Kindling codebase for contributors. It explains how the pieces fit together and where to find things.

## High-Level Overview

Kindling is a desktop application built with [Tauri](https://tauri.app/), which combines a Rust backend with a web-based frontend. The frontend uses Svelte 5 and communicates with the Rust backend via Tauri's IPC (Inter-Process Communication) system.

```
┌─────────────────────────────────────────────────────────────────┐
│                         Desktop Window                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                  Svelte 5 Frontend                      │   │
│   │  ┌──────────┐  ┌──────────┐  ┌─────────────────────┐   │   │
│   │  │ Sidebar  │  │  Scene   │  │  References Panel   │   │   │
│   │  │          │  │  Panel   │  │                     │   │   │
│   │  └──────────┘  └──────────┘  └─────────────────────┘   │   │
│   │                     │                                   │   │
│   │              ┌──────┴──────┐                            │   │
│   │              │   Stores    │  (Svelte 5 runes)          │   │
│   │              └──────┬──────┘                            │   │
│   └─────────────────────┼───────────────────────────────────┘   │
│                         │ invoke()                              │
│   ┌─────────────────────┼───────────────────────────────────┐   │
│   │                Tauri IPC Bridge                         │   │
│   └─────────────────────┼───────────────────────────────────┘   │
│                         │                                       │
│   ┌─────────────────────┼───────────────────────────────────┐   │
│   │                  Rust Backend                           │   │
│   │  ┌──────────┐  ┌────┴─────┐  ┌──────────────────────┐   │   │
│   │  │ Parsers  │  │ Commands │  │      Database        │   │   │
│   │  │ (import) │  │  (IPC)   │  │      (SQLite)        │   │   │
│   │  └──────────┘  └──────────┘  └──────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
kindling/
├── src/                      # Svelte frontend
│   ├── lib/
│   │   ├── components/       # UI components
│   │   ├── stores/           # State management (Svelte 5 runes)
│   │   └── types.ts          # TypeScript interfaces
│   ├── App.svelte            # Root component
│   └── main.ts               # Entry point
│
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs            # App initialization, command registration
│   │   ├── commands/         # Tauri IPC command handlers (by module)
│   │   ├── models/           # Data structures (Project, Scene, ReferenceItem, etc.)
│   │   ├── db/               # SQLite schema and queries
│   │   └── parsers/          # Import format parsers (Plottr, Markdown, yWriter, Longform)
│   └── tauri.conf.json       # Tauri configuration
│
├── e2e/                      # End-to-end tests (WebdriverIO)
├── docs/                     # Documentation
└── test-data/                # Test fixtures
```

## Frontend Architecture

### Components (`src/lib/components/`)

| Component                | Purpose                                                                                |
| ------------------------ | -------------------------------------------------------------------------------------- |
| `App.svelte`             | Root component, routes between StartScreen and Editor                                  |
| `StartScreen.svelte`     | Project selection and import UI                                                        |
| `Sidebar.svelte`         | Chapter/scene tree navigation                                                          |
| `ScenePanel.svelte`      | Main editing area with beats                                                           |
| `ReferencesPanel.svelte` | Reference cards across types (characters, locations, items, objectives, organizations) |
| `Onboarding.svelte`      | First-launch tutorial flow                                                             |
| `ContextMenu.svelte`     | Right-click context menus                                                              |
| `ConfirmDialog.svelte`   | Confirmation modals                                                                    |
| `RenameDialog.svelte`    | Rename modals                                                                          |
| `ArchivePanel.svelte`    | View and restore archived items                                                        |

### State Management (`src/lib/stores/`)

Kindling uses Svelte 5's runes-based reactivity. State is managed through class-based stores:

**`project.svelte.ts`** - Project data state:

```typescript
// Access via: currentProject
currentProject.value; // Current Project or null
currentProject.chapters; // Chapter[]
currentProject.scenes; // Scene[] (for current chapter)
currentProject.beats; // Beat[] (for current scene)
currentProject.characters; // Character[]
currentProject.locations; // Location[]
```

**`ui.svelte.ts`** - UI state:

```typescript
// Access via: ui
ui.currentView; // 'start' | 'editor'
ui.sidebarCollapsed; // boolean
ui.referencesPanelCollapsed; // boolean
ui.focusMode; // boolean
ui.showOnboarding; // boolean
ui.isImporting; // boolean
```

### Data Flow

1. User interacts with a component
2. Component calls `invoke()` to send command to Rust
3. Rust processes command and returns data
4. Component updates the store
5. Reactive UI updates automatically

Example:

```typescript
// In a component
import { invoke } from "@tauri-apps/api/core";
import { currentProject } from "./lib/stores/project.svelte";

async function loadChapters(projectId: string) {
  const chapters = await invoke("get_chapters", { projectId });
  currentProject.setChapters(chapters);
}
```

## Backend Architecture

### Commands (`src-tauri/src/commands/`)

All frontend-backend communication goes through Tauri commands. Commands are async functions decorated with `#[tauri::command]`:

```rust
#[tauri::command]
pub async fn get_chapters(
    project_id: String,
    state: State<'_, AppState>
) -> Result<Vec<Chapter>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_chapters(&conn, &project_id).map_err(|e| e.to_string())
}
```

Commands are registered in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_chapters,
    commands::create_chapter,
    // ...
])
```

**Command categories:**

- **Import**: `import_plottr`, `import_markdown`, `import_ywriter`, `import_longform`
- **CRUD**: `get_*`, `create_*`, `delete_*`, `rename_*`
- **Reorder**: `reorder_chapters`, `reorder_scenes`, `move_scene_to_chapter`
- **Sync**: `get_sync_preview`, `apply_sync`, `reimport_project`
- **Archive**: `archive_*`, `restore_*`, `get_archived_items`
- **Lock**: `lock_*`, `unlock_*`
- **Export**: `export_to_docx`, `export_to_markdown`, `export_to_longform`, `export_to_epub`
- **Snapshot**: `create_snapshot`, `list_snapshots`, `preview_snapshot`, `restore_snapshot`, `delete_snapshot`
- **Settings**: `get_app_settings`, `update_app_settings`, `update_project_settings`

### Models (`src-tauri/src/models/`)

Each model maps to a database table:

| Model           | Description                                                               |
| --------------- | ------------------------------------------------------------------------- |
| `Project`       | Top-level container, tracks source file                                   |
| `Chapter`       | Groups scenes, has position for ordering                                  |
| `Scene`         | Writing unit, contains synopsis and prose                                 |
| `Beat`          | Story beat within a scene                                                 |
| `Character`     | Character reference with attributes                                       |
| `Location`      | Location reference with attributes                                        |
| `ReferenceItem` | Typed reference (characters, locations, items, objectives, organizations) |

### Database (`src-tauri/src/db/`)

- **`schema.rs`**: Table definitions and migrations
- **`queries.rs`**: CRUD operations
- **`mod.rs`**: Module exports

The database is SQLite, stored in the app's data directory (`kindling.db`).

**Key tables:**

```
projects → chapters → scenes → beats
    ↓
characters, locations, reference_items (with scene reference links)
```

### Parsers (`src-tauri/src/parsers/`)

Native Rust parsers for importing outlines:

| Parser        | File Type | Notes                                   |
| ------------- | --------- | --------------------------------------- |
| `plottr.rs`   | `.pltr`   | JSON-based, extracts timeline/beats     |
| `markdown.rs` | `.md`     | Heading-based outline format            |
| `ywriter.rs`  | `.yw7`    | yWriter project import                  |
| `longform.rs` | `.md`     | Longform/Obsidian index or vault import |

Each parser returns a `ParsedProject` struct that gets inserted into the database.

## Data Model

```
Project
├── Chapters (ordered by position)
│   └── Scenes (ordered by position)
│       └── Beats (ordered by position)
├── Characters
│   └── Attributes (key-value pairs)
└── Locations
    └── Attributes (key-value pairs)
```

**Key concepts:**

- `source_id`: Links imported items back to their source file IDs (for re-import sync)
- `position`: Integer for ordering within parent
- `archived`: Soft-delete flag
- `locked`: Prevents editing

## Import & Sync Flow

### Initial Import

1. User selects file via system dialog
2. Frontend calls `import_*` command
3. Parser extracts structure from source file
4. Data inserted into SQLite with `source_id` mappings
5. Project returned to frontend, UI updates

### Re-import Sync

1. User triggers reimport on existing project
2. `get_sync_preview` parses source file and compares to DB
3. Returns `SyncPreview` with additions and changes
4. User selects which changes to apply
5. `apply_sync` updates only selected items
6. Prose written in Kindling is preserved

## Testing

### Frontend Tests

- **Location**: `src/**/*.test.ts`
- **Framework**: Vitest + Testing Library
- **Run**: `npm test`

### Backend Tests

- **Location**: `src-tauri/src/**/*.rs` (inline `#[cfg(test)]` modules)
- **Framework**: Rust's built-in test framework
- **Run**: `cd src-tauri && cargo test`

### E2E Tests

- **Location**: `e2e/specs/`
- **Framework**: WebdriverIO with Tauri driver
- **Run**: See `e2e/README.md`

## Key Patterns

### Error Handling

- Rust commands return `Result<T, String>`
- Frontend wraps `invoke()` in try/catch
- Errors displayed via toast notifications (planned)

### Async Operations

- All Tauri commands are async
- Frontend uses `$effect()` for reactive data loading
- Import progress tracked via UI store

### Persistence

- UI preferences stored in `localStorage`
- Project data stored in SQLite
- Session state (last opened scene) stored per-project

## Adding a New Feature

### Adding a new Tauri command:

1. Add function in the appropriate `src-tauri/src/commands/` module
2. Register in `lib.rs` invoke_handler
3. Add TypeScript types to `types.ts`
4. Call via `invoke()` from frontend

### Adding a new component:

1. Create `.svelte` file in `src/lib/components/`
2. Add `data-testid` attributes for E2E tests
3. Import and use in parent component

### Adding a database field:

1. Add migration in `schema.rs`
2. Update model struct
3. Update relevant queries
4. Update TypeScript types

## Resources

- [Tauri Documentation](https://tauri.app/v2/start/)
- [Svelte 5 Runes](https://svelte.dev/docs/svelte/what-are-runes)
- [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
