import { afterEach, describe, expect, it } from "vitest";
import { LOCALES, setLocale } from "./i18n";
import type { Locale } from "./i18n/types";
import {
  DEFAULT_REFERENCE_TYPES,
  REFERENCE_TYPE_OPTIONS,
  normalizeReferenceTypes,
  referenceTypeText,
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

describe("referenceTypeText", () => {
  afterEach(() => setLocale("en"));
  const bases = [
    "references.referencesPanel.empty",
    "references.referencesPanel.emptyHint",
    "references.referenceEditDialog.titleAdd",
    "references.referenceEditDialog.titleEdit",
  ];

  it("has a whole phrase for every type and the neutral fallback in every locale", () => {
    for (const locale of Object.keys(LOCALES) as Locale[]) {
      setLocale(locale);
      for (const base of [...bases, "references.referencesPanel.add"])
        for (const option of REFERENCE_TYPE_OPTIONS)
          expect(referenceTypeText(base, option.id), `${locale} ${base}`).not.toContain(base);
      for (const base of bases)
        expect(referenceTypeText(base, null), `${locale} ${base}`).not.toContain(base);
    }
  });

  it("falls back to the neutral wording for an unknown or missing type", () => {
    expect(referenceTypeText("references.referencesPanel.empty", "characters")).toBe(
      "No characters yet"
    );
    expect(referenceTypeText("references.referencesPanel.empty", "dragons")).toBe(
      "No references yet"
    );
    setLocale("ru");
    expect(referenceTypeText("references.referenceEditDialog.titleEdit", "locations")).toBe(
      "Редактировать место"
    );
    expect(referenceTypeText("references.referenceEditDialog.titleAdd", undefined)).toBe(
      "Добавить ссылку"
    );
  });
});
