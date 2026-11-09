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
import { createSubscriber } from "svelte/reactivity";
import type { Catalog, LanguagePreference, Locale, LocaleEntry } from "./types";

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
  de: { languages: ["de"] },
  en: { languages: ["en"] },
  es: { languages: ["es"] },
  fr: { languages: ["fr"] },
  "pt-BR": { languages: ["pt"] },
  ru: { languages: ["ru"] },
  "zh-CN": { languages: ["zh"] },
} as const satisfies Record<string, LocaleEntry>;

export const FALLBACK_LOCALE: Locale = "en";

/**
 * Each locale named in its own language, for the Settings language picker. A writer looking
 * for their language should recognise it whatever the interface currently reads.
 */
export const LANGUAGE_NAMES: Record<Locale, string> = {
  en: "English",
  "zh-CN": "简体中文",
  es: "Español",
  "pt-BR": "Português (Brasil)",
  ru: "Русский",
  de: "Deutsch",
  fr: "Français",
};

/** localStorage key for the writer's interface-language choice, next to `kindling:theme`. */
export const LANGUAGE_STORAGE_KEY = "kindling:language";

export const isLocale = (value: unknown): value is Locale =>
  typeof value === "string" && Object.prototype.hasOwnProperty.call(LOCALES, value);

/** The stored language choice, or `"system"` when none (or an unknown one) is stored. */
export function storedLanguagePreference(
  storage: Pick<Storage, "getItem"> | undefined
): LanguagePreference {
  let stored: string | null = null;
  try {
    stored = storage?.getItem(LANGUAGE_STORAGE_KEY) ?? null;
  } catch {
    // Storage can be unavailable; the system language still applies.
  }
  return isLocale(stored) ? stored : "system";
}

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
 * The locale a language choice reads. A chosen language always wins. Under "system", a
 * dev-server `VITE_KINDLING_LOCALE` (set directly or through `KINDLING_LOCALE`, see
 * `vite.config.ts`) stands in for the system language, so visual QA can screenshot every
 * locale on one machine. Keep `pick_locale` in `src-tauri/src/menu.rs` in step.
 */
export function localeFor(
  preference: LanguagePreference,
  env: ImportMetaEnv,
  systemTag: string | undefined
): Locale {
  if (preference !== "system") return preference;
  const override = env.DEV ? env.VITE_KINDLING_LOCALE : undefined;
  return resolveLocale(override || systemTag);
}

/** The locale at startup. Tests always start in English, whatever is stored. */
export function initialLocale(
  env: ImportMetaEnv,
  systemTag: string | undefined,
  preference: LanguagePreference = "system"
): Locale {
  if (env.MODE === "test" || env.VITEST) return FALLBACK_LOCALE;
  return localeFor(preference, env, systemTag);
}

/** The system language tag the webview reports, if any. */
export const systemLanguageTag = (): string | undefined =>
  typeof navigator === "undefined" ? undefined : navigator.language;

const DEFAULT_LOCALE: Locale = initialLocale(
  import.meta.env,
  systemLanguageTag(),
  storedLanguagePreference(typeof localStorage === "undefined" ? undefined : localStorage)
);

let currentLocale: Locale = DEFAULT_LOCALE;
let localeChanged: (() => void) | undefined;

/**
 * Reading the locale inside a template, `$derived` or `$effect` subscribes to it, so every
 * `t()` / `tn()` call on screen re-renders when `setLocale()` switches language. Copy that
 * must follow a switch is therefore read lazily (a getter or a render-time call), never
 * captured into a constant at module load.
 */
const subscribe = createSubscriber((update) => {
  localeChanged = update;
  return () => (localeChanged = undefined);
});

export function getLocale(): Locale {
  subscribe();
  return currentLocale;
}

/** Switch the interface language. Everything reading the locale reactively re-renders. */
export function setLocale(locale: Locale): void {
  if (locale === currentLocale) return;
  currentLocale = locale;
  // Native controls and assistive tech follow the document language.
  if (typeof document !== "undefined") document.documentElement.lang = locale;
  localeChanged?.();
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
  return interpolate(catalogFor(getLocale())[key] ?? EN[key] ?? key, params);
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
  const locale = getLocale();
  const catalog = catalogFor(locale);
  const text =
    catalog[`${key}.${pluralCategory(locale, count)}`] ??
    catalog[`${key}.other`] ??
    EN[`${key}.${pluralCategory(FALLBACK_LOCALE, count)}`] ??
    EN[`${key}.other`] ??
    key;
  return interpolate(text, { count, ...params });
}
