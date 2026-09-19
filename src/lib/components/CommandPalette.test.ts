import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/svelte";
import CommandPalette from "./CommandPalette.svelte";
import KeyboardSettings from "./KeyboardSettings.svelte";
import { shortcuts } from "../stores/shortcuts.svelte";
import { defaultBindings } from "../utils/keyboardShortcuts";

beforeEach(() => {
  shortcuts.bindings = { ...defaultBindings, settings: "Mod+Alt+Shift+P" };
  shortcuts.ready = true;
  shortcuts.suspended = true;
  shortcuts.error = null;
  HTMLElement.prototype.scrollIntoView = vi.fn();
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  shortcuts.bindings = { ...defaultBindings };
});

it.each([
  ["MacIntel", "⌥⇧⌘P", "⌘K"],
  ["Win32", "Ctrl+Alt+Shift+P", "Ctrl+K"],
  ["Linux x86_64", "Ctrl+Alt+Shift+P", "Ctrl+K"],
])(
  "renders settings and palette shortcuts for %s without hardcoded Command icons",
  (platform, settingsLabel, paletteLabel) => {
    vi.spyOn(navigator, "platform", "get").mockReturnValue(platform);
    const settings = render(KeyboardSettings);
    const binding = settings.getByRole("button", { name: "Shortcut for Settings" });
    expect(binding.textContent?.trim()).toBe(settingsLabel);
    settings.unmount();

    const palette = render(CommandPalette, {
      open: true,
      commands: [
        {
          id: "settings",
          label: "Settings",
          category: "Help",
          shortcut: shortcuts.label("settings"),
          action: vi.fn(),
        },
      ],
    });
    const displayed = Array.from(palette.container.querySelectorAll("kbd"), (element) =>
      element.textContent?.trim()
    );
    expect(displayed).toEqual([paletteLabel, settingsLabel]);
    expect(palette.container.querySelector("svg.lucide-command")).toBeNull();
    expect(palette.container.querySelector("svg.lucide-keyboard")).not.toBeNull();
    if (platform !== "MacIntel") expect(palette.container.textContent).not.toMatch(/[⌘⌥⇧]/);
  }
);
