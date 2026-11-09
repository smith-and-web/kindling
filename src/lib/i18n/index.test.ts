import { afterEach, describe, expect, it } from "vitest";
import {
  LANGUAGE_NAMES,
  LANGUAGE_STORAGE_KEY,
  LOCALES,
  getLocale,
  initialLocale,
  isLocale,
  localeFor,
  storedLanguagePreference,
  mergeCatalogs,
  pluralCategory,
  resolveLocale,
  setLocale,
  t,
  tn,
} from "./index";
import type { Catalog, Locale } from "./types";
import { countLabel, type CountNoun } from "../utils/plural";
import detection from "./detection.json";
import pluralOutput from "./plural-output.fixture.json";

type CatalogModule = { default: Catalog };
const modules = import.meta.glob<CatalogModule>("./locales/*/*.ts", { eager: true });

/** `{ locale: { group: catalog } }`, discovered the same way the runtime discovers them. */
const discovered: Record<string, Record<string, Catalog>> = {};
for (const [path, module] of Object.entries(modules)) {
  const [locale, file] = path.split("/").slice(-2);
  (discovered[locale] ??= {})[file.replace(/\.ts$/, "")] = module.default;
}
const locales = Object.keys(discovered).sort();
const others = locales.filter((locale) => locale !== "en");

const placeholders = (text: string) => [...text.matchAll(/\{([a-zA-Z0-9_]+)\}/g)].map((m) => m[1]);

const CATEGORIES = ["zero", "one", "two", "few", "many", "other"];
const pluralBase = (key: string) => {
  const dot = key.lastIndexOf(".");
  return dot > 0 && CATEGORIES.includes(key.slice(dot + 1)) ? key.slice(0, dot) : null;
};

/** The plural categories a locale's rules return for whole numbers 0-200. */
const integerCategories = (tag: string) => {
  const rules = new Intl.PluralRules(tag);
  return [...new Set(Array.from({ length: 201 }, (_, n) => rules.select(n)))].sort();
};

/**
 * Plural keys a catalog gets wrong against English: a base key with a missing category
 * the locale needs, or a category it never selects.
 */
function pluralProblems(english: Catalog, catalog: Catalog, tag: string): string[] {
  const needed: string[] = integerCategories(tag);
  const bases = new Set(Object.keys(english).map(pluralBase).filter(Boolean) as string[]);
  const problems: string[] = [];
  for (const base of bases) {
    const present = CATEGORIES.filter((category) => `${base}.${category}` in catalog);
    for (const category of needed)
      if (!present.includes(category)) problems.push(`${base}.${category} missing`);
    for (const category of present)
      if (!needed.includes(category) && category !== "other")
        problems.push(`${base}.${category} unused`);
  }
  return problems;
}

afterEach(() => setLocale("en"));

describe("t()", () => {
  it("looks keys up by catalog namespace", () => {
    expect(t("common.cancel")).toBe("Cancel");
    expect(t("find.findReplaceDialog.title")).toBe("Find and Replace");
  });

  it("interpolates parameters", () => {
    expect(t("common.closeNamed", { name: "Kindling" })).toBe("Close Kindling");
    expect(t("find.findReplaceDialog.position", { current: 2, total: 9 })).toBe("2 of 9");
  });

  it("returns the key when neither catalog has it", () => {
    expect(t("missing.key")).toBe("missing.key");
    expect(t("missing.key", { count: 3 })).toBe("missing.key");
  });

  it("uses the Chinese catalog after a locale change", () => {
    setLocale("zh-CN");
    expect(t("common.cancel")).toBe("取消");
    expect(t("common.closeNamed", { name: "Kindling" })).toBe("关闭Kindling");
  });
});

describe("locale accessors", () => {
  it("reports the active locale", () => {
    expect(getLocale()).toBe("en");
    setLocale("zh-CN");
    expect(getLocale()).toBe("zh-CN");
    setLocale("en");
    expect(getLocale()).toBe("en");
  });
});

