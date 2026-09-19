# References in Kindling

References help you track people, places, and story elements across scenes. Each reference can include notes and custom attributes, and you can link them to specific scenes.

## Reference Types

Kindling supports these reference types:

- Characters
- Locations
- Items
- Objectives
- Organizations

## References Panel Overview

The References panel shows tabs for each enabled reference type and lets you create, edit, and link references.

![Screenshot: References panel](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/references-panel.png)

## Linking References to Scenes

When a scene is selected:

- References linked to the scene appear first
- Unlinked references appear after
- Drag references to set per-scene ordering
- Expand/collapse state is saved per scene

![Screenshot: Linked references in a scene](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/linked-references.png)

## Editing Reference Details

Each reference includes:

- Name
- Description
- Notes
- Custom attributes (key/value pairs)

![Screenshot: Reference edit dialog](https://raw.githubusercontent.com/smith-and-web/kindling/main/docs/assets/reference-edit-dialog.png)

## Reference Type Settings

Open **File → Settings → Reference Types**, then choose a project to enable or disable its reference types.


## Copying References Between Projects

Open the book that should receive the references, then choose **Copy references from project…** in the References panel header. Choose the source project and review the references before copying.

All references are selected initially, including references in categories disabled in the source project. Search by name, select individual references or categories, or clear the selection. Searching hides nonmatching entries without deselecting them; the selected count always includes hidden entries.

Copies include descriptions, notes, legacy attributes, custom field definitions and values, and assigned tags with their parent tags. Categories containing copied references become available in the destination project. Scene links, manuscript content, and external import connections are not copied.

Possible duplicates are matched against existing destination references by category and name, ignoring capitalization and surrounding spaces. They are skipped by default. Choose **Keep both** to create another reference with the name shown in the preview, such as `Mara (copy)`. Same-name references within the source are all copied unless they match an existing destination reference; any allocated names are shown in the preview. Existing references are never overwritten. Copying again does not update an earlier copy, and a renamed reference may not be recognized as a possible duplicate.

Review the field and tag summary before copying. Compatible definitions are reused; conflicting fields or tags receive distinct names. New fields become available on existing destination references in that category too, without changing their stored values.

**These are independent copies. Changes won't update other projects.** This lets each book evolve separately. Shared references and ongoing updates across a series are outside this feature.

A failed transfer rolls back the entire copy. If the data changes while you review it, refresh the preview and review it again. If the copy succeeds but the panel cannot refresh, use **Refresh references**; the copy has already completed.

There is no batch undo. Remove unwanted copied references and unused fields or tags through their existing controls. Snapshots do not currently include typed fields or project tags, so snapshot restore is not a complete undo for a transfer.
