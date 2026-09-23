import { describe, expect, it } from "vitest";
import { countLabel } from "./plural";

describe("countLabel", () => {
  it("agrees the noun with the count", () => {
    expect(countLabel(1, "scene")).toBe("1 scene");
    expect(countLabel(0, "reference")).toBe("0 references");
    expect(countLabel(3, "chapter")).toBe("3 chapters");
    expect(countLabel(2, "match", "matches")).toBe("2 matches");
  });
});
