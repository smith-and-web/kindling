import { afterEach, describe, expect, it } from "vitest";
import { setLocale } from "./index";
import { formatDate, formatDecimal, formatFileSize, formatNumber } from "./format";

afterEach(() => setLocale("en"));

// Noon UTC keeps the calendar day stable in every test runner time zone.
const date = new Date("2026-03-04T12:00:00Z");

describe("formatDate()", () => {
  it("formats with the interface locale, not the OS locale", () => {
    const options = { month: "long", day: "numeric", year: "numeric" } as const;
    expect(formatDate(date, options)).toBe("March 4, 2026");
    setLocale("de");
    expect(formatDate(date, options)).toBe("4. März 2026");
    setLocale("ru");
    expect(formatDate(date, options)).toBe("4 марта 2026 г.");
  });

  it("accepts ISO strings and timestamps", () => {
    const options = { year: "numeric" } as const;
    expect(formatDate(date.toISOString(), options)).toBe("2026");
    expect(formatDate(date.getTime(), options)).toBe("2026");
  });

  it("defaults to a medium date", () => {
    expect(formatDate(date)).toBe("Mar 4, 2026");
  });

  it("reads an unparseable value as empty", () => {
    expect(formatDate("not a date")).toBe("");
  });
});

describe("formatNumber()", () => {
  it("groups digits per locale", () => {
    expect(formatNumber(12345)).toBe("12,345");
    setLocale("de");
    expect(formatNumber(12345)).toBe("12.345");
    setLocale("ru");
    expect(formatNumber(12345)).toBe("12 345");
  });

  it("uses the locale's decimal separator", () => {
    expect(formatDecimal(1.25)).toBe("1.3");
    expect(formatDecimal(2)).toBe("2.0");
    setLocale("pt-BR");
    expect(formatDecimal(0.5, 2)).toBe("0,50");
    expect(formatNumber(1.6)).toBe("1,6");
  });
});

describe("formatFileSize()", () => {
  it("picks bytes, kilobytes or megabytes", () => {
    expect(formatFileSize(512)).toBe("512 B");
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("localizes digits and unit labels", () => {
    setLocale("ru");
    expect(formatFileSize(1536)).toBe("1,5 КБ");
    expect(formatFileSize(3 * 1024 * 1024 + 1024 * 512)).toBe("3,5 МБ");
    setLocale("de");
    expect(formatFileSize(1536)).toBe("1,5 KB");
  });
});
