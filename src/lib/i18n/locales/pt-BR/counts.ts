import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 cenas`. One key per CLDR plural category the
 * locale uses (`.one`/`.other` in pt-BR); `{count}` is the number.
 * CLDR pt: one = n = 0..1; other = n ≥ 2.
 */
const catalog: Catalog = {
  "beat.one": "{count} beat",
  "beat.other": "{count} beats",
  "chapter.one": "{count} capítulo",
  "chapter.other": "{count} capítulos",
  "match.one": "{count} correspondência",
  "match.other": "{count} correspondências",
  "reference.one": "{count} referência",
  "reference.other": "{count} referências",
  "scene.one": "{count} cena",
  "scene.other": "{count} cenas",
  "sequence.one": "{count} sequência",
  "sequence.other": "{count} sequências",
};

export default catalog;
