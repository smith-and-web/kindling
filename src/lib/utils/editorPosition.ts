import type { Editor } from "@tiptap/core";
import { session } from "../stores/session.svelte";

/** Bind an editor to the identifiers captured when it mounted, including its own scroll area. */
export function trackEditorPosition(
  editor: Editor,
  scroller: HTMLElement,
  projectId: string,
  sceneId: string,
  beatId: string | null
) {
  let restoring = true;
  let disposed = false;
  let frame = 0;
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
  } else if (sameBeat && beatId && session.restoring) {
    editor.commands.setTextSelection(1);
    editor.view.focus();
  }

  // Apply after layout/fonts; focusing must not scroll away from the saved viewport.
  void Promise.resolve(document.fonts?.ready).then(() => {
    if (disposed) return;
    frame = requestAnimationFrame(() => {
      scroller.scrollTop = sameBeat ? (saved?.editor_scroll_position ?? 0) : 0;
      restoring = false;
    });
  });

  const save = () => {
    if (restoring || disposed || editor.isDestroyed) return;
    session.update(projectId, sceneId, {
      current_beat_id: beatId,
      cursor_position: editor.state.selection.head,
      editor_scroll_position: scroller.scrollTop,
    });
  };
  const saveSelection = () => {
    if (editor.isFocused) save();
  };
  editor.on("selectionUpdate", saveSelection);
  editor.on("focus", saveSelection);
  // Edits can move the selection without a separate selectionUpdate event.
  editor.on("update", saveSelection);
  scroller.addEventListener("scroll", save);

  return () => {
    disposed = true;
    cancelAnimationFrame(frame);
    editor.off("selectionUpdate", saveSelection);
    editor.off("focus", saveSelection);
    editor.off("update", saveSelection);
    scroller.removeEventListener("scroll", save);
  };
}
