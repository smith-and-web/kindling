import pressTokens from "../../styles/press/tokens.json";

/** The Press tag palette, in the order the tag editor offers it. */
export const TAG_COLOR_NAMES = [
  "red",
  "orange",
  "yellow",
  "green",
  "teal",
  "blue",
  "purple",
  "pink",
  "slate",
  "cyan",
] as const;

const BY_STORED_VALUE = new Map(
  TAG_COLOR_NAMES.map((name) => [
    pressTokens.light[`--tag-${name}` as keyof typeof pressTokens.light].toLowerCase(),
    name,
  ])
);

/**
 * Tags persist the stable light `--tag-*` value (user data must not change
 * with the theme). Render them through `--color-tag-*`, which Press swaps for
 * the dark palette in dark chrome. Colours outside the palette (older projects,
 * imports) render as stored.
 */
export function tagColor(stored: string | null | undefined): string | null {
  if (!stored) return null;
  const name = BY_STORED_VALUE.get(stored.trim().toLowerCase());
  return name ? `var(--color-tag-${name})` : stored;
}
