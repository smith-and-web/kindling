import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import StartScreen from "./StartScreen.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import type { Project } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

function project(overrides: Partial<Project> = {}): Project {
  return {
    id: "project-1",
    name: "The Letter",
    source_type: "Markdown",
    source_path: null,
    created_at: "2026-09-10T12:00:00Z",
    modified_at: "2026-09-10T12:00:00Z",
    author_pen_name: null,
    genre: null,
    description: null,
    word_target: null,
    reference_types: ["characters"],
    project_type: "novel",
    target_page_count: null,
    ...overrides,
  };
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  currentProject.setProject(null);
  ui.setView("start");
});

afterEach(() => {
  cleanup();
  ui.finishImport();
});

describe("StartScreen", () => {
  it("lists recent projects as rows with plain metadata and a labelled delete action", () => {
    render(StartScreen, { recentProjects: [project()], onNewProject: vi.fn() });

    const list = screen.getByTestId("recent-projects");
    expect(within(list).getByRole("heading", { name: "Recent projects" })).toBeTruthy();
    const row = within(list).getByTestId("project-card");
    expect(row.textContent).toContain("The Letter");
    // The importer id reads as sentence-case metadata, not "(markdown)".
    expect(row.textContent).toContain("Markdown");
    expect(row.textContent).not.toContain("(");
    expect(row.querySelector("time")?.getAttribute("datetime")).toBe("2026-09-10T12:00:00Z");
    expect(row.querySelector("time")?.getAttribute("aria-label")).toMatch(/^Modified /);
    expect(within(list).getByRole("button", { name: "Delete The Letter" })).toBeTruthy();
  });

  const many = (n: number) =>
    Array.from({ length: n }, (_, i) => project({ id: `p${i}`, name: `Book ${i + 1}` }));

  it("keeps View all as the first control when there are more projects", () => {
    render(StartScreen, { recentProjects: many(6) });
    const first = screen.getByTestId("recent-projects").querySelector("button");
    expect(first?.textContent?.trim()).toBe("View all");
    expect(first?.getAttribute("aria-pressed")).toBe("false");
    expect(screen.getAllByRole("listitem").length).toBeGreaterThanOrEqual(5);
    expect(screen.queryByText("Book 6")).toBeNull();
  });

  it("offers no View all when every project is already listed", () => {
    render(StartScreen, { recentProjects: many(5) });
    expect(screen.queryByRole("button", { name: "View all" })).toBeNull();
    expect(screen.getByText("Book 5")).toBeTruthy();
  });

  it("lists every project, then returns to recent without refetching", async () => {
    const all = many(12);
    vi.mocked(invoke).mockResolvedValueOnce(all);
    render(StartScreen, { recentProjects: all.slice(0, 10) });
    await fireEvent.click(screen.getByRole("button", { name: "View all" }));
    expect(await screen.findByRole("heading", { name: "All projects" })).toBeTruthy();
    expect(screen.getByText("Book 12")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("get_all_projects");

    await fireEvent.click(screen.getByRole("button", { name: "Show recent" }));
    expect(screen.getByRole("heading", { name: "Recent projects" })).toBeTruthy();
    expect(screen.queryByText("Book 6")).toBeNull();
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("explains the empty library", () => {
    render(StartScreen, { recentProjects: [] });
    expect(screen.getByText("Your projects will appear here")).toBeTruthy();
    expect(screen.queryByTestId("recent-projects")).toBeNull();
    expect(screen.getByTestId("import-section")).toBeTruthy();
  });

  it("shows an open failure beside the list and keeps the row", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("The project file could not be read.");
    render(StartScreen, { recentProjects: [project()] });

    await fireEvent.click(screen.getByTestId("project-card"));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Couldn’t open “The Letter”");
    expect(alert.textContent).toContain("The project file could not be read.");
    expect(screen.getByTestId("project-card")).toBeTruthy();
    expect(currentProject.value).toBeNull();
  });

  it("marks the row busy while a project opens", async () => {
    let resolve!: (value: Project) => void;
    vi.mocked(invoke).mockReturnValueOnce(new Promise<Project>((r) => (resolve = r)));
    render(StartScreen, { recentProjects: [project()] });

    await fireEvent.click(screen.getByTestId("project-card"));
    expect(screen.getByText("Opening…")).toBeTruthy();
    expect(screen.getByTestId("project-card").closest("li")?.getAttribute("aria-busy")).toBe(
      "true"
    );

    resolve(project());
    await waitFor(() => expect(currentProject.value?.id).toBe("project-1"));
    expect(ui.currentView).toBe("editor");
  });

  it("spells source tools the way they spell themselves", () => {
    render(StartScreen, {
      recentProjects: [
        project({ id: "a", source_type: "YWriter" }),
        project({ id: "b", source_type: "NovelWriter" }),
      ],
    });
    const rows = screen.getAllByTestId("project-card").map((row) => row.textContent);
    expect(rows[0]).toContain("yWriter");
    expect(rows[1]).toContain("novelWriter");
  });

  it("switches to all projects only after they load", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("offline");
    render(StartScreen, { recentProjects: many(6) });
    await fireEvent.click(screen.getByRole("button", { name: "View all" }));
    await waitFor(() => expect(ui.toast?.message).toContain("Failed to load projects"));
    expect(screen.getByRole("heading", { name: "Recent projects" })).toBeTruthy();
  });

  it("names the consequence when confirming a delete", async () => {
    render(StartScreen, { recentProjects: [project()] });
    await fireEvent.click(screen.getByRole("button", { name: "Delete The Letter" }));
    expect(screen.getByText("Delete “The Letter”?")).toBeTruthy();
    expect(screen.getByTestId("dialog-message").textContent).toContain("It can’t be undone.");
  });
});
