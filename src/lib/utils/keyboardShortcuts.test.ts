import { describe, it, expect } from "vitest";
import definitions from "../shortcutDefinitions.json";
import {
  bindingProblem,
  defaultBindings,
  formatShortcut,
  shortcutFromEvent,
} from "./keyboardShortcuts";

describe("keyboard shortcuts", () => {
  it("has unique valid defaults and detects collisions including formatting", () => {
    for (const def of definitions)
      expect(bindingProblem(def.id, def.binding, defaultBindings)).toBeNull();
    expect(bindingProblem("settings", "Mod+B", defaultBindings)).toContain("Bold");
    expect(bindingProblem("settings", "Mod+Comma", defaultBindings)).toBeNull();
    expect(bindingProblem("settings", "", defaultBindings)).toBeNull();
    expect(bindingProblem("settings", "Mod+C", defaultBindings)).toContain("reserved");
  });
  it.each([
    [{ key: "k", code: "KeyK", metaKey: true }, true, "Mod+K"],
    [{ key: "K", code: "KeyK", ctrlKey: true, shiftKey: true }, false, "Mod+Shift+K"],
    [{ key: "µ", code: "KeyM", metaKey: true, altKey: true }, true, "Mod+Alt+M"],
    [{ key: "?", code: "Slash", ctrlKey: true, shiftKey: true }, false, "Mod+Shift+Slash"],
    [{ key: ",", metaKey: true }, true, "Mod+Comma"],
    [{ key: "7", code: "Digit7", metaKey: true, altKey: true }, true, "Mod+Alt+7"],
    [{ key: "F12", ctrlKey: true }, false, "Mod+F12"],
    [{ key: "k", ctrlKey: true }, true, null],
    [{ key: "k", metaKey: true }, false, null],
    [{ key: "k", ctrlKey: true, metaKey: true }, false, null],
    [{ key: "a" }, false, null],
    [{ key: "Tab", ctrlKey: true }, false, null],
    [{ key: "Dead", ctrlKey: true }, false, null],
    [{ key: "k", ctrlKey: true, isComposing: true }, false, null],
  ] as const)("normalizes keyboard event %j", (init, mac, expected) => {
    expect(shortcutFromEvent(new KeyboardEvent("keydown", init), mac)).toBe(expected);
  });
  it("ignores AltGraph input", () => {
    const event = new KeyboardEvent("keydown", { key: "a", ctrlKey: true });
    Object.defineProperty(event, "getModifierState", { value: () => true });
    expect(shortcutFromEvent(event, false)).toBeNull();
  });
  it("formats each platform and named punctuation", () => {
    expect(formatShortcut("Mod+Alt+Shift+Comma", true)).toBe("⌥⇧⌘,");
    expect(formatShortcut("Mod+Alt+Shift+Backslash", false)).toBe("Ctrl+Alt+Shift+\\");
    expect(formatShortcut("", true)).toBe("");
  });
});

it("uses logical AZERTY letters and punctuation rather than US key positions", () => {
  expect(
    shortcutFromEvent(
      new KeyboardEvent("keydown", { key: "m", code: "Semicolon", metaKey: true }),
      true
    )
  ).toBe("Mod+M");
  expect(
    shortcutFromEvent(new KeyboardEvent("keydown", { key: ",", code: "KeyM", metaKey: true }), true)
  ).toBe("Mod+Comma");
  expect(bindingProblem("settings", "Mod+M", defaultBindings)).toContain("reserved");
  expect(bindingProblem("settings", "Mod+Shift+V", defaultBindings)).toContain("reserved");
  expect(bindingProblem("settings", "Mod+Alt+Shift+V", defaultBindings)).toContain("reserved");
});
