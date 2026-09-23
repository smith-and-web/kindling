import { describe, expect, it } from "vitest";
import pressTokens from "../../styles/press/tokens.json";
import { TAG_COLOR_NAMES, tagColor } from "./tagColor";

describe("tagColor", () => {
  it("renders every stored palette value through its themed token", () => {
    for (const name of TAG_COLOR_NAMES) {
      const stored = pressTokens.light[`--tag-${name}` as keyof typeof pressTokens.light];
      expect(tagColor(stored)).toBe(`var(--color-tag-${name})`);
      expect(tagColor(stored.toLowerCase())).toBe(`var(--color-tag-${name})`);
    }
  });

  it("keeps colours outside the palette as stored", () => {
    expect(tagColor("#123456")).toBe("#123456");
  });

  it("returns null when a tag has no colour", () => {
    expect(tagColor(null)).toBeNull();
    expect(tagColor(undefined)).toBeNull();
    expect(tagColor("")).toBeNull();
  });
});
