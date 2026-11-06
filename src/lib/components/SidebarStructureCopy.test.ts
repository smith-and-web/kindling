import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar.svelte";
import { currentProject } from "../stores/project.svelte";
import type { Chapter, Scene } from "../types";
import { mockProject } from "../../dev/mock-data";
import { setLocale } from "../i18n";
import type { Locale } from "../i18n/types";

// Structure copy has one key per kind, so each language agrees in gender, number and case.

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

function chapter(overrides: Partial<Chapter> = {}): Chapter {
  return {
    id: "chapter",
    project_id: mockProject.id,
    title: "Arrival",
    position: 0,
    source_id: null,
    archived: false,
    locked: false,
    is_part: false,
    synopsis: null,
    planning_status: "fixed",
    ...overrides,
  };
}
const scene: Scene = {
  id: "scene",
  chapter_id: "chapter",
  title: "The Gate",
  synopsis: null,
  prose: null,
  position: 0,
  source_id: null,
  archived: false,
  locked: false,
  scene_type: "normal",
  scene_status: "draft",
  planning_status: "fixed",
  editor_mode: "beat",
};

function mockOutline(chapters: Chapter[]) {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_chapters") return chapters;
    if (command === "get_scenes") return [scene];
    if (command === "get_page_count_estimate") return { pages: 1.5, words: 300, target: "90" };
    return [];
  });
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(mockProject);
});
afterEach(() => {
  cleanup();
  setLocale("en");
  currentProject.setProject(null);
});

it.each([
  ["en", "novel", "New chapter", "New part", "Chapter menu", "Chapter title…"],
  ["de", "novel", "Neues Kapitel", "Neuer Teil", "Kapitel-Menü", "Titel des Kapitels…"],
  ["ru", "novel", "Новая глава", "Новая часть", "Меню главы", "Название главы…"],
  ["pt-BR", "novel", "Novo capítulo", "Nova parte", "Menu do capítulo", "Título do capítulo…"],
  ["ru", "screenplay", "Новая секвенция", "Новый акт", "Меню секвенции", "Название секвенции…"],
  [
    "es",
    "screenplay",
    "Nueva secuencia",
    "Nuevo acto",
    "Menú de la secuencia",
    "Título de la secuencia…",
  ],
] as const)(
  "%s %s structure copy agrees with its noun",
  async (locale, type, newChapter, newPart, menu, placeholder) => {
    setLocale(locale as Locale);
    currentProject.setProject({ ...mockProject, project_type: type });
    mockOutline([chapter()]);
    render(Sidebar);
    const create = await screen.findByTestId("new-chapter-button");
    expect(create.textContent?.trim()).toBe(newChapter);
    expect(await screen.findByRole("button", { name: menu })).toBeTruthy();
    await fireEvent.click(screen.getByTestId("new-dropdown-button"));
    expect(screen.getByTestId("dropdown-new-part").textContent?.trim()).toBe(newPart);
    await fireEvent.click(screen.getByTestId("dropdown-new-chapter"));
    await tick();
    expect(screen.getByPlaceholderText(placeholder)).toBeTruthy();
    if (type === "screenplay") expect(screen.getByText("1,5 / 90")).toBeTruthy();
  }
);

it("reports a failed chapter creation without lowercasing the noun in code", async () => {
  setLocale("de");
  const showError = vi.spyOn((await import("../stores/ui.svelte")).ui, "showError");
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_chapters") return [chapter()];
    if (command === "get_scenes") return [scene];
    if (command === "create_chapter") throw new Error("disk full");
    return [];
  });
  render(Sidebar);
  await fireEvent.click(await screen.findByTestId("new-dropdown-button"));
  await fireEvent.click(screen.getByTestId("dropdown-new-chapter"));
  await tick();
  const input = screen.getByPlaceholderText("Titel des Kapitels…");
  await fireEvent.input(input, { target: { value: "Neu" } });
  await fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() =>
    expect(showError).toHaveBeenCalledWith("Kapitel konnte nicht erstellt werden: Error: disk full")
  );
});

it("lets the scene filter segments wrap rather than overlap in long languages", async () => {
  const source = (await import("./Sidebar.svelte?raw")).default;
  const rule = (selector: string) =>
    source.slice(
      source.indexOf(`${selector} {`),
      source.indexOf("}", source.indexOf(`${selector} {`))
    );
  expect(rule(".sb-filter-segments .ka-segment-track")).toContain("flex-wrap: wrap");
  const segment = rule(".sb-filter-segments .ka-segment");
  expect(segment).toContain("white-space: nowrap");
  // A segment may never shrink below its label, and spacing comes from Press tokens.
  expect(segment).not.toMatch(/min-width:\s*0/);
  expect(segment).not.toMatch(/\d+px/);
});
