import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { mockProject } from "../../dev/mock-data";
import { currentProject } from "../stores/project.svelte";
import ClassicExportDialog from "./ClassicExportDialog.svelte";

const values = new Map<string, string>();

beforeEach(() => {
  values.clear();
  // A remembered folder makes Markdown exportable without a file picker.
  values.set("kindling:lastExportPath", "/tmp/exports");
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
  currentProject.setProject(mockProject);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command: string) =>
    command === "get_project_word_count" ? 1200 : { files_created: 1, output_path: "/tmp/exports" }
  );
});

afterEach(() => {
  currentProject.setProject(null);
  vi.unstubAllGlobals();
  document.querySelectorAll("[data-opener]").forEach((el) => el.remove());
});

function open() {
  const opener = document.createElement("button");
  opener.dataset.opener = "";
  opener.textContent = "Manuscript";
  document.body.append(opener);
  opener.focus();
  const view = { unmount: () => {} };
  const onClose = vi.fn(() => view.unmount());
  const onSuccess = vi.fn();
  view.unmount = render(ClassicExportDialog, {
    props: {
      scope: "project",
      scopeId: null,
      scopeTitle: mockProject.name,
      onClose,
      onSuccess,
      onCustomize: vi.fn(),
    },
  }).unmount;
  return { opener, onClose, onSuccess };
}

const exported = () =>
  vi.mocked(invoke).mock.calls.filter(([command]) => String(command).startsWith("export_to"));

async function chooseMarkdown() {
  const markdown = screen.getByTestId("export-format-markdown") as HTMLInputElement;
  await fireEvent.click(markdown);
  await waitFor(() =>
    expect((screen.getByTestId("export-confirm") as HTMLButtonElement).disabled).toBe(false)
  );
  return markdown;
}

describe("ClassicExportDialog focus and keys", () => {
  it("moves focus to the chosen format on open", async () => {
    open();
    await waitFor(() =>
      expect(document.activeElement).toBe(screen.getByTestId("export-format-docx"))
    );
  });

  it("does not export when Enter is pressed on Cancel", async () => {
    open();
    await chooseMarkdown();
    const cancel = screen.getByRole("button", { name: "Cancel" });
    cancel.focus();
    await fireEvent.keyDown(cancel, { key: "Enter" });
    expect(exported()).toEqual([]);
  });

  it("exports when Enter is pressed on a format", async () => {
    const { onSuccess } = open();
    const markdown = await chooseMarkdown();
    markdown.focus();
    await fireEvent.keyDown(markdown, { key: "Enter" });
    await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
    expect(exported().map(([command]) => command)).toEqual(["export_to_markdown"]);
  });

  it("wraps Tab from the last control back to the first", async () => {
    open();
    await chooseMarkdown();
    const confirm = screen.getByTestId("export-confirm");
    confirm.focus();
    await fireEvent.keyDown(confirm, { key: "Tab" });
    expect(document.activeElement).toBe(screen.getByTestId("export-close"));
    await fireEvent.keyDown(document.activeElement!, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(confirm);
  });

  it("keeps typed keys and Escape from reaching the page behind, and restores focus", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    const { opener, onClose } = open();
    await waitFor(() =>
      expect(document.activeElement).toBe(screen.getByTestId("export-format-docx"))
    );

    // The editor behind is where focus was; it must not see these.
    await fireEvent.keyDown(document.activeElement!, { key: "x" });
    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });

    expect(behind).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(opener);
    window.removeEventListener("keydown", behind);
  });

  it("pulls focus back in if the page behind grabs it", async () => {
    const { opener } = open();
    const docx = screen.getByTestId("export-format-docx");
    await waitFor(() => expect(document.activeElement).toBe(docx));
    opener.focus();
    expect(document.activeElement).toBe(docx);
  });
});
