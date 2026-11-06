import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 сцены`. One key per CLDR plural category the
 * locale uses (`.one`/`.few`/`.many` in Russian); `{count}` is the number.
 * CLDR ru: one = n mod 10 = 1 and n mod 100 ≠ 11; few = n mod 10 = 2..4 and n mod 100 ≠ 12..14;
 * many = everything else.
 */
const catalog: Catalog = {
  "beat.one": "{count} бит",
  "beat.few": "{count} бита",
  "beat.many": "{count} битов",
  "chapter.one": "{count} глава",
  "chapter.few": "{count} главы",
  "chapter.many": "{count} глав",
  "match.one": "{count} совпадение",
  "match.few": "{count} совпадения",
  "match.many": "{count} совпадений",
  "reference.one": "{count} ссылка",
  "reference.few": "{count} ссылки",
  "reference.many": "{count} ссылок",
  "scene.one": "{count} сцена",
  "scene.few": "{count} сцены",
  "scene.many": "{count} сцен",
  "sequence.one": "{count} секвенция",
  "sequence.few": "{count} секвенции",
  "sequence.many": "{count} секвенций",
};

export default catalog;
