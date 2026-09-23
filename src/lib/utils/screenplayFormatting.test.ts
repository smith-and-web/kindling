import { describe, expect, it } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { classifyScreenplay, ScreenplayFormatting } from "./screenplayFormatting";

describe("classifyScreenplay", () => {
  it("reads headings, action, cues, parentheticals, dialogue and transitions", () => {
    expect(
      classifyScreenplay([
        "INT. LIGHTHOUSE - NIGHT",
        "A lamp throws their shadows up the spiral stair. ELEANOR, 18, works her knife.",
        "THOMAS",
        "(whispering)",
        "He'll hear us.",
        "ELEANOR (V.O.)",
        "Then let him explain what we find.",
        "The step lifts.",
        "CUT TO:",
      ])
    ).toEqual([
      "heading",
      "action",
      "cue",
      "parenthetical",
      "dialogue",
      "cue",
      "dialogue",
      "action",
      "transition",
    ]);
  });

  it("keeps shouted action, a closing capital line and blanks as action", () => {
    expect(classifyScreenplay(["BOOM!", "", "THE END"])).toEqual(["action", "action", "action"]);
  });

  it("allows a parenthetical between lines of one speech", () => {
    expect(classifyScreenplay(["SILAS", "No.", "(beat)", "Not yet."])).toEqual([
      "cue",
      "dialogue",
      "parenthetical",
      "dialogue",
    ]);
  });
});

describe("ScreenplayFormatting", () => {
  it("decorates paragraphs by role and follows edits", () => {
    const element = document.createElement("div");
    const editor = new Editor({
      element,
      extensions: [StarterKit, ScreenplayFormatting],
      content: "<p>THOMAS</p><p>He'll hear us.</p>",
    });
    const classes = () => [...element.querySelectorAll("p")].map((p) => p.className);
    expect(classes()).toEqual(["sp-cue", "sp-dialogue"]);

    editor.commands.setContent("<p>The step lifts.</p>");
    expect(classes()).toEqual(["sp-action"]);
    editor.destroy();
  });
});
