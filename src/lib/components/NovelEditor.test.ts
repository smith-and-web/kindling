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
