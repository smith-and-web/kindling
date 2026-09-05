import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { platform } from "@tauri-apps/plugin-os";
import { ui } from "$lib/stores/ui.svelte";
import type { Project } from "$lib/types";

import { IMPORT_FORMATS } from "../importFormats";
import type { ImportType } from "../importFormats";
export type { ImportType } from "../importFormats";

/**
 * Opens a file dialog, invokes the backend import command, and returns the
 * resulting Project. Returns `null` if the user cancels the dialog.
 * Handles import progress and error toasts automatically.
 */
function basename(p: string): string {
  const normalized = p.replace(/\\/g, "/");
  const i = normalized.lastIndexOf("/");
  return i === -1 ? normalized : normalized.slice(i + 1);
}

/**
 * Pick a `.scriv` bundle: file picker on macOS (package), folder picker elsewhere
 * where `.scriv` is a directory.
 */
export async function pickScrivenerProjectPath(): Promise<string | null> {
  const isMacos = platform() === "macos";
  const path = await open({
    multiple: false,
    title: isMacos ? undefined : "Select Scrivener project folder (.scriv)",
    ...(isMacos ? { filters: IMPORT_FORMATS.scrivener.filters } : { directory: true }),
  });
  if (!path) return null;
  if (!isMacos) {
    const name = basename(path).toLowerCase();
    if (!name.endsWith(".scriv")) {
      ui.showError("Please select a folder whose name ends with .scriv");
      return null;
    }
  }
  return path;
}

export async function runImport(type: ImportType): Promise<Project | null> {
  const config = IMPORT_FORMATS[type];
  if (!config) throw new Error(`Unknown import type: ${type}`);

  const path =
    type === "scrivener"
      ? await pickScrivenerProjectPath()
      : await open({
          multiple: false,
          ...(config.directory ? { directory: true } : { filters: config.filters }),
        });

  if (!path) return null;

  ui.startImport();
  try {
    return await invoke<Project>(config.command, { path });
  } catch (e) {
    console.error(`Failed to import ${config.label}:`, e);
    ui.showError(`Import failed: ${e}`);
    return null;
  } finally {
    ui.finishImport();
  }
}
