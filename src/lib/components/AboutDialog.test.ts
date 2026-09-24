import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import AboutDialog from "./AboutDialog.svelte";

function open() {
  const opener = document.createElement("button");
  opener.textContent = "Help";
  document.body.append(opener);
  opener.focus();
  const view = { unmount: () => {} };
  const onClose = vi.fn(() => view.unmount());
  const onSendFeedback = vi.fn();
  view.unmount = render(AboutDialog, { props: { onClose, onSendFeedback } }).unmount;
  return { opener, onClose, onSendFeedback };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("AboutDialog focus", () => {
  it("moves focus inside on open", async () => {
    open();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("about-close")));
  });

  it("wraps Tab from the last control to the first, and Shift+Tab back", async () => {
    open();
    const close = screen.getByTestId("about-close");
    await waitFor(() => expect(document.activeElement).toBe(close));
    const last = screen.getByRole("button", { name: "Release notes" });
    last.focus();
    await fireEvent.keyDown(last, { key: "Tab" });
    expect(document.activeElement).toBe(close);
    await fireEvent.keyDown(close, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("closes on Escape, restores focus, and keeps keys from the window behind", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    const { opener, onClose } = open();
    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("about-close")));

    await fireEvent.keyDown(document.activeElement!, { key: "a" });
    await fireEvent.keyDown(document.activeElement!, { key: "Escape" });

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(behind).not.toHaveBeenCalled();
    expect(screen.queryByTestId("about-dialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
    window.removeEventListener("keydown", behind);
  });

  it("pulls focus back if the page behind takes it", async () => {
    const { opener } = open();
    const close = screen.getByTestId("about-close");
    await waitFor(() => expect(document.activeElement).toBe(close));
    opener.focus();
    expect(document.activeElement).toBe(close);
  });
});
