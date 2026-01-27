import { describe, expect, it } from "vitest";
import {
  DEFAULT_REFERENCE_TYPES,
  REFERENCE_TYPE_OPTIONS,
  normalizeReferenceTypes,
} from "./referenceTypes";

describe("referenceTypes", () => {
  it("builds default types from options", () => {
    const expectedDefaults = REFERENCE_TYPE_OPTIONS.filter((option) => option.isDefault).map(
      (option) => option.id
    );

    expect(DEFAULT_REFERENCE_TYPES).toEqual(expectedDefaults);
  });

  it("normalizes to defaults when types are missing", () => {
    expect(normalizeReferenceTypes()).toEqual(DEFAULT_REFERENCE_TYPES);
    expect(normalizeReferenceTypes(null)).toEqual(DEFAULT_REFERENCE_TYPES);
  });

  it("filters unknown and duplicate reference types", () => {
    const input = ["characters", "items", "characters", "unknown", "locations"];
    expect(normalizeReferenceTypes(input)).toEqual(["characters", "items", "locations"]);
  });

  it("returns empty when no allowed types are present", () => {
    expect(normalizeReferenceTypes(["unknown", "invalid"])).toEqual([]);
  });
});
