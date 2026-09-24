import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import QuickStartDialog from "./QuickStartDialog.svelte";
import { tabbableElements } from "../utils/modalFocus";

function open() {
  const opener = document.createElement("button");
  opener.textContent = "Help";
  document.body.append(opener);
  opener.focus();
  const view = { unmount: () => {} };
  const onClose = vi.fn(() => view.unmount());
  view.unmount = render(QuickStartDialog, { props: { onClose } }).unmount;
  return { opener, onClose };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("QuickStartDialog focus", () => {
  it("moves focus inside on open and keeps Tab inside", async () => {
    open();
    const close = screen.getByTestId("quick-start-close");
    await waitFor(() => expect(document.activeElement).toBe(close));
    const items = tabbableElements(close.closest<HTMLElement>('[role="dialog"]')!);
    const last = items[items.length - 1];
    last.focus();
    await fireEvent.keyDown(last, { key: "Tab" });
    expect(document.activeElement).toBe(items[0]);
    await fireEvent.keyDown(items[0], { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("closes on Escape, restores focus, and keeps the key from the window behind", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    const { opener, onClose } = open();
    await waitFor(() =>
      expect(document.activeElement).toBe(screen.getByTestId("quick-start-close"))
    );

    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(behind).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(opener);
    window.removeEventListener("keydown", behind);
  });
});
