import type { LOCALES } from "./index";

/** A flat catalog of UI strings: `key` -> text with optional `{param}` placeholders. */
export type Catalog = Record<string, string>;

/** A registry entry: the system language subtags (`zh`, `pt`) a locale serves. */
export interface LocaleEntry {
  languages: readonly string[];
}

/** A supported interface locale, named after its catalog directory under `locales/`. */
export type Locale = keyof typeof LOCALES;

/** The writer's interface-language choice: a specific locale, or follow the system. */
export type LanguagePreference = Locale | "system";
