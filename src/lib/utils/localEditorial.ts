import { Fragment, Slice } from "@tiptap/pm/model";
import {
  manuscript,
  editorialSchema,
  locateChange,
  type EditorialSource,
  type EditorialChange,
} from "./editorial";
import { activeDocuments, isAnchored, type SceneReview, type Annotation } from "./revisions";

export interface LocalAnnotation {
  key: string;
  review: SceneReview;
  annotation: Annotation;
  change: EditorialChange;
  from: number;
  to: number;
  conflict: boolean;
  current: string;
  locked: boolean;
}
function textSlice(text: string) {
  return text ? new Slice(Fragment.from(editorialSchema.text(text)), 0, 0).toJSON() : null;
}
/** Adapt stored scene annotations for the continuous view without rewriting their
 * anchors or losing inactive prose/history. Decisions still use the original CAS. */
export function localAnnotations(
  reviews: SceneReview[],
  sources: EditorialSource[]
): LocalAnnotation[] {
  const starts = new Map<string, number>();
  let position = 0;
  for (const source of sources) {
    starts.set(source.id, position);
    position += manuscript([source]).content.size;
  }
  return reviews.flatMap((review) =>
    review.data.annotations.flatMap((annotation) => {
      const source = sources.find(
        (s) => s.scene_id === review.scene_id && s.id === annotation.document_id
      );
      if (!source || !activeDocuments(review).some((d) => d.id === source.id)) return [];
      const base = manuscript([{ ...source, html: annotation.anchor_html }]);
      const current = manuscript([source]);
      const from = Math.max(0, Math.min(annotation.from, base.content.size));
      const to = Math.max(from, Math.min(annotation.to, base.content.size));
      const change: EditorialChange = {
        id: `scene:${review.scene_id}:${annotation.id}`,
        revision: 1,
        kind: annotation.replacement === null ? "comment" : "suggestion",
        from,
        to,
        before: base.slice(from, to).toJSON(),
        after: textSlice(annotation.replacement ?? ""),
        state: annotation.state === "open" ? "open" : "resolved",
        messages: annotation.messages.map((m, i) => ({ ...m, id: `${annotation.id}:${i}` })),
      };
      const range = locateChange(base, current, change);
      const offset = starts.get(source.id)!;
      return [
        {
          key: change.id,
          review,
          annotation,
          change,
          from: offset + range.from,
          to: offset + range.to,
          conflict: !isAnchored(annotation, review.documents),
          current: current.textBetween(range.from, Math.max(range.from, range.to), "\n"),
          locked: source.locked,
        },
      ];
    })
  );
}

export function localSelection(sources: EditorialSource[], from: number, to: number) {
  let offset = 0;
  for (const source of sources) {
    const doc = manuscript([source]);
    if (from >= offset + 1 && to <= offset + doc.content.size - 1) {
      return {
        source,
        from: from - offset,
        to: to - offset,
        quote: doc.textBetween(from - offset, to - offset, "\n"),
      };
    }
    offset += doc.content.size;
  }
  throw new Error(
    "Select a passage within this scene’s original prose to re-anchor its saved annotation."
  );
}
