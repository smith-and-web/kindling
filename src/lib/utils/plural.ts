import { tn } from "../i18n";

/** Nouns with a `counts.<noun>` entry in every catalog. */
export type CountNoun = "beat" | "chapter" | "match" | "reference" | "scene" | "sequence";

/** "1 scene", "2 scenes", "0 references": a count with its noun in agreement. */
export function countLabel(count: number, noun: CountNoun): string {
  return tn(`counts.${noun}`, count);
}
