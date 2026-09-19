import { describe, it, expect } from "vitest";
import { Fragment, Slice } from "@tiptap/pm/model";
import { Transform } from "@tiptap/pm/transform";
import {
  manuscript,
  normalizeOwnership,
  sourceHtml,
  changesBetween,
  trackChanges,
  projectedRange,
  sliceText,
  prepareAcceptance,
  locateChange,
  withdrawSuggestion,
  searchManuscript,
  message,
  editorialSchema,
  validateEditorialPackage,
  sliceHtml,
  restoreTracking,
  type EditorialSource,
  type EditorialFeedback,
  type FeedbackEntry,
} from "./editorial";

function source(id: string, html: string): EditorialSource {
  return {
    id,
    scene_id: `scene-${id}`,
    chapter_id: "chapter",
    chapter: "The Letter",
    scene: id,
    mode: "beat",
    html,
    locked: false,
  };
}
const sources = [
  source("a", "<p>The <em>sea</em> was still.</p>"),
  source("b", "<p>Eleanor waited.</p>"),
  source("c", "<p>Thomas arrived.</p>"),
];
function feedback(next: ReturnType<typeof manuscript>): EditorialFeedback {
  return {
    round: {
      id: "round",
      project_id: "project",
      title: "The Letter",
      name: "Pass",
      brief: "",
      created_at: "",
      sources,
    },
    sources,
    version: 0,
    entries: trackChanges(manuscript(sources), next, []).map((change) => ({
      key: change.id,
      reviewer: "Editor",
      change,
      decision: "open",
    })),
  };
}

