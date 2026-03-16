import { describe, it, expect } from "vitest";
import { COMMAND_DEFS, fuzzyMatch, fuzzyScore } from "./commands";

describe("COMMAND_DEFS", () => {
  it("has unique ids", () => {
    const ids = COMMAND_DEFS.map((c) => c.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("has unique shortcuts", () => {
    const shortcuts = COMMAND_DEFS.map((c) => c.shortcut);
    expect(new Set(shortcuts).size).toBe(shortcuts.length);
  });

  it("every command has a non-empty label", () => {
    for (const cmd of COMMAND_DEFS) {
      expect(cmd.label.length).toBeGreaterThan(0);
    }
  });

  it("every command belongs to a valid category", () => {
    const validCategories = ["File", "View", "Edit", "Help", "Project"];
    for (const cmd of COMMAND_DEFS) {
      expect(validCategories).toContain(cmd.category);
    }
  });

  it("contains expected commands", () => {
    const ids = COMMAND_DEFS.map((c) => c.id);
    expect(ids).toContain("export");
    expect(ids).toContain("toggle_sidebar");
    expect(ids).toContain("quick_start");
    expect(ids).toContain("sync");
  });
});

describe("fuzzyMatch", () => {
  it("matches when query chars appear in order", () => {
    expect(fuzzyMatch("exp", "Export project")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(fuzzyMatch("EXP", "export project")).toBe(true);
    expect(fuzzyMatch("exp", "EXPORT PROJECT")).toBe(true);
  });

  it("returns true for empty query", () => {
    expect(fuzzyMatch("", "anything")).toBe(true);
    expect(fuzzyMatch("  ", "anything")).toBe(true);
  });

  it("returns false when chars do not appear in order", () => {
    expect(fuzzyMatch("zxy", "Export project")).toBe(false);
  });

  it("matches non-contiguous characters", () => {
    expect(fuzzyMatch("epr", "Export project")).toBe(true);
  });

  it("returns false when query is longer than can match", () => {
    expect(fuzzyMatch("abcdefghijklmnop", "abc")).toBe(false);
  });

  it("handles single character queries", () => {
    expect(fuzzyMatch("e", "Export")).toBe(true);
    expect(fuzzyMatch("z", "Export")).toBe(false);
  });
});

describe("fuzzyScore", () => {
  it("returns 0 for empty query", () => {
    expect(fuzzyScore("", "anything")).toBe(0);
    expect(fuzzyScore("  ", "anything")).toBe(0);
  });

  it("returns -1 when no match", () => {
    expect(fuzzyScore("zxy", "Export project")).toBe(-1);
  });

  it("returns positive score for valid matches", () => {
    expect(fuzzyScore("exp", "Export project")).toBeGreaterThan(0);
  });

  it("scores earlier matches higher", () => {
    const earlyScore = fuzzyScore("e", "export");
    const lateScore = fuzzyScore("t", "export");
    expect(earlyScore).toBeGreaterThan(lateScore);
  });

  it("gives word boundary bonus", () => {
    const boundaryScore = fuzzyScore("p", "export project");
    const midWordScore = fuzzyScore("r", "export project");
    expect(boundaryScore).toBeGreaterThan(midWordScore);
  });

  it("is case-insensitive", () => {
    expect(fuzzyScore("EXP", "export")).toBe(fuzzyScore("exp", "export"));
  });

  it("scores exact prefix highly", () => {
    const prefixScore = fuzzyScore("exp", "export");
    expect(prefixScore).toBeGreaterThan(0);
  });

  it("returns -1 for partial non-match", () => {
    expect(fuzzyScore("exz", "export")).toBe(-1);
  });
});
