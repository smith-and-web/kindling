import { describe, it, expect } from "vitest";
import {
  acceptSuggestions,
  activeDocuments,
  diffText,
  draftOf,
  isAnchored,
  parseReviewHtml,
  type Annotation,
  type SceneReview,
} from "./revisions";

function fixture(): SceneReview {
  return {
    scene_id: "scene",
    version: 0,
    mode: "page",
    documents: [
      { id: "scene", label: "Scene page", html: "<p>The <em>red</em> fox ran.</p>" },
      { id: "beat", label: "Beat 1", html: "<p>Other prose.</p>" },
    ],
    data: { status: "first_draft", drafts: [], annotations: [] },
  };
}
function annotation(
  review: SceneReview,
  from = 5,
  to = 8,
  replacement: string | null = "blue",
  id = "a"
): Annotation {
  return {
    id,
    document_id: "scene",
    from,
    to,
    quote: parseReviewHtml(review.documents[0].html).textBetween(from, to, "\n"),
    anchor_html: review.documents[0].html,
    replacement,
    state: "open",
    messages: [],
  };
}
describe("editorial suggestions", () => {
  it("preserves formatting and a recoverable original draft, remapping disjoint comments", () => {
    const review = fixture();
    const a = annotation(review);
    const comment = annotation(review, 9, 12, null, "comment");
    review.data.annotations = [a, comment];
    const { data, next } = acceptSuggestions(review, ["a"]);
    expect(next.documents[0].html).toContain("<em>blue</em>");
    expect(next.documents[1]).toEqual(review.documents[1]);
    expect(data.annotations[0].state).toBe("accepted");
    expect(data.annotations[1].from).toBe(10);
    expect(isAnchored(data.annotations[1], next.documents)).toBe(true);
    expect(data.drafts[0].documents).toEqual(review.documents);
    expect(review.data.annotations[0].state).toBe("open");
    expect(data.status).toBe("revised");
  });
  it("accepts adjacent ranges together and keeps adjacent anchors valid individually", () => {
    const review = fixture();
    review.data.annotations = [annotation(review, 1, 4, "A"), annotation(review, 4, 8, "", "b")];
    const batch = acceptSuggestions(review, ["a", "b"]);
    expect(parseReviewHtml(batch.next.documents[0].html).textContent).toBe("A fox ran.");
    const first = acceptSuggestions(review, ["a"]);
    expect(isAnchored(first.data.annotations[1], first.next.documents)).toBe(true);
    const second = acceptSuggestions(
      { ...review, documents: first.next.documents, data: first.data },
      ["b"]
    );
    expect(second.next.documents).toEqual(batch.next.documents);
    review.data.annotations = [annotation(review, 1, 1, "A"), annotation(review, 1, 4, "", "b")];
    expect(() => acceptSuggestions(review, ["a", "b"])).toThrow("overlap");
  });
  it("inserts literal text, deletes ranges, and handles Unicode positions", () => {
    const review = fixture();
    review.data.annotations = [annotation(review, 1, 1, "<b>🌲 & </b>")];
    const inserted = acceptSuggestions(review, ["a"]);
    expect(inserted.next.documents[0].html).toContain("&lt;b&gt;🌲 &amp; &lt;/b&gt;");
    review.documents[0].html = "<p>🌲 café!</p>";
    review.data.annotations = [annotation(review, 1, 4, "")];
    expect(acceptSuggestions(review, ["a"]).next.documents[0].html).toBe("<p>café!</p>");
  });
  it("applies disjoint batches in reverse order and rejects overlapping or stale batches atomically", () => {
    const review = fixture();
    review.data.annotations = [annotation(review), annotation(review, 9, 12, "wolf", "b")];
    const result = acceptSuggestions(review, ["a", "b"]);
    expect(result.next.documents[0].html).toContain("wolf ran.");
    expect(result.data.annotations.every((a) => a.state === "accepted")).toBe(true);
    review.data.annotations[1] = annotation(review, 5, 12, "wolf", "b");
    expect(() => acceptSuggestions(review, ["a", "b"])).toThrow("overlap");
    review.documents[0].html = "<p>Someone edited this</p>";
    expect(() => acceptSuggestions(review, ["a"])).toThrow("Prose changed");
    expect(review.data.drafts).toHaveLength(0);
  });
  it("leaves overlapping annotations outdated and ignores resolved or unrelated anchors", () => {
    const review = fixture();
    const overlap = annotation(review, 5, 8, null, "overlap");
    const closed = { ...annotation(review, 9, 12, null, "closed"), state: "resolved" as const };
    const stale = { ...annotation(review, 9, 12, null, "stale"), anchor_html: "old" };
    review.data.annotations = [annotation(review), overlap, closed, stale];
    const result = acceptSuggestions(review, ["a"]);
    expect(isAnchored(result.data.annotations[1], result.next.documents)).toBe(false);
    expect(result.data.annotations[2]).toEqual(closed);
    expect(result.data.annotations[3]).toEqual(stale);
  });
  it("validates missing, closed and non-suggestion IDs and invalid range anchors", () => {
    const review = fixture();
    expect(() => acceptSuggestions(review, ["missing"])).toThrow("no longer open");
    review.data.annotations = [annotation(review, 1, 2, null)];
    expect(() => acceptSuggestions(review, ["a"])).toThrow("no longer open");
    review.data.annotations[0].replacement = "X";
    review.data.annotations[0].state = "rejected";
    expect(() => acceptSuggestions(review, ["a"])).toThrow("no longer open");
    const a = annotation(review);
    expect(isAnchored(a, [])).toBe(false);
    expect(isAnchored({ ...a, from: 0 }, review.documents)).toBe(false);
    expect(isAnchored({ ...a, to: 1 }, review.documents)).toBe(false);
    expect(isAnchored({ ...a, to: 500 }, review.documents)).toBe(false);
    expect(isAnchored({ ...a, quote: "wrong" }, review.documents)).toBe(false);
  });
  it("selects the active prose mode while drafts retain both modes independently", () => {
    const review = fixture();
    expect(activeDocuments(review).map((d) => d.id)).toEqual(["scene"]);
    review.mode = "beat";
    expect(activeDocuments(review).map((d) => d.id)).toEqual(["beat"]);
    const draft = draftOf(review, "  First draft  ");
    review.documents[0].html = "changed";
    expect(draft.name).toBe("First draft");
    expect(draft.documents[0].html).not.toBe("changed");
    review.documents = [review.documents[0]];
    expect(activeDocuments(review).map((d) => d.id)).toEqual(["scene"]);
  });
});
describe("draft comparison", () => {
  it("shows insertions, deletions and shared words with whitespace intact", () => {
    expect(diffText("The red fox.", "The blue fox.")).toEqual([
      { kind: "same", text: "The " },
      { kind: "delete", text: "red" },
      { kind: "insert", text: "blue" },
      { kind: "same", text: " fox." },
    ]);
    expect(diffText("", "")).toEqual([]);
    expect(diffText("", "one two")).toEqual([{ kind: "insert", text: "one two" }]);
    expect(diffText("one two", "")).toEqual([{ kind: "delete", text: "one two" }]);
    expect(diffText("same", "same")).toEqual([{ kind: "same", text: "same" }]);
  });
  it("bounds memory for long scenes while reconstructing both originals exactly", () => {
    for (const [before, after] of [
      ["a ".repeat(600) + "old end", "a ".repeat(600) + "new end"],
      ["a ".repeat(600), "b ".repeat(600)],
      ["same ".repeat(600), "same ".repeat(600)],
    ]) {
      const parts = diffText(before, after);
      expect(
        parts
          .filter((p) => p.kind !== "insert")
          .map((p) => p.text)
          .join("")
      ).toBe(before);
      expect(
        parts
          .filter((p) => p.kind !== "delete")
          .map((p) => p.text)
          .join("")
      ).toBe(after);
    }
  });
});