describe("locale registry", () => {
  it("has one entry per catalog directory", () => {
    expect(Object.keys(LOCALES).sort()).toEqual(locales);
  });

  it("keys merged catalogs by directory and prefixes keys with the file namespace", () => {
    const merged = mergeCatalogs({
      "./locales/xx/common.ts": { default: { ok: "OK" } },
      "./locales/xx/sidebar.ts": { default: { "scene.one": "{count} scene" } },
      "./locales/yy/common.ts": { default: { ok: "Bon" } },
    });
    expect(merged).toEqual({
      xx: { "common.ok": "OK", "sidebar.scene.one": "{count} scene" },
      yy: { "common.ok": "Bon" },
    });
  });

  it("maps system language tags to supported locales", () => {
    // detection.json is shared with the Rust menu tests so both tables agree.
    for (const [tag, locale] of Object.entries(detection))
      expect(resolveLocale(tag), tag || "(empty)").toBe(locale);
    expect(resolveLocale(undefined)).toBe("en");
    expect(resolveLocale(null)).toBe("en");
  });

  it("prefers an exact locale over a language match", () => {
    expect(resolveLocale("ZH-cn")).toBe("zh-CN");
  });

  /** The `import.meta.env` fields `initialLocale()` reads. */
  const env = (vars: Record<string, string | boolean>) => vars as unknown as ImportMetaEnv;
  const dev = { MODE: "development", DEV: true };

  it("reads English in tests whatever the system or override says", () => {
    expect(
      initialLocale(env({ MODE: "test", DEV: true, VITE_KINDLING_LOCALE: "zh-CN" }), "zh")
    ).toBe("en");
    expect(initialLocale(env({ ...dev, VITEST: "true" }), "zh")).toBe("en");
  });

  it("follows the system language outside tests", () => {
    expect(initialLocale(env(dev), "zh-TW")).toBe("zh-CN");
    expect(initialLocale(env(dev), "fr-FR")).toBe("fr");
    expect(initialLocale(env(dev), undefined)).toBe("en");
  });

  it("lets VITE_KINDLING_LOCALE override the system language on the dev server", () => {
    expect(initialLocale(env({ ...dev, VITE_KINDLING_LOCALE: "zh-CN" }), "en-US")).toBe("zh-CN");
    expect(initialLocale(env({ ...dev, VITE_KINDLING_LOCALE: "en" }), "zh")).toBe("en");
    expect(initialLocale(env({ ...dev, VITE_KINDLING_LOCALE: "" }), "zh")).toBe("zh-CN");
  });

  it("ignores the override in production builds", () => {
    const build = { MODE: "production", DEV: false, VITE_KINDLING_LOCALE: "zh-CN" };
    expect(initialLocale(env(build), "en-US")).toBe("en");
  });

  it("starts in the language chosen in Settings, over the override and system", () => {
    expect(initialLocale(env(dev), "en-US", "ru")).toBe("ru");
    expect(initialLocale(env({ ...dev, VITE_KINDLING_LOCALE: "zh-CN" }), "en-US", "fr")).toBe("fr");
    expect(initialLocale(env(dev), "de-DE", "system")).toBe("de");
    // Tests still start in English.
    expect(initialLocale(env({ MODE: "test" }), "en-US", "ru")).toBe("en");
  });

  it("resolves a choice even under test, for in-app switching", () => {
    expect(localeFor("pt-BR", env({ MODE: "test" }), "en-US")).toBe("pt-BR");
    expect(localeFor("system", env(dev), "es-MX")).toBe("es");
    expect(localeFor("system", env({ ...dev, VITE_KINDLING_LOCALE: "de" }), "es-MX")).toBe("de");
  });
});

describe("language preference", () => {
  const storage = (value: string | null) => ({ getItem: () => value });

  it("reads a stored locale and treats anything else as the system language", () => {
    expect(storedLanguagePreference(storage("zh-CN"))).toBe("zh-CN");
    expect(storedLanguagePreference(storage("zh"))).toBe("system");
    expect(storedLanguagePreference(storage("toString"))).toBe("system");
    expect(storedLanguagePreference(storage(null))).toBe("system");
    expect(storedLanguagePreference(undefined)).toBe("system");
    const broken = {
      getItem: () => {
        throw new Error("denied");
      },
    };
    expect(storedLanguagePreference(broken)).toBe("system");
    expect(LANGUAGE_STORAGE_KEY).toBe("kindling:language");
  });

  it("names every locale in its own language", () => {
    expect(Object.keys(LANGUAGE_NAMES).sort()).toEqual(Object.keys(LOCALES).sort());
    expect(LANGUAGE_NAMES).toMatchObject({
      en: "English",
      "zh-CN": "简体中文",
      es: "Español",
      "pt-BR": "Português (Brasil)",
      ru: "Русский",
      de: "Deutsch",
      fr: "Français",
    });
    expect(isLocale("fr")).toBe(true);
    expect(isLocale("system")).toBe(false);
    expect(isLocale(3)).toBe(false);
  });
});

