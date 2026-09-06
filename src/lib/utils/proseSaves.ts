import { invoke } from "@tauri-apps/api/core";

export type ProseSave = { projectId: string; kind: "beat" | "page"; id: string; prose: string };

/** Only known storage failures may block work for an automatic retry.
 * Unknown rejections remain recoverable, but cannot poison the project-wide queue.
 */
function isTerminal(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return !/^(database is (locked|busy)|database table is locked|database or disk is full|disk full|disk I\/O error|unable to open database file)$/i.test(
    message
  );
}

/** Retry transient failures; retain terminal drafts separately without blocking other work. */
export class ProseSaveQueue {
  private pending = new Map<string, ProseSave>();
  private recovery = new Map<string, { draft: ProseSave; error: unknown }>();
  private queue: Promise<void> = Promise.resolve();

  save(save: ProseSave, retryRecovered = false): Promise<void> {
    const recovered = this.recovery.get(save.id);
    if (
      !retryRecovered &&
      recovered?.draft.prose === save.prose &&
      recovered.draft.projectId === save.projectId &&
      recovered.draft.kind === save.kind
    ) {
      return Promise.reject(recovered.error);
    }
    this.recovery.delete(save.id);
    this.pending.set(save.id, save);
    const writing = this.queue.then(async () => {
      try {
        await invoke(save.kind === "beat" ? "save_beat_prose" : "save_scene_page_prose", {
          [save.kind === "beat" ? "beatId" : "sceneId"]: save.id,
          prose: save.prose,
        });
        if (this.pending.get(save.id) === save) this.pending.delete(save.id);
      } catch (error) {
        if (isTerminal(error) && this.pending.get(save.id) === save) {
          this.pending.delete(save.id);
          this.recovery.set(save.id, { draft: save, error });
        }
        throw error;
      }
    });
    this.queue = writing.catch(() => {});
    return writing;
  }

  pendingFor(projectId: string): ProseSave[] {
    return [...this.pending.values()].filter((save) => save.projectId === projectId);
  }

  draftsForRecovery(projectId: string): ProseSave[] {
    return [...this.recovery.values()]
      .map((entry) => entry.draft)
      .filter((save) => save.projectId === projectId)
      .concat(this.pendingFor(projectId));
  }

  /** Only a user-requested retry may resubmit a terminal draft (e.g. after unlocking). */
  async retryRecovered(projectId: string): Promise<ProseSave[]> {
    await this.queue;
    const saved: ProseSave[] = [];
    for (const { draft } of [...this.recovery.values()]) {
      if (draft.projectId !== projectId) continue;
      try {
        await this.save(draft, true);
        saved.push(draft);
      } catch {
        /* The appropriate queue retains the draft. */
      }
    }
    return saved;
  }

  async discard(drafts: ProseSave[], onDiscarded?: () => void): Promise<void> {
    await this.queue;
    if (
      drafts.some(
        (draft) => (this.pending.get(draft.id) ?? this.recovery.get(draft.id)?.draft) !== draft
      )
    ) {
      throw new Error("Unsaved drafts changed. Retry loading before discarding them.");
    }
    for (const draft of drafts) {
      this.pending.delete(draft.id);
      this.recovery.delete(draft.id);
    }
    // Clear editor copies in the same turn, before they can be submitted again.
    onDiscarded?.();
  }

  async flush(projectId: string, onSaved?: (save: ProseSave) => void): Promise<ProseSave[]> {
    await this.queue;
    const saved: ProseSave[] = [];
    const errors: string[] = [];
    for (const save of this.pendingFor(projectId)) {
      try {
        await this.save(save);
      } catch (e) {
        errors.push(String(e));
        continue;
      }
      saved.push(save);
      // Publish each success before another failure can reject the overall flush.
      onSaved?.(save);
    }
    if (this.pendingFor(projectId).length) {
      throw new Error(
        `Could not save the latest prose. ${errors.join("; ")} Retry saving or review the unsaved drafts in Find and Replace.`
      );
    }
    return saved;
  }
}

export const proseSaves = new ProseSaveQueue();
