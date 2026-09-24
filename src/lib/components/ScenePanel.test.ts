import { afterEach, beforeAll, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ScenePanel from "./ScenePanel.svelte";
import { currentProject } from "../stores/project.svelte";
import { session } from "../stores/session.svelte";
import { mockProject, mockChapters, mockScenes } from "../../dev/mock-data";

vi.hoisted(() => {
  const data = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => data.set(k, v),
    removeItem: (k: string) => data.delete(k),
  });
});

// Vitest serves CSS modules (even `?raw`) as empty strings, so read the real
// stylesheets from disk. The specifier is a variable so the check stays free
// of Node typings.
let disabledRules = "";
let tokensCss = "";
beforeAll(async () => {
  const fsModule = "node:fs";
  const { readFileSync } = await import(/* @vite-ignore */ fsModule);
  const read = (path: string): string =>
    readFileSync(new URL(path, import.meta.url).pathname, "utf8");
  const appCss = read("../../app.css");
  tokensCss = read("../../styles/press/tokens.css");
  // The app's disabled-control rules: the global disabled pair through the
  // segment adapter, after Press's `.ka-segment.ka-selected` accent fill.
  const start = appCss.indexOf("/* Disabled state is a token pair");
  const end = appCss.indexOf("/* Focus follows the Press application layer", start);
  expect(start).toBeGreaterThan(-1);
  expect(end).toBeGreaterThan(start);
  disabledRules =
    ".ka-segment.ka-selected{background-color:var(--color-accent-text);color:var(--color-on-accent);}\n" +
    appCss.slice(start, end).replace(/\/\*[\s\S]*?\*\//g, "");
});

/** Resolve a Press token to its hex value in a theme, following var() chains. */
function token(name: string, theme: "light" | "dark"): string {
  const blocks = new Map<string, Map<string, string>>();
  const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, "");
  for (const [, selector, body] of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const values = blocks.get(selector.trim()) ?? new Map<string, string>();
    for (const [, prop, value] of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g))
      values.set(prop, value.trim());
    blocks.set(selector.trim(), values);
  }
  const lookup = (prop: string): string => {
    const value =
      (theme === "dark" ? blocks.get('[data-theme="dark"]')?.get(prop) : undefined) ??
      blocks.get(":root")?.get(prop) ??
      "";
    const reference = value.match(/^var\((--[\w-]+)\)$/);
    return reference ? lookup(reference[1]) : value;
  };
  return lookup(name);
}
function contrast(a: string, b: string): number {
  const luminance = (hex: string) => {
    const [r, g, bl] = [1, 3, 5].map((i) => {
      const c = parseInt(hex.slice(i, i + 2), 16) / 255;
      return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

beforeEach(() => {
  HTMLElement.prototype.scrollIntoView = vi.fn();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_scene_reference_items" ? {} : []
  );
  currentProject.setProject(mockProject);
  currentProject.setCurrentChapter(mockChapters[0]);
});
afterEach(async () => {
  cleanup();
  await session.flush();
  currentProject.setProject(null);
});

it.each(["page", "beat"] as const)(
  "keeps the active %s view visibly selected when a locked scene disables the switch",
  async (mode) => {
    currentProject.setCurrentScene({
      ...mockScenes[0],
      chapter_id: mockChapters[0].id,
      editor_mode: mode,
      planning_status: "fixed",
      locked: true,
    });
    render(ScenePanel);
    const active = (await screen.findByTestId(
      mode === "page" ? "view-page" : "view-beats"
    )) as HTMLButtonElement;
    const other = screen.getByTestId(mode === "page" ? "view-beats" : "view-page");
    expect(active.disabled).toBe(true);
    expect((other as HTMLButtonElement).disabled).toBe(true);
    expect(active.classList.contains("ka-selected")).toBe(true);
    expect(active.getAttribute("aria-pressed")).toBe("true");
    expect(other.classList.contains("ka-selected")).toBe(false);
    expect(other.getAttribute("aria-pressed")).toBe("false");

    // The global disabled pair must not flatten the chosen segment into its
    // track: it keeps its own fill and a ring, uses the disabled ink, and
    // never dims with opacity. (jsdom keeps var() unresolved.)
    // jsdom drops !important from its CSSOM and cascade, so resolve the
    // winning declarations here: among the app's rules matching an element,
    // the last !important declaration of a property wins (each rule after
    // the global disabled pair is at least as specific as it).
    const winning = (element: Element, property: string) => {
      let value = "";
      for (const [, selector, body] of disabledRules.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
        if (!element.matches(selector.trim())) continue;
        const declaration = body.match(new RegExp(`(?:^|[;\\s])${property}:\\s*([^;]+?)\\s*;`));
        if (declaration && (declaration[1].endsWith("!important") || !value))
          value = declaration[1];
      }
      return value;
    };
    expect(winning(other, "background-color")).toBe("var(--color-disabled-bg) !important");
    expect(winning(active, "background-color")).toBe("var(--color-surface) !important");
    expect(winning(active, "box-shadow")).toBe("inset 0 0 0 1px var(--color-disabled-text)");
    expect(winning(active, "color")).toBe("var(--color-disabled-text) !important");
    expect(winning(active, "opacity")).toBe("1 !important");

    // In both themes the chosen fill differs from the track, and the ring and
    // text stand out from it.
    for (const theme of ["light", "dark"] as const) {
      const fill = token("--color-surface", theme);
      const track = token("--color-disabled-bg", theme);
      const ink = token("--color-disabled-text", theme);
      expect(fill).toMatch(/^#[0-9a-f]{6}$/i);
      expect(fill).not.toBe(track);
      expect(contrast(ink, fill)).toBeGreaterThanOrEqual(4.5);
    }
  }
);

it.each(["a dialog is open", "the Escape was already handled"])(
  "keeps a discovery-note draft when Escape reaches the window while %s",
  async (situation) => {
    currentProject.setCurrentScene({
      ...mockScenes[0],
      chapter_id: mockChapters[0].id,
      planning_status: "fixed",
    });
    render(ScenePanel);
    await fireEvent.click(await screen.findByRole("button", { name: /Discovery notes/ }));
    await fireEvent.click(await screen.findByRole("button", { name: "Add note" }));
    const draft = screen.getByLabelText("New discovery note") as HTMLTextAreaElement;
    await fireEvent.input(draft, { target: { value: "The well is dry" } });

    const modal = document.createElement("div");
    modal.setAttribute("aria-modal", "true");
    if (situation === "a dialog is open") document.body.append(modal);
    const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    if (situation !== "a dialog is open") escape.preventDefault();
    await fireEvent(window, escape);

    expect((screen.getByLabelText("New discovery note") as HTMLTextAreaElement).value).toBe(
      "The well is dry"
    );
    modal.remove();
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByLabelText("New discovery note")).toBeNull();
  }
);
