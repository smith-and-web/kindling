# Scene Workflow in Kindling

Scenes are where you turn your outline into prose. Each scene combines beats, synopsis, and metadata to keep your draft structured.

## Scene Panel Layout

When you open a scene, the Scene panel shows its beats, synopsis, and metadata controls in one place.

![Screenshot: Scene panel layout](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/scene-panel.png)

## Beats and Prose

- Beats are collapsible cards that act as prompts
- Expand a beat to write prose directly beneath it
- Prose auto-saves as you write
- Collapse a beat to keep the outline visible while you draft

![Screenshot: Beat editor with prose](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/beat-with-prose.png)

## Returning to a Project

Reopening a project restores the last scene you viewed, the expanded beat, your prose cursor, and your scroll position. This works in both beat and page mode, including after restarting Kindling. Each project remembers its own position. If the saved scene was deleted or archived, the project opens with the usual chapter selection.

## Synopsis Editing

Use the synopsis field to capture a short summary for the scene. Synopses can be edited inline and are saved automatically.

## Scene Type and Status

Scene metadata helps you filter and organize your outline:

- **Type**: Normal, Notes, ToDo, or Unused
- **Status**: Draft, Revised, Final

These controls influence scene filters in the sidebar and are used in exports.

## Scene Locking

Locked scenes (or scenes inside locked chapters) are read-only. Unlock the scene to edit beats, prose, or metadata.

## Cancelling quit after a save failure

If saving fails while quitting, choose **Keep editing** or press **Escape** to
return to the same writing editor and selection. Kindling restores the caret only
while that editor still belongs to the same scene and project and you have not
moved focus to another control outside the quit prompt.

## Snapshot independence

Each newly created project snapshot has its own backing file, including snapshots
created in rapid succession. Deleting one leaves the others available to preview
and restore.
