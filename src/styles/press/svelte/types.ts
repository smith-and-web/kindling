export interface Option { value: string; label: string; disabled?: boolean }
export interface MenuItem { id: string; label: string; shortcut?: string; disabled?: boolean; danger?: boolean }
export interface Command extends MenuItem { keywords?: string[]; group?: string }
export interface TreeNode { id: string; label: string; description?: string; meta?: string; expanded?: boolean; disabled?: boolean; children?: TreeNode[] }
export interface SearchResult { id: string; title: string; path?: string; excerpt: string }
