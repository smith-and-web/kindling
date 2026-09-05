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

/**
 * Class applied to <html> for the duration of a theme switch. app.css disables
 * transitions under it so every surface flips at once instead of each element
 * animating from its old colour, and the forced style flush below makes WebKit
 * recompute var()-driven colours on the side panels immediately.
 */
const SWITCHING_CLASS = "theme-switching";

function applyTheme(theme: "dark" | "light") {
  const root = document.documentElement;
  root.classList?.add(SWITCHING_CLASS);
  root.setAttribute("data-theme", theme);
  // Reading a layout property forces a synchronous style recalculation.
  void root.offsetHeight;
  const release = () => root.classList?.remove(SWITCHING_CLASS);
  if (typeof requestAnimationFrame === "function") requestAnimationFrame(release);
  else release();
}

export function getStoredPreference(): ThemePreference {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") return stored;
  return "light";
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
