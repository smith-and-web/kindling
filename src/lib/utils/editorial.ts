import { Extension, getSchema } from "@tiptap/core";
import { ChangeSet, type Change as TrackedChange } from "@tiptap/pm/changeset";
import { DOMParser, DOMSerializer, Fragment, Node, Slice } from "@tiptap/pm/model";
import { Mapping, StepMap, Transform } from "@tiptap/pm/transform";
import { reviewExtensions } from "./revisions";
import type { EditorMode } from "../types";

export interface EditorialSource {
  id: string;
  scene_id: string;
  chapter_id: string;
  chapter: string;
  scene: string;
  mode: EditorMode;
  html: string;
  locked: boolean;
}
export interface EditorialRound {
  id: string;
  project_id: string;
  title: string;
  name: string;
  brief: string;
  created_at: string;
  sources: EditorialSource[];
}
export interface EditorialMessage {
  id: string;
  author: string;
  text: string;
  created_at: string;
}
export interface EditorialChange {
  id: string;
  revision: number;
  kind: "suggestion" | "comment";
  from: number;
  to: number;
  before: ReturnType<Slice["toJSON"]>;
  after: ReturnType<Slice["toJSON"]>;
  state: "open" | "resolved" | "withdrawn";
  messages: EditorialMessage[];
  anchor_offset?: [number, number] | null;
  writer_decision?: string | null;
}
export interface EditorialSession {
  reviewer_id: string;
  name: string;
  generation: number;
  document: ReturnType<Node["toJSON"]>;
  changes: EditorialChange[];
  position: number;
  reading_position?: number;
  writer_version?: number;
}
export interface EditorialPackage {
  format: "kindling-editorial";
  version: 1;
  kind: "review" | "feedback";
  round: EditorialRound;
  session: EditorialSession | null;
}
export interface FeedbackEntry {
  key: string;
  reviewer: string;
  change: EditorialChange;
  decision: "open" | "accepted" | "rejected" | "resolved" | "withdrawn";
}
export interface EditorialFeedback {
  round: EditorialRound;
  entries: FeedbackEntry[];
  sources: EditorialSource[];
  version: number;
}

/** Packages are data, not trusted editor state. Validate positions and rich
 * attributes before constructing decorations or offering manuscript decisions. */
export function validateEditorialPackage(packageData: EditorialPackage) {
  const base = manuscript(packageData.round.sources);
  const ids = new Set(packageData.round.sources.map((s) => s.id));
  function validateNode(node: Node) {
    node.check();
    const attributes = (child: Node) => {
      if (child.attrs.source && !ids.has(child.attrs.source))
        throw new Error("The review contains an unknown manuscript source.");
      if (
        child.attrs.textAlign &&
        !["left", "right", "center", "justify"].includes(child.attrs.textAlign)
      )
        throw new Error("The review contains unsupported paragraph formatting.");
      for (const mark of child.marks) {
        if (mark.type.name === "link" && !/^(https?:|mailto:)/i.test(mark.attrs.href ?? ""))
          throw new Error("The review contains an unsupported link.");
      }
    };
    attributes(node);
    node.descendants(attributes);
  }
  validateNode(base);
  const session = packageData.session;
  if (!session) return;
  const doc = Node.fromJSON(editorialSchema, session.document);
  validateNode(doc);
  normalizeOwnership(doc, packageData.round.sources);
  for (const change of session.changes) {
    if (
      !Number.isSafeInteger(change.from) ||
      !Number.isSafeInteger(change.to) ||
      change.from < 0 ||
      change.to < change.from ||
      change.to > base.content.size
    ) {
      throw new Error("A feedback annotation refers to a passage outside this manuscript.");
    }
    if (!Slice.fromJSON(editorialSchema, change.before).eq(base.slice(change.from, change.to)))
      throw new Error("A feedback annotation does not match its original passage.");
    if (
      change.anchor_offset &&
      (change.kind !== "comment" ||
        change.anchor_offset.length !== 2 ||
        change.anchor_offset.some((v) => !Number.isSafeInteger(v) || v < 0) ||
        change.anchor_offset[1] < change.anchor_offset[0])
    )
      throw new Error("A comment has an invalid passage anchor.");
    const after = Slice.fromJSON(editorialSchema, change.after);
    after.content.forEach(validateNode);
  }
}

