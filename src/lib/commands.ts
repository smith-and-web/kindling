/**
 * Command registry for the command palette (US-1.1-5)
 *
 * Each command has an id, label, shortcut, category, and optional keywords for search.
 * Shortcuts here are formatted defaults; the shortcut store supplies live bindings.
 * Actions are bound at runtime by App.svelte.
 */

export type CommandCategory = "File" | "View" | "Edit" | "Help" | "Project";

export interface CommandDef {
  id: string;
  label: string;
  shortcut: string;
  category: CommandCategory;
  /** Extra keywords for fuzzy search (e.g. "sync", "reimport") */
  keywords?: string[];
  /** Only show when project is open */
  requiresProject?: boolean;
  /** Only show when project has a source path (for sync) */
  requiresSourcePath?: boolean;
}

import definitions from "./shortcutDefinitions.json";
import { formatShortcut } from "./utils/keyboardShortcuts";

export const COMMAND_DEFS: CommandDef[] = definitions
  .filter((def) => def.category !== "Editor")
  .map((def) => ({
    ...def,
    category: def.category as CommandCategory,
    shortcut: formatShortcut(def.binding),
  }));

/** Simple fuzzy match: query chars must appear in order in the text */
export function fuzzyMatch(query: string, text: string): boolean {
  const q = query.toLowerCase().trim();
  if (!q) return true;
  const t = text.toLowerCase();
  let ti = 0;
  for (let i = 0; i < q.length; i++) {
    const idx = t.indexOf(q[i], ti);
    if (idx === -1) return false;
    ti = idx + 1;
  }
  return true;
}

/** Score for ranking: prefer matches at word boundaries and earlier in string */
export function fuzzyScore(query: string, text: string): number {
  const q = query.toLowerCase().trim();
  if (!q) return 0;
  const t = text.toLowerCase();
  let score = 0;
  let ti = 0;
  for (let i = 0; i < q.length; i++) {
    const idx = t.indexOf(q[i], ti);
    if (idx === -1) return -1;
    score += 100 - idx; // Earlier matches score higher
    if (idx === 0 || t[idx - 1] === " " || t[idx - 1] === "-") {
      score += 50; // Word boundary bonus
    }
    ti = idx + 1;
  }
  return score;
}
