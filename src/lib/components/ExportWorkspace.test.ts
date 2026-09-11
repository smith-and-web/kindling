import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { mockProject } from "../../dev/mock-data";
import { currentProject } from "../stores/project.svelte";
import { decodeProfiles, profileStorageKey } from "../utils/exportPrototype";
import ExportWorkspace from "./ExportWorkspace.svelte";
import ClassicExportDialog from "./ClassicExportDialog.svelte";

const key = profileStorageKey(mockProject.id);
const values = new Map<string, string>();
const chapters = ["a", "b"].map((id) => ({
  id,
  title: `Chapter ${id}`,
  part: null,
  scenes: [
    {
      id: `scene-${id}`,
      title: `Scene ${id}`,
      synopsis: null,
      blocks: [{ heading: null, html: "<p>Saved prose.</p>" }],
    },
  ],
}));
beforeEach(() => {
  values.clear();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
  window.structuredClone = structuredClone;
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
  currentProject.setProject(mockProject);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(chapters);
});
afterEach(() => {
  cleanup();
  currentProject.setProject(null);
  vi.unstubAllGlobals();
});

async function mount(scope: "project" | "chapter" | "scene" = "project") {
  const app = render(ExportWorkspace, {
    scope,
    scopeId: scope === "scene" ? "scene-a" : scope === "chapter" ? "a" : null,
    onClose: vi.fn(),
    onClassic: vi.fn(),
  });
  await waitFor(() =>
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_export_prototype_document", {
      projectId: mockProject.id,
    })
  );
  await tick();
  return app;
}
async function clearSize() {
  await fireEvent.click(screen.getByRole("button", { name: /Text & page/ }));
  await fireEvent.input(screen.getByLabelText("Size (pt)"), { target: { value: "" } });
}
function saved() {
  return decodeProfiles(values.get(key)!);
}

it("keeps the saved profile collection readable after an incomplete duplicate attempt", async () => {
  const app = await mount();
  await clearSize();
  await fireEvent.click(screen.getByRole("button", { name: "Duplicate profile" }));
  expect(saved().profiles).toHaveLength(3);
  expect(screen.getByText(/Could not duplicate profile/)).toBeTruthy();
  app.unmount();
  await mount();
  await fireEvent.click(screen.getByRole("button", { name: /Text & page/ }));
  expect((screen.getByLabelText("Size (pt)") as HTMLInputElement).value).toBe("12");
});

it.each(["markdown", "novelwriter"])(
  "saves a readable %s profile after hiding an incomplete typography field",
  async (format) => {
    const app = await mount();
    await clearSize();
    await fireEvent.change(screen.getByLabelText("Output format"), { target: { value: format } });
    await fireEvent.click(screen.getByRole("button", { name: "Save profile" }));
    expect(saved().profiles[0]).toMatchObject({ format, fontSize: 12 });
    await fireEvent.click(screen.getByRole("button", { name: "Duplicate profile" }));
    expect(saved().profiles).toHaveLength(4);
    const active = saved().activeId;
    app.unmount();
    await mount();
    expect((screen.getByLabelText("Export profile") as HTMLSelectElement).value).toBe(active);
  }
);

it.each(["chapter", "scene"] as const)(
  "keeps unsaved %s context out of later project exports",
  async (scope) => {
    const app = await mount(scope);
    await fireEvent.click(screen.getByRole("button", { name: /Content.*Choose what goes in/ }));
    expect((screen.getByLabelText("Include in this export") as HTMLSelectElement).value).toBe(
      "selected"
    );
    app.unmount();
    await mount();
    await fireEvent.click(screen.getByRole("button", { name: /Content.*Choose what goes in/ }));
    expect((screen.getByLabelText("Include in this export") as HTMLSelectElement).value).toBe(
      "all"
    );
    expect(saved().profiles[0].sceneId).toBeNull();
  }
);

it("only persists contextual scope when explicitly saved, including across profile switches", async () => {
  let app = await mount("chapter");
  await fireEvent.change(screen.getByLabelText("Export profile"), { target: { value: "readers" } });
  expect(saved().profiles[0].selection).toBe("all");
  app.unmount();
  app = await mount("scene");
  await fireEvent.click(screen.getByRole("button", { name: "Save profile" }));
  expect(saved().profiles.find((p) => p.id === "readers")).toMatchObject({
    selection: "selected",
    sceneId: "scene-a",
  });
  app.unmount();
  await mount();
  await fireEvent.click(screen.getByRole("button", { name: /Content.*Choose what goes in/ }));
  expect(screen.getByText(/Only the selected scene is included/)).toBeTruthy();
});

