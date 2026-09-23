import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import Onboarding from "./Onboarding.svelte";
import { ui } from "../stores/ui.svelte";
import { currentProject } from "../stores/project.svelte";
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

const project = { id: "p1", name: "Guided novel" } as Project;
const preview = {
  project_name: "Guided novel",
  chapter_count: 2,
  scene_count: 5,
  beat_count: 9,
  character_count: 0,
  location_count: 3,
};

beforeEach(() => {
  ui.startOnboarding();
  vi.mocked(invoke).mockReset();
  vi.mocked(open).mockReset();
});

afterEach(() => {
  cleanup();
  ui.finishImport();
});

const button = (name: string | RegExp) => screen.getByRole("button", { name });

describe("Onboarding", () => {
  it("welcomes with a labelled dialog, progress and the sample as the primary", () => {
    render(Onboarding);
    expect(screen.getByRole("dialog", { name: "Welcome to kindling" })).toBeTruthy();
    expect(screen.getByText("Step 1 of 5")).toBeTruthy();
    expect(screen.getByRole("progressbar", { name: "Getting started, step 1 of 5" })).toBeTruthy();
    expect(document.activeElement).toBe(button("Try sample project"));
  });

  it("skips from the close button, the ghost action and Escape", async () => {
    render(Onboarding);
    await fireEvent.click(screen.getByTestId("skip-onboarding"));
    expect(ui.showOnboarding).toBe(false);

    ui.startOnboarding();
    await waitFor(() => expect(screen.getByTestId("onboarding")).toBeTruthy());
    await fireEvent.click(button("Skip and start importing"));
    expect(ui.showOnboarding).toBe(false);

    ui.startOnboarding();
    await waitFor(() => expect(screen.getByTestId("onboarding")).toBeTruthy());
    await fireEvent.keyDown(screen.getByTestId("onboarding"), { key: "Escape" });
    expect(ui.showOnboarding).toBe(false);
  });

  it("walks the tour forwards and back, then finishes into import", async () => {
    render(Onboarding);
    await fireEvent.click(button("Take the tour"));
    expect(screen.getByRole("dialog", { name: "Chapters and scenes" })).toBeTruthy();
    expect(screen.getByText("Step 2 of 5")).toBeTruthy();

    await fireEvent.click(button("Next"));
    expect(screen.getByRole("dialog", { name: "Synopsis and beats" })).toBeTruthy();
    await fireEvent.click(button("Back"));
    expect(ui.onboardingStep).toBe("tour-sidebar");
    await fireEvent.click(button("Next"));
    await fireEvent.click(button("Next"));
    expect(screen.getByRole("dialog", { name: "References" })).toBeTruthy();

    await fireEvent.click(button("Start importing"));
    expect(localStorage.getItem("kindling:onboardingCompleted")).toBe("true");
    expect(screen.getByRole("dialog", { name: "Import your outline" })).toBeTruthy();
    expect(screen.getByText("Step 5 of 5")).toBeTruthy();

    await fireEvent.click(button("Back to tour"));
    expect(ui.onboardingStep).toBe("tour-references");
  });

  it("skips the tour straight to import, and can defer importing", async () => {
    ui.goToStep("tour-editor");
    render(Onboarding);
    await fireEvent.click(button("Skip tour"));
    expect(ui.onboardingStep).toBe("import");
    await fireEvent.click(button("I’ll import later"));
    expect(ui.showOnboarding).toBe(false);
  });

  it("previews an outline before importing it", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue("/outline.pltr");
    vi.mocked(invoke).mockImplementation(async (cmd: string) =>
      cmd === "preview_import" ? preview : project
    );
    const onImportComplete = vi.fn();
    render(Onboarding, { onImportComplete });

    await fireEvent.click(button(/^Plottr/));
    expect(
      await screen.findByRole("dialog", { name: "Ready to import “Guided novel”" })
    ).toBeTruthy();
    expect(screen.getByText("Locations")).toBeTruthy();
    expect(screen.queryByText("Characters")).toBeNull();

    await fireEvent.click(screen.getByTestId("guided-import-confirm"));
    await waitFor(() => expect(onImportComplete).toHaveBeenCalledWith(project, "plottr"));
    expect(invoke).toHaveBeenCalledWith("import_plottr", { path: "/outline.pltr" });
    expect(ui.showOnboarding).toBe(false);
  });

  it("returns from a preview to the format list", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue("/outline.md");
    vi.mocked(invoke).mockResolvedValue(preview);
    render(Onboarding);

    await fireEvent.click(button(/^Markdown/));
    await fireEvent.click(await screen.findByRole("button", { name: "Choose a different file" }));
    expect(screen.getByRole("dialog", { name: "Import your outline" })).toBeTruthy();
  });

  it("stays on the format list when the file picker is cancelled", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue(null);
    render(Onboarding);
    await fireEvent.click(button(/^yWriter/));
    await waitFor(() => expect(open).toHaveBeenCalled());
    expect(invoke).not.toHaveBeenCalled();
    expect(button(/^yWriter/)).toBeTruthy();
  });

  it("explains an unreadable outline and offers another file", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue("/broken.pltr");
    vi.mocked(invoke).mockRejectedValue(new Error("Not a Plottr file"));
    render(Onboarding);

    await fireEvent.click(button(/^Plottr/));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Not a Plottr file");
    await fireEvent.click(button("Try a different file"));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("reports a failed import", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue("/outline.pltr");
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "preview_import") return preview;
      throw "disk full";
    });
    const showError = vi.spyOn(ui, "showError");
    render(Onboarding);

    await fireEvent.click(button(/^Plottr/));
    await fireEvent.click(await screen.findByTestId("guided-import-confirm"));
    await waitFor(() => expect(showError).toHaveBeenCalledWith("Import failed: disk full"));
    showError.mockRestore();
  });

  it("hands Longform to the parent when it provides a handler", async () => {
    ui.goToStep("import");
    const onImportLongform = vi.fn();
    render(Onboarding, { onImportLongform });
    await fireEvent.click(button(/^Longform/));
    expect(onImportLongform).toHaveBeenCalled();
    expect(open).not.toHaveBeenCalled();
  });

  it("falls back to the guided Longform import without a handler", async () => {
    ui.goToStep("import");
    vi.mocked(open).mockResolvedValue(null);
    render(Onboarding);
    await fireEvent.click(button(/^Longform/));
    await waitFor(() => expect(open).toHaveBeenCalled());
  });

  it("opens the sample project, or reports why it couldn't", async () => {
    vi.mocked(invoke).mockResolvedValue(project);
    render(Onboarding);
    await fireEvent.click(button("Try sample project"));
    await waitFor(() => expect(currentProject.value?.id).toBe("p1"));
    expect(invoke).toHaveBeenCalledWith("create_sample_project");
    expect(ui.showOnboarding).toBe(false);

    ui.startOnboarding();
    ui.goToStep("import");
    vi.mocked(invoke).mockRejectedValue("no space");
    const showError = vi.spyOn(ui, "showError");
    await fireEvent.click(await screen.findByRole("button", { name: "Try the sample project" }));
    await waitFor(() =>
      expect(showError).toHaveBeenCalledWith("Failed to create sample project: no space")
    );
    showError.mockRestore();
  });

  it("starts from scratch by closing onboarding", async () => {
    ui.goToStep("import");
    render(Onboarding);
    await fireEvent.click(button("Start a new project from scratch"));
    expect(ui.showOnboarding).toBe(false);
  });
});
