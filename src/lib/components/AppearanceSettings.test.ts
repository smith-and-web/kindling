import { afterEach, beforeEach, expect, it, vi } from "vitest";

// jsdom here has no usable localStorage, and the ui store reads it on import, so stub a
// Map-backed one before any import runs.
vi.hoisted(() => {
  const store = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, String(value)),
    removeItem: (key: string) => void store.delete(key),
  });
});
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import AppearanceSettings from "./AppearanceSettings.svelte";
import { LANGUAGE_STORAGE_KEY, setLocale } from "../i18n";

beforeEach(() => {
  vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
  localStorage.removeItem(LANGUAGE_STORAGE_KEY);
});

afterEach(() => {
  cleanup();
  localStorage.removeItem(LANGUAGE_STORAGE_KEY);
  setLocale("en");
});

const languageSelect = () => screen.getByLabelText("Language") as HTMLSelectElement;

it("offers the system default and every shipped language by its own name", () => {
  render(AppearanceSettings);
  const select = languageSelect();
  expect(select.value).toBe("system");
  const options = within(select)
    .getAllByRole("option")
    .map((option) => [(option as HTMLOptionElement).value, option.textContent]);
  expect(options).toEqual([
    ["system", "System default"],
    ["en", "English"],
    ["de", "Deutsch"],
    ["es", "Español"],
    ["fr", "Français"],
    ["pt-BR", "Português (Brasil)"],
    ["ru", "Русский"],
    ["zh-CN", "简体中文"],
  ]);
  expect(screen.getByText(/follows your operating system’s language/)).toBeTruthy();
});

it("switches the interface live and keeps language names untranslated", async () => {
  render(AppearanceSettings);
  await fireEvent.change(languageSelect(), { target: { value: "fr" } });

  expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe("fr");
  expect(invoke).toHaveBeenCalledWith("set_interface_language", { locale: "fr" });
  // Already-rendered copy follows the switch without a remount.
  expect(screen.getByText("Apparence")).toBeTruthy();
  const select = screen.getByLabelText("Langue") as HTMLSelectElement;
  expect(select.value).toBe("fr");
  expect(within(select).getByRole("option", { name: "Langue du système" })).toBeTruthy();
  expect(within(select).getByRole("option", { name: "Deutsch" })).toBeTruthy();

  await fireEvent.change(select, { target: { value: "system" } });
  expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBeNull();
  expect(invoke).toHaveBeenLastCalledWith("set_interface_language", { locale: null });
  expect(screen.getByText("Appearance")).toBeTruthy();
});

it("shows a stored choice as selected", () => {
  localStorage.setItem(LANGUAGE_STORAGE_KEY, "zh-CN");
  render(AppearanceSettings);
  expect(languageSelect().value).toBe("zh-CN");
});
