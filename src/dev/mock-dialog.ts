/**
 * Mock @tauri-apps/plugin-dialog for browser-only dev.
 * Returns fake paths immediately so import/export flows complete without native dialogs.
 */

interface DialogFilter {
  name: string;
  extensions: string[];
}

interface OpenDialogOptions {
  filters?: DialogFilter[];
  multiple?: boolean;
  directory?: boolean;
}

function pickExtension(filters?: DialogFilter[]): string {
  if (filters?.length && filters[0]?.extensions?.length) {
    return filters[0].extensions[0]!;
  }
  return "pltr";
}

export async function open(options?: OpenDialogOptions): Promise<string | string[] | null> {
  const ext = pickExtension(options?.filters);
  const path = options?.directory ? "/mock/path/longform-vault" : `/mock/path/story.${ext}`;
  console.debug("[mock-dialog] open() ->", path);
  return options?.multiple ? [path] : path;
}

export async function save(): Promise<string | null> {
  const path = "/mock/path/export/output.docx";
  console.debug("[mock-dialog] save() ->", path);
  return path;
}

export async function message(): Promise<string> {
  console.debug("[mock-dialog] message() (no-op)");
  return "Ok";
}

export async function ask(): Promise<boolean> {
  console.debug("[mock-dialog] ask() (no-op, returns true)");
  return true;
}

export async function confirm(): Promise<boolean> {
  console.debug("[mock-dialog] confirm() (no-op, returns true)");
  return true;
}
