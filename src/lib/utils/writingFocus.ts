/** Preserve a live writing selection while an exit attempt temporarily makes it inert. */
export function captureWritingFocus(
  isCurrent: () => boolean,
  isTemporaryFocus: (target: EventTarget | null) => boolean = () => false
): (restore: boolean) => void {
  const editor = document.activeElement;
  if (
    !(editor instanceof HTMLElement) ||
    !editor.closest('[data-testid="scene-panel"]') ||
    !(editor instanceof HTMLTextAreaElement || editor.isContentEditable)
  )
    return () => {};

  const textarea = editor instanceof HTMLTextAreaElement ? editor : null;
  const selection = textarea
    ? {
        start: textarea.selectionStart,
        end: textarea.selectionEnd,
        direction: textarea.selectionDirection,
      }
    : null;
  const domSelection = window.getSelection();
  const range =
    !textarea && domSelection?.rangeCount ? domSelection.getRangeAt(0).cloneRange() : null;
  let movedFocus = false;
  const trackFocus = (event: FocusEvent) => {
    if (
      event.target !== editor &&
      event.target !== document.body &&
      !isTemporaryFocus(event.target)
    )
      movedFocus = true;
  };
  document.addEventListener("focusin", trackFocus, true);

  return (restore) => {
    document.removeEventListener("focusin", trackFocus, true);
    if (!restore || movedFocus || !isCurrent() || !editor.isConnected || editor.closest("[inert]"))
      return;
    if (document.activeElement !== document.body && document.activeElement !== editor) return;
    editor.focus({ preventScroll: true });
    if (textarea && selection) {
      textarea.setSelectionRange(selection.start, selection.end, selection.direction);
    } else if (
      range &&
      editor.contains(range.startContainer) &&
      editor.contains(range.endContainer)
    ) {
      const current = window.getSelection();
      current?.removeAllRanges();
      current?.addRange(range);
    }
  };
}
