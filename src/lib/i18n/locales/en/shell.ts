import type { Catalog } from "../../types";

/** App shell: quit flow, save failures, toasts and dialog chrome. */
const catalog: Catalog = {
  quitReviewTitle: "Quit without saving review changes?",
  quitWritingTitle: "Quit without saving writing changes?",
  quitSynopsisTitle: "Quit without saving synopsis changes?",
  quitReviewMessage:
    "Your review could not be saved. Keep editing to retry or export a recovery copy. Quitting and discarding removes all unsaved review, prose, and synopsis changes.",
  quitWritingMessage:
    "Some prose or synopsis changes could not be saved. Quit and discard these unsaved writing changes, or keep editing to retry saving.",
  quitSynopsisMessage:
    "Some synopsis changes could not be saved. Quit and discard these unsaved synopsis changes, or keep editing to retry saving.",
  quitAndDiscard: "Quit and discard",
  keepEditing: "Keep editing",
  couldNotQuit: "Could not quit: {error}",
  sceneUnavailable: "This scene is no longer available.",
  proseUnsaved: "Prose changes could not be saved. Review the unsaved drafts in Find and Replace.",
  synopsisUnsaved: "Your synopsis changes have not been saved.",
  retryAllSynopsisSaves: "Retry all synopsis saves",
  retrySaving: "Retry saving",
  savingShort: "Saving…",
  dismissError: "Dismiss error",
  dismiss: "Dismiss",
  closeConfirmation: "Close confirmation",
};

export default catalog;
