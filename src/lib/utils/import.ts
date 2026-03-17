import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ui } from "$lib/stores/ui.svelte";
import type { Project } from "$lib/types";

interface FileFilter {
  name: string;
  extensions: string[];
}

interface ImportOptions {
  command: string;
  filters?: FileFilter[];
  directory?: boolean;
  label: string;
}

const IMPORT_CONFIGS: Record<string, ImportOptions> = {
  plottr: {
    command: "import_plottr",
    filters: [{ name: "Plottr", extensions: ["pltr"] }],
    label: "Plottr file",
  },
  markdown: {
    command: "import_markdown",
    filters: [{ name: "Markdown", extensions: ["md", "markdown"] }],
    label: "Markdown file",
  },
  ywriter: {
    command: "import_ywriter",
    filters: [{ name: "yWriter 7", extensions: ["yw7"] }],
    label: "yWriter file",
  },
  longform: {
    command: "import_longform",
    filters: [{ name: "Longform Index", extensions: ["md", "markdown"] }],
    label: "Longform index",
  },
  longformVault: {
    command: "import_longform",
    directory: true,
    label: "Longform vault",
  },
  scrivener: {
    command: "import_scrivener",
    directory: true,
    label: "Scrivener project",
  },
};

export type ImportType = keyof typeof IMPORT_CONFIGS;

/**
 * Opens a file dialog, invokes the backend import command, and returns the
 * resulting Project. Returns `null` if the user cancels the dialog.
 * Handles import progress and error toasts automatically.
 */
export async function runImport(type: ImportType): Promise<Project | null> {
  const config = IMPORT_CONFIGS[type];
  if (!config) throw new Error(`Unknown import type: ${type}`);

  const path = await open({
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
