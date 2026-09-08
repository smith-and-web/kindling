import { getSchema } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import TextAlign from "@tiptap/extension-text-align";
import { DOMParser as ProseParser, DOMSerializer } from "@tiptap/pm/model";
import { Transform } from "@tiptap/pm/transform";
import type { EditorMode } from "../types";

export const reviewExtensions = [
  StarterKit.configure({
    heading: false,
    bulletList: false,
    orderedList: false,
    listItem: false,
    codeBlock: false,
    horizontalRule: false,
  }),
  TextAlign.configure({ types: ["paragraph"] }),
];
const schema = getSchema(reviewExtensions);
export interface ReviewDocument {
  id: string;
  label: string;
  html: string;
}
export interface ReviewDraft {
  name: string;
  created_at: string;
  mode: EditorMode;
  documents: ReviewDocument[];
}
export interface ReviewMessage {
  author: string;
  text: string;
  created_at: string;
}
export interface Annotation {
  id: string;
  document_id: string;
  anchor_html: string;
  from: number;
  to: number;
  quote: string;
  replacement: string | null;
  state: "open" | "resolved" | "accepted" | "rejected";
  messages: ReviewMessage[];
}
export interface ReviewData {
  status: string;
  drafts: ReviewDraft[];
  annotations: Annotation[];
}
export interface SceneReview {
  scene_id: string;
  version: number;
  mode: EditorMode;
  documents: ReviewDocument[];
  data: ReviewData;
}
export interface RevisionOverview {
  scene_id: string;
  title: string;
  chapter: string;
  status: string;
  drafts: number;
}
export const revisionStatuses = {
  first_draft: "First Draft",
  editor_review: "Editor Review",
  revised: "Revised",
  final: "Final",
};

export function parseReviewHtml(html: string) {
  const root = document.createElement("div");
  root.innerHTML = html;
  return ProseParser.fromSchema(schema).parse(root);
}

export function draftOf(review: SceneReview, name: string): ReviewDraft {
  return {
    name: name.trim(),
    created_at: new Date().toISOString(),
    mode: review.mode,
    documents: structuredClone(review.documents),
  };
}

export function activeDocuments(review: Pick<SceneReview, "scene_id" | "mode" | "documents">) {
  return review.mode === "page" || review.documents.length === 1
    ? review.documents.filter((d) => d.id === review.scene_id)
    : review.documents.filter((d) => d.id !== review.scene_id);
}

export function isAnchored(
  annotation: Annotation,
  documents: ReviewDocument[],
  parsedDocument?: ReturnType<typeof parseReviewHtml>
) {
  const doc = documents.find((d) => d.id === annotation.document_id);
  if (!doc || doc.html !== annotation.anchor_html) return false;
  const parsed = parsedDocument ?? parseReviewHtml(doc.html);
  return (
    annotation.from >= 1 &&
    annotation.to >= annotation.from &&
    annotation.to < parsed.content.size &&
    parsed.textBetween(annotation.from, annotation.to, "\n") === annotation.quote
  );
}

/** Nonempty ranges are half-open. Cursor insertions at a shared edge are
 * intentionally reviewed individually because their ordering is ambiguous. */
function overlaps(a: Annotation, b: Annotation) {
  if (a.from === a.to || b.from === b.to) return a.from <= b.to && b.from <= a.to;
  return a.from < b.to && b.from < a.to;
}

/** Apply against exact source HTML. Remap disjoint anchors; invalidate overlaps
 * instead of ever applying an edit to text the editor did not review. */
export function acceptSuggestions(
  review: SceneReview,
  ids: string[]
): { data: ReviewData; next: ReviewDraft } {
  const data: ReviewData = structuredClone(review.data);
  const next = draftOf(review, "Accepted editorial changes");
  const targets = ids.map((id) => {
    const a = data.annotations.find((a) => a.id === id);
    if (!a || a.state !== "open" || a.replacement === null)
      throw new Error("Suggestion is no longer open");
    return a;
  });
  // Fail the whole batch for overlaps or outdated anchors; never partially accept all.
  for (let i = 0; i < targets.length; i++) {
    const a = targets[i];
    if (!isAnchored(a, next.documents))
      throw new Error("Prose changed. Re-anchor outdated suggestions before accepting them.");
    for (const b of targets.slice(i + 1)) {
      if (a.document_id === b.document_id && overlaps(a, b))
        throw new Error("Suggestions overlap. Review and accept them individually.");
    }
  }
  for (const a of targets.sort((a, b) => b.from - a.from)) {
    const doc = next.documents.find((d) => d.id === a.document_id)!;
    const oldHtml = doc.html;
    const parsed = parseReviewHtml(oldHtml);
    const tr = new Transform(parsed);
    if (a.replacement)
      tr.replaceWith(
        a.from,
        a.to,
        schema.text(
          a.replacement,
          (a.from < a.to ? parsed.nodeAt(a.from)?.marks : undefined) ??
            parsed.resolve(a.from).marks()
        )
      );
    else tr.delete(a.from, a.to);
    const container = document.createElement("div");
    container.append(DOMSerializer.fromSchema(schema).serializeFragment(tr.doc.content));
    doc.html = container.innerHTML;
    a.state = "accepted";
    for (const other of data.annotations) {
      if (
        other === a ||
        other.state !== "open" ||
        other.document_id !== doc.id ||
        other.anchor_html !== oldHtml
      )
        continue;
      if (overlaps(other, a)) continue; // Retain old HTML to flag outdated.
      other.from = tr.mapping.map(other.from, 1);
      other.to = tr.mapping.map(other.to, -1);
      other.anchor_html = doc.html;
    }
  }
  data.drafts.push(draftOf(review, "Before accepting changes"));
  data.status = "revised";
  return { data, next };
}

