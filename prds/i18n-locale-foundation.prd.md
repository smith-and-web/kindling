---
contract_version: 1
component: i18n-locale-foundation
version: v1
primary_target_repo: smith-and-web/kindling
layers:
  rust: auto
  frontend: auto
untouchables:
  - "No AI, cloud, or subscription features or dependencies in the product"
  - "English catalog wording (src/lib/i18n/locales/en/*.ts values stay byte-identical; only plural key suffixes are renamed)"
  - "src-tauri/src/db/schema.rs (SQLite schema — changes require explicit human review)"
  - "Cargo.lock and package-lock.json (no unsupervised dependency changes)"
  - "src/styles/press/**, static/fonts/**, static/brand/** and DESIGN_GUIDE.md (generated Press mirrors)"
  - "Standard Manuscript Format rules in the DOCX exporter (manuscript text is the writer's language, not the UI's)"
work_units:
  - id: WU-01
    title: "Replace the zh-only locale switch with a locale registry and a dev override"
    layers: [frontend]
    target_modules:
      - "src/lib/i18n/index.ts"
      - "src/lib/i18n/types.ts"
      - "src/lib/i18n/index.test.ts"
    test_contract: "vitest: catalogs for every directory under locales/ are discovered by one glob and keyed by directory name; detection maps zh, zh-TW and zh-Hant to zh-CN, en-GB and unknown tags (fr, uk) to en; a VITE_KINDLING_LOCALE value overrides detection outside tests; the parity tests iterate every discovered locale instead of naming zh-CN, so adding locales/<tag>/ needs no test edit. All existing i18n tests pass unchanged in intent."
    depends_on: []
  - id: WU-02
    title: "Add CLDR plural selection with tn() and a plural-coverage parity test"
    layers: [frontend]
    target_modules:
      - "src/lib/i18n/index.ts"
      - "src/lib/i18n/index.test.ts"
      - "src/lib/utils/plural.ts"
    test_contract: "vitest: tn(key, count, params) resolves `${key}.${new Intl.PluralRules(locale).select(count)}`, falling back to `${key}.other` in the same locale, then English, then the key; count is injected as {count}. A parity test fails when any locale is missing a plural category its Intl.PluralRules returns for integers 0-200 (en: one/other; zh-CN: other; a stub ru fixture: one/few/many). countLabel(3, 'scene') still renders '3 scenes' in English and '3 个场景' in zh-CN."
    depends_on: [WU-01]
  - id: WU-03
    title: "Move count call sites and the noun inventory onto tn()"
    layers: [frontend]
    target_modules:
      - "src/lib/i18n/locales/en/sidebar.ts"
      - "src/lib/i18n/locales/en/editorial.ts"
      - "src/lib/i18n/locales/en/editor.ts"
      - "src/lib/i18n/locales/zh-CN/sidebar.ts"
      - "src/lib/i18n/locales/zh-CN/editorial.ts"
      - "src/lib/i18n/locales/zh-CN/editor.ts"
      - "src/lib/components/EditorialWorkspace.svelte"
      - "src/lib/components/RevisionsPanel.svelte"
      - "src/lib/components/WritingProgress.svelte"
      - "src/lib/components/TemplateBrowser.svelte"
      - "src/lib/components/PartDeleteDialog.svelte"
      - "src/lib/utils/plural.ts"
    test_contract: "vitest: the 11 `.one`/`.many` pairs become `.one`/`.other` and every call site uses tn() instead of `count === 1`; the ZH_NOUNS map and the `{label}s` concatenation in PartDeleteDialog are gone, replaced by per-noun plural keys in the catalogs; rendered English and Chinese output for 0, 1, 2 and 5 is identical to before (snapshot the strings, not the DOM). Coverage thresholds hold."
    depends_on: [WU-02]
  - id: WU-04
    title: "Give native menus a per-locale string table"
    layers: [rust]
    target_modules:
      - "src-tauri/src/menu.rs"
    test_contract: "cargo test: menu labels come from a MenuStrings struct with one const per locale (EN, ZH_CN), so a missing label is a compile error; menu_locale() maps zh* to zh-CN and everything else to en, mirroring the TypeScript detection table; a KINDLING_LOCALE env var overrides it; a test asserts every field of every const is non-empty and that EN matches the current English labels exactly."
    depends_on: []
  - id: WU-05
    title: "Document how to add a language, with a shared terminology glossary"
    layers: [frontend]
    target_modules:
      - "docs/ARCHITECTURE.md"
      - "docs/i18n-glossary.md"
    test_contract: "npm run format:check passes; ARCHITECTURE.md's Internationalization section lists the add-a-language checklist (catalog directory, registry entry, MenuStrings const, plural categories, glossary row, QA override) and no longer says only zh-CN exists; i18n-glossary.md has a row per core writing term (beat, scene, chapter, part, outline, synopsis, manuscript, reference, snapshot, revision, editorial review, feedback) with en and zh-CN columns filled from the shipped catalogs."
    depends_on: [WU-01, WU-02, WU-03, WU-04]
