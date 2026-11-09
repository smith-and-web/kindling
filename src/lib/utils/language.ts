/**
 * The writer's interface-language choice.
 *
 * Stored in localStorage next to the theme, because the webview has to know the language
 * synchronously before the first translated string renders. The native menu is built by Rust
 * before the webview exists, so every change (and every startup) is also mirrored to the
 * backend with `set_interface_language`, which persists it beside `shortcuts.json` and
 * rebuilds the menu when it differs.
 */
import { invoke } from "@tauri-apps/api/core";
import {
  LANGUAGE_STORAGE_KEY,
  localeFor,
  setLocale,
  storedLanguagePreference,
  systemLanguageTag,
} from "../i18n";
import type { LanguagePreference } from "../i18n/types";

export function getLanguagePreference(): LanguagePreference {
  return storedLanguagePreference(typeof localStorage === "undefined" ? undefined : localStorage);
}

/** Tell the backend which language the native menu should use. Never rejects. */
export async function syncNativeMenuLanguage(preference: LanguagePreference): Promise<void> {
  try {
    await invoke("set_interface_language", {
      locale: preference === "system" ? null : preference,
    });
  } catch (error) {
    // The interface already switched; a stale native menu is not worth interrupting writing.
    console.error("Failed to update the menu language:", error);
  }
}

/** Persist a language choice, switch the interface to it, and update the native menu. */
export function setLanguagePreference(preference: LanguagePreference): Promise<void> {
  try {
    if (preference === "system") localStorage.removeItem(LANGUAGE_STORAGE_KEY);
    else localStorage.setItem(LANGUAGE_STORAGE_KEY, preference);
  } catch (error) {
    console.error("Failed to save the interface language:", error);
  }
  setLocale(localeFor(preference, import.meta.env, systemLanguageTag()));
  return syncNativeMenuLanguage(preference);
}
