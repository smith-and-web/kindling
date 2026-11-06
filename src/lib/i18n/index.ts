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
import type { Catalog, Locale, LocaleEntry } from "./types";

/**
 * Supported interface locales, keyed by their catalog directory under `locales/`.
 * `languages` lists the system language subtags a locale serves. A system tag that names a
 * locale exactly (`pt-BR`) wins over a language match (`pt`), and anything unmatched reads
 * English. Traditional Chinese systems (`zh-TW`, `zh-HK`, `zh-Hant`) read the Simplified copy,
 * the closest translation available until a `zh-Hant` catalog exists.
 *
 * Keep `src-tauri/src/menu.rs` in step: `detection.json` holds the cases both test suites check.
 */
export const LOCALES = {
  en: { languages: ["en"] },
  "zh-CN": { languages: ["zh"] },
  es: { languages: ["es"] },
  "pt-BR": { languages: ["pt"] },
} as const satisfies Record<string, LocaleEntry>;

export const FALLBACK_LOCALE: Locale = "en";

type CatalogModule = { default: Catalog };

const modules = import.meta.glob<CatalogModule>("./locales/*/*.ts", { eager: true });

/** Group catalog modules by locale directory, prefixing each key with its file's namespace. */
export function mergeCatalogs(found: Record<string, CatalogModule>): Record<string, Catalog> {
  const catalogs: Record<string, Catalog> = {};
  for (const [path, module] of Object.entries(found)) {
    const [locale, file] = path.split("/").slice(-2);
    const namespace = file.replace(/\.ts$/, "");
    const catalog = (catalogs[locale] ??= {});
    for (const [key, text] of Object.entries(module.default)) catalog[`${namespace}.${key}`] = text;
  }
  return catalogs;
}

const CATALOGS = mergeCatalogs(modules);
const EN = CATALOGS[FALLBACK_LOCALE];

const catalogFor = (locale: Locale): Catalog => CATALOGS[locale] ?? EN;

/** Map a BCP 47 tag such as `zh-Hant-TW` or `en-GB` to the supported locale that serves it. */
export function resolveLocale(tag: string | undefined | null): Locale {
  const wanted = (tag ?? "").trim().replace(/_/g, "-").toLowerCase();
  const locales = Object.keys(LOCALES) as Locale[];
  const exact = locales.find((locale) => locale.toLowerCase() === wanted);
  if (exact) return exact;
  const language = wanted.split("-")[0];
  const match = locales.find((locale) =>
    (LOCALES[locale].languages as readonly string[]).includes(language)
  );
  return match ?? FALLBACK_LOCALE;
}

/**
 * The locale at startup. Tests always read English. Otherwise a dev-server
 * `VITE_KINDLING_LOCALE` (set directly or through `KINDLING_LOCALE`, see `vite.config.ts`)
 * wins over the system language, so visual QA can screenshot every locale on one machine.
 */
export function initialLocale(env: ImportMetaEnv, systemTag: string | undefined): Locale {
  if (env.MODE === "test" || env.VITEST) return FALLBACK_LOCALE;
  const override = env.DEV ? env.VITE_KINDLING_LOCALE : undefined;
  return resolveLocale(override || systemTag);
}

/**
 * Resolved once at module load. Module-level copy (command labels, reference types, import
 * formats) is translated when it is imported, so a future in-app language switch has to
 * re-render those rather than only calling `setLocale()`.
 */
const DEFAULT_LOCALE: Locale = initialLocale(
  import.meta.env,
  typeof navigator === "undefined" ? undefined : navigator.language
);

let currentLocale: Locale = DEFAULT_LOCALE;

export const getLocale = (): Locale => currentLocale;

export function setLocale(locale: Locale): void {
  currentLocale = locale;
}

function interpolate(text: string, params?: Record<string, string | number>): string {
  if (!params) return text;
  let out = text;
  for (const [name, value] of Object.entries(params))
    out = out.replaceAll(`{${name}}`, String(value));
  return out;
}

/**
 * Translate a key, interpolating `{name}` placeholders.
 * Falls back to English, then to the key itself so a missing entry stays visible but harmless.
 */
export function t(key: string, params?: Record<string, string | number>): string {
  return interpolate(catalogFor(currentLocale)[key] ?? EN[key] ?? key, params);
}

const pluralRules = new Map<Locale, Intl.PluralRules>();

/** CLDR plural category (`one`, `few`, `many`, `other`, ...) for a count in a locale. */
export function pluralCategory(locale: Locale, count: number): Intl.LDMLPluralRule {
  let rules = pluralRules.get(locale);
  if (!rules) pluralRules.set(locale, (rules = new Intl.PluralRules(locale)));
  return rules.select(count);
}

/**
 * Translate a counted phrase. Catalogs hold one key per CLDR plural category the locale
 * uses (`sidebar.sceneCount.one`, `sidebar.sceneCount.other`; Russian adds `.few` and
 * `.many`), and `count` is interpolated as `{count}`. A missing category falls back to
 * `.other`, then to English, then to the key.
 */
export function tn(key: string, count: number, params?: Record<string, string | number>): string {
  const catalog = catalogFor(currentLocale);
  const text =
    catalog[`${key}.${pluralCategory(currentLocale, count)}`] ??
    catalog[`${key}.other`] ??
    EN[`${key}.${pluralCategory(FALLBACK_LOCALE, count)}`] ??
    EN[`${key}.other`] ??
    key;
  return interpolate(text, { count, ...params });
}
