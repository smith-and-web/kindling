import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 scènes`. One key per CLDR plural category the
 * locale uses (`.one`/`.other` in French); `{count}` is the number.
 * French uses the singular for 0 and 1, the plural for 2 and above.
 */
const catalog: Catalog = {
  "beat.one": "{count} temps fort",
  "beat.other": "{count} temps forts",
  "chapter.one": "{count} chapitre",
  "chapter.other": "{count} chapitres",
  "match.one": "{count} résultat",
  "match.other": "{count} résultats",
  "reference.one": "{count} fiche",
  "reference.other": "{count} fiches",
  "scene.one": "{count} scène",
  "scene.other": "{count} scènes",
  "sequence.one": "{count} séquence",
  "sequence.other": "{count} séquences",
};

export default catalog;