/** Internal block ownership is kept in JSON, never copied from pasted HTML. */
const ownership = Extension.create({
  name: "manuscriptOwnership",
  addGlobalAttributes() {
    return [
      {
        types: ["paragraph", "blockquote"],
        attributes: {
          source: { default: null, rendered: false, parseHTML: () => null },
        },
      },
    ];
  },
});
export const editorialExtensions = [...reviewExtensions, ownership];
export const editorialSchema = getSchema(editorialExtensions);

function own(node: Node, source: string): Node {
  if (node.isText) return node;
  if (node.attrs.source === source && (node.isTextblock || node.type.name !== "blockquote"))
    return node;
  return node.type.create(
    { ...node.attrs, source },
    Fragment.fromArray(
      Array.from({ length: node.childCount }, (_, i) => own(node.child(i), source))
    ),
    node.marks
  );
}

const manuscripts = new WeakMap<EditorialSource[], Node>();
export function manuscript(sources: EditorialSource[]): Node {
  const cached = manuscripts.get(sources);
  if (cached) return cached;
  const blocks: Node[] = [];
  for (const source of sources) {
    const root = document.createElement("div");
    root.innerHTML = source.html;
    const parsed = DOMParser.fromSchema(editorialSchema).parse(root);
    parsed.forEach((node) => blocks.push(own(node, source.id)));
  }
  const result = editorialSchema.node(
    "doc",
    null,
    blocks.length ? blocks : [editorialSchema.nodes.paragraph.create()]
  );
  manuscripts.set(sources, result);
  return result;
}

/** Pasted blocks inherit their insertion source. Consumed sources may be empty;
 * records and outline order survive even a replacement spanning many scenes. */
const ownedDocuments = new WeakMap<Node, { ids: string[]; doc: Node }>();
export function normalizeOwnership(doc: Node, sources: EditorialSource[]): Node {
  const cached = ownedDocuments.get(doc);
  if (
    cached &&
    cached.ids.length === sources.length &&
    sources.every((s, i) => s.id === cached.ids[i])
  )
    return cached.doc;
  const ids = new Set(sources.map((s) => s.id));
  let last = sources[0]?.id;
  const blocks: Node[] = [];
  doc.forEach((node) => {
    const source = ids.has(node.attrs.source) ? (node.attrs.source as string) : last;
    if (!source) throw new Error("The review has no manuscript sources.");
    blocks.push(own(node, source));
    last = source;
  });
  const order = blocks.map((b) => sources.findIndex((s) => s.id === b.attrs.source));
  if (order.some((v, i) => i > 0 && v < order[i - 1])) {
    throw new Error(
      "This edit would reorder manuscript sources. Copy the passage as text instead."
    );
  }
  const normalized = doc.type.create(doc.attrs, blocks);
  // Native transaction nodes are immutable. Reuse normalization across change
  // tracking and markup so the shared-schema conversion runs once per edit.
  const entry = { ids: sources.map((s) => s.id), doc: normalized };
  ownedDocuments.set(doc, entry);
  ownedDocuments.set(normalized, entry);
  return normalized;
}

export function sourceHtml(doc: Node, sources: EditorialSource[]): Map<string, string> {
  const groups = new Map(sources.map((s) => [s.id, [] as Node[]]));
  doc.forEach((node) => {
    const group = groups.get(node.attrs.source);
    if (!group)
      throw new Error(
        "A manuscript paragraph has lost its source. Reopen the review before continuing."
      );
    group.push(node);
  });
  return new Map(
    [...groups].map(([id, blocks]) => {
      const root = document.createElement("div");
      root.append(
        DOMSerializer.fromSchema(editorialSchema).serializeFragment(Fragment.fromArray(blocks))
      );
      return [id, root.innerHTML];
    })
  );
}

const markKeys = new WeakMap<readonly import("@tiptap/pm/model").Mark[], string>();
const encoder = {
  encodeCharacter: (char: number, marks: readonly import("@tiptap/pm/model").Mark[]) => {
    let key = markKeys.get(marks);
    if (key === undefined) {
      key = JSON.stringify(marks.map((m) => m.toJSON()));
      markKeys.set(marks, key);
    }
    return `${char}:${key}`;
  },
  encodeNodeStart: (node: Node) => `${node.type.name}:${JSON.stringify(node.attrs)}`,
  encodeNodeEnd: () => "/",
  compareTokens: (a: string, b: string) => a === b,
};

