import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ScrivenerMatchDialog from "./ScrivenerMatchDialog.svelte";
import type { ScrivenerMatchPreview } from "../types";

const matches: ScrivenerMatchPreview[] = [
  {
    scene_id: "s1",
    scene_title: "The Village",
    chapter_title: "The Awakening",
    matched_scriv_title: "Village (draft 2)",
    match_method: "source_id",
  },
  {
    scene_id: "s2",
    scene_title: "The Stranger",
    chapter_title: "The Awakening",
    matched_scriv_title: "The Stranger",
    match_method: "title",
  },
  {
    scene_id: "s3",
    scene_title: "Ashes",
    chapter_title: "The Fall",
    matched_scriv_title: null,
    match_method: null,
  },
];

function setup() {
  const props = {
    projectId: "p1",
    scrivPath: "/Users/writer/Novel.scriv",
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
  };
  render(ScrivenerMatchDialog, props);
  return props;
}

function proceed() {
  return screen.getByRole("button", { name: "Proceed with export" }) as HTMLButtonElement;
}

afterEach(() => {
  cleanup();
  vi.mocked(invoke).mockReset();
});

describe("ScrivenerMatchDialog", () => {
  it("shows a loading state and disables Proceed while matches are analysed", async () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    setup();

    expect(screen.getByRole("dialog", { name: "Scrivener match preview" })).toBeTruthy();
    expect(screen.getByRole("status").textContent).toContain("Analyzing matches…");
    expect(screen.getByRole("progressbar", { name: "Analyzing matches" })).toBeTruthy();
    expect(proceed().disabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith("preview_scrivener_matches", {
      projectId: "p1",
      scrivPath: "/Users/writer/Novel.scriv",
    });
  });

  it("shows an error notice and keeps Proceed disabled when the preview fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Not a Scrivener project"));
    setup();

    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("Couldn’t preview the Scrivener matches")).toBeTruthy();
    expect(within(alert).getByText("Not a Scrivener project")).toBeTruthy();
    expect(screen.queryByRole("status")).toBeNull();
    expect(proceed().disabled).toBe(true);
  });

  it("shows a string rejection as the error detail", async () => {
    vi.mocked(invoke).mockRejectedValue("scrivx missing");
    setup();

    const alert = await screen.findByRole("alert");
    expect(within(alert).getByText("scrivx missing")).toBeTruthy();
  });

  it("shows an empty state when there are no scenes to match", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    setup();

    expect(await screen.findByRole("heading", { name: "No scenes to match" })).toBeTruthy();
    expect(screen.queryByRole("list")).toBeNull();
    expect(proceed().disabled).toBe(false);
  });

  it("lists matched and unmatched scenes with badges and a summary", async () => {
    vi.mocked(invoke).mockResolvedValue(matches);
    setup();

    const list = await screen.findByRole("list");
    expect(screen.getByText("2 matched")).toBeTruthy();
    expect(screen.getByText("1 unmatched (will be created)")).toBeTruthy();

    const rows = within(list).getAllByRole("listitem");
    expect(rows).toHaveLength(3);

    expect(within(rows[0]).getByText("Matched")).toBeTruthy();
    expect(within(rows[0]).getByText("The Awakening · The Village")).toBeTruthy();
    expect(within(rows[0]).getByText("→ Village (draft 2) (via ID)")).toBeTruthy();

    expect(within(rows[1]).getByText("Matched")).toBeTruthy();
    expect(within(rows[1]).getByText("→ The Stranger (via title)")).toBeTruthy();

    expect(within(rows[2]).getByText("New")).toBeTruthy();
    expect(within(rows[2]).getByText("The Fall · Ashes")).toBeTruthy();
    expect(within(rows[2]).getByText("New document")).toBeTruthy();

    expect(proceed().disabled).toBe(false);
  });

  it("hides the unmatched badge when every scene matched", async () => {
    vi.mocked(invoke).mockResolvedValue(matches.slice(0, 2));
    setup();

    expect(await screen.findByText("2 matched")).toBeTruthy();
    expect(screen.queryByText(/unmatched/)).toBeNull();
  });

  it("calls onConfirm from Proceed with export", async () => {
    vi.mocked(invoke).mockResolvedValue(matches);
    const props = setup();
    await screen.findByRole("list");

    await fireEvent.click(proceed());

    expect(props.onConfirm).toHaveBeenCalledTimes(1);
    expect(props.onCancel).not.toHaveBeenCalled();
  });

  it("calls onCancel from Cancel and from the header close button", async () => {
    vi.mocked(invoke).mockResolvedValue(matches);
    const props = setup();
    await screen.findByRole("list");

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(props.onCancel).toHaveBeenCalledTimes(1);

    await fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(props.onCancel).toHaveBeenCalledTimes(2);
    expect(props.onConfirm).not.toHaveBeenCalled();
  });

  it("does not cancel when clicking inside the dialog surface", async () => {
    vi.mocked(invoke).mockResolvedValue(matches);
    const props = setup();
    await screen.findByRole("list");

    await fireEvent.click(screen.getByText("New document"));
    expect(props.onCancel).not.toHaveBeenCalled();
  });
});
