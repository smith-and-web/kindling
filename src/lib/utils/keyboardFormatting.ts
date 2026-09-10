import { Extension } from "@tiptap/core";
import { Plugin } from "@tiptap/pm/state";
import { shortcuts } from "../stores/shortcuts.svelte";
import { shortcutFromEvent } from "./keyboardShortcuts";

// Suppress built-in formatting keys after reassignment, including defaults that
// collide with Kindling's native commands (code/export and align/import/panels).
const builtIn = new Set([
  "Mod+B",
  "Mod+I",
  "Mod+U",
  "Mod+Shift+X",
  "Mod+Shift+S",
  "Mod+Shift+I",
  "Mod+Shift+U",
  "Mod+E",
  "Mod+Shift+B",
  "Mod+Shift+L",
  "Mod+Shift+E",
  "Mod+Shift+R",
  "Mod+Shift+J",
]);
export const KeyboardFormatting = Extension.create({
  name: "keyboardFormatting",
  priority: 1000,
  addProseMirrorPlugins() {
    const editor = this.editor;
    return [
      new Plugin({
        props: {
          handleKeyDown: (_view, event) => {
            const id = shortcuts.match(event);
            const commands: Record<string, () => boolean> = {
              bold: () => editor.commands.toggleBold(),
              italic: () => editor.commands.toggleItalic(),
              underline: () => editor.commands.toggleUnderline(),
              strike: () => editor.commands.toggleStrike(),
              code: () => editor.commands.toggleCode(),
              blockquote: () => editor.commands.toggleBlockquote(),
              align_left: () => editor.commands.setTextAlign("left"),
              align_center: () => editor.commands.setTextAlign("center"),
              align_right: () => editor.commands.setTextAlign("right"),
              align_justify: () => editor.commands.setTextAlign("justify"),
            };
            if (id && Object.prototype.hasOwnProperty.call(commands, id)) {
              event.preventDefault();
              if (editor.isEditable && !event.repeat) commands[id]();
              return true;
            }
            if (builtIn.has(shortcutFromEvent(event) ?? "")) {
              event.preventDefault();
              return true;
            }
            return false;
          },
        },
      }),
    ];
  },
});
