import { nounLabel, nounPluralLabel } from "../i18n";

/** "1 scene", "2 scenes", "0 references": a count with its noun in agreement. */
export function countLabel(count: number, noun: string, pluralNoun = `${noun}s`): string {
  const label = count === 1 ? nounLabel(noun) : nounPluralLabel(noun, pluralNoun);
  return `${count} ${label}`;
}
