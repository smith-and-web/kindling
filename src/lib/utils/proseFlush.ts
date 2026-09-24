import { proseSaves } from "./proseSaves";

let flushEditors: (() => Promise<void>) | null = null;

/** The writing surface registers how to push its debounced edits into the save queue. */
export function registerProseFlush(flush: (() => Promise<void>) | null): void {
  flushEditors = flush;
}

/**
 * Saves pending prose before something reads it back from the database (export,
 * sync). Throws, like opening Revisions does, if any draft is still unsaved, so
 * the caller stops instead of using stale text.
 */
export async function saveProseBefore(projectId: string, action: string): Promise<void> {
  await flushEditors?.();
  await proseSaves.flush(projectId);
  if (proseSaves.draftsForRecovery(projectId).length) {
    throw new Error(
      `Save or recover unsaved prose before ${action}. Review the unsaved drafts in Find and Replace.`
    );
  }
}
