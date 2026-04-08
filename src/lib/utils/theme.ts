export type ThemePreference = "dark" | "light" | "system";

const STORAGE_KEY = "kindling:theme";
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

function resolveTheme(pref: ThemePreference): "dark" | "light" {
  if (pref === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return pref;
}

function applyTheme(theme: "dark" | "light") {
  document.documentElement.setAttribute("data-theme", theme);
}

export function getStoredPreference(): ThemePreference {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") return stored;
  return "dark";
}

export function setThemePreference(pref: ThemePreference) {
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(resolveTheme(pref));
  setupSystemListener(pref);
}

function setupSystemListener(pref: ThemePreference) {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener("change", mediaListener);
    mediaListener = null;
  }

  if (pref === "system") {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaListener = (e: MediaQueryListEvent) => {
      applyTheme(e.matches ? "dark" : "light");
    };
    mediaQuery.addEventListener("change", mediaListener);
  }
}

/**
 * Call once on app startup (before first paint if possible).
 * Reads stored preference and applies the correct data-theme attribute.
 */
export function initTheme() {
  const pref = getStoredPreference();
  applyTheme(resolveTheme(pref));
  setupSystemListener(pref);
}
