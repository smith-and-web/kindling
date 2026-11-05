import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 scenes`. One key per CLDR plural category the
 * locale uses (`.one`/`.other` in English); `{count}` is the number.
 */
const catalog: Catalog = {
  "beat.one": "{count} beat",
  "beat.other": "{count} beats",
  "chapter.one": "{count} chapter",
  "chapter.other": "{count} chapters",
  "match.one": "{count} match",
  "match.other": "{count} matches",
  "reference.one": "{count} reference",
  "reference.other": "{count} references",
  "scene.one": "{count} scene",
  "scene.other": "{count} scenes",
  "sequence.one": "{count} sequence",
  "sequence.other": "{count} sequences",
};

export default catalog;
