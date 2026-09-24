import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import FeedbackDialog from "./FeedbackDialog.svelte";

const invokeMock = vi.mocked(invoke);

const expectedLocale = globalThis.navigator?.language ?? "en-US";

function renderDialog(onClose = vi.fn()) {
  render(FeedbackDialog, { props: { onClose } });
  return { onClose };
}

describe("FeedbackDialog", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("requires a message for bug feedback and does not submit", async () => {
    renderDialog();

    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const validation = await screen.findByTestId("feedback-validation");
    expect(validation.textContent).toContain("Please enter a message");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("rejects an over-long summary", async () => {
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Summary/), {
      target: { value: "a".repeat(121) },
    });
    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "valid message" },
    });

    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const validation = await screen.findByTestId("feedback-validation");
    expect(validation.textContent).toContain("120 characters or fewer");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("rejects an over-long message", async () => {
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "m".repeat(2001) },
    });

    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const validation = await screen.findByTestId("feedback-validation");
    expect(validation.textContent).toContain("2000 characters or fewer");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("requires a rating for rating feedback, then submits the rating payload", async () => {
    invokeMock.mockResolvedValue(undefined);
    renderDialog();

    await fireEvent.click(screen.getByRole("button", { name: "Rating" }));

    // Submitting with no star selected fails validation.
    await fireEvent.click(screen.getByTestId("feedback-submit"));
    expect((await screen.findByTestId("feedback-validation")).textContent).toContain(
      "select a rating"
    );
    expect(invokeMock).not.toHaveBeenCalled();

    // Picking a star and submitting sends the expected payload.
    await fireEvent.click(screen.getByRole("button", { name: "4 stars" }));
    await fireEvent.click(screen.getByTestId("feedback-submit"));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("submit_feedback", {
        input: {
          feedbackType: "rating",
          locale: expectedLocale,
          rating: 4,
        },
      });
    });
  });

  it("submits a bug report with the correct payload", async () => {
    invokeMock.mockResolvedValue(undefined);
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Summary/), {
      target: { value: "  Crash report  " },
    });
    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "  It crashed on launch  " },
    });

    await fireEvent.click(screen.getByTestId("feedback-submit"));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("submit_feedback", {
        input: {
          feedbackType: "bug",
          locale: expectedLocale,
          summary: "Crash report",
          message: "It crashed on launch",
        },
      });
    });
  });

  it("disables the submit button while sending and shows a success state", async () => {
    let resolveInvoke: (value?: unknown) => void = () => {};
    invokeMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveInvoke = resolve;
        })
    );
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "dark mode please" },
    });
    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const submit = screen.getByTestId("feedback-submit") as HTMLButtonElement;
    await waitFor(() => {
      expect(submit.disabled).toBe(true);
    });
    expect(submit.textContent).toContain("Sending");

    resolveInvoke(undefined);

    const success = await screen.findByTestId("feedback-success");
    expect(success.textContent).toContain("Thanks for your feedback");
    expect(screen.queryByTestId("feedback-error")).toBeNull();
  });

  it("renders a retryable error state when submission fails", async () => {
    // The backend rejects with the error's Display string (SubmitFeedbackError serialises as text).
    const reason = "Couldn't reach the feedback service. Check your connection and try again.";
    invokeMock.mockRejectedValueOnce(reason);
    invokeMock.mockResolvedValueOnce(undefined);
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "something broke" },
    });
    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const errorBox = await screen.findByTestId("feedback-error");
    expect(errorBox.textContent).toContain("Couldn’t send your feedback");
    expect(errorBox.textContent).toContain(reason);
    expect(errorBox.textContent).not.toContain("[object Object]");
    expect(screen.queryByTestId("feedback-success")).toBeNull();

    // The error is retryable.
    await fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    const success = await screen.findByTestId("feedback-success");
    expect(success.textContent).toContain("Thanks for your feedback");
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });

  it("says what a submission sends", () => {
    renderDialog();
    const note = screen.getByTestId("feedback-disclosure").textContent ?? "";
    expect(note).toMatch(/version, operating system and\s+language/);
    expect(note).toContain("Nothing from your manuscript");
  });

  it("closes via the Escape key", async () => {
    const { onClose } = renderDialog();
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  describe("focus", () => {
    function openFrom() {
      const opener = document.createElement("button");
      opener.textContent = "Help";
      document.body.append(opener);
      opener.focus();
      const view = { unmount: () => {} };
      const onClose = vi.fn(() => view.unmount());
      view.unmount = render(FeedbackDialog, { props: { onClose } }).unmount;
      return { opener, onClose };
    }

    it("moves focus to the selected feedback type on open", async () => {
      renderDialog();
      await waitFor(() =>
        expect(document.activeElement).toBe(screen.getByRole("button", { name: "Bug" }))
      );
    });

    it("wraps Tab from Send feedback to the first control, and Shift+Tab back", async () => {
      renderDialog();
      const close = screen.getByRole("button", { name: "Close" });
      const submit = screen.getByTestId("feedback-submit");
      submit.focus();
      await fireEvent.keyDown(submit, { key: "Tab" });
      expect(document.activeElement).toBe(close);
      await fireEvent.keyDown(close, { key: "Tab", shiftKey: true });
      expect(document.activeElement).toBe(submit);
    });

    it("keeps typing inside the dialog, closes on Escape and restores focus", async () => {
      const behind = vi.fn();
      window.addEventListener("keydown", behind);
      const { opener, onClose } = openFrom();
      const message = screen.getByLabelText(/Message/);
      message.focus();

      await fireEvent.keyDown(message, { key: "Enter" });
      await fireEvent.keyDown(message, { key: "Escape" });

      expect(behind).not.toHaveBeenCalled();
      expect(onClose).toHaveBeenCalledTimes(1);
      expect(document.activeElement).toBe(opener);
      window.removeEventListener("keydown", behind);
      opener.remove();
    });
  });
});
