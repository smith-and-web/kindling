import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";
import ImportProgressDialog from "./ImportProgressDialog.svelte";
import { ui } from "../stores/ui.svelte";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

beforeAll(() => {
  // jsdom has no top layer; model showModal/close on the open attribute.
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
});

afterEach(() => {
  ui.finishImport();
  cleanup();
});

describe("ImportProgressDialog", () => {
  it("renders nothing while idle", () => {
    render(ImportProgressDialog);
    expect(screen.queryByTestId("import-progress")).toBeNull();
  });

  it("shows the named work, its status and a visible percentage, with no close control", async () => {
    render(ImportProgressDialog);
    ui.startImport("Opening sample project", "Creating the sample project…");
    ui.updateImportProgress(42.4, "Creating scenes");
    await tick();

    const dialog = screen.getByRole("dialog", { name: "Opening sample project" });
    expect(dialog.hasAttribute("open")).toBe(true);
    expect(within(dialog).getByRole("status").textContent).toContain("Creating scenes");
    expect(dialog.textContent).toContain("42%");
    expect(dialog.querySelector("progress")?.getAttribute("value")).toBe("42.4");
    expect(within(dialog).queryByRole("button")).toBeNull();
  });

  it("cannot be dismissed with Escape", async () => {
    render(ImportProgressDialog);
    ui.startImport();
    await tick();
    const dialog = screen.getByTestId("import-progress");
    const cancel = new Event("cancel", { cancelable: true });
    dialog.dispatchEvent(cancel);
    expect(cancel.defaultPrevented).toBe(true);
  });

  it("closes and returns focus when the work finishes", async () => {
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    render(ImportProgressDialog);
    ui.startImport();
    await tick();
    ui.finishImport();
    await tick();
    expect(screen.queryByTestId("import-progress")).toBeNull();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });
});
