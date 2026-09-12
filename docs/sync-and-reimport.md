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

![Screenshot: Sync preview dialog](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/sync-preview.png)

## Reimport Workflow

1. Open the project.
2. Choose **Reimport** to re-read the source file.
3. Review the summary after completion.

![Screenshot: Reimport summary dialog](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/sync-summary.png)

## What Sync Updates

Sync/reimport focuses on outline structure:

- Chapter, scene, and beat additions
- Title and synopsis updates (Markdown title changes require label alignment first)
- Beat content updates (Markdown beat text changes require label alignment first)

Locked chapters or scenes are skipped, and prose inside Kindling is preserved.

## Markdown identity and ambiguous changes

Markdown has no permanent outline IDs. Kindling matches unique chapter titles,
scene titles within a chapter, and beat text within a scene. Inserting or
reordering nodes keeps existing prose attached to its original scene and beat.

If sibling labels are duplicated, or unmatched labels could represent a rename
or replacement, sync and reimport stop without applying changes. Give siblings
unique labels and explicitly align renamed titles or beat text in Kindling and
the source, then retry. This also applies to older imported Markdown projects;
positions alone are never used to attach prose to incoming outline nodes.

Local-only nodes remain independent and do not block source additions. Archived
chapters and scenes stay archived; sync neither updates nor recreates them or
their descendants. If a scene or beat moved between parents in the source,
align that move in Kindling before syncing. For beat moves, move the source beat
back to its original scene before syncing; copying prose into a new beat is a
separate, explicit editing operation.

A successful preview may repair missing or duplicate legacy identity metadata.
Cancelling the preview leaves outline content and prose unchanged; the repaired
identities remain saved.

## Troubleshooting

### "Project has no source path"

- Reimport/sync only works for projects created from a source file.

### "Source file not found"

- Move the original source file back to its original location, or reimport from the new path.

### Changes not detected

- Ensure the source file saved successfully before syncing.
- Markdown sources only include outline structure (no references), so reference data will not change.