export interface DiffPart {
  kind: "same" | "insert" | "delete";
  text: string;
}
/** Myers diff bounds work by edit distance, so a few edits in a long scene
 * remain precise. Very heavily rewritten scenes use the coarse fallback. */
function sparseDiff(a: string[], b: string[]): DiffPart[] | null {
  const frontier = new Map<number, number>([[1, 0]]);
  const trace: Map<number, number>[] = [];
  for (let depth = 0; depth <= Math.min(a.length + b.length, 1024); depth++) {
    trace.push(new Map(frontier));
    for (let k = -depth; k <= depth; k += 2) {
      let x =
        k === -depth || (k !== depth && frontier.get(k - 1)! < frontier.get(k + 1)!)
          ? frontier.get(k + 1)!
          : frontier.get(k - 1)! + 1;
      let y = x - k;
      while (x < a.length && y < b.length && a[x] === b[y]) {
        x++;
        y++;
      }
      frontier.set(k, x);
      if (x < a.length || y < b.length) continue;
      const reversed: DiffPart[] = [];
      for (let d = trace.length - 1; d >= 0; d--) {
        const v = trace[d],
          diagonal = x - y;
        const previousK =
          diagonal === -d || (diagonal !== d && v.get(diagonal - 1)! < v.get(diagonal + 1)!)
            ? diagonal + 1
            : diagonal - 1;
        const previousX = v.get(previousK)!,
          previousY = previousX - previousK;
        while (x > previousX && y > previousY) {
          reversed.push({ kind: "same", text: a[--x] });
          y--;
        }
        if (d === 0) break;
        if (x === previousX) reversed.push({ kind: "insert", text: b[--y] });
        else reversed.push({ kind: "delete", text: a[--x] });
      }
      return reversed.reverse();
    }
  }
  return null;
}

/** Word diff with bounded memory. Scenes with more than 1024 changed tokens
 * and a large comparison retain common edges and show the middle as a replacement. */
export function diffText(before: string, after: string): DiffPart[] {
  const a = before.match(/\s+|[^\s]+/gu) ?? [];
  const b = after.match(/\s+|[^\s]+/gu) ?? [];
  const parts: DiffPart[] = [];
  const add = (kind: DiffPart["kind"], text: string) => {
    if (!text) return;
    if (parts[parts.length - 1]?.kind === kind) parts[parts.length - 1].text += text;
    else parts.push({ kind, text });
  };
  if (a.length * b.length > 1_000_000) {
    const precise = sparseDiff(a, b);
    if (precise) {
      for (const part of precise) add(part.kind, part.text);
      return parts;
    }
    let start = 0,
      end = 0;
    while (start < Math.min(a.length, b.length) && a[start] === b[start]) start++;
    while (
      end < Math.min(a.length, b.length) - start &&
      a[a.length - end - 1] === b[b.length - end - 1]
    )
      end++;
    add("same", a.slice(0, start).join(""));
    add("delete", a.slice(start, a.length - end).join(""));
    add("insert", b.slice(start, b.length - end).join(""));
    add("same", a.slice(a.length - end).join(""));
    return parts;
  }
  const rows = Array.from({ length: a.length + 1 }, () => new Uint32Array(b.length + 1));
  for (let i = a.length - 1; i >= 0; i--)
    for (let j = b.length - 1; j >= 0; j--)
      rows[i][j] =
        a[i] === b[j] ? rows[i + 1][j + 1] + 1 : Math.max(rows[i + 1][j], rows[i][j + 1]);
  let i = 0,
    j = 0;
  while (i < a.length || j < b.length) {
    if (i < a.length && j < b.length && a[i] === b[j]) {
      add("same", a[i++]);
      j++;
    } else if (j < b.length && (i === a.length || rows[i][j + 1] > rows[i + 1][j]))
      add("insert", b[j++]);
    else add("delete", a[i++]);
  }
  return parts;
}
