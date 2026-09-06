import { invoke } from "@tauri-apps/api/core";
import type { Scene, SessionState } from "../types";

class SessionStore {
  value = $state<SessionState | null>(null);
  restoring = $state(false);
  ready = $state(false);
  private projectId: string | null = null;
  private generation = 0;
  private pending = new Map<string, SessionState>();
  private writing: Promise<void> | null = null;

  open(projectId: string | null) {
    this.generation++;
    this.projectId = projectId;
    this.value = null;
    this.restoring = false;
    this.ready = false;
  }

  async load(projectId: string) {
    const generation = this.generation;
    // Reopening immediately after closing must read the most recent queued save.
    await this.flush();
    try {
      const saved = await invoke<SessionState | null>("get_session_state", { projectId });
      if (generation !== this.generation || projectId !== this.projectId || this.value) return null;
      if (!saved?.current_scene_id) return null;
      this.value = saved;
      this.restoring = true;
      return saved;
    } catch (error) {
      console.error("Failed to restore writing position:", error);
      return null;
    }
  }

  selectScene(scene: Scene) {
    if (!this.projectId || this.value?.current_scene_id === scene.id) return;
    this.restoring = false;
    this.ready = false;
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
    this.value = { ...this.value!, ...changes };
    this.persist();
  }

  private persist() {
    if (!this.value) return;
    // Snapshot the identifiers now. Old component cleanup must never write into a new project.
    this.pending.set(this.value.project_id, { ...this.value });
    void this.flush();
  }

  async flush(): Promise<void> {
    while (this.writing || this.pending.size) {
      if (this.writing) {
        await this.writing;
      } else {
        this.writing = this.drain();
        await this.writing;
        this.writing = null;
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
        console.error("Failed to save writing position:", error);
      }
    }
  }
}

export const session = new SessionStore();
