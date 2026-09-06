import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { trackScrollPosition } from "./scrollPosition";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

it("ignores delayed restore events even when the browser clamps the saved offset", async () => {
  const node = document.createElement("div");
  let position = 0;
  Object.defineProperty(node, "scrollTop", {
    get: () => position,
    set: (requested: number) => {
      position = Math.min(requested, 200);
      // The event is delivered after the restore frame, as it is in a browser.
      setTimeout(() => node.dispatchEvent(new Event("scroll")), 16);
    },
  });
  const save = vi.fn();
  const tracker = trackScrollPosition(node, save);
  tracker.restore(1800);
  await vi.advanceTimersByTimeAsync(80);
  expect(node.scrollTop).toBe(200);
  expect(save).not.toHaveBeenCalled();
  node.scrollTop = 120;
  await vi.advanceTimersByTimeAsync(20);
  expect(save).toHaveBeenCalledExactlyOnceWith(120);
  node.dispatchEvent(new Event("scroll"));
  expect(save).toHaveBeenCalledTimes(1);
  tracker.destroy();
});

it("cancels superseded and unmounted restores without suppressing later user scrolling", async () => {
  const node = document.createElement("div");
  const save = vi.fn();
  const tracker = trackScrollPosition(node, save);
  const cancelOld = tracker.restore(1800);
  const complete = vi.fn();
  tracker.restore(200, complete);
  cancelOld();
  await vi.advanceTimersByTimeAsync(30);
  expect(node.scrollTop).toBe(200);
  expect(complete).toHaveBeenCalledOnce();
  tracker.restore(800);
  await vi.advanceTimersByTimeAsync(0);
  tracker.destroy();
  await vi.advanceTimersByTimeAsync(30);
  expect(node.scrollTop).toBe(200);
  node.scrollTop = 300;
  node.dispatchEvent(new Event("scroll"));
  expect(save).not.toHaveBeenCalled();
});

it("lets user input interrupt a pending restore", async () => {
  const node = document.createElement("div");
  const save = vi.fn();
  const tracker = trackScrollPosition(node, save);
  const complete = vi.fn();
  tracker.restore(1800, complete);
  node.scrollTop = 50;
  node.dispatchEvent(new Event("scroll"));
  expect(save).not.toHaveBeenCalled();
  node.dispatchEvent(new Event("wheel"));
  node.scrollTop = 100;
  node.dispatchEvent(new Event("scroll"));
  await vi.advanceTimersByTimeAsync(30);
  expect(node.scrollTop).toBe(100);
  expect(save).toHaveBeenCalledExactlyOnceWith(100);
  expect(complete).toHaveBeenCalledOnce();
  tracker.destroy();
});
