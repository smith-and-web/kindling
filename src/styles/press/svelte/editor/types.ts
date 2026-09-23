import type { Editor } from '@tiptap/core';

export interface ShortcutAdapter {
  match(event: KeyboardEvent): string | null;
  label(command: string): string;
}
export interface EditorAttachment {
  editor: Editor;
  scroller: HTMLElement;
  projectId?: string;
  sceneId?: string;
  beatId: string | null;
}
export type SaveStatus = 'idle' | 'saving' | 'error';

