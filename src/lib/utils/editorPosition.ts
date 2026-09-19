import type { Editor } from "@tiptap/core";
import { session } from "../stores/session.svelte";
import { trackScrollPosition } from "./scrollPosition";

/** Bind an editor to the identifiers captured when it mounted, including its own scroll area. */
export function trackEditorPosition(
  editor: Editor,
  scroller: HTMLElement,
  projectId: string,
  sceneId: string,
  beatId: string | null
) {
  let disposed = false;
  const saved = session.matches(projectId, sceneId) ? session.value : null;
  const sameBeat = saved?.current_beat_id === beatId;

  if (saved && !sameBeat) {
    session.update(projectId, sceneId, {
      current_beat_id: beatId,
      cursor_position: null,
      editor_scroll_position: 0,
    });
  }

  const cursor = sameBeat ? saved?.cursor_position : null;
  if (cursor !== null && cursor !== undefined) {
    editor.commands.setTextSelection(
      Math.max(1, Math.min(cursor, editor.state.doc.content.size - 1))
    );
    editor.view.focus();
  } else if (sameBeat && beatId) {
    editor.commands.setTextSelection(1);
    editor.view.focus();
  }

  const scroll = trackScrollPosition(scroller, (position) => {
    session.update(projectId, sceneId, {
      current_beat_id: beatId,
      editor_scroll_position: position,
    });
  });
  scroll.restore(sameBeat ? (saved?.editor_scroll_position ?? 0) : 0);

  let previousCursor = editor.state.selection.head;
  const saveSelection = () => {
    if (disposed || editor.isDestroyed || !editor.isFocused) return;
    const cursor = editor.state.selection.head;
    if (cursor === previousCursor) return;
    previousCursor = cursor;
    session.update(projectId, sceneId, {
      current_beat_id: beatId,
      cursor_position: cursor,
    });
  };
  editor.on("selectionUpdate", saveSelection);
  editor.on("focus", saveSelection);
  // Edits can move the selection without a separate selectionUpdate event.
  editor.on("update", saveSelection);

  return () => {
    disposed = true;
    scroll.destroy();
    editor.off("selectionUpdate", saveSelection);
    editor.off("focus", saveSelection);
    editor.off("update", saveSelection);
  };
}
