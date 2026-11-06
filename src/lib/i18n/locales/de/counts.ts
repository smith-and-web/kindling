import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 Szenen`. One key per CLDR plural category the
 * locale uses (`.one`/`.other` in German); `{count}` is the number.
 */
const catalog: Catalog = {
  "beat.one": "{count} Beat",
  "beat.other": "{count} Beats",
  "chapter.one": "{count} Kapitel",
  "chapter.other": "{count} Kapitel",
  "match.one": "{count} Treffer",
  "match.other": "{count} Treffer",
  "reference.one": "{count} Referenz",
  "reference.other": "{count} Referenzen",
  "scene.one": "{count} Szene",
  "scene.other": "{count} Szenen",
  "sequence.one": "{count} Sequenz",
  "sequence.other": "{count} Sequenzen",
};

export default catalog;
