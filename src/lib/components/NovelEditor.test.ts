import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import NovelEditor from "./NovelEditor.svelte";

afterEach(cleanup);

it("names the page editor as the scene's prose", async () => {
  render(NovelEditor, { content: "<p>Opening line.</p>", sceneId: "scene" });
  const editor = await screen.findByRole("textbox", { name: "Scene prose" });
  expect(editor.classList.contains("ProseMirror")).toBe(true);
  expect(editor.getAttribute("aria-multiline")).toBe("true");
  expect(editor.getAttribute("aria-readonly")).toBe("false");
});

it("names a beat's editor as the beat's prose", async () => {
  render(NovelEditor, { content: "<p>Beat line.</p>", sceneId: "scene", beatId: "beat" });
  expect(await screen.findByRole("textbox", { name: "Beat prose" })).toBeTruthy();
  expect(screen.queryByRole("textbox", { name: "Scene prose" })).toBeNull();
});

it("keeps its name and reports read-only when a scene locks", async () => {
  const view = render(NovelEditor, { content: "<p>Line.</p>", sceneId: "scene" });
  const editor = await screen.findByRole("textbox", { name: "Scene prose" });
  await view.rerender({ content: "<p>Line.</p>", sceneId: "scene", readonly: true });
  await waitFor(() => expect(editor.getAttribute("aria-readonly")).toBe("true"));
  expect(editor.getAttribute("contenteditable")).toBe("false");
  expect(editor.getAttribute("aria-label")).toBe("Scene prose");
});

it("keeps the save slot mounted so the indicator never changes the toolbar layout", async () => {
  const props = { content: "<p>Line.</p>", sceneId: "scene", beatId: "beat" } as const;
  const view = render(NovelEditor, props);
  await screen.findByRole("textbox", { name: "Beat prose" });

  const slot = screen.getByTestId("save-slot");
  expect(screen.queryByTestId("save-indicator")).toBeNull();

  await view.rerender({ ...props, saveStatus: "saving" });
  expect(screen.getByTestId("save-indicator")).toBeTruthy();
  expect(screen.getByTestId("save-slot")).toBe(slot);

  await view.rerender({ ...props, saveStatus: "idle" });
  expect(screen.queryByTestId("save-indicator")).toBeNull();
  expect(screen.getByTestId("save-slot")).toBe(slot);

  await view.rerender({ ...props, saveStatus: "error" });
  expect(screen.getByRole("alert")).toBeTruthy();
  expect(screen.getByTestId("save-slot")).toBe(slot);
});

// The slot must claim no row width of its own (#426 reserved 5rem, which opened an
// empty second toolbar row on panes the buttons alone fit). jsdom has no layout, so
// this checks the component's own rules for the slot and the indicator over it.
it("places the save slot at the end of the toolbar row without claiming width", async () => {
  const props = { content: "<p>Line.</p>", sceneId: "scene", beatId: "beat" } as const;
  const view = render(NovelEditor, props);
  await screen.findByRole("textbox", { name: "Beat prose" });

  const toolbar = screen.getByRole("toolbar");
  const slot = screen.getByTestId("save-slot");
  const extra = slot.parentElement!;
  expect(extra.classList.contains("extra")).toBe(true);
  expect(extra.parentElement).toBe(toolbar);
  expect(toolbar.lastElementChild).toBe(extra);
  expect(extra.lastElementChild).toBe(slot);
  expect(slot.childElementCount).toBe(0);

  await view.rerender({ ...props, saveStatus: "saving" });
  const indicator = screen.getByTestId("save-indicator");
  expect(indicator.parentElement).toBe(slot);
  expect(indicator.textContent).toContain("Saving…");

  // A save error is lasting and must be readable, so it is laid out in the row
  // beside the slot rather than squeezed into it.
  await view.rerender({ ...props, saveStatus: "error" });
  const alert = screen.getByRole("alert");
  expect(slot.contains(alert)).toBe(false);
  expect(alert.parentElement).toBe(extra);
  expect(slot.childElementCount).toBe(0);

  const fsModule = "node:fs";
  const { readFileSync } = await import(/* @vite-ignore */ fsModule);
  const urlModule = "node:url";
  const { fileURLToPath } = await import(/* @vite-ignore */ urlModule);
  const componentPath = "./NovelEditor.svelte";
  const source: string = readFileSync(
    fileURLToPath(new URL(componentPath, import.meta.url)),
    "utf8"
  );
  const rule = (selector: string): Record<string, string> => {
    const start = source.indexOf(`\n  ${selector} {`);
    expect(start, selector).toBeGreaterThan(-1);
    const body = source.slice(source.indexOf("{", start) + 1, source.indexOf("}", start));
    return Object.fromEntries(
      body
        .split(";")
        .map((line) => line.trim())
        .filter((line) => line && !line.startsWith("/*"))
        .map((line) => line.split(/:\s*/, 2) as [string, string])
    );
  };

  const slotRule = rule(".save-slot");
  expect(slotRule.flex).toBe("1 1 0");
  expect(slotRule["min-width"]).toBe("0");
  expect(slotRule).not.toHaveProperty("width");
  expect(slotRule).not.toHaveProperty("min-height");

  // The trailing area grows into the row's leftover space but has no footprint of its
  // own: no padding, and a start margin that cancels the toolbar gap before it.
  const extraRule = rule(".novel-editor :global(.prose-toolbar .extra)");
  expect(extraRule.flex).toBe("1 1 0");
  expect(extraRule["padding-inline"]).toBe("0");
  expect(extraRule["margin-left"]).toBe("calc(-1 * var(--space-2xs))");

  // The indicator is drawn over the slot, one line tall, dropping its label to a
  // clipped second line when the row is short of room.
  const indicatorRule = rule(".save-status.saving");
  expect(indicatorRule.position).toBe("absolute");
  expect(indicatorRule.inset).toBe("0");
  expect(indicatorRule.height).toBe("calc(var(--text-small) * 1.5)");
  expect(indicatorRule.overflow).toBe("hidden");
  expect(indicatorRule["flex-wrap"]).toBe("wrap");
  expect(slotRule["container-type"]).toBe("inline-size");
});
