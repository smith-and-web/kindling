/**
 * Auto-updater: checks GitHub Releases on launch, downloads in background,
 * shows banner when ready. User clicks Restart to install and relaunch.
 */

import { writable } from "svelte/store";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateState {
  /** Update is downloaded and ready to install */
  ready: boolean;
  /** Version string of the available update */
  version: string;
  /** Release notes (optional) */
  body: string | null;
  /** The update object for install/relaunch */
  update: Awaited<ReturnType<typeof check>>;
}

export const updateState = writable<UpdateState | null>(null);

export async function checkForUpdate(): Promise<void> {
  if (import.meta.env.DEV) return;

  try {
    const update = await check();
    if (!update) return;

    await update.download();
    updateState.set({
      ready: true,
      version: update.version,
      body: update.body ?? null,
      update,
    });
  } catch (e) {
    console.warn("Update check failed:", e);
  }
}

export async function installAndRelaunch(state: UpdateState): Promise<void> {
  if (!state.update) return;
  try {
    await state.update.install();
    await relaunch();
  } catch (e) {
    console.error("Failed to install update:", e);
  }
}

export function dismissUpdate(): void {
  updateState.set(null);
}
