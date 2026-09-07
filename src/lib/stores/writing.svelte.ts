import { invoke } from "@tauri-apps/api/core";
import type { WritingStats } from "../types";

export class WritingStore {
  value = $state<WritingStats | null>(null);
  error = $state<string | null>(null);
  private projectId: string | null = null;
  private request = 0;

  open(projectId: string | null) {
    this.projectId = projectId;
    this.request++;
    this.value = null;
    this.error = null;
    if (projectId) void this.refresh(projectId);
  }

  async refresh(projectId = this.projectId) {
    if (!projectId || projectId !== this.projectId) return;
    const request = ++this.request;
    try {
      const stats = await invoke<WritingStats>("get_writing_stats", { projectId });
      if (request !== this.request) return;
      this.value = stats;
      this.error = null;
    } catch (error) {
      if (request === this.request)
        this.error = `Could not load writing statistics: ${String(error)}`;
    }
  }

  async reset(projectId: string) {
    try {
      await invoke("reset_writing_session", { projectId });
      await this.refresh(projectId);
    } catch (error) {
      if (projectId === this.projectId) this.error = `Could not reset session: ${String(error)}`;
    }
  }
}

export const writing = new WritingStore();
