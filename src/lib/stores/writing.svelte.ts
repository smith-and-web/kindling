import { invoke } from "@tauri-apps/api/core";
import type { WritingStats } from "../types";

export class WritingStore {
  value = $state<WritingStats | null>(null);
  error = $state<string | null>(null);
  private projectId: string | null = null;
  private request = 0;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  open(projectId: string | null) {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = null;
    this.projectId = projectId;
    this.request++;
    this.value = null;
    this.error = null;
    if (projectId) void this.refresh(projectId);
  }

  scheduleRefresh(projectId: string) {
    if (projectId !== this.projectId) return;
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = setTimeout(() => {
      this.refreshTimer = null;
      void this.refresh(projectId);
    }, 250);
  }

  async refresh(projectId = this.projectId) {
    if (!projectId || projectId !== this.projectId) return;
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = null;
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
