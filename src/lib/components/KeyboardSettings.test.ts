import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import KeyboardSettings from "./KeyboardSettings.svelte";
import { shortcuts } from "../stores/shortcuts.svelte";
import { defaultBindings } from "../utils/keyboardShortcuts";
const mod = /Mac/.test(navigator.platform) ? { metaKey: true } : { ctrlKey: true };
beforeEach(() => {
  shortcuts.bindings = { ...defaultBindings };
  shortcuts.ready = true;
  shortcuts.suspended = true;
  shortcuts.error = null;
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(
    async (_cmd, args) => (args as { bindings: unknown })?.bindings
  );
});
afterEach(cleanup);
it("records, rejects conflicts, clears, and resets shortcuts", async () => {
  render(KeyboardSettings);
  const button = screen.getByRole("button", { name: "Shortcut for Settings" });
  await fireEvent.click(button);
  await fireEvent.keyDown(button, { key: "e", ...mod });
  expect(screen.getByRole("alert").textContent).toContain("Export project");
  expect(invoke).not.toHaveBeenCalled();
  await fireEvent.keyDown(button, { key: "c", ...mod });
  expect(screen.getByRole("alert").textContent).toContain("reserved");
  await fireEvent.keyDown(button, { key: "p", altKey: true, ...mod });
  await screen.findByText("Keyboard shortcuts saved.");
  expect(shortcuts.bindings.settings).toBe("Mod+Alt+P");
  await fireEvent.click(screen.getByRole("button", { name: "Clear shortcut for Settings" }));
  await waitFor(() => expect(shortcuts.bindings.settings).toBe(""));
  await fireEvent.click(screen.getByRole("button", { name: "Reset all to defaults" }));
  await waitFor(() => expect(shortcuts.bindings).toEqual(defaultBindings));
});
it("cancels recording, filters commands, and reports failed saves", async () => {
  render(KeyboardSettings);
  const button = screen.getByRole("button", { name: "Shortcut for Settings" });
  await fireEvent.click(button);
  await fireEvent.keyDown(button, { key: "Escape" });
  expect(button.getAttribute("aria-pressed")).toBe("false");
  await fireEvent.click(button);
  await fireEvent.keyDown(button, { key: "Tab" });
  expect(button.getAttribute("aria-pressed")).toBe("false");
  await fireEvent.click(button);
  await fireEvent.keyDown(button, { key: "p" });
  expect(screen.getByRole("alert").textContent).toContain("Use Command");
  vi.mocked(invoke).mockRejectedValueOnce("disk full");
  await fireEvent.keyDown(button, { key: "p", altKey: true, ...mod });
  await screen.findByText(/Could not save keyboard shortcuts: disk full/);
  expect(shortcuts.bindings.settings).toBe("Mod+Comma");
  await fireEvent.input(screen.getByRole("searchbox"), { target: { value: "Bold" } });
  expect(screen.queryByRole("button", { name: "Shortcut for Settings" })).toBeNull();
  expect(screen.getByRole("button", { name: "Shortcut for Bold" })).toBeTruthy();
});
it("keeps recording disabled until native menu shortcuts are suspended", () => {
  shortcuts.suspended = false;
  render(KeyboardSettings);
  expect(
    (screen.getByRole("button", { name: "Shortcut for Settings" }) as HTMLButtonElement).disabled
  ).toBe(true);
});

it("allows resetting a malformed saved configuration without a successful load", async () => {
  shortcuts.ready = false;
  vi.mocked(invoke).mockRejectedValueOnce("malformed shortcuts.json");
  render(KeyboardSettings);
  await screen.findByText(/Could not load keyboard shortcuts/);
  const reset = screen.getByRole("button", { name: "Reset all to defaults" }) as HTMLButtonElement;
  expect(reset.disabled).toBe(false);
  vi.mocked(invoke).mockResolvedValue(defaultBindings);
  await fireEvent.click(reset);
  await screen.findByText("Keyboard shortcuts saved.");
  expect(shortcuts.ready).toBe(true);
  expect(shortcuts.error).toBeNull();
  const button = screen.getByRole("button", { name: "Shortcut for Settings" });
  expect(document.getElementById(button.getAttribute("aria-describedby")!)?.textContent).toBe(
    shortcuts.label("settings")
  );
});
