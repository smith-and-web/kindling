import { afterEach, expect, it, vi } from "vitest";
import { dragWithMouseEvents } from "../../e2e/specs/helpers.js";

afterEach(() => vi.unstubAllGlobals());

function setup({ gestureError, assertionError, pointerUpError, cleanupError } = {}) {
  const pointerUp = vi.fn(async () => {
    if (pointerUpError) throw pointerUpError;
  });
  const releaseActions = vi.fn(async () => {
    if (cleanupError) throw cleanupError;
  });
  const waitUntil = vi.fn(async (check) => expect(await check()).toBe(true));
  const browser = {
    execute: vi
      .fn()
      .mockResolvedValueOnce(["source", "target"])
      .mockResolvedValue(["target", "source"]),
    performActions: vi.fn(async ([pointer]) => {
      if (pointer.actions[0].type === "pointerUp") return pointerUp();
      if (gestureError) throw gestureError;
    }),
    releaseActions,
    waitUntil,
  };
  vi.stubGlobal("browser", browser);
  const element = (id) => ({
    getAttribute: async () => id,
    getLocation: async () => ({ x: 10, y: 20 }),
    getSize: async () => ({ width: 20, height: 20 }),
    $: async () => element(id),
  });
  const whileDragging = vi.fn(async () => {
    if (assertionError) throw assertionError;
  });
  return {
    run: () => dragWithMouseEvents(element("source"), element("target"), whileDragging),
    pointerUp,
    releaseActions,
    waitUntil,
    whileDragging,
  };
}

it.each(["gestureError", "assertionError", "pointerUpError", "cleanupError"])(
  "preserves the original %s when later cleanup also fails",
  async (first) => {
    const original = new Error(first);
    const options = {
      pointerUpError: new Error("Pointer release failed"),
      cleanupError: new Error("Cleanup failed"),
      [first]: original,
    };
    if (first === "cleanupError") delete options.pointerUpError;
    const test = setup(options);
    await expect(test.run()).rejects.toBe(original);
    expect(test.pointerUp).toHaveBeenCalledOnce();
    expect(test.releaseActions).toHaveBeenCalledOnce();
    expect(test.waitUntil).not.toHaveBeenCalled();
  }
);

it("releases the drag and verifies the order when no step fails", async () => {
  const test = setup();
  await test.run();
  expect(test.whileDragging).toHaveBeenCalledOnce();
  expect(test.pointerUp).toHaveBeenCalledOnce();
  expect(test.releaseActions).toHaveBeenCalledOnce();
  expect(test.waitUntil).toHaveBeenCalledOnce();
});
