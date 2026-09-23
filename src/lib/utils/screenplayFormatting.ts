import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";

/**
 * Screenplay prose is stored as plain paragraphs. This reads each paragraph's
 * role from its text, the way a screenwriter types it, so the page can show
 * screenplay layout without changing what is saved or exported.
 */
export type ScreenplayRole =
  | "heading"
  | "action"
  | "cue"
  | "parenthetical"
  | "dialogue"
  | "transition";

const HEADING = /^(INT|EXT|EST|INT\.?\/EXT|I\/E)[.\s]/;
const TRANSITION = /(TO:|^FADE (IN|OUT)[.:]?|^CUT TO BLACK[.:]?)$/;
const MAX_CUE_LENGTH = 38;

function isUppercase(text: string): boolean {
  return /[A-Z]/.test(text) && text === text.toUpperCase();
}

export function classifyScreenplay(paragraphs: string[]): ScreenplayRole[] {
  const roles: ScreenplayRole[] = [];
  paragraphs.forEach((raw, index) => {
    const text = raw.trim();
    const previous = roles[index - 1];
    const inSpeech = previous === "cue" || previous === "parenthetical" || previous === "dialogue";
    const hasNext = paragraphs.slice(index + 1).some((p) => p.trim().length > 0);

    if (!text) roles.push("action");
    else if (HEADING.test(text) && isUppercase(text)) roles.push("heading");
    else if (isUppercase(text) && TRANSITION.test(text)) roles.push("transition");
    else if (inSpeech && /^\(.*\)$/.test(text)) roles.push("parenthetical");
    else if (isUppercase(text) && text.length <= MAX_CUE_LENGTH && hasNext && !/[.!?]$/.test(text))
      roles.push("cue");
    else if (previous === "cue" || previous === "parenthetical") roles.push("dialogue");
    else roles.push("action");
  });
  return roles;
}

function decorate(doc: ProseMirrorNode): DecorationSet {
  const blocks: { pos: number; node: ProseMirrorNode }[] = [];
  doc.forEach((node, pos) => {
    if (node.type.name === "paragraph") blocks.push({ pos, node });
  });
  const roles = classifyScreenplay(blocks.map((b) => b.node.textContent));
  return DecorationSet.create(
    doc,
    blocks.map(({ pos, node }, i) =>
      Decoration.node(pos, pos + node.nodeSize, { class: `sp-${roles[i]}` })
    )
  );
}

const key = new PluginKey("screenplay-formatting");

export const ScreenplayFormatting = Extension.create({
  name: "screenplayFormatting",
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key,
        state: {
          init: (_, { doc }) => decorate(doc),
          apply: (tr, old) => (tr.docChanged ? decorate(tr.doc) : old),
        },
        props: {
          decorations(state) {
            return key.getState(state);
          },
        },
      }),
    ];
  },
});