describe("tn()", () => {
  it("picks the English category and injects the count", () => {
    expect(tn("sidebar.sidebar.sceneCount", 1)).toBe("1 scene");
    expect(tn("sidebar.sidebar.sceneCount", 0)).toBe("0 scenes");
    expect(tn("sidebar.sidebar.sceneCount", 2)).toBe("2 scenes");
  });

  it("passes extra parameters through", () => {
    expect(tn("editorial.revisionsPanel.prunedRemoved", 2, { names: "“A” and “B”" })).toBe(
      "“A” and “B” were removed because only the newest automatic drafts are kept."
    );
  });

  it("uses the active locale's categories", () => {
    setLocale("zh-CN");
    expect(tn("sidebar.sidebar.sceneCount", 1)).toBe("1 个场景");
    expect(tn("sidebar.sidebar.sceneCount", 5)).toBe("5 个场景");
  });

  it("falls back to .other, then English, then the key", () => {
    // zh-CN has only `.other`; a key it lacks entirely falls back to English.
    setLocale("zh-CN");
    expect(tn("counts.scene", 1)).toBe("1 个场景");
    setLocale("missing" as Locale);
    expect(tn("counts.scene", 1)).toBe("1 scene");
    expect(tn("counts.scene", 3)).toBe("3 scenes");
    expect(tn("missing.key", 3)).toBe("missing.key");
  });

  it("reports CLDR categories per locale", () => {
    expect(pluralCategory("en", 1)).toBe("one");
    expect(pluralCategory("en", 0)).toBe("other");
    expect(pluralCategory("zh-CN", 1)).toBe("other");
  });
});

describe("word counts", () => {
  // Every word count is a plural key, so 1 is singular in English and 0 is singular in French.
  it.each([
    ["en", 1, "Scene: 1 word"],
    ["en", 0, "Scene: 0 words"],
    ["fr", 0, "Scène\u00a0: 0 mot"],
    ["fr", 2, "Scène\u00a0: 2 mots"],
    ["de", 1, "Szene: 1 Wort"],
    ["ru", 1, "Сцена: 1 слово"],
    ["ru", 3, "Сцена: 3 слова"],
    ["ru", 5, "Сцена: 5 слов"],
    ["zh-CN", 1, "场景：1 字"],
  ] as [Locale, number, string][])("renders %s %i", (locale, count, text) => {
    setLocale(locale);
    expect(tn("editor.writingStatusBar.sceneWords", count)).toBe(text);
  });

  it("counts copied references and skipped duplicates separately", () => {
    setLocale("fr");
    expect(tn("references.copyReferencesDialog.previewSkipped", 0)).toBe(
      "0 doublon possible ignoré"
    );
    expect(
      tn("references.copyReferencesDialog.copiedMessage", 2, { source: "A", destination: "B" })
    ).toBe("2 fiches copiées de A vers B.");
    setLocale("ru");
    expect(tn("references.copyReferencesDialog.copiedSkipped", 2)).toBe(
      "Пропущено 2 возможных дубликата."
    );
  });
});

describe("counted phrases", () => {
  // A frame that carries a noun must go through tn(): a fixed t() form reads wrong as soon as
  // the count is 1 in English, and in Russian also for 2-4 and for 5 and up.
  const cases: [Locale, number, string, string][] = [
    ["en", 1, "sidebar.sidebar.wordsCount", "1 word"],
    ["en", 3, "sidebar.sidebar.wordsCount", "3 words"],
    ["ru", 1, "sidebar.sidebar.wordsCount", "1 слово"],
    ["ru", 3, "sidebar.sidebar.wordsCount", "3 слова"],
    ["ru", 5, "sidebar.sidebar.wordsCount", "5 слов"],
    ["ru", 21, "sidebar.sidebar.wordsCount", "21 слово"],
    ["ru", 2, "sidebar.tagManager.childTags", "2 дочерних тега"],
    ["ru", 3, "sidebar.templateBrowser.beatsCount", "3 бита"],
    ["ru", 1, "references.referencesPanel.expandWithCount", "Развернуть панель ссылок, 1 ссылка"],
    [
      "ru",
      1,
      "editorial.editorialManuscript.marginMarker",
      "1 комментарий или изменение в этом абзаце",
    ],
    ["ru", 3, "export.exportWorkspace.settingsFound", "3 настройки найдены"],
    ["ru", 5, "export.exportWorkspace.settingsFound", "5 настроек найдено"],
  ];

  it.each(cases)("%s agrees %s with %i", (locale, count, key, text) => {
    setLocale(locale);
    expect(tn(key, count)).toBe(text);
  });
});