const comparisons = new WeakMap<Node, WeakMap<Node, readonly TrackedChange<string>[]>>();
const tracking = new WeakMap<Node, { doc: Node; set: ChangeSet<string> }>();
const schemaDocuments = new WeakMap<Node, Node>();
function sharedSchema(doc: Node): Node {
  if (doc.type.schema === editorialSchema) return doc;
  let shared = schemaDocuments.get(doc);
  if (!shared) {
    shared = Node.fromJSON(editorialSchema, doc.toJSON());
    schemaDocuments.set(doc, shared);
  }
  return shared;
}

function alignedSourceMaps(before: Node, after: Node): StepMap[] | null {
  const groups = (doc: Node) => {
    const result: { id: string; from: number; content: Fragment }[] = [];
    doc.forEach((node, offset) => {
      const last = result[result.length - 1];
      if (last?.id === node.attrs.source) last.content = last.content.append(Fragment.from(node));
      else result.push({ id: node.attrs.source, from: offset, content: Fragment.from(node) });
    });
    return result;
  };
  const old = groups(before),
    next = groups(after);
  if (old.length !== next.length || old.some((g, i) => !g.id || g.id !== next[i].id)) return null;
  return old
    .flatMap((g, i) =>
      g.content.eq(next[i].content)
        ? []
        : [new StepMap([g.from, g.content.size, next[i].content.size])]
    )
    .reverse();
}

function remember(base: Node, doc: Node, set: ChangeSet<string>) {
  tracking.set(base, { doc, set });
  if (!comparisons.has(base)) comparisons.set(base, new WeakMap());
  comparisons.get(base)!.set(doc, set.changes);
  return set.changes;
}

/** Restore the individual ranges, rather than diffing a whole edited novel and
 * risking the diff library's large-change fallback coalescing unrelated edits. */
export function restoreTracking(base: Node, doc: Node, changes: EditorialChange[]) {
  base = sharedSchema(base);
  doc = sharedSchema(doc);
  const ranges = changes
    .filter((c) => c.kind === "suggestion" && c.state === "open")
    .sort((a, b) => a.from - b.from);
  // ChangeSet consumes sequential step maps. Apply from the end so each saved
  // base position remains valid while reconstructing the original operation order.
  const maps = ranges
    .reverse()
    .map(
      (c) => new StepMap([c.from, c.to - c.from, Slice.fromJSON(editorialSchema, c.after).size])
    );
  return remember(
    base,
    doc,
    ChangeSet.create<string>(base, (a) => a, encoder).addSteps(doc, maps, "change")
  );
}

export function changesBetween(base: Node, next: Node): readonly TrackedChange<string>[] {
  base = sharedSchema(base);
  next = sharedSchema(next);
  const cached = comparisons.get(base)?.get(next);
  if (cached) return cached;
  const previous = tracking.get(base) ?? {
    doc: base,
    set: ChangeSet.create<string>(base, (a) => a, encoder),
  };
  if (previous.doc === base) {
    const maps = alignedSourceMaps(base, next);
    if (maps) return remember(base, next, previous.set.addSteps(next, maps, "change"));
  }
  const start = previous.doc.content.findDiffStart(next.content);
  if (start === null) return remember(base, next, previous.set);
  const end = previous.doc.content.findDiffEnd(next.content)!;
  const overlap = start - Math.min(end.a, end.b);
  const result = previous.set.addSteps(
    next,
    [
      new StepMap([
        start,
        end.a + Math.max(0, overlap) - start,
        end.b + Math.max(0, overlap) - start,
      ]),
    ],
    "change"
  );
  return remember(base, next, result);
}

export function changeMap(changes: readonly TrackedChange<string>[]): Mapping {
  return new Mapping([
    new StepMap(changes.flatMap((c) => [c.fromA, c.toA - c.fromA, c.toB - c.fromB])),
  ]);
}

