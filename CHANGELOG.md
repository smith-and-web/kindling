# Changelog

All notable changes to this project will be documented in this file.

This changelog is automatically generated from [Conventional Commits](https://www.conventionalcommits.org/).

## [0.2.0-alpha](https://github.com/smith-and-web/kindling/compare/v0.1.0-alpha...v0.2.0-alpha) (2026-01-18)

### Features

* **sdlc:** add changelog generation with conventional-changelog ([941f7ca](https://github.com/smith-and-web/kindling/commit/941f7ca))
* **sdlc:** add CODEOWNERS for automatic reviewer assignment ([3426bed](https://github.com/smith-and-web/kindling/commit/3426bed))
* **sdlc:** enhance Dependabot config with grouping and labels ([3426bed](https://github.com/smith-and-web/kindling/commit/3426bed))
* **sdlc:** add git hooks setup to development setup script ([be3307a](https://github.com/smith-and-web/kindling/commit/be3307a))

### Performance

* **ci:** optimize workflows with sccache and improved caching ([7a4bfcf](https://github.com/smith-and-web/kindling/commit/7a4bfcf))
* **ci:** add cargo registry caching across all workflows
* **ci:** cache tauri-driver and cargo-audit binaries

### Bug Fixes

* **ci:** disable incremental compilation for sccache compatibility ([3464a57](https://github.com/smith-and-web/kindling/commit/3464a57))
* **ci:** add checks write permission for JUnit report annotations ([fdfa5c1](https://github.com/smith-and-web/kindling/commit/fdfa5c1))
* **ci:** add shell bash for Windows and move permissions to workflow level ([d87fa35](https://github.com/smith-and-web/kindling/commit/d87fa35))
* **e2e:** improve reimport test reliability with safer sync button clicks ([063a0da](https://github.com/smith-and-web/kindling/commit/063a0da))

### Chores

* **deps:** bump the actions group with 6 updates ([#80](https://github.com/smith-and-web/kindling/issues/80))
* **deps:** bump the e2e-dependencies group with 5 updates ([#79](https://github.com/smith-and-web/kindling/issues/79))
* regenerate package-lock.json for npm ci compatibility ([e3e02da](https://github.com/smith-and-web/kindling/commit/e3e02da))

---

## [0.1.0-alpha](https://github.com/smith-and-web/kindling/compare/v0.0.1-alpha...v0.1.0-alpha) (2026-01-15)

### Features

* **e2e:** add WebdriverIO e2e testing infrastructure ([290de9a](https://github.com/smith-and-web/kindling/commit/290de9a)), closes [#38](https://github.com/smith-and-web/kindling/issues/38)
* **e2e:** add data-testid attributes for e2e testing ([29b9e7b](https://github.com/smith-and-web/kindling/commit/29b9e7b))
* **e2e:** improve E2E testing setup and developer experience ([3647430](https://github.com/smith-and-web/kindling/commit/3647430))
* **onboarding:** add first-run onboarding flow with Lucide icons ([e392f61](https://github.com/smith-and-web/kindling/commit/e392f61)), closes [#18](https://github.com/smith-and-web/kindling/issues/18)
* **references:** multi-select accordions, sorting, and drag-drop ([65c50dd](https://github.com/smith-and-web/kindling/commit/65c50dd))
* **ui:** add expandable chapter/scene tree view ([7a2df85](https://github.com/smith-and-web/kindling/commit/7a2df85)), closes [#10](https://github.com/smith-and-web/kindling/issues/10)
* **ui:** add read-only scene content panel ([a3629ea](https://github.com/smith-and-web/kindling/commit/a3629ea)), closes [#11](https://github.com/smith-and-web/kindling/issues/11)
* **ui:** add resizable References panel ([33731a4](https://github.com/smith-and-web/kindling/commit/33731a4)), closes [#36](https://github.com/smith-and-web/kindling/issues/36)
* **ui:** add scene display and references panels ([3aa513b](https://github.com/smith-and-web/kindling/commit/3aa513b)), closes [#11](https://github.com/smith-and-web/kindling/issues/11) [#12](https://github.com/smith-and-web/kindling/issues/12) [#13](https://github.com/smith-and-web/kindling/issues/13)
* **ui:** add v0.2.0 UI components for content management ([cb1c0c9](https://github.com/smith-and-web/kindling/commit/cb1c0c9))
* **ui:** apply brand guidelines to app ([2e8309f](https://github.com/smith-and-web/kindling/commit/2e8309f))
* **ui:** dynamic max width for References panel ([97c5a9f](https://github.com/smith-and-web/kindling/commit/97c5a9f))
* add comprehensive tests and fixture for Scrivener parser ([749e9d2](https://github.com/smith-and-web/kindling/commit/749e9d2)), closes [#20](https://github.com/smith-and-web/kindling/issues/20)
* add context menu with rename, duplicate, archive, and lock ([8f81d4c](https://github.com/smith-and-web/kindling/commit/8f81d4c))
* add release workflow and installation documentation ([426ab58](https://github.com/smith-and-web/kindling/commit/426ab58)), closes [#41](https://github.com/smith-and-web/kindling/issues/41)
* add sdlc improvements including coverage gating, security scanning, and commit linting ([781e909](https://github.com/smith-and-web/kindling/commit/781e909))
* add v0.2.0 backend commands for content management ([b7c5131](https://github.com/smith-and-web/kindling/commit/b7c5131))
* complete Plottr parser with real file format support ([4066272](https://github.com/smith-and-web/kindling/commit/4066272))
* create beats from imported content ([6ec3a1e](https://github.com/smith-and-web/kindling/commit/6ec3a1e))
* implement granular sync preview and selective change approval ([2069ae9](https://github.com/smith-and-web/kindling/commit/2069ae9))
* improve markdown parser with comprehensive tests and fixtures ([47bbd45](https://github.com/smith-and-web/kindling/commit/47bbd45)), closes [#21](https://github.com/smith-and-web/kindling/issues/21)
* improve save indicator and add sync confirmation dialog ([f4dc6ff](https://github.com/smith-and-web/kindling/commit/f4dc6ff))

### Bug Fixes

* **e2e:** achieve 100% E2E test pass rate (47/47) ([d625a58](https://github.com/smith-and-web/kindling/commit/d625a58))
* **e2e:** add package-lock.json and remove invalid tauri-driver dep ([5969799](https://github.com/smith-and-web/kindling/commit/5969799))
* **e2e:** align data-testid attributes with E2E test expectations ([fe07eb3](https://github.com/smith-and-web/kindling/commit/fe07eb3))
* **e2e:** fix app-launch tests to match actual app behavior ([50a2d0c](https://github.com/smith-and-web/kindling/commit/50a2d0c))
* **e2e:** fix WebDriver config and optimize CI build ([e543f9f](https://github.com/smith-and-web/kindling/commit/e543f9f))
* **e2e:** handle onboarding flow in e2e tests ([83a967d](https://github.com/smith-and-web/kindling/commit/83a967d))
* **e2e:** improve E2E test compatibility and Plottr parser ([5fc6fff](https://github.com/smith-and-web/kindling/commit/5fc6fff))
* **e2e:** improve E2E test reliability to 80% pass rate (36/45) ([5656ffc](https://github.com/smith-and-web/kindling/commit/5656ffc))
* **e2e:** improve test reliability and fix common issues ([0df3b09](https://github.com/smith-and-web/kindling/commit/0df3b09))
* **e2e:** match official Tauri WebdriverIO pattern ([ca26202](https://github.com/smith-and-web/kindling/commit/ca26202))
* **e2e:** remove invalid browserName from capabilities ([dd8c9e7](https://github.com/smith-and-web/kindling/commit/dd8c9e7))
* **references:** improve icons and fix drag-and-drop ([df015ee](https://github.com/smith-and-web/kindling/commit/df015ee))
* **references:** use pointer events for drag-and-drop ([a1fef44](https://github.com/smith-and-web/kindling/commit/a1fef44))
* **ui:** display description in character/location expanded view ([07a92d0](https://github.com/smith-and-web/kindling/commit/07a92d0)), closes [#35](https://github.com/smith-and-web/kindling/issues/35)
* **ui:** remove duplicate description from expanded view ([25aaf69](https://github.com/smith-and-web/kindling/commit/25aaf69))
* **ui:** sidebar collapse and project navigation ([4938e09](https://github.com/smith-and-web/kindling/commit/4938e09)), closes [#31](https://github.com/smith-and-web/kindling/issues/31) [#32](https://github.com/smith-and-web/kindling/issues/32)
* avoid duplicating single-sentence synopsis as beat in Scrivener ([673deb4](https://github.com/smith-and-web/kindling/commit/673deb4))
* cast usize to i64 for rusqlite 0.38 compatibility ([3dbc22a](https://github.com/smith-and-web/kindling/commit/3dbc22a))
* expose Tauri invoke for E2E testing via __KINDLING_TEST__ ([ffb7d27](https://github.com/smith-and-web/kindling/commit/ffb7d27))
* improve save indicator and sidebar width ([a5c2a53](https://github.com/smith-and-web/kindling/commit/a5c2a53))
* mute reimport prose preserved text ([f07f78a](https://github.com/smith-and-web/kindling/commit/f07f78a))
* prevent loading race condition in Sidebar ([99e1be5](https://github.com/smith-and-web/kindling/commit/99e1be5))
* refresh project list when returning to start screen ([9032217](https://github.com/smith-and-web/kindling/commit/9032217))
* resolve parser duplication and empty scene bugs ([ffe200c](https://github.com/smith-and-web/kindling/commit/ffe200c)), closes [#26](https://github.com/smith-and-web/kindling/issues/26) [#27](https://github.com/smith-and-web/kindling/issues/27) [#28](https://github.com/smith-and-web/kindling/issues/28)
* resolve three Sidebar bugs ([9bf8b43](https://github.com/smith-and-web/kindling/commit/9bf8b43))
* update @tauri-apps/plugin-dialog to v2.6.0 to match Rust crate version ([ece5f4f](https://github.com/smith-and-web/kindling/commit/ece5f4f))
* update scrivener parser for quick-xml 0.39 API changes ([55df1de](https://github.com/smith-and-web/kindling/commit/55df1de))
