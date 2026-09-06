import { invoke } from "@tauri-apps/api/core";
import type { Scene, SessionState } from "../types";

class SessionStore {
  value = $state<SessionState | null>(null);
  viewport = $state.raw<{ projectId: string; sceneId: string; position: number } | null>(null);
  private projectId: string | null = null;
  private generation = 0;
  private pending = new Map<string, SessionState>();
  private writing: Promise<void> | null = null;
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

  open(projectId: string | null) {
    void this.flush().catch((error) => console.error("Failed to save writing position:", error));
    this.generation++;
    this.projectId = projectId;
    this.value = null;
    this.viewport = null;
  }

  async load(projectId: string) {
    const generation = this.generation;
    // Reopening immediately after closing must read the most recent queued save.
    try {
      await this.flush();
      const saved = await invoke<SessionState | null>("get_session_state", { projectId });
      if (generation !== this.generation || projectId !== this.projectId || this.value) return null;
      if (!saved?.current_scene_id) return null;
      this.value = saved;
      return saved;
    } catch (error) {
      console.error("Failed to restore writing position:", error);
      return null;
    }
  }

  selectScene(scene: Scene) {
    if (!this.projectId || this.value?.current_scene_id === scene.id) return;
    this.viewport = null;
    this.value = {
      project_id: this.projectId,
      current_chapter_id: scene.chapter_id,
      current_scene_id: scene.id,
      current_beat_id: null,
      cursor_position: null,
      scroll_position: 0,
      editor_scroll_position: 0,
      last_opened_at: null,
    };
    this.persist();
  }

  matches(projectId: string, sceneId: string) {
    return this.value?.project_id === projectId && this.value.current_scene_id === sceneId;
  }

  // Only a navigation request that finished loading its scene and beats publishes a viewport.
  // This is an immutable request, not a flag that can disable future position saving.
  restoreViewport(projectId: string, sceneId: string, position: number) {
    if (this.matches(projectId, sceneId)) this.viewport = { projectId, sceneId, position };
  }

  update(
    projectId: string,
    sceneId: string,
    changes: Partial<
      Pick<
        SessionState,
        "current_beat_id" | "cursor_position" | "scroll_position" | "editor_scroll_position"
      >
    >
  ) {
    if (!this.matches(projectId, sceneId)) return;
    if (
      Object.entries(changes).every(
        ([key, value]) => this.value![key as keyof SessionState] === value
      )
    )
      return;
    this.value = { ...this.value!, ...changes };
    this.persist();
  }

  private persist() {
    if (!this.value) return;
    // Snapshot the identifiers now. Old component cleanup must never write into a new project.
    this.pending.set(this.value.project_id, { ...this.value });
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => {
      void this.flush().catch((error) => console.error("Failed to save writing position:", error));
    }, 500);
  }

  async flush(): Promise<void> {
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = null;
    while (this.writing || this.pending.size) {
      if (this.writing) {
        await this.writing;
      } else {
        this.writing = this.drain();
        try {
          await this.writing;
        } finally {
          this.writing = null;
        }
      }
      // Every caller also waits for updates queued while an earlier write was finishing.
    }
  }

  private async drain() {
    for (const [projectId, session] of this.pending) {
      this.pending.delete(projectId);
      try {
        await invoke("save_session_state", { session });
      } catch (error) {
        // Keep the newest snapshot available for a later retry, without retrying in a loop.
        if (!this.pending.has(projectId)) this.pending.set(projectId, session);
        throw error;
      }
    }
  }
}

export const session = new SessionStore();