export function trackChanges(
  base: Node,
  next: Node,
  previous: EditorialChange[],
  edit?: { before: Node; mapping: Mapping }
): EditorialChange[] {
  const result = previous.filter((c) => c.kind === "comment").map((c) => structuredClone(c));
  const commentRanges = edit
    ? result.map((c) => ({ change: c, range: projectedRange(base, edit.before, c) }))
    : [];
  const deltas = changesBetween(base, next);
  const inverse = changeMap(deltas).invert();
  for (const { change, range } of commentRanges) {
    const start = edit!.mapping.map(range.from, 1),
      end = Math.max(start, edit!.mapping.map(range.to, -1));
    const containing = deltas.find((d) => d.fromB <= start && d.toB >= end && d.fromB < d.toB);
    const from = inverse.map(start, -1),
      to = Math.max(from, inverse.map(end, 1));
    const offset: [number, number] | null = containing
      ? [start - containing.fromB, end - containing.fromB]
      : null;
    if (
      change.from !== from ||
      change.to !== to ||
      JSON.stringify(change.anchor_offset ?? null) !== JSON.stringify(offset)
    )
      change.revision++;
    change.from = from;
    change.to = to;
    change.before = base.slice(from, to).toJSON();
    change.anchor_offset = offset;
    if (containing) change.after = next.slice(start, end).toJSON();
  }
  const used = new Set<string>();
  for (const delta of deltas) {
    const before = base.slice(delta.fromA, delta.toA).toJSON();
    const after = next.slice(delta.fromB, delta.toB).toJSON();
    const old =
      previous.find(
        (c) =>
          c.kind === "suggestion" && !used.has(c.id) && c.from === delta.fromA && c.to === delta.toA
      ) ??
      previous.find(
        (c) =>
          c.kind === "suggestion" && !used.has(c.id) && c.from <= delta.toA && c.to >= delta.fromA
      );
    const merged = previous.filter(
      (c) =>
        c.kind === "suggestion" && !used.has(c.id) && c.from <= delta.toA && c.to >= delta.fromA
    );
    for (const change of merged) used.add(change.id);
    const messages = new Map(merged.flatMap((c) => c.messages).map((m) => [m.id, m]));
    const unchanged =
      old &&
      old.from === delta.fromA &&
      old.to === delta.toA &&
      Slice.fromJSON(editorialSchema, old.after).eq(Slice.fromJSON(editorialSchema, after)) &&
      Slice.fromJSON(editorialSchema, old.before).eq(Slice.fromJSON(editorialSchema, before));
    result.push({
      id: old?.id ?? crypto.randomUUID(),
      revision: unchanged ? old.revision : (old?.revision ?? 0) + 1,
      kind: "suggestion",
      from: delta.fromA,
      to: delta.toA,
      before,
      after,
      state: "open",
      messages: [...messages.values()],
      writer_decision: unchanged ? old.writer_decision : undefined,
    });
  }
  return result;
}

export function projectedRange(base: Node, next: Node, change: EditorialChange) {
  const deltas = changesBetween(base, next);
  const matching = deltas.find((d) => d.fromA === change.from && d.toA === change.to);
  if (change.kind === "comment" && change.anchor_offset && matching) {
    return {
      from: Math.min(matching.toB, matching.fromB + change.anchor_offset[0]),
      to: Math.min(matching.toB, matching.fromB + change.anchor_offset[1]),
    };
  }
  if (change.kind === "suggestion" && matching) return { from: matching.fromB, to: matching.toB };
  const mapping = changeMap(deltas);
  return { from: mapping.map(change.from, 1), to: mapping.map(change.to, -1) };
}

export function sliceText(json: ReturnType<Slice["toJSON"]>): string {
  return Slice.fromJSON(editorialSchema, json).content.textBetween(
    0,
    Slice.fromJSON(editorialSchema, json).content.size,
    "\n"
  );
}

/** Render only the prose schema, never package HTML or arbitrary attributes. */
export function sliceHtml(json: ReturnType<Slice["toJSON"]>): string {
  const root = document.createElement("div");
  root.append(
    DOMSerializer.fromSchema(editorialSchema).serializeFragment(
      Slice.fromJSON(editorialSchema, json).content
    )
  );
  return root.innerHTML;
}

export function message(author: string, text: string): EditorialMessage {
  return {
    id: crypto.randomUUID(),
    author: author.trim(),
    text: text.trim(),
    created_at: new Date().toISOString(),
  };
}

function overlaps(a: { from: number; to: number }, b: { from: number; to: number }) {
  if (a.from === a.to || b.from === b.to) return a.from <= b.to && b.from <= a.to;
  return a.from < b.to && b.from < a.to;
}

export function locateChange(base: Node, current: Node, change: EditorialChange) {
  if (change.kind === "comment" && change.anchor_offset)
    return { ...projectedRange(base, current, change), conflict: false };
  const differences = changesBetween(base, current);
  const conflict = differences.some((d) => overlaps(change, { from: d.fromA, to: d.toA }));
  const map = changeMap(differences);
  return { from: map.map(change.from, 1), to: map.map(change.to, -1), conflict };
}

/** Resolve against the current manuscript, not a response's proposed document.
 * All replacements are prepared together before the backend's atomic lock/CAS check. */
