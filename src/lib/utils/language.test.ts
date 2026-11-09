import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getLocale, LANGUAGE_STORAGE_KEY, setLocale, t } from "../i18n";
import { getLanguagePreference, setLanguagePreference, syncNativeMenuLanguage } from "./language";

// jsdom here has no usable localStorage; stub a Map-backed one, as theme.test.ts does.
const store = new Map<string, string>();
const localStorageMock = {
  getItem: vi.fn((key: string) => store.get(key) ?? null),
  setItem: vi.fn((key: string, value: string) => void store.set(key, String(value))),
  removeItem: vi.fn((key: string) => void store.delete(key)),
};
vi.stubGlobal("localStorage", localStorageMock);

describe("interface language preference", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
    localStorage.removeItem(LANGUAGE_STORAGE_KEY);
  });

  afterEach(() => {
    localStorage.removeItem(LANGUAGE_STORAGE_KEY);
    setLocale("en");
    vi.mocked(console.error).mockRestore?.();
  });

  it("follows the system until a language is chosen", () => {
    expect(getLanguagePreference()).toBe("system");
    localStorage.setItem(LANGUAGE_STORAGE_KEY, "nope");
    expect(getLanguagePreference()).toBe("system");
  });

  it("persists a choice, switches the interface and updates the native menu", async () => {
    await setLanguagePreference("de");
    expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe("de");
    expect(getLanguagePreference()).toBe("de");
    expect(getLocale()).toBe("de");
    expect(document.documentElement.lang).toBe("de");
    expect(t("settings.appearance.language")).toBe("Sprache");
    expect(invoke).toHaveBeenCalledWith("set_interface_language", { locale: "de" });
  });

  it("returns to the system language and clears the stored choice", async () => {
    await setLanguagePreference("ru");
    await setLanguagePreference("system");
    expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBeNull();
    // jsdom reports an English system language.
    expect(getLocale()).toBe("en");
    expect(invoke).toHaveBeenLastCalledWith("set_interface_language", { locale: null });
  });

  it("keeps the switch when the backend refuses the menu update", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValueOnce("menu failed");
    await expect(setLanguagePreference("fr")).resolves.toBeUndefined();
    expect(getLocale()).toBe("fr");
    expect(error).toHaveBeenCalledWith("Failed to update the menu language:", "menu failed");
  });

  it("still switches when storage is unavailable", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    localStorageMock.setItem.mockImplementationOnce(() => {
      throw new Error("quota");
    });
    await setLanguagePreference("es");
    expect(getLocale()).toBe("es");
    expect(error).toHaveBeenCalledWith("Failed to save the interface language:", expect.any(Error));
  });

  it("syncs the stored choice to the backend", async () => {
    await syncNativeMenuLanguage("zh-CN");
    expect(invoke).toHaveBeenCalledWith("set_interface_language", { locale: "zh-CN" });
  });
});
