/**
 * UI translation layer.
 *
 * Catalogs live in `src/lib/i18n/locales/<locale>/<group>.ts`. Each module default-exports a
 * flat `{ key: text }` record and becomes the namespace from its file name, so keys look like
 * `sidebar.newScene` or `common.ok`.
 *
 * The English catalog mirrors the original wording exactly, which keeps unit tests (they run
 * with `import.meta.env.MODE === "test"`, i.e. English) and the source of truth in sync.
 */
import type { Catalog, Locale } from "./types";

type CatalogModule = { default: Catalog };

const enModules = import.meta.glob<CatalogModule>("./locales/en/*.ts", { eager: true });
const zhModules = import.meta.glob<CatalogModule>("./locales/zh-CN/*.ts", { eager: true });

function merge(modules: Record<string, CatalogModule>): Catalog {
  const catalog: Catalog = {};
  for (const [path, module] of Object.entries(modules)) {
    const namespace = path.split("/").pop()!.replace(/\.ts$/, "");
    for (const [key, text] of Object.entries(module.default)) catalog[`${namespace}.${key}`] = text;
  }
  return catalog;
}

const EN = merge(enModules);
const ZH = merge(zhModules);

const catalogFor = (locale: Locale): Catalog => (locale === "zh-CN" ? ZH : EN);

/**
 * Follow the system language: Chinese systems get the translated catalog, everything
 * else keeps the original English copy. Traditional Chinese locales (`zh-TW`, `zh-HK`,
 * `zh-Hant`) match too, so they read the Simplified copy — the closest translation
 * available until a `zh-Hant` catalog exists.
 */
function detectedLocale(): Locale {
  const tag = typeof navigator === "undefined" ? "en" : navigator.language;
  return tag.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

/**
 * Resolved once at module load. Module-level copy (command labels, reference types, import
 * formats) is translated when it is imported, so a future in-app language switch has to
 * re-render those rather than only calling `setLocale()`.
 */
const DEFAULT_LOCALE: Locale =
  import.meta.env.MODE === "test" || import.meta.env.VITEST ? "en" : detectedLocale();

let currentLocale: Locale = DEFAULT_LOCALE;

export const getLocale = (): Locale => currentLocale;

export function setLocale(locale: Locale): void {
  currentLocale = locale;
}

/**
 * Translate a key, interpolating `{name}` placeholders.
 * Falls back to English, then to the key itself so a missing entry stays visible but harmless.
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const text = catalogFor(currentLocale)[key] ?? EN[key] ?? key;
  if (!params) return text;
  let out = text;
  for (const [name, value] of Object.entries(params))
    out = out.replaceAll(`{${name}}`, String(value));
  return out;
}

/** Noun inventory for `countLabel()`; Chinese needs a measure word that English does not. */
const ZH_NOUNS: Record<string, string> = {
  backup: "个备份",
  beat: "个节拍",
  chapter: "章",
  character: "个角色",
  entry: "个条目",
  file: "个文件",
  item: "项",
  line: "行",
  match: "处匹配",
  note: "条笔记",
  page: "页",
  part: "部",
  project: "个项目",
  reference: "条引用",
  revision: "个修订版本",
  scene: "个场景",
  tag: "个标签",
  word: "个字",
};

/** Locale-aware noun phrase for counts, e.g. `3 scenes` / `3 个场景`. */
export function nounLabel(noun: string): string {
  if (currentLocale !== "zh-CN") return noun;
  return ZH_NOUNS[noun] ?? noun;
}

export function nounPluralLabel(noun: string, pluralNoun: string): string {
  if (currentLocale !== "zh-CN") return pluralNoun;
  return ZH_NOUNS[noun] ?? pluralNoun;
}
