/** Track a viewport independently of the navigation request that restores it. */
export function trackScrollPosition(node: HTMLElement, onChange: (position: number) => void) {
  let previous = node.scrollTop;
  let cancelPending: (() => void) | undefined;
  let interruptPending: (() => void) | undefined;

  function set(position: number) {
    node.scrollTop = position;
    // Read back the actual (possibly clamped) offset. Scroll events are asynchronous,
    // so the requested offset and a one-frame suppression flag are both insufficient.
    previous = node.scrollTop;
  }

  const onScroll = () => {
    const position = node.scrollTop;
    if (position === previous) return;
    previous = position;
    if (!cancelPending) onChange(position);
  };
  const onInteraction = () => interruptPending?.();
  node.addEventListener("scroll", onScroll);
  node.addEventListener("wheel", onInteraction, { passive: true });
  node.addEventListener("pointerdown", onInteraction);
  node.addEventListener("keydown", onInteraction);

  function restore(position: number, onComplete?: () => void) {
    cancelPending?.();
    let cancelled = false;
    let frame = 0;
    const cancel = () => {
      cancelled = true;
      cancelAnimationFrame(frame);
      if (cancelPending === cancel) {
        cancelPending = undefined;
        interruptPending = undefined;
      }
    };
    cancelPending = cancel;
    interruptPending = () => {
      cancel();
      // A user's navigation consumes the request, so a later layout refresh cannot replay it.
      onComplete?.();
    };
    // Mount/update DOM work finishes before this frame. Local fonts can change its height.
    void Promise.resolve(document.fonts?.ready)
      .catch(() => {})
      .then(() => {
        if (cancelled) return;
        frame = requestAnimationFrame(() => {
          set(position);
          cancelPending = undefined;
          interruptPending = undefined;
          onComplete?.();
        });
      });
    return cancel;
  }

  return {
    set,
    restore,
    destroy() {
      cancelPending?.();
      node.removeEventListener("scroll", onScroll);
      node.removeEventListener("wheel", onInteraction);
      node.removeEventListener("pointerdown", onInteraction);
      node.removeEventListener("keydown", onInteraction);
    },
  };
}
