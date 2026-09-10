import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { Shortcuts } from "./shortcuts.svelte";
import { defaultBindings } from "../utils/keyboardShortcuts";
beforeEach(() => vi.mocked(invoke).mockReset());
it("loads once, applies saved bindings, and reloads them in a new session", async () => {
  const saved = { ...defaultBindings, command_palette: "Mod+Alt+K" };
  vi.mocked(invoke).mockResolvedValue(saved);
  const store = new Shortcuts();
  await Promise.all([store.load(), store.load()]);
  await store.load();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(store.ready).toBe(true);
  expect(store.bindings.command_palette).toBe("Mod+Alt+K");
  await store.save(saved);
  const next = new Shortcuts();
  await next.load();
  expect(next.bindings).toEqual(saved);
  expect(next.label("missing")).toBe("");
  expect(next.label("command_palette")).toContain("K");
  const mod = /Mac/.test(navigator.platform) ? { metaKey: true } : { ctrlKey: true };
  expect(next.match(new KeyboardEvent("keydown", { key: "k", altKey: true, ...mod }))).toBe(
    "command_palette"
  );
  expect(next.match(new KeyboardEvent("keydown", { key: "k", ...mod }))).toBeUndefined();
  expect(next.match(new KeyboardEvent("keydown", { key: "k" }))).toBeUndefined();
});
it("reports load failures and retries; failed saves preserve the active bindings", async () => {
  const store = new Shortcuts();
  vi.mocked(invoke).mockRejectedValue(new Error("disk unavailable"));
  await store.load();
  expect(store.error).toContain("disk unavailable");
  expect(store.ready).toBe(false);
  await expect(store.save({ ...defaultBindings, quit: "" })).rejects.toThrow("disk unavailable");
  expect(store.bindings.quit).toBe(defaultBindings.quit);
  vi.mocked(invoke).mockResolvedValue(defaultBindings);
  await store.load();
  expect(store.ready).toBe(true);
  expect(store.error).toBeNull();
});
it("serializes menu suspension and reports failures without breaking subsequent calls", async () => {
  const store = new Shortcuts();
  vi.mocked(invoke).mockRejectedValueOnce("menu failed");
  await store.suspend(true);
  expect(store.error).toContain("menu failed");
  expect(store.suspended).toBe(false);
  vi.mocked(invoke).mockResolvedValue(undefined);
  store.ready = true;
  await Promise.all([store.suspend(true), store.suspend(false)]);
  expect(store.suspended).toBe(false);
  expect(store.error).toBeNull();
  expect(vi.mocked(invoke).mock.calls.slice(-2)).toEqual([
    ["suspend_keyboard_shortcuts", { suspended: true }],
    ["suspend_keyboard_shortcuts", { suspended: false }],
  ]);
});