describe("countLabel()", () => {
  it("renders English and Chinese counted nouns", () => {
    expect(countLabel(3, "scene")).toBe("3 scenes");
    expect(countLabel(1, "match")).toBe("1 match");
    setLocale("zh-CN");
    expect(countLabel(3, "scene")).toBe("3 个场景");
    expect(countLabel(1, "match")).toBe("1 处匹配");
  });
});

describe("plural output", () => {
  // plural-output.fixture.json was rendered from the catalogs and `count === 1` logic before
  // tn() existed. PartDeleteDialog rows are checked in PartDeleteDialog.test.ts.
  const rendered = (id: string, count: number): string | null => {
    if (id.startsWith("partDelete.")) return null;
    if (id.startsWith("countLabel.")) return countLabel(count, id.split(".")[1] as CountNoun);
    return tn(id, count, { names: "“A”" });
  };

  for (const [locale, byCount] of Object.entries(pluralOutput)) {
    it(`matches the pre-tn() strings in ${locale}`, () => {
      setLocale(locale as Locale);
      for (const [count, rows] of Object.entries(byCount)) {
        for (const [id, expected] of Object.entries(rows)) {
          const actual = rendered(id, Number(count));
          if (actual !== null) expect(actual, `${id} (${count})`).toBe(expected);
        }
      }
    });
  }
});

describe("catalogs", () => {
  const en = discovered.en;

  it.each(others)("%s ships the same groups as English", (locale) => {
    expect(Object.keys(discovered[locale]).sort()).toEqual(Object.keys(en).sort());
  });

  it.each(others)("%s translates every non-plural key of every group", (locale) => {
    const singular = (catalog: Catalog) =>
      Object.keys(catalog)
        .filter((key) => pluralBase(key) === null)
        .sort();
    for (const group of Object.keys(en)) {
      expect(singular(discovered[locale][group]), group).toEqual(singular(en[group]));
    }
  });

  it.each(locales)("%s has every plural category its rules select", (locale) => {
    for (const group of Object.keys(en)) {
      expect(pluralProblems(en[group], discovered[locale][group], locale), group).toEqual([]);
    }
  });

  // tn() falls back to `.other` before English, so a locale without it shows English copy for
  // the counts its rules put outside one/few/many (fractions, and 0 in some languages).
  it.each(locales)("%s defines .other for every plural base", (locale) => {
    for (const group of Object.keys(en)) {
      for (const key of Object.keys(en[group])) {
        const base = pluralBase(key);
        if (base)
          expect(`${base}.other` in discovered[locale][group], `${group}: ${base}`).toBe(true);
      }
    }
  });

  it.each(others)("%s keeps placeholders aligned with English", (locale) => {
    for (const group of Object.keys(en)) {
      for (const [key, text] of Object.entries(discovered[locale][group])) {
        const base = pluralBase(key);
        const english = en[group][base ? `${base}.other` : key];
        // tn() always supplies {count}, so a plural form may drop or add it.
        const names = (s: string) =>
          placeholders(s)
            .filter((name) => !base || name !== "count")
            .sort();
        expect(names(text), `${group}.${key}`).toEqual(names(english));
      }
    }
  });

  it("only uses plural categories on keys English pluralizes", () => {
    for (const locale of others) {
      for (const group of Object.keys(en)) {
        for (const key of Object.keys(discovered[locale][group])) {
          const base = pluralBase(key);
          if (base) expect(`${base}.other` in en[group], `${locale} ${group}.${key}`).toBe(true);
        }
      }
    }
  });

  describe("plural coverage check", () => {
    // A stub Russian catalog: integers need one (1, 21), few (2-4, 22) and many (0, 5-20).
    const english = { "scene.one": "{count} scene", "scene.other": "{count} scenes" };

    it("passes a catalog with every category", () => {
      const ru = {
        "scene.one": "{count} сцена",
        "scene.few": "{count} сцены",
        "scene.many": "{count} сцен",
      };
      expect(integerCategories("ru")).toEqual(["few", "many", "one"]);
      expect(pluralProblems(english, ru, "ru")).toEqual([]);
    });

    it("fails a catalog missing a category", () => {
      const ru = { "scene.one": "{count} сцена", "scene.other": "{count} сцены" };
      expect(pluralProblems(english, ru, "ru")).toEqual([
        "scene.few missing",
        "scene.many missing",
      ]);
    });

    it("fails a catalog with a category its rules never select", () => {
      const zh = { "scene.one": "{count} 个场景", "scene.other": "{count} 个场景" };
      expect(pluralProblems(english, zh, "zh-CN")).toEqual(["scene.one unused"]);
    });
  });
});
