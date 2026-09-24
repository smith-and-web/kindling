import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import RenameDialog from "./RenameDialog.svelte";

afterEach(cleanup);

it("shows the backend's reason when a rename is refused", async () => {
  // Tauri rejects with the Rust error string, not an Error.
  const onSave = vi.fn().mockRejectedValue("This scene is locked. Unlock it to rename it.");
  render(RenameDialog, {
    props: { title: "Rename scene", currentName: "Opening", onSave, onClose: vi.fn() },
  });
  await fireEvent.input(screen.getByRole("textbox"), { target: { value: "Cold open" } });
  await fireEvent.click(screen.getByRole("button", { name: /^(Rename|Save)$/ }));
  expect(await screen.findByText("This scene is locked. Unlock it to rename it.")).toBeTruthy();
});
