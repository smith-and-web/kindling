import { afterEach, beforeEach, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import TextAlign from "@tiptap/extension-text-align";
import { KeyboardFormatting } from "./keyboardFormatting";
import { defaultBindings } from "./keyboardShortcuts";
import { shortcuts } from "../stores/shortcuts.svelte";
let editor: Editor;
const mod = /Mac/.test(navigator.platform) ? { metaKey: true } : { ctrlKey: true };
beforeEach(() => {
  shortcuts.bindings = { ...defaultBindings };
  editor = new Editor({
    extensions: [StarterKit, TextAlign.configure({ types: ["paragraph"] }), KeyboardFormatting],
    content: "<p>Words</p>",
  });
  editor.commands.setTextSelection({ from: 1, to: 6 });
});
afterEach(() => {
  editor.destroy();
});
function press(key: string, extra = {}) {
  const event = new KeyboardEvent("keydown", {
    key,
    ...mod,
    ...extra,
    bubbles: true,
    cancelable: true,
  });
  editor.view.dom.dispatchEvent(event);
  return event;
}
it("remaps formatting immediately and suppresses the old built-in key", () => {
  shortcuts.bindings.bold = "Mod+Alt+P";
  expect(press("b").defaultPrevented).toBe(true);
  expect(editor.isActive("bold")).toBe(false);
  press("p", { altKey: true });
  expect(editor.isActive("bold")).toBe(true);
  shortcuts.bindings.bold = "";
  press("p", { altKey: true });
  expect(editor.isActive("bold")).toBe(true);
});
it.each([
  "bold",
  "italic",
  "underline",
  "strike",
  "code",
  "blockquote",
  "align_left",
  "align_center",
  "align_right",
  "align_justify",
])("executes %s through its custom binding", (id) => {
  shortcuts.bindings[id] = "Mod+Alt+P";
  press("p", { altKey: true });
  expect(
    id.startsWith("align_") ? editor.isActive({ textAlign: id.slice(6) }) : editor.isActive(id)
  ).toBe(true);
});
it("does not mutate read-only prose or toggle twice on a held key", () => {
  editor.setEditable(false);
  press("b");
  expect(editor.isActive("bold")).toBe(false);
  editor.setEditable(true);
  press("b", { repeat: true });
  expect(editor.isActive("bold")).toBe(false);
  expect(press("Escape").defaultPrevented).toBe(false);
});

it.each(["B", "I", "U", "S"])(
  "suppresses legacy shifted %s aliases after clearing formatting",
  (key) => {
    for (const id of [
      "bold",
      "italic",
      "underline",
      "strike",
      "blockquote",
      "sync",
      "import_scrivener",
    ])
      shortcuts.bindings[id] = "";
    press(key, { shiftKey: true });
    expect(editor.getHTML()).toBe("<p>Words</p>");
  }
);