it("remembers Custom and the chosen profile across the standard dialog and workspace", async () => {
  const props = {
    scope: "project" as const,
    scopeId: null,
    scopeTitle: mockProject.name,
    onClose: vi.fn(),
    onSuccess: vi.fn(),
    onCustomize: vi.fn(),
  };
  let app = render(ClassicExportDialog, props);
  await tick();
  await fireEvent.click(screen.getByTestId("export-format-custom"));
  await fireEvent.change(screen.getByLabelText("Export profile"), { target: { value: "website" } });
  app.unmount();
  const workspace = await mount();
  expect((screen.getByLabelText("Export profile") as HTMLSelectElement).value).toBe("website");
  await fireEvent.change(screen.getByLabelText("Export profile"), { target: { value: "readers" } });
  workspace.unmount();
  app = render(ClassicExportDialog, props);
  await tick();
  expect((screen.getByTestId("export-format-custom") as HTMLInputElement).checked).toBe(true);
  expect((screen.getByLabelText("Export profile") as HTMLSelectElement).value).toBe("readers");
  await fireEvent.click(screen.getByTestId("export-format-docx"));
  app.unmount();
  render(ClassicExportDialog, props);
  await tick();
  expect((screen.getByTestId("export-format-custom") as HTMLInputElement).checked).toBe(true);
});

it("exports EPUB without first saving when an unfinished Word margin becomes hidden", async () => {
  await mount();
  await fireEvent.click(screen.getByRole("button", { name: /Text & page/ }));
  await fireEvent.input(screen.getByLabelText("Margins (in)"), { target: { value: "" } });
  await fireEvent.change(screen.getByLabelText("Output format"), { target: { value: "epub" } });
  vi.mocked(save).mockResolvedValueOnce("/tmp/book.epub");
  await fireEvent.click(screen.getByRole("button", { name: "Export EPUB" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith(
      "export_workspace_document",
      expect.objectContaining({
        format: "epub",
        document: expect.objectContaining({ layout: expect.objectContaining({ margin: 1 }) }),
      })
    )
  );
  await fireEvent.click(screen.getByRole("button", { name: "Save profile" }));
  expect(saved().profiles[0].margin).toBe(1);
});
it("saves title-only headings after the unfinished start number is hidden", async () => {
  await mount();
  await fireEvent.click(screen.getByRole("button", { name: /Headings & breaks/ }));
  await fireEvent.input(screen.getByLabelText("Start at"), { target: { value: "" } });
  await fireEvent.change(screen.getByLabelText("Chapter heading"), { target: { value: "title" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save profile" }));
  expect(saved().profiles[0]).toMatchObject({ chapterHeading: "title", startNumber: 1 });
  await fireEvent.change(screen.getByLabelText("Export profile"), { target: { value: "readers" } });
  expect((screen.getByLabelText("Export profile") as HTMLSelectElement).value).toBe("readers");
});

it.each(["markdown", "txt"])(
  "keeps the %s preview usable while visible settings are incomplete",
  async (format) => {
    await mount();
    await fireEvent.change(screen.getByLabelText("Output format"), { target: { value: format } });
    await fireEvent.input(screen.getByLabelText("Profile name"), { target: { value: "" } });
    expect(
      (screen.getByRole("button", { name: "Save profile" }) as HTMLButtonElement).disabled
    ).toBe(true);
    expect((document.querySelector(".source-preview") as HTMLTextAreaElement).value).toContain(
      "Saved prose"
    );
    await fireEvent.input(screen.getByLabelText("Profile name"), {
      target: { value: "My profile" },
    });
    await fireEvent.click(screen.getByRole("button", { name: /Headings & breaks/ }));
    await fireEvent.input(screen.getByLabelText("Start at"), { target: { value: "" } });
    expect(
      (screen.getByRole("button", { name: "Save profile" }) as HTMLButtonElement).disabled
    ).toBe(true);
    expect((document.querySelector(".source-preview") as HTMLTextAreaElement).value).toContain(
      "Saved prose"
    );
    await fireEvent.input(screen.getByLabelText("Start at"), { target: { value: "2" } });
    expect(
      (screen.getByRole("button", { name: "Save profile" }) as HTMLButtonElement).disabled
    ).toBe(false);
    expect((document.querySelector(".source-preview") as HTMLTextAreaElement).value).toContain(
      "Chapter 2"
    );
  }
);
