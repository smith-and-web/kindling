import { invoke } from "@tauri-apps/api/core";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { currentProject } from "./project.svelte";

export type SynopsisDraft = { projectId: string; sceneId: string; synopsis: string | null };

/** Drafts outlive scene panels. Only successful writes retire them. */
export class SynopsisSaveQueue {
  private drafts = new SvelteMap<string, SynopsisDraft>();
  private errors = new SvelteMap<string, string>();
  private saving = new SvelteSet<string>();
  private timers = new SvelteMap<string, ReturnType<typeof setTimeout>>();
  private queue: Promise<void> = Promise.resolve();

  private key(projectId: string, sceneId: string) {
    return JSON.stringify([projectId, sceneId]);
  }

  get failedCount() {
    return this.errors.size;
  }

  getState(projectId: string | undefined, sceneId: string | undefined) {
    const key = this.key(projectId ?? "", sceneId ?? "");
    return {
      draft: this.drafts.get(key),
      error: this.errors.get(key),
      saving: this.saving.has(key),
    };
  }

  stage(draft: SynopsisDraft) {
    const key = this.key(draft.projectId, draft.sceneId);
    this.drafts.set(key, draft);
    this.errors.delete(key);
    this.clearTimer(key);
    this.timers.set(
      key,
      setTimeout(() => {
        // Errors remain visible and retryable; never discard a rejected draft.
        void this.flush(draft.projectId, draft.sceneId).catch(() => {});
      }, 1000)
    );
  }

  private clearTimer(key: string) {
    const timer = this.timers.get(key);
    if (timer) clearTimeout(timer);
    this.timers.delete(key);
  }

  /** Drain the latest edits, including edits made during a write; retry failures once per call. */
  flush(projectId?: string, sceneId?: string): Promise<void> {
    const matches = (draft: SynopsisDraft) =>
      (projectId === undefined || draft.projectId === projectId) &&
      (sceneId === undefined || draft.sceneId === sceneId);
    const writing = this.queue.then(async () => {
      const attempted = new SvelteSet<SynopsisDraft>();
      let next: [string, SynopsisDraft] | undefined;
      while (
        (next = [...this.drafts].find(([, draft]) => matches(draft) && !attempted.has(draft)))
      ) {
        const [key, draft] = next;
        attempted.add(draft);
        this.clearTimer(key);
        this.saving.add(key);
        try {
          await invoke("save_scene_synopsis", { sceneId: draft.sceneId, synopsis: draft.synopsis });
          if (currentProject.value?.id === draft.projectId) {
            currentProject.updateSceneSynopsis(draft.sceneId, draft.synopsis);
          }
          if (this.drafts.get(key) === draft) {
            this.drafts.delete(key);
            this.errors.delete(key);
          }
        } catch (error) {
          if (this.drafts.get(key) === draft) {
            this.errors.set(key, error instanceof Error ? error.message : String(error));
          }
        } finally {
          this.saving.delete(key);
        }
      }
      const failures = [...this.drafts].filter(([, draft]) => matches(draft));
      if (failures.length) {
        throw new Error(
          `Synopsis not saved: ${failures.map(([key]) => this.errors.get(key)).join("; ")}`
        );
      }
    });
    this.queue = writing.catch(() => {});
    return writing;
  }
}

export const synopsisSaves = new SynopsisSaveQueue();
