import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ImportLongformDialog from "./ImportLongformDialog.svelte";

function setup() {
  const props = { onSelectIndex: vi.fn(), onSelectVault: vi.fn(), onClose: vi.fn() };
  render(ImportLongformDialog, props);
  return props;
}

afterEach(() => {
  cleanup();
});

describe("ImportLongformDialog", () => {
  it("renders a labelled dialog with both import choices", () => {
    setup();

    expect(screen.getByRole("dialog", { name: "Import Longform project" })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Choose Longform index file/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Choose Obsidian vault folder/ })).toBeTruthy();
  });

  it("calls onSelectIndex from the index file choice only", async () => {
    const props = setup();

    await fireEvent.click(screen.getByRole("button", { name: /Choose Longform index file/ }));

    expect(props.onSelectIndex).toHaveBeenCalledTimes(1);
    expect(props.onSelectVault).not.toHaveBeenCalled();
    expect(props.onClose).not.toHaveBeenCalled();
  });

  it("calls onSelectVault from the vault folder choice only", async () => {
    const props = setup();

    await fireEvent.click(screen.getByRole("button", { name: /Choose Obsidian vault folder/ }));

    expect(props.onSelectVault).toHaveBeenCalledTimes(1);
    expect(props.onSelectIndex).not.toHaveBeenCalled();
    expect(props.onClose).not.toHaveBeenCalled();
  });

  it("closes from Cancel", async () => {
    const props = setup();

    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(props.onClose).toHaveBeenCalledTimes(1);
    expect(props.onSelectIndex).not.toHaveBeenCalled();
    expect(props.onSelectVault).not.toHaveBeenCalled();
  });

  it("closes from the header close button", async () => {
    const props = setup();

    await fireEvent.click(screen.getByRole("button", { name: "Close Import Longform project" }));

    expect(props.onClose).toHaveBeenCalledTimes(1);
  });

  it("closes once on Escape and ignores other keys", async () => {
    const props = setup();

    await fireEvent.keyDown(window, { key: "Enter" });
    expect(props.onClose).not.toHaveBeenCalled();

    await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(props.onClose).toHaveBeenCalledTimes(1);
  });
});