export function prepareAcceptance(
  feedback: EditorialFeedback,
  entries: FeedbackEntry[],
  reanchor?: { from: number; to: number }
) {
  const original = manuscript(feedback.round.sources);
  if (!feedback.sources.length)
    throw new Error(
      "There are no current manuscript passages in this review. Restore a scene before applying feedback."
    );
  const current = manuscript(feedback.sources);
  const targets = entries.map((entry) => {
    const location =
      reanchor && entries.length === 1
        ? { ...reanchor, conflict: false }
        : locateChange(original, current, entry.change);
    if (location.conflict)
      throw new Error(
        "This passage changed. Compare the original, current, and suggested prose, then select where the change belongs."
      );
    return { entry, ...location };
  });
  if (targets.some((a, i) => targets.slice(i + 1).some((b) => overlaps(a, b)))) {
    throw new Error("These suggestions overlap. Review and accept them individually.");
  }
  // Apply each affected part in its own source. A normal ProseMirror join across
  // paragraphs otherwise moves the untouched suffix of the last scene into the
  // first one. The editor's gesture remains one atomic suggestion.
  let offset = 0;
  const parts = feedback.sources.map((source) => {
    const doc = manuscript([source]);
    const part = { source, start: offset, end: offset + doc.content.size, doc, empty: false };
    offset = part.end;
    return part;
  });
  for (const target of targets.sort((a, b) => b.from - a.from)) {
    const affected = parts.filter((part) =>
      target.from === target.to
        ? target.from >= part.start && (target.from < part.end || part === parts[parts.length - 1])
        : target.from < part.end && target.to > part.start
    );
    if (!affected.length) throw new Error("Select a passage within the manuscript.");
    affected.forEach((part, index) => {
      const from = Math.max(0, target.from - part.start),
        to = Math.min(part.end - part.start, target.to - part.start);
      const replacement =
        index === 0 ? Slice.fromJSON(editorialSchema, target.entry.change.after) : Slice.empty;
      const owned = new Slice(
        Fragment.fromArray(
          Array.from({ length: replacement.content.childCount }, (_, i) =>
            own(replacement.content.child(i), part.source.id)
          )
        ),
        replacement.openStart,
        replacement.openEnd
      );
      part.empty = from === 0 && to === part.doc.content.size && owned.size === 0;
      part.doc = new Transform(part.doc).replaceRange(from, to, owned).doc;
    });
  }
  const before = sourceHtml(current, feedback.sources);
  const after = new Map(
    parts.map((part) => [
      part.source.id,
      part.empty
        ? ""
        : sourceHtml(normalizeOwnership(part.doc, [part.source]), [part.source]).get(
            part.source.id
          )!,
    ])
  );
  return feedback.sources
    .filter((s) => before.get(s.id) !== after.get(s.id))
    .map((s) => ({ id: s.id, expected: s.html, html: after.get(s.id)! }));
}

export function withdrawSuggestion(base: Node, proposed: Node, change: EditorialChange): Node {
  const range = projectedRange(base, proposed, change);
  return new Transform(proposed).replaceRange(
    range.from,
    range.to,
    Slice.fromJSON(editorialSchema, change.before)
  ).doc;
}

const searchIndexes = new WeakMap<Node, { text: string; starts: number[]; ends: number[] }>();
function foldCharacter(character: string) {
  return character === "ς" ? "σ" : character.toLowerCase();
}

export function searchManuscript(doc: Node, query: string): { from: number; to: number }[] {
  if (!query) return [];
  let index = searchIndexes.get(doc);
  if (!index) {
    const chunks: string[] = [],
      starts: number[] = [],
      ends: number[] = [];
    doc.descendants((node, pos) => {
      if (node.isText) {
        let offset = 0;
        for (const character of node.text!) {
          const folded = foldCharacter(character);
          chunks.push(folded);
          for (let i = 0; i < folded.length; i++) {
            starts.push(pos + offset);
            ends.push(pos + offset + character.length);
          }
          offset += character.length;
        }
      } else if (node.isTextblock && chunks.length) {
        chunks.push("\n");
        starts.push(pos);
        ends.push(pos + 1);
      }
    });
    index = { text: chunks.join(""), starts, ends };
    searchIndexes.set(doc, index);
  }
  const needle = [...query].map(foldCharacter).join("");
  const matches = [];
  for (
    let i = index.text.indexOf(needle);
    i !== -1;
    i = index.text.indexOf(needle, i + needle.length)
  )
    matches.push({ from: index.starts[i], to: index.ends[i + needle.length - 1] });
  return matches;
}
