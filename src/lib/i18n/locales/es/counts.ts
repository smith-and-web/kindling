import type { Catalog } from "../../types";

/**
 * Counted nouns for `countLabel()`, e.g. `3 escenas`. One key per CLDR plural category the
 * locale uses for whole numbers (`.one`/`.other` in Spanish); `{count}` is the number.
 */
const catalog: Catalog = {
  "beat.one": "{count} tiempo",
  "beat.other": "{count} tiempos",
  "chapter.one": "{count} capítulo",
  "chapter.other": "{count} capítulos",
  "match.one": "{count} coincidencia",
  "match.other": "{count} coincidencias",
  "reference.one": "{count} referencia",
  "reference.other": "{count} referencias",
  "scene.one": "{count} escena",
  "scene.other": "{count} escenas",
  "sequence.one": "{count} secuencia",
  "sequence.other": "{count} secuencias",
};

export default catalog;
