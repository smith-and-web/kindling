/**
 * Locale-aware formatting for dates, numbers and file sizes.
 *
 * Always formats with the active interface locale (`getLocale()`), never the OS locale, so a
 * Russian interface on an English system shows Russian month names and decimal commas.
 */
import { getLocale, t } from "./index";

type DateInput = Date | string | number;

const toDate = (value: DateInput) => (value instanceof Date ? value : new Date(value));

/** A date (and optionally time) in the active locale; an unparseable value reads as "". */
export function formatDate(value: DateInput, options?: Intl.DateTimeFormatOptions): string {
  const date = toDate(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(getLocale(), options ?? { dateStyle: "medium" });
}

/** A number in the active locale: digit grouping and decimal separator. */
export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return value.toLocaleString(getLocale(), options);
}

/** A number with exactly one decimal place, as `toFixed(1)` would give but localized. */
export const formatDecimal = (value: number, digits = 1): string =>
  formatNumber(value, { minimumFractionDigits: digits, maximumFractionDigits: digits });

/** Bytes as B, KB or MB with one decimal, in the active locale's digits and unit labels. */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return t("common.sizeBytes", { size: formatNumber(bytes) });
  if (bytes < 1024 * 1024) return t("common.sizeKilobytes", { size: formatDecimal(bytes / 1024) });
  return t("common.sizeMegabytes", { size: formatDecimal(bytes / (1024 * 1024)) });
}
