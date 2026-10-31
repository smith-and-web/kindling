import type { Catalog } from "../../types";

/** Copy produced outside components: stores, utils, import formats and command labels. */
const catalog: Catalog = {
  // --- UI store ---
  "ui.importing": "Importing",
  "ui.startingImport": "Starting import…",
  "ui.importComplete": "Import complete!",

  // --- Shortcut store and binding checks ---
  "shortcuts.loadFailed": "Could not load keyboard shortcuts: {error}",
  "shortcuts.updateFailed": "Could not update native keyboard shortcuts: {error}",
  "shortcuts.reserved": "This shortcut is reserved for standard text editing or window controls.",
  "shortcuts.conflict": "Already used by {label}. Clear or change that shortcut first.",

  // --- Import helpers ---
  "import.selectScrivenerFolder": "Select Scrivener project folder (.scriv)",
  "import.scrivenerFolderOnly": "Please select a folder whose name ends with .scriv",
  "import.unknownType": "Unknown import type: {type}",
  "import.failed": "Import failed: {error}",

  // --- Import formats ---
  "importFormat.plottr": "Plottr file",
  "importFormat.markdown": "Markdown file",
  "importFormat.ywriter": "yWriter file",
  "importFormat.longform": "Longform index",
  "importFormat.longformVault": "Longform vault",
  "importFormat.novelwriter": "novelWriter project",
  "importFormat.scrivener": "Scrivener project",

  // --- Reference types ---
  "referenceType.characters": "Characters",
  "referenceType.locations": "Locations",
  "referenceType.items": "Items",
  "referenceType.objectives": "Objectives",
  "referenceType.organizations": "Organizations",
  "referenceType.timelines": "Timelines",
  "referenceType.custom": "Notes",

  // --- Editorial review validation ---
  "editorial.unknownSource": "The review contains an unknown manuscript source.",
  "editorial.paragraphFormatting": "The review contains unsupported paragraph formatting.",
  "editorial.unsupportedLink": "The review contains an unsupported link.",
  "editorial.outsidePassage": "A feedback annotation refers to a passage outside this manuscript.",
  "editorial.mismatchedPassage": "A feedback annotation does not match its original passage.",
  "editorial.invalidAnchor": "A comment has an invalid passage anchor.",
  "editorial.noSources": "The review has no manuscript sources.",
  "editorial.reorderSources":
    "This edit would reorder manuscript sources. Copy the passage as text instead.",
  "editorial.lostSource":
    "A manuscript paragraph has lost its source. Reopen the review before continuing.",
  "editorial.noCurrentPassages":
    "There are no current manuscript passages in this review. Restore a scene before applying feedback.",
  "editorial.passageChanged":
    "This passage changed. Compare the original, current, and suggested prose, then select where the change belongs.",
  "editorial.overlap": "These suggestions overlap. Review and accept them individually.",
  "editorial.selectPassage": "Select a passage within the manuscript.",

  // --- Prose save queue ---
  "proseSaves.draftsChanged": "Unsaved drafts changed. Retry loading before discarding them.",
  "proseSaves.flushFailed":
    "Could not save the latest prose. {errors} Retry saving or review the unsaved drafts in Find and Replace.",
  "proseFlush.message":
    "Save or recover unsaved prose before {action}. Review the unsaved drafts in Find and Replace.",
  "proseFlush.action.exporting": "exporting",
  "proseFlush.action.syncing": "syncing",

  // --- Editorial recovery and re-anchoring ---
  "editorialSaves.recoveredMismatch":
    "The recovered review has a different manuscript. Keep this window open and export a recovery copy.",
  "localEditorial.reanchor":
    "Select a passage within this scene’s original prose to re-anchor its saved annotation.",

  // --- Review suggestions ---
  "revisions.suggestionClosed": "Suggestion is no longer open",
  "revisions.proseChanged": "Prose changed. Re-anchor outdated suggestions before accepting them.",
  "revisions.overlap": "Suggestions overlap. Review and accept them individually.",

  // --- Utility errors ---
  "referenceCopy.failed": "Could not copy references. Please try again.",
  "synopsisSaves.changed": "Synopsis drafts changed. Review the latest changes before discarding.",
  "exportPrototype.nameRequired": "Give this profile a name before saving.",
  "exportPrototype.startNumberInteger": "Chapter start number must be a whole number.",
  "exportPrototype.unsupportedFormat": "Saved prototype profiles have an unsupported format.",
  "exportPrototype.invalidProfile": "A saved prototype profile is invalid.",
  "exportPrototype.unsupportedOption": "A saved prototype option is unsupported.",
  "exportPrototype.duplicateIds": "Saved prototype profiles have duplicate IDs.",
  "exportPrototype.fontSize": "Font size",
  "exportPrototype.lineSpacing": "Line spacing",
  "exportPrototype.paragraphSpacing": "Paragraph spacing",
  "exportPrototype.indent": "First-line indent",
  "exportPrototype.margin": "Margins",
  "exportPrototype.startNumber": "Chapter start number",
  "exportPrototype.rangeError": "{label} must be between {min} and {max}.",

  // --- Command categories ---
  "commandCategory.File": "File",
  "commandCategory.View": "View",
  "commandCategory.Edit": "Edit",
  "commandCategory.Help": "Help",
  "commandCategory.Project": "Project",
  "commandCategory.Editor": "Editor",

  // --- Command labels (mirror shortcutDefinitions.json) ---
  "command.editorial_open.label": "Open review or feedback file",
  "command.editorial_project.label": "Editorial review and packages",
  "command.find.label": "Find in scene",
  "command.find_replace.label": "Find and replace",
  "command.find_project.label": "Find and replace in project",
  "command.export.label": "Export project",
  "command.close_project.label": "Close project",
  "command.quit.label": "Quit Kindling",
  "command.import_plottr.label": "Import Plottr (.pltr)",
  "command.import_markdown.label": "Import Markdown (.md)",
  "command.import_longform.label": "Import Longform",
  "command.import_ywriter.label": "Import yWriter 7 (.yw7)",
  "command.import_scrivener.label": "Import Scrivener 3 (.scriv)",
  "command.import_novelwriter.label": "Import novelWriter",
  "command.sync.label": "Sync from source",
  "command.toggle_sidebar.label": "Toggle sidebar",
  "command.toggle_references.label": "Toggle references panel",
  "command.toggle_discovery_notes.label": "Toggle discovery notes",
  "command.toggle_editor_mode.label": "Toggle beat/page view",
  "command.detect_references.label": "Detect references in scene",
  "command.detect_all_references.label": "Detect references in all scenes",
  "command.about.label": "About kindling",
  "command.quick_start.label": "Quick start guide",
  "command.settings.label": "Settings",
  "command.new_project.label": "New project",
  "command.command_palette.label": "Command palette",
  "command.send_feedback.label": "Send feedback",
  "command.editorial_comment.label": "Add editorial comment",
  "command.bold.label": "Bold",
  "command.italic.label": "Italic",
  "command.underline.label": "Underline",
  "command.strike.label": "Strikethrough",
  "command.code.label": "Monospace",
  "command.blockquote.label": "Blockquote",
  "command.align_left.label": "Align left",
  "command.align_center.label": "Align center",
  "command.align_right.label": "Align right",
  "command.align_justify.label": "Justify",
};

export default catalog;
