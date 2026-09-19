declare global {
  interface Window {
    __KINDLING_QA_BACKGROUND__?: boolean;
  }
}

export const backgroundQA = () => import.meta.env.DEV && window.__KINDLING_QA_BACKGROUND__ === true;

// Hidden WebKit windows may suspend native rAF even with timer throttling off.
// Keep the application's layout callbacks running in the isolated QA process.
// This supplies scheduling, not presented frames: WKWebView snapshotting still
// performs the actual rendering. Normal and production apps retain native rAF.
export function installBackgroundFrames() {
  if (!backgroundQA()) return;
  let nextId = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const callbacks = new Map<number, FrameRequestCallback>();
  const schedule = () => {
    if (timer !== undefined) return;
    timer = setTimeout(() => {
      timer = undefined;
      const frame = [...callbacks.keys()];
      const now = performance.now();
      for (const id of frame) {
        const callback = callbacks.get(id);
        callbacks.delete(id);
        if (callback) {
          try {
            callback(now);
          } catch (error) {
            // One failing callback must not swallow the rest of the frame.
            setTimeout(() => {
              throw error;
            }, 0);
          }
        }
      }
    }, 16);
  };
  window.requestAnimationFrame = (callback) => {
    const id = ++nextId;
    callbacks.set(id, callback);
    schedule();
    return id;
  };
  window.cancelAnimationFrame = (id) => {
    callbacks.delete(id);
  };
}
