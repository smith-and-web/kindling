/**
 * Command registry for the command palette (US-1.1-5)
 *
 * Each command has an id, label, shortcut, category, and optional keywords for search.
 * Shortcuts use format "Mod+Key" where Mod = Cmd on Mac, Ctrl on Windows/Linux.
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

export const COMMAND_DEFS: CommandDef[] = [
  // File
  {
    id: "export",
    label: "Export project",
    shortcut: "⌘E",
    category: "File",
    keywords: ["export", "docx", "manuscript"],
    requiresProject: true,
  },
  {
    id: "close_project",
    label: "Close project",
    shortcut: "⌘W",
    category: "File",
    keywords: ["close"],
    requiresProject: true,
  },
  {
    id: "quit",
    label: "Quit Kindling",
    shortcut: "⌘Q",
    category: "File",
    keywords: ["quit", "exit"],
    requiresProject: false,
  },
  {
    id: "import_plottr",
    label: "Import Plottr (.pltr)",
    shortcut: "⌘⇧O",
    category: "File",
    keywords: ["import", "plottr"],
    requiresProject: false,
  },
  {
    id: "import_markdown",
    label: "Import Markdown (.md)",
    shortcut: "⌘⇧M",
    category: "File",
    keywords: ["import", "markdown", "md"],
    requiresProject: false,
  },
  {
    id: "import_longform",
    label: "Import Longform",
    shortcut: "⌘⇧L",
    category: "File",
    keywords: ["import", "longform", "obsidian"],
    requiresProject: false,
  },
  {
    id: "import_ywriter",
    label: "Import yWriter 7 (.yw7)",
    shortcut: "⌘⇧Y",
    category: "File",
    keywords: ["import", "ywriter", "yw7"],
    requiresProject: false,
  },
  // Project
  {
    id: "project_settings",
    label: "Project settings",
    shortcut: "⌘⇧P",
    category: "Project",
    keywords: ["settings", "project", "configure"],
    requiresProject: true,
  },
  {
    id: "sync",
    label: "Sync from source",
    shortcut: "⌘⇧S",
    category: "Project",
    keywords: ["sync", "reimport", "refresh", "reload"],
    requiresProject: true,
    requiresSourcePath: true,
  },
  // View
  {
    id: "toggle_sidebar",
    label: "Toggle sidebar",
    shortcut: "⌘\\",
    category: "View",
    keywords: ["sidebar", "outline", "collapse", "expand"],
    requiresProject: true,
  },
  {
    id: "toggle_references",
    label: "Toggle references panel",
    shortcut: "⌘⇧R",
    category: "View",
    keywords: ["references", "characters", "panel", "collapse", "expand"],
    requiresProject: true,
  },
  {
    id: "toggle_discovery_notes",
    label: "Toggle discovery notes",
    shortcut: "⌘D",
    category: "View",
    keywords: ["discovery", "notes", "draft"],
    requiresProject: true,
  },
  // Help
  {
    id: "about",
    label: "About Kindling",
    shortcut: "",
    category: "Help",
    keywords: ["about", "version", "info"],
    requiresProject: false,
  },
  {
    id: "quick_start",
    label: "Quick start guide",
    shortcut: "⌘⇧H",
    category: "Help",
    keywords: ["help", "quick", "start", "guide", "docs"],
    requiresProject: false,
  },
  {
    id: "kindling_settings",
    label: "Kindling settings",
    shortcut: "⌘,",
    category: "Help",
    keywords: ["settings", "kindling", "preferences", "author"],
    requiresProject: false,
  },
];

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
