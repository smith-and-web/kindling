import type { Catalog } from "../../types";

/** Find and Replace dialog. */
const catalog: Catalog = {
  "findReplaceDialog.title": "Find and Replace",
  "findReplaceDialog.find": "Find",
  "findReplaceDialog.previousMatch": "Previous match (Shift+Enter)",
  "findReplaceDialog.previous": "Previous",
  "findReplaceDialog.nextMatch": "Next match (Enter)",
  "findReplaceDialog.next": "Next",
  "findReplaceDialog.loadingProse": "Loading prose…",
  "findReplaceDialog.enterQuery": "Enter text to find.",
  "findReplaceDialog.searchIn": "Search in",
  "findReplaceDialog.currentScene": "Current scene",
  "findReplaceDialog.entireProject": "Entire project",
  "findReplaceDialog.matchCase": "Match case",
  "findReplaceDialog.wholeWords": "Whole words",
  "findReplaceDialog.replace": "Replace",
  "findReplaceDialog.replaceWith": "Replace with",
  "findReplaceDialog.scopeHelp":
    "Searches visible prose in Fixed scenes. Flexible, Undefined and archived scenes are excluded; locked scenes are searchable but cannot be replaced.",
  "findReplaceDialog.position": "{current} of {total}",
  "findReplaceDialog.locked": " · Locked",
  "findReplaceDialog.unsavedDraft": " · Unsaved draft",
  "findReplaceDialog.openScene": "Open scene",
  "findReplaceDialog.noMoreMatches": "No more matches ahead. Use Next or Previous to continue.",
  "findReplaceDialog.noMatches": "No matches found.",
  "findReplaceDialog.confirmReplace":
    "Replace {count} matches in {scope}? {skipped} locked or unsaved matches will be skipped.",
  "findReplaceDialog.scopeEntireProject": "the entire project",
  "findReplaceDialog.scopeCurrentScene": "the current scene",
  "findReplaceDialog.confirmReplaceAll": "Confirm replace all",
  "findReplaceDialog.retryLoading": "Retry loading",
  "findReplaceDialog.retrySavingDrafts": "Retry saving drafts",
  "findReplaceDialog.draftsHelp":
    "Unsaved drafts are retained for this session, including after closing the project. Locked or missing documents and unrecognized save errors are not retried automatically and do not block other scenes. You can retry after resolving the save error, or copy these drafts before discarding them. Matches in documents with unsaved drafts cannot be replaced.",
  "findReplaceDialog.unsavedScene": "Unsaved scene {index}",
  "findReplaceDialog.unsavedBeat": "Unsaved beat {index}",
  "findReplaceDialog.draftText": "Draft text (select to copy)",
  "findReplaceDialog.discardQuestion":
    "Discard these unsaved drafts? Their changes will be lost. Copy any text you want to keep first.",
  "findReplaceDialog.confirmDiscardDrafts": "Confirm discard drafts",
  "findReplaceDialog.keepDrafts": "Keep drafts",
  "findReplaceDialog.discardDrafts": "Discard unsaved drafts…",
  "findReplaceDialog.undoReplacement": "Undo replacement",
  "findReplaceDialog.replaceAll": "Replace all",
  "findReplaceDialog.replaceMatch": "Replace match",
  "findReplaceDialog.replacementUndone": "Replacement undone.",
  "findReplaceDialog.replacementSaved": "Replacement saved.",
  "findReplaceDialog.noChangesNeeded": "No changes needed.",
};

export default catalog;
