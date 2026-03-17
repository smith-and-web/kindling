import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

let store: Record<string, string> = {};
const localStorageMock = {
  getItem: vi.fn((key: string) => store[key] || null),
  setItem: vi.fn((key: string, value: string) => {
    store[key] = value;
  }),
  removeItem: vi.fn((key: string) => {
    delete store[key];
  }),
  clear: vi.fn(() => {
    store = {};
  }),
};

vi.stubGlobal("localStorage", localStorageMock);

const setAttributeSpy = vi.fn();
vi.stubGlobal("document", {
  documentElement: { setAttribute: setAttributeSpy },
});

let matchMediaMatches = false;
const addListenerSpy = vi.fn();
const removeListenerSpy = vi.fn();
vi.stubGlobal("window", {
  ...globalThis.window,
  matchMedia: vi.fn(() => ({
    matches: matchMediaMatches,
    addEventListener: addListenerSpy,
    removeEventListener: removeListenerSpy,
  })),
});

const { getStoredPreference, setThemePreference, initTheme } = await import("./theme");

describe("theme utility", () => {
  beforeEach(() => {
    store = {};
    vi.clearAllMocks();
    matchMediaMatches = false;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("getStoredPreference", () => {
    it("returns 'dark' when nothing stored", () => {
      expect(getStoredPreference()).toBe("dark");
    });

    it("returns 'light' when stored", () => {
      store["kindling:theme"] = "light";
      expect(getStoredPreference()).toBe("light");
    });

    it("returns 'system' when stored", () => {
      store["kindling:theme"] = "system";
      expect(getStoredPreference()).toBe("system");
    });

    it("returns 'dark' for invalid stored value", () => {
      store["kindling:theme"] = "invalid";
      expect(getStoredPreference()).toBe("dark");
    });
  });

  describe("setThemePreference", () => {
    it("persists and applies dark theme", () => {
      setThemePreference("dark");
      expect(localStorageMock.setItem).toHaveBeenCalledWith("kindling:theme", "dark");
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "dark");
    });

    it("persists and applies light theme", () => {
      setThemePreference("light");
      expect(localStorageMock.setItem).toHaveBeenCalledWith("kindling:theme", "light");
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "light");
    });

    it("resolves system to dark when OS prefers dark", () => {
      matchMediaMatches = true;
      setThemePreference("system");
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "dark");
      expect(addListenerSpy).toHaveBeenCalledWith("change", expect.any(Function));
    });

    it("resolves system to light when OS prefers light", () => {
      matchMediaMatches = false;
      setThemePreference("system");
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "light");
    });

    it("removes previous system listener when switching away", () => {
      setThemePreference("system");
      expect(addListenerSpy).toHaveBeenCalled();

      setThemePreference("dark");
      expect(removeListenerSpy).toHaveBeenCalled();
    });

    it("system listener applies theme on OS change", () => {
      setThemePreference("system");
      const listener = addListenerSpy.mock.calls[0][1];

      listener({ matches: true } as MediaQueryListEvent);
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "dark");

      listener({ matches: false } as MediaQueryListEvent);
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "light");
    });
  });

  describe("initTheme", () => {
    it("applies theme from storage on init", () => {
      store["kindling:theme"] = "light";
      initTheme();
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "light");
    });

    it("defaults to dark when nothing stored", () => {
      initTheme();
      expect(setAttributeSpy).toHaveBeenCalledWith("data-theme", "dark");
    });

    it("sets up system listener when preference is system", () => {
      store["kindling:theme"] = "system";
      initTheme();
      expect(addListenerSpy).toHaveBeenCalledWith("change", expect.any(Function));
    });
  });
});
