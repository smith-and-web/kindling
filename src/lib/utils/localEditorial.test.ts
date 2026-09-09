import { describe, expect, it } from "vitest";
import { Transform } from "@tiptap/pm/transform";
import { localAnnotations, localSelection } from "./localEditorial";
import { editorialSchema, lockedProseChanged, manuscript, type EditorialSource } from "./editorial";
import type { SceneReview } from "./revisions";
const source: EditorialSource = {
  id: "scene",
  scene_id: "scene",
  chapter_id: "c",
  chapter: "Chapter",
  scene: "Scene",
  mode: "page",
  html: "<p>The red fox.</p>",
  locked: false,
};
function review(): SceneReview {
  return {
    scene_id: "scene",
    version: 0,
    mode: "page",
    documents: [
      { id: "scene", label: "Page", html: source.html },
      { id: "inactive", label: "Beat", html: "<p>Hidden</p>" },
    ],
    data: {
      status: "first_draft",
      drafts: [],
      annotations: [
        {
          id: "note",
          document_id: "scene",
          anchor_html: source.html,
          from: 5,
          to: 8,
          quote: "red",
          replacement: "blue",
          state: "open",
          messages: [{ author: "Rowan", text: "Colour?", created_at: "today" }],
        },
      ],
    },
  };
}
describe("saved scene annotations in continuous review", () => {
  it("offsets saved anchors across sources without mutating stored feedback or inactive prose", () => {
    const saved = review();
    saved.data.annotations.push({
      ...saved.data.annotations[0],
      id: "hidden",
      document_id: "inactive",
    });
    const before = structuredClone(saved);
    const prefix = { ...source, id: "previous", scene_id: "previous" };
    const entries = localAnnotations([saved], [prefix, source]);
    expect(entries).toHaveLength(1);
    expect(entries[0]).toMatchObject({
      from: manuscript([prefix]).content.size + 5,
      conflict: false,
      locked: false,
      change: { kind: "suggestion", messages: [{ id: "note:0", text: "Colour?" }] },
    });
    expect(saved).toEqual(before);
    expect(localSelection([prefix, source], entries[0].from, entries[0].to)).toMatchObject({
      source,
      from: 5,
      to: 8,
      quote: "red",
    });
    expect(() => localSelection([prefix, source], 1, entries[0].to)).toThrow("Select a passage");
  });
  it("keeps outdated, locked and resolved comments reviewable", () => {
    const saved = review();
    saved.documents[0].html = "<p>The old red fox.</p>";
    saved.data.annotations[0].replacement = null;
    saved.data.annotations[0].state = "resolved";
    const entry = localAnnotations(
      [saved],
      [{ ...source, html: saved.documents[0].html, locked: true }]
    )[0];
    expect(entry).toMatchObject({
      conflict: true,
      locked: true,
      change: { kind: "comment", state: "resolved", after: null },
    });
  });
  it("rejects text and formatting changes in locked prose while allowing adjacent edits", () => {
    const sources = [source, { ...source, id: "locked", scene_id: "locked", locked: true }];
    const base = manuscript(sources);
    expect(
      lockedProseChanged(
        base,
        new Transform(base).insert(1, editorialSchema.text("Hi ")).doc,
        sources
      )
    ).toBe(false);
    const position = manuscript([source]).content.size + 1;
    expect(
      lockedProseChanged(
        base,
        new Transform(base).insert(position, editorialSchema.text("Hi ")).doc,
        sources
      )
    ).toBe(true);
    expect(
      lockedProseChanged(
        base,
        new Transform(base).addMark(position, position + 3, editorialSchema.marks.bold.create())
          .doc,
        sources
      )
    ).toBe(true);
  });
});
