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
        feedbackType: "rating",
        locale: expectedLocale,
        rating: 4,
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
        feedbackType: "bug",
        locale: expectedLocale,
        summary: "Crash report",
        message: "It crashed on launch",
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
    invokeMock.mockRejectedValueOnce(new Error("network error: offline"));
    invokeMock.mockResolvedValueOnce(undefined);
    renderDialog();

    await fireEvent.input(screen.getByLabelText(/Message/), {
      target: { value: "something broke" },
    });
    await fireEvent.click(screen.getByTestId("feedback-submit"));

    const errorBox = await screen.findByTestId("feedback-error");
    expect(errorBox.textContent).toContain("Couldn't send your feedback");
    expect(screen.queryByTestId("feedback-success")).toBeNull();

    // The error is retryable.
    await fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    const success = await screen.findByTestId("feedback-success");
    expect(success.textContent).toContain("Thanks for your feedback");
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });

  it("closes via the Escape key", async () => {
    const { onClose } = renderDialog();
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
