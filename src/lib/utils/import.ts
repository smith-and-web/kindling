import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { platform } from "@tauri-apps/plugin-os";
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
    filters: [{ name: "Scrivener Project", extensions: ["scriv"] }],
    label: "Scrivener project",
  },
};

export type ImportType = keyof typeof IMPORT_CONFIGS;

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
    ...(isMacos ? { filters: IMPORT_CONFIGS.scrivener.filters } : { directory: true }),
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
  const config = IMPORT_CONFIGS[type];
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
