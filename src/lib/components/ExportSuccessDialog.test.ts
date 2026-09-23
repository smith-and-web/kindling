import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import ExportSuccessDialog from "./ExportSuccessDialog.svelte";
import type { ExportResult } from "../types";

const result: ExportResult = {
  output_path: "/Users/writer/Exports/Ember & Ash.docx",
  files_created: 1,
  chapters_exported: 12,
  scenes_exported: 1,
};

afterEach(() => {
  cleanup();
  vi.mocked(revealItemInDir).mockReset();
  vi.restoreAllMocks();
});

describe("ExportSuccessDialog", () => {
  it("shows the export counts and the saved path", () => {
    render(ExportSuccessDialog, { result, onClose: vi.fn() });

    const dialog = screen.getByRole("dialog", { name: "Export complete" });
    const counts = within(dialog)
      .getAllByRole("listitem")
      .map((item) => item.textContent?.trim());
    expect(counts).toEqual(["12 chapters", "1 scene", "1 file created"]);
    expect(within(dialog).getByText("Saved to")).toBeTruthy();
    expect(within(dialog).getByText(result.output_path)).toBeTruthy();
    expect(within(dialog).queryByRole("alert")).toBeNull();
  });

  it("omits zero chapter and scene counts but always shows files", () => {
    render(ExportSuccessDialog, {
      result: { ...result, chapters_exported: 0, scenes_exported: 0, files_created: 3 },
      onClose: vi.fn(),
    });

    const counts = screen.getAllByRole("listitem").map((item) => item.textContent?.trim());
    expect(counts).toEqual(["3 files created"]);
  });

  it("reveals the export in the file browser from Open folder", async () => {
    vi.mocked(revealItemInDir).mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(ExportSuccessDialog, { result, onClose });

    await fireEvent.click(screen.getByRole("button", { name: "Open folder" }));

    expect(revealItemInDir).toHaveBeenCalledTimes(1);
    expect(revealItemInDir).toHaveBeenCalledWith(result.output_path);
    expect(screen.queryByRole("alert")).toBeNull();
    expect(onClose).not.toHaveBeenCalled();
  });

  it("shows an error when the folder cannot be opened", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(revealItemInDir).mockRejectedValue(new Error("Path not found"));
    const onClose = vi.fn();
    render(ExportSuccessDialog, { result, onClose });

    await fireEvent.click(screen.getByRole("button", { name: "Open folder" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toBe("Couldn’t open the folder: Path not found");
    expect(onClose).not.toHaveBeenCalled();
    // The saved path stays visible so the writer can find it by hand.
    expect(screen.getByText(result.output_path)).toBeTruthy();
  });

  it("reports a non-Error rejection and clears the error on a successful retry", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(revealItemInDir).mockRejectedValueOnce("opener unavailable");
    render(ExportSuccessDialog, { result, onClose: vi.fn() });

    const open = screen.getByRole("button", { name: "Open folder" });
    await fireEvent.click(open);
    expect((await screen.findByRole("alert")).textContent).toBe(
      "Couldn’t open the folder: opener unavailable"
    );

    vi.mocked(revealItemInDir).mockResolvedValueOnce(undefined);
    await fireEvent.click(open);
    await vi.waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(revealItemInDir).toHaveBeenCalledTimes(2);
  });

  it("calls onClose from the footer Close button", async () => {
    const onClose = vi.fn();
    render(ExportSuccessDialog, { result, onClose });

    const footerClose = screen
      .getAllByRole("button", { name: "Close" })
      .find((button) => button.textContent?.trim() === "Close");
    expect(footerClose).toBeTruthy();
    await fireEvent.click(footerClose!);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(revealItemInDir).not.toHaveBeenCalled();
  });

  it("calls onClose when the backdrop itself is clicked, not the surface", async () => {
    const onClose = vi.fn();
    render(ExportSuccessDialog, { result, onClose });

    await fireEvent.click(screen.getByText(result.output_path));
    expect(onClose).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole("dialog"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
