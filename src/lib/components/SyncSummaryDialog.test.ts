import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import SyncSummaryDialog from "./SyncSummaryDialog.svelte";
import type { ReimportSummary } from "../types";

function summary(overrides: Partial<ReimportSummary> = {}): ReimportSummary {
  return {
    chapters_added: 0,
    chapters_updated: 0,
    scenes_added: 0,
    scenes_updated: 0,
    beats_added: 0,
    beats_updated: 0,
    prose_preserved: 0,
    prose_updated: 0,
    ...overrides,
  };
}

afterEach(() => {
  cleanup();
});

describe("SyncSummaryDialog", () => {
  it("renders totals as badges and one row per changed kind", () => {
    render(SyncSummaryDialog, {
      summary: summary({
        chapters_added: 1,
        chapters_updated: 2,
        scenes_added: 3,
        beats_updated: 4,
        prose_updated: 1,
        prose_preserved: 5,
      }),
      onClose: vi.fn(),
    });

    const dialog = screen.getByRole("dialog", { name: "Sync complete" });
    const body = within(dialog).getByTestId("sync-summary");

    // Added = 1 + 3 + 0; updated = 2 + 0 + 4 + 1 prose.
    expect(within(body).getByText("4 added")).toBeTruthy();
    expect(within(body).getByText("7 updated")).toBeTruthy();
    expect(within(body).getByText("5 preserved")).toBeTruthy();

    const rows = within(body).getAllByRole("listitem");
    expect(rows.map((row) => row.textContent?.replace(/\s+/g, " ").trim())).toEqual([
      "Chapters 1 added, 2 updated",
      "Scenes 3 added, 0 updated",
      "Beats 0 added, 4 updated",
      "Prose 1 prose item updated 5 prose items preserved",
    ]);
    expect(within(body).queryByText("Nothing changed")).toBeNull();
  });

  it("omits kinds and badges with nothing to report", () => {
    render(SyncSummaryDialog, {
      summary: summary({ scenes_updated: 2 }),
      onClose: vi.fn(),
    });

    const body = screen.getByTestId("sync-summary");
    expect(within(body).getByText("2 updated")).toBeTruthy();
    expect(within(body).queryByText(/added$/)).toBeNull();
    expect(within(body).queryByText(/preserved/)).toBeNull();
    expect(within(body).queryByText("Chapters")).toBeNull();
    expect(within(body).queryByText("Prose")).toBeNull();
    expect(within(body).getAllByRole("listitem")).toHaveLength(1);
  });

  it("shows an empty state when nothing changed", () => {
    render(SyncSummaryDialog, { summary: summary(), onClose: vi.fn() });

    const body = screen.getByTestId("sync-summary");
    expect(within(body).getByRole("heading", { name: "Nothing changed" })).toBeTruthy();
    expect(within(body).getByText("No changes were applied.")).toBeTruthy();
    expect(within(body).queryByRole("list")).toBeNull();
  });

  it("treats preserved-only prose as no changes", () => {
    render(SyncSummaryDialog, { summary: summary({ prose_preserved: 3 }), onClose: vi.fn() });

    expect(screen.getByText("Nothing changed")).toBeTruthy();
  });

  it("calls onClose from Done", async () => {
    const onClose = vi.fn();
    render(SyncSummaryDialog, { summary: summary({ beats_added: 1 }), onClose });

    await fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("calls onClose from the header close button", async () => {
    const onClose = vi.fn();
    render(SyncSummaryDialog, { summary: summary(), onClose });

    const close = screen.getByTestId("dialog-close");
    expect(close.getAttribute("aria-label")).toBe("Close Sync complete");
    await fireEvent.click(close);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