it("shows separated edits precisely in long scenes and reconstructs insertions and deletions", () => {
  const words = Array.from({ length: 2000 }, (_, i) => `word${i}`);
  const before = words.join(" ");
  const changed = [...words];
  changed[100] = "red";
  changed[1500] = "blue";
  const parts = diffText(before, changed.join(" "));
  expect(parts.filter((p) => p.kind === "delete").map((p) => p.text)).toEqual([
    "word100",
    "word1500",
  ]);
  expect(parts.filter((p) => p.kind === "insert").map((p) => p.text)).toEqual(["red", "blue"]);
  for (const [a, b] of [
    [before, "new " + before],
    ["new " + before, before],
    [before, before + " end"],
    [before + " end", before],
  ]) {
    const diff = diffText(a, b);
    expect(
      diff
        .filter((p) => p.kind !== "insert")
        .map((p) => p.text)
        .join("")
    ).toBe(a);
    expect(
      diff
        .filter((p) => p.kind !== "delete")
        .map((p) => p.text)
        .join("")
    ).toBe(b);
  }
});

it("keeps ordinary editorial passes precise past 256 token edits", () => {
  const words = Array.from({ length: 1500 }, (_, i) => `word${i}`);
  const revised = words.map((word, i) => (i % 10 === 0 ? `edited${i}` : word));
  const parts = diffText(words.join(" "), revised.join(" "));
  expect(parts.filter((p) => p.kind === "delete")).toHaveLength(150);
  expect(parts.filter((p) => p.kind === "insert")).toHaveLength(150);
  expect(parts.filter((p) => p.kind === "delete").every((p) => !p.text.includes(" "))).toBe(true);
  const removed = diffText(
    words.join(" "),
    [...words.slice(0, 500), ...words.slice(630)].join(" ")
  );
  expect(removed.filter((p) => p.kind === "insert")).toEqual([]);
  expect(
    removed
      .filter((p) => p.kind === "delete")
      .map((p) => p.text)
      .join("")
      .trim()
      .split(" ")
  ).toHaveLength(130);
});
