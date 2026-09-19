import { afterEach, expect, it, vi } from "vitest";
import { installBackgroundFrames } from "./qaMode";
const originalRAF = window.requestAnimationFrame;
const originalCancel = window.cancelAnimationFrame;
afterEach(() => {
  delete window.__KINDLING_QA_BACKGROUND__;
  window.requestAnimationFrame = originalRAF;
  window.cancelAnimationFrame = originalCancel;
  vi.useRealTimers();
});
it("leaves native frame scheduling untouched outside isolated QA", () => {
  installBackgroundFrames();
  expect(window.requestAnimationFrame).toBe(originalRAF);
  expect(window.cancelAnimationFrame).toBe(originalCancel);
});
it("runs hidden layout callbacks in batches, supports cancellation and defers nested frames", () => {
  vi.useFakeTimers();
  window.__KINDLING_QA_BACKGROUND__ = true;
  installBackgroundFrames();
  const events: string[] = [];
  let cancelled = 0;
  window.requestAnimationFrame(() => {
    events.push("first");
    window.cancelAnimationFrame(cancelled);
    window.requestAnimationFrame(() => events.push("next"));
  });
  cancelled = window.requestAnimationFrame(() => events.push("cancelled"));
  window.requestAnimationFrame(() => events.push("second"));
  expect(events).toEqual([]);
  vi.advanceTimersByTime(16);
  expect(events).toEqual(["first", "second"]);
  vi.advanceTimersByTime(16);
  expect(events).toEqual(["first", "second", "next"]);
});
