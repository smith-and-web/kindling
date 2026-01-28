# Sync and Reimport

Kindling can keep source-backed projects in sync with their original files while preserving the prose you write in the app.

## Sync vs. Reimport

- **Sync preview** compares your source file to the current project and lets you apply selected changes.
- **Reimport** re-reads the source file and updates the outline structure in one pass.

Both options preserve existing prose written in Kindling.

## Supported Sources

Sync/reimport is available for:

- Plottr (`.pltr`)
- Markdown (`.md`)
- yWriter (`.yw7`)
- Longform/Obsidian (Longform index file)

## Sync Preview Workflow

1. Open the project.
2. Choose **Sync** to generate a preview.
3. Review additions and changes, then apply the ones you want.

TODO: Add screenshot of the sync preview dialog.
![Screenshot: Sync preview dialog](docs/assets/placeholder-sync-preview.png)

## Reimport Workflow

1. Open the project.
2. Choose **Reimport** to re-read the source file.
3. Review the summary after completion.

TODO: Add screenshot of the reimport summary dialog.
![Screenshot: Reimport summary dialog](docs/assets/placeholder-reimport-summary.png)

## What Sync Updates

Sync/reimport focuses on outline structure:

- Chapter, scene, and beat additions
- Title and synopsis updates
- Beat content updates

Locked chapters or scenes are skipped, and prose inside Kindling is preserved.

## Troubleshooting

### "Project has no source path"

- Reimport/sync only works for projects created from a source file.

### "Source file not found"

- Move the original source file back to its original location, or reimport from the new path.

### Changes not detected

- Ensure the source file saved successfully before syncing.
- Markdown sources only include outline structure (no references), so reference data will not change.
