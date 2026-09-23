/** "1 scene", "2 scenes", "0 references": a count with its noun in agreement. */
export function countLabel(count: number, noun: string, pluralNoun = `${noun}s`): string {
  return `${count} ${count === 1 ? noun : pluralNoun}`;
}