---

# i18n Locale Foundation — Product Requirements Document

## 1. Purpose

kindling shipped its first translation (zh-CN, #420) with locale handling hard-coded as
"Chinese or English". Spanish (#424) is waiting to merge, and Russian and Brazilian
Portuguese are next. Adding them to the current code would mean another `else if` in four
places, and Russian cannot be translated correctly at all: plurals are chosen by
`count === 1`, and Russian needs one/few/many forms.

This PRD makes adding a language a data-only change: a catalog directory, one registry
line and one Rust const. It lands before Spanish, Russian and Portuguese.

Success: after this lands, a new locale needs no edits to `index.ts` logic, the parity
tests or `create_menu`, and its plural forms are checked by a test.

## 2. Scope

**In bounds (v1):**
- A locale registry that discovers catalogs by directory and maps system language tags to
  supported locales, with a dev/QA override (`VITE_KINDLING_LOCALE`, `KINDLING_LOCALE`) so
  visual QA can screenshot every locale on one machine.
- `tn()` plural selection with `Intl.PluralRules`, a parity test for plural categories,
  and the existing 11 one/many pairs, `countLabel` nouns and `{label}s` concatenations
  moved onto it.
- A compile-time-complete per-locale string table for the native menus.
- An "adding a language" checklist and a cross-locale glossary of core terms.

**Out of bounds:**
- An in-app language picker. The UI still follows the system language. The module-level
  copy note in `index.ts` stays true.
- Any new translation. es, ru and pt-BR have their own plans.
- Number and date formatting (already follows the system through `toLocaleString`).
- Manuscript and export text such as the DOCX title-page word count.

## 3. Untouchables

English wording stays byte-identical: the unit tests run in English and the English
catalog is the source of truth. Do not touch the Press mirrors, the SQLite schema, the
Standard Manuscript Format rules or the lockfiles. Add no dependency: `Intl.PluralRules`
is built into the webview. Add no network or telemetry.

## 4. Acceptance criteria

1. **AC-1** — Adding `locales/xx/` plus one registry entry is enough for the frontend to
   detect, load and parity-test a new locale.
2. **AC-2** — `tn()` picks the CLDR category for the active locale, and a missing category
   in any locale fails the parity test.
3. **AC-3** — No component chooses a plural with `count === 1`; `ZH_NOUNS` and `{label}s`
   concatenation are gone. English and Chinese output is unchanged for 0, 1, 2 and 5.
4. **AC-4** — Native menu labels come from one const per locale; a missing label does not
   compile. `KINDLING_LOCALE` overrides the system language.
5. **AC-5** — `npm run check:all`, `npm test -- --coverage` and `cargo test --all-features`
   pass.

## Open questions

- **macOS predefined Edit items** (Undo, Cut, Copy, Paste, Select All) are drawn by AppKit.
  Without `CFBundleLocalizations` in the bundle they probably stay English for every
  locale, including zh-CN today. Verify on a non-English macOS before adding the key.
- **Detection tables in two languages.** TypeScript and Rust each map tags to locales. A
  shared fixture both test suites read would keep them in step; decide during WU-04.