describe("continuous editorial manuscript", () => {
  it("keeps edits separate across a substantial manuscript and across reopening", () => {
    const book = Array.from({ length: 60 }, (_, i) =>
      source(`book-${i}`, `<p>${`Passage ${i} beside the lighthouse. `.repeat(200)}</p>`)
    );
    const base = manuscript(book);
    let proposed = base;
    let tracked: ReturnType<typeof trackChanges> = [];
    const started = performance.now();
    for (let i = 0; i < 20; i++) {
      let pos = 1;
      for (let j = 0; j < i * 3; j++) pos += proposed.child(j).nodeSize;
      proposed = new Transform(proposed).insert(
        pos,
        editorialSchema.text(`Editorial addition ${i}. `.repeat(15))
      ).doc;
      tracked = trackChanges(base, proposed, tracked);
    }
    expect(tracked).toHaveLength(20);
    const reopened = editorialSchema.nodeFromJSON(proposed.toJSON());
    restoreTracking(base, reopened, tracked);
    expect(trackChanges(base, reopened, tracked)).toEqual(tracked);
    expect(performance.now() - started).toBeLessThan(5000);
  });
  it("keeps both discussions when ordinary typing merges adjacent suggestions", () => {
    const base = manuscript(sources);
    let next = new Transform(base).insert(1, editorialSchema.text("A")).doc;
    let changes = trackChanges(base, next, []);
    next = new Transform(next).insert(5, editorialSchema.text("B")).doc;
    changes = trackChanges(base, next, changes);
    expect(changes).toHaveLength(2);
    changes[0].messages.push(message("Rowan", "Opening discussion"));
    changes[1].messages.push(message("Writer", "Second discussion"));
    next = new Transform(next).delete(2, 5).doc;
    changes = trackChanges(base, next, changes);
    expect(changes).toHaveLength(1);
    expect(changes[0].messages.map((m) => m.text)).toEqual([
      "Opening discussion",
      "Second discussion",
    ]);
  });
  it("accepts an intact passage after an unrelated scene is removed", () => {
    const base = manuscript(sources);
    const review = feedback(new Transform(base).insert(1, editorialSchema.text("Cold. ")).doc);
    review.sources = sources.slice(0, 2);
    expect(prepareAcceptance(review, review.entries)[0].html).toContain("Cold. The");
  });
  it("maps case folding to original positions, including text after dotted I", () => {
    const doc = manuscript([source("unicode", "<p>İstanbul beside the sea.</p>")]);
    const found = searchManuscript(doc, "sea.");
    expect(doc.textBetween(found[0].from, found[0].to)).toBe("sea.");
    const dotted = searchManuscript(doc, "İstanbul")[0];
    expect(doc.textBetween(dotted.from, dotted.to)).toBe("İstanbul");
    const greek = manuscript([source("greek", "<p>ΟΣ</p>")]);
    expect(searchManuscript(greek, "ΟΣ")).toHaveLength(1);
    expect(searchManuscript(greek, "ος")).toHaveLength(1);
  });
  it("keeps a comment within an inserted passage", () => {
    const base = manuscript(sources);
    const next = new Transform(base).insert(1, editorialSchema.text("At dusk, ")).doc;
    const change = trackChanges(base, next, [])[0];
    const comment = {
      ...change,
      kind: "comment" as const,
      anchor_offset: [3, 7] as [number, number],
    };
    const range = projectedRange(base, next, comment);
    expect(next.textBetween(range.from, range.to)).toBe("dusk");
    const edit = new Transform(next).insert(1, editorialSchema.text("cold "));
    const updated = trackChanges(base, edit.doc, [change, comment], {
      before: next,
      mapping: edit.mapping,
    });
    const anchored = projectedRange(base, edit.doc, updated.find((c) => c.kind === "comment")!);
    expect(edit.doc.textBetween(anchored.from, anchored.to)).toBe("dusk");
  });
  it("validates returned document positions, original anchors, and formatting", () => {
    const base = manuscript(sources);
    const next = new Transform(base).insert(1, editorialSchema.text("Cold. ")).doc;
    const review = feedback(next);
    const data = {
      format: "kindling-editorial" as const,
      version: 1 as const,
      kind: "feedback" as const,
      round: review.round,
      session: {
        reviewer_id: "editor",
        name: "Rowan",
        generation: 1,
        document: next.toJSON(),
        changes: review.entries.map((e) => e.change),
        position: 1,
      },
    };
    expect(() => validateEditorialPackage(data)).not.toThrow();
    expect(sliceHtml(data.session.changes[0].after)).toBe("Cold. ");
    const invalid = structuredClone(data);
    invalid.session.changes[0].to = 100000;
    expect(() => validateEditorialPackage(invalid)).toThrow("outside");
    invalid.session.changes[0].to = 0;
    expect(() => validateEditorialPackage(invalid)).toThrow("outside");
    invalid.session.changes[0].to = 2;
    expect(() => validateEditorialPackage(invalid)).toThrow("original passage");
    expect(() => validateEditorialPackage({ ...data, session: null })).not.toThrow();
  });
  it("validates and retains replacement revisions after a Rust JSON round trip", () => {
    const base = manuscript(sources);
    const next = new Transform(base).replaceWith(1, 4, editorialSchema.text("A cold")).doc;
    const review = feedback(next);
    // serde_json sorts object keys; rich slices must compare structurally.
    const sorted = (value: unknown): unknown => {
      if (Array.isArray(value)) return value.map(sorted);
      if (value && typeof value === "object") {
        return Object.fromEntries(
          Object.entries(value)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([k, v]) => [k, sorted(v)])
        );
      }
      return value;
    };
    const data = JSON.parse(
      JSON.stringify(
        sorted({
          format: "kindling-editorial",
          version: 1,
          kind: "feedback",
          round: review.round,
          session: {
            reviewer_id: "editor",
            name: "Rowan",
            generation: 1,
            document: next.toJSON(),
            changes: review.entries.map((e) => e.change),
            position: 1,
          },
        })
      )
    );
    expect(() => validateEditorialPackage(data)).not.toThrow();
    const reopened = editorialSchema.nodeFromJSON(data.session.document);
    restoreTracking(base, reopened, data.session.changes);
    const retained = trackChanges(base, reopened, data.session.changes);
    expect(retained[0].id).toBe(data.session.changes[0].id);
    expect(retained[0].revision).toBe(data.session.changes[0].revision);
  });
  it("preserves rich prose and source order including blockquotes", () => {
    const rich = [
      source(
        "a",
        '<p style="text-align: right;"><strong>Bold</strong> <em>italic</em> <u>underlined</u></p><blockquote><p>Letter</p></blockquote>'
      ),
    ];
    const doc = manuscript(rich);
    expect(sourceHtml(doc, rich).get("a")).toBe(rich[0].html);
    expect(changesBetween(doc, doc)).toEqual([]);
  });

  it("tracks distinct edits, refinements, withdrawal and undo without altering the original", () => {
    const base = manuscript(sources);
    const first = new Transform(base).insert(1, editorialSchema.text("Cold. ")).doc;
    let changes = trackChanges(base, first, []);
    expect(changes).toHaveLength(1);
    expect(sliceText(changes[0].after)).toBe("Cold. ");
    const refined = new Transform(first).insert(3, editorialSchema.text("very ")).doc;
    const updated = trackChanges(base, refined, changes);
    expect(updated[0].id).toBe(changes[0].id);
    expect(updated[0].revision).toBeGreaterThan(changes[0].revision);
    expect(trackChanges(base, refined, updated)[0].revision).toBe(updated[0].revision);
    expect(withdrawSuggestion(base, refined, updated[0]).eq(base)).toBe(true);
    expect(trackChanges(base, base, updated)).toEqual([]);
    expect(base.textContent).toContain("The sea was still.");
    const second = new Transform(first).insert(
      first.content.size - 2,
      editorialSchema.text(" quietly")
    ).doc;
    changes = trackChanges(base, second, changes);
    expect(changes).toHaveLength(2);
    expect(projectedRange(base, second, changes[0]).to).toBe(7);
  });

  it("treats formatting-only changes as reviewable suggestions", () => {
    const base = manuscript(sources);
    const next = new Transform(base).addMark(1, 4, editorialSchema.marks.bold.create()).doc;
    const review = feedback(next);
    expect(review.entries).toHaveLength(1);
    const replacements = prepareAcceptance(review, review.entries);
    expect(replacements[0].html).toContain("<strong>The</strong>");
    expect(replacements[0].expected).toBe(sources[0].html);
  });

  it("accepts one natural replacement across beats and scenes, retaining all source records", () => {
    const base = manuscript(sources);
    const end = base.child(0).nodeSize + base.child(1).nodeSize + 7;
    const next = new Transform(base).replaceRange(
      5,
      end,
      new Slice(Fragment.from(editorialSchema.text("the lighthouse. ")), 0, 0)
    ).doc;
    const normalized = normalizeOwnership(next, sources);
    const review = feedback(normalized);
    expect(review.entries).toHaveLength(1);
    const replacements = prepareAcceptance(review, review.entries);
    expect(replacements.some((r) => r.id === "b" && r.html === "")).toBe(true);
    expect(replacements.map((r) => r.id)).toContain("a");
    expect(replacements.find((r) => r.id === "c")?.html).toBe("<p> arrived.</p>");
    expect(replacements.find((r) => r.id === "a")?.html).not.toContain("arrived");
    expect(sources).toHaveLength(3);
    expect(sourceHtml(normalized, sources).size).toBe(3);
  });

  it("maps disjoint writer edits and blocks conflicting changes without partial application", () => {
    const base = manuscript(sources);
    const next = new Transform(base).insert(1, editorialSchema.text("Cold. ")).doc;
    const review = feedback(next);
    review.sources = sources.map((s) =>
      s.id === "c" ? { ...s, html: "<p>Thomas never arrived.</p>" } : s
    );
    expect(prepareAcceptance(review, review.entries)).toHaveLength(1);
    review.sources = sources.map((s) => (s.id === "a" ? { ...s, html: "<p>A storm.</p>" } : s));
    expect(() => prepareAcceptance(review, review.entries)).toThrow("passage changed");
    expect(prepareAcceptance(review, review.entries, { from: 1, to: 1 })[0].html).toContain(
      "Cold. A storm."
    );
    review.sources = sources.slice(1);
    expect(() => prepareAcceptance(review, review.entries)).toThrow("passage changed");
    expect(prepareAcceptance(review, review.entries, { from: 1, to: 1 })[0].html).toContain(
      "Cold. Eleanor"
    );
  });

  it("rejects overlapping bulk changes and supports plain deletions", () => {
    const base = manuscript(sources);
    const next = new Transform(base).delete(1, 5).doc;
    const review = feedback(next);
    const duplicate: FeedbackEntry = { ...review.entries[0], key: "second" };
    expect(() => prepareAcceptance(review, [...review.entries, duplicate])).toThrow("overlap");
    expect(prepareAcceptance(review, review.entries)[0].html).not.toContain("The ");
    expect(sliceText(review.entries[0].change.after)).toBe("");
  });

  it("normalizes pasted paragraphs and rejects reordered or missing ownership", () => {
    const base = manuscript(sources);
    const pasted = editorialSchema.node("paragraph", null, editorialSchema.text("New paragraph"));
    const doc = new Transform(base).insert(base.child(0).nodeSize, pasted).doc;
    expect(normalizeOwnership(doc, sources).child(1).attrs.source).toBe("a");
    const reordered = base.type.create(null, [base.child(1), base.child(0)]);
    expect(() => normalizeOwnership(reordered, sources)).toThrow("reorder");
    expect(() => sourceHtml(editorialSchema.node("doc", null, [pasted]), sources)).toThrow(
      "source"
    );
    expect(() => normalizeOwnership(editorialSchema.node("doc", null, [pasted]), [])).toThrow(
      "no manuscript"
    );
  });

  it("searches across paragraph boundaries and locates general comments", () => {
    const base = manuscript(sources);
    const results = searchManuscript(base, "still.\nEleanor");
    expect(results).toHaveLength(1);
    expect(base.textBetween(results[0].from, results[0].to, "\n")).toBe("still.\nEleanor");
    expect(searchManuscript(base, "")).toEqual([]);
    const comment = {
      id: "c",
      revision: 1,
      kind: "comment" as const,
      from: 1,
      to: 4,
      before: base.slice(1, 4).toJSON(),
      after: null,
      state: "open" as const,
      messages: [message(" Editor ", " Note ")],
    };
    expect(comment.messages[0].text).toBe("Note");
    expect(trackChanges(base, base, [comment])).toEqual([comment]);
    expect(projectedRange(base, base, comment)).toEqual({ from: 1, to: 4 });
    expect(locateChange(base, base, comment).conflict).toBe(false);
  });
});
