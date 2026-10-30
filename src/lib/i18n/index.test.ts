import { afterEach, describe, expect, it } from "vitest";
import { getLocale, nounLabel, nounPluralLabel, setLocale, t } from "./index";
import { countLabel } from "../utils/plural";

type CatalogModule = { default: Record<string, string> };
const enModules = import.meta.glob<CatalogModule>("./locales/en/*.ts", { eager: true });
const zhModules = import.meta.glob<CatalogModule>("./locales/zh-CN/*.ts", { eager: true });

const groups = (modules: Record<string, CatalogModule>) =>
  Object.fromEntries(
    Object.entries(modules).map(([path, module]) => [
      path.split("/").pop()!.replace(/\.ts$/, ""),
      module.default,
    ])
  );

const placeholders = (text: string) => [...text.matchAll(/\{([a-zA-Z0-9_]+)\}/g)].map((m) => m[1]);

afterEach(() => setLocale("en"));

describe("t()", () => {
  it("looks keys up by catalog namespace", () => {
    expect(t("common.cancel")).toBe("Cancel");
    expect(t("find.findReplaceDialog.title")).toBe("Find and Replace");
  });

  it("interpolates parameters", () => {
    expect(t("common.closeNamed", { name: "Kindling" })).toBe("Close Kindling");
    expect(t("find.findReplaceDialog.scopeEntireProject")).toBe("the entire project");
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

describe("noun labels", () => {
  it("keeps English nouns and their plurals", () => {
    expect(nounLabel("scene")).toBe("scene");
    expect(nounPluralLabel("scene", "scenes")).toBe("scenes");
    expect(countLabel(1, "scene")).toBe("1 scene");
    expect(countLabel(3, "scene")).toBe("3 scenes");
  });

  it("adds the Chinese measure word for known nouns", () => {
    setLocale("zh-CN");
    expect(nounLabel("scene")).toBe("个场景");
    expect(nounPluralLabel("scene", "scenes")).toBe("个场景");
    expect(countLabel(3, "scene")).toBe("3 个场景");
    expect(countLabel(1, "match", "matches")).toBe("1 处匹配");
  });

  it("keeps unknown nouns untouched", () => {
    setLocale("zh-CN");
    expect(nounLabel("sausage")).toBe("sausage");
    expect(nounPluralLabel("sausage", "sausages")).toBe("sausages");
    expect(countLabel(2, "sausage", "sausages")).toBe("2 sausages");
  });
});

describe("catalogs", () => {
  const en = groups(enModules);
  const zh = groups(zhModules);

  it("ships the same groups in every locale", () => {
    expect(Object.keys(zh).sort()).toEqual(Object.keys(en).sort());
  });

  it("translates every key of every group", () => {
    for (const group of Object.keys(en)) {
      expect(Object.keys(zh[group]).sort(), group).toEqual(Object.keys(en[group]).sort());
    }
  });

  it("keeps placeholders aligned between locales", () => {
    for (const group of Object.keys(en)) {
      for (const [key, english] of Object.entries(en[group])) {
        expect(placeholders(zh[group][key]).sort(), `${group}.${key}`).toEqual(
          placeholders(english).sort()
        );
      }
    }
  });
});
