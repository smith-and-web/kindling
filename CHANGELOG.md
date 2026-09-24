# Changelog

All notable changes to this project will be documented in this file.

This changelog is automatically generated from [Conventional Commits](https://www.conventionalcommits.org/).

## [1.3.0](https://github.com/smith-and-web/kindling/compare/v1.2.0...v1.3.0) (2026-09-24)

### Features

* **ui:** a new look: kindling is reskinned on the Press design system, with light, dark and system themes and a manuscript that stays on paper in dark mode ([#343](https://github.com/smith-and-web/kindling/issues/343))
* **editorial:** offline editorial review: send an editor a `.kindling-review` package and read their `.kindling-feedback` back, with comments, threaded replies and tracked suggestions you accept or reject ([#288](https://github.com/smith-and-web/kindling/issues/288))
* **editor:** scene revisions: save and compare drafts of a scene and restore an earlier one ([#286](https://github.com/smith-and-web/kindling/issues/286))
* **editor:** find and replace across a scene or the whole project
* **editor:** writing goals, session tracking and statistics ([#284](https://github.com/smith-and-web/kindling/issues/284)), closes [#67](https://github.com/smith-and-web/kindling/issues/67) [#225](https://github.com/smith-and-web/kindling/issues/225)
* **editor:** kindling remembers where you were writing in each project, closes [#65](https://github.com/smith-and-web/kindling/issues/65)
* **editor:** see the end of the previous scene while you write ([#289](https://github.com/smith-and-web/kindling/issues/289))
* **export:** customizable export profiles and an export workspace with a live preview ([#339](https://github.com/smith-and-web/kindling/issues/339))
* **novelwriter:** import, export and prose sync with novelWriter projects
* **references:** copy references between projects ([#278](https://github.com/smith-and-web/kindling/issues/278))
* **settings:** preferences and project navigation in one Settings window ([#290](https://github.com/smith-and-web/kindling/issues/290))
* **shortcuts:** customizable keyboard shortcuts ([#292](https://github.com/smith-and-web/kindling/issues/292))
* **feedback:** **Help → Send Feedback…** sends a bug report, feature request or rating to the kindling team. It includes what you type plus the app version, operating system and language, never anything from your manuscript, and is sent only when you press **Send**. It is the only network request besides the update check.
* **db:** the first launch of a new version saves a copy of your library to `backups/` in the kindling data folder before upgrading it (the newest three are kept)
* **ui:** a loading screen from the moment the app starts ([#291](https://github.com/smith-and-web/kindling/issues/291))

### Bug Fixes

* **linux:** attempted fix for the AppImage opening to a blank white window on some GPU and compositor setups ([#252](https://github.com/smith-and-web/kindling/issues/252)), by disabling WebKitGTK's DMABUF renderer at startup. We could not test this on an affected system. An affected app can't show the update prompt, so download 1.3.0 from the releases page, and if it still opens blank, please report it on #252.
* **editor:** clicking the already-selected Beats or Page button no longer overwrites the scene with an older copy of its prose
* **editor:** undo after switching scenes in Page view no longer brings back the previous scene's text and saves it into the current one
* **editor:** a beat whose save failed reopens with your unsaved text instead of the last saved version
* **editor:** a chapter synopsis or beat title you typed is kept when saving it fails
* **editor:** keep editing focus and save drafts before exit ([#274](https://github.com/smith-and-web/kindling/issues/274)), closes [#272](https://github.com/smith-and-web/kindling/issues/272); keep restored writing positions ([#65](https://github.com/smith-and-web/kindling/issues/65))
* **editor:** export and sync save pending prose first, and refuse to run over a draft that could not be saved
* **db:** locked scenes are protected from chapter deletion, moves, reordering, beat renames and view switches
* protect writing and core workflows ([#341](https://github.com/smith-and-web/kindling/issues/341)), closes [#327](https://github.com/smith-and-web/kindling/issues/327) [#304](https://github.com/smith-and-web/kindling/issues/304) [#318](https://github.com/smith-and-web/kindling/issues/318) [#337](https://github.com/smith-and-web/kindling/issues/337) [#322](https://github.com/smith-and-web/kindling/issues/322) [#323](https://github.com/smith-and-web/kindling/issues/323) [#324](https://github.com/smith-and-web/kindling/issues/324) [#321](https://github.com/smith-and-web/kindling/issues/321) [#330](https://github.com/smith-and-web/kindling/issues/330) [#298](https://github.com/smith-and-web/kindling/issues/298)
* prevent high-priority data loss across sync, editing, imports and exports ([#340](https://github.com/smith-and-web/kindling/issues/340)), closes [#295](https://github.com/smith-and-web/kindling/issues/295) [#308](https://github.com/smith-and-web/kindling/issues/308) [#312](https://github.com/smith-and-web/kindling/issues/312) [#314](https://github.com/smith-and-web/kindling/issues/314) [#296](https://github.com/smith-and-web/kindling/issues/296) [#331](https://github.com/smith-and-web/kindling/issues/331) [#305](https://github.com/smith-and-web/kindling/issues/305) [#306](https://github.com/smith-and-web/kindling/issues/306) [#315](https://github.com/smith-and-web/kindling/issues/315) [#333](https://github.com/smith-and-web/kindling/issues/333)
* preserve writing focus, accessible labels, exports and snapshots ([#294](https://github.com/smith-and-web/kindling/issues/294)), closes [#280](https://github.com/smith-and-web/kindling/issues/280) [#281](https://github.com/smith-and-web/kindling/issues/281) [#283](https://github.com/smith-and-web/kindling/issues/283) [#285](https://github.com/smith-and-web/kindling/issues/285)
* resolve writing exit and reference management bugs ([#279](https://github.com/smith-and-web/kindling/issues/279))
* **sync:** scenes, beats and chapters added by sync take their own place instead of sharing a position with an existing item
* **scrivener:** Mac Scrivener imports keep their paragraphs, curly quotes, dashes and tabs, decode the project's code page, and no longer pick up image data or placeholders as prose
* **scrivener:** **Create new** refuses to replace an existing `.scriv` project instead of overwriting it
* **longform:** prose lines containing `::` are no longer deleted as Dataview fields
* **longform:** import only reads notes inside the vault
* **plottr:** cards with a title but no description are imported. Syncing a Plottr project imported with an earlier version offers those cards as new scenes, placed where Plottr has them.
* **import:** novelWriter appears on the home screen's import options
* **export:** Word and EPUB exports no longer break when prose contains control characters (such as a line break pasted from Word); the same goes for Scrivener and novelWriter project files
* **export:** a failed Word, EPUB or treatment export leaves the previous file intact
* **export:** "Delete existing export folder" only replaces a Markdown or Longform folder kindling created, and never removes files you added to it
* **export:** Notes, To-do and Unused scenes are left out of Word, EPUB and Markdown manuscripts, matching the export workspace
* **export:** smart quotes curl correctly across italics, `&mdash;` and `&hellip;` become real characters, and Word sets curly quotes and dashes in the manuscript font
* **export:** chapter and Part headings in Word manuscripts are set at body size and weight
* **export:** the export workspace's Agent submission Word manuscript follows Standard Manuscript Format: a running header with surname, short title and page number, a contact block on the title page, and no indent after a heading
* **export:** **Show in folder** works after a workspace export
* **feedback:** send errors are readable and a request gives up after 15 seconds instead of spinning
* **editorial:** emoji edits no longer make a review unsaveable
* **editorial:** separate suggestions in a long scene stay separate
* **editorial:** the review workspace loads on macOS versions before Safari 16.4
* **editorial:** Draft history can always be closed
* **editorial:** review packages can't navigate the app away through links
* **revisions:** automatic drafts are capped per scene; named drafts are always kept
* **snapshots:** replacing a project from a snapshot keeps its draft history
* **references:** descriptions imported from other apps are shown as plain formatted prose
* **ui:** every dialog takes keyboard focus, keeps Tab inside and returns focus when it closes, and Escape closes only the topmost one
* **ui:** menu shortcuts are ignored while a dialog or file picker is open, so Cmd+W during an import no longer closes the project
* **ui:** sidebar menus work from the keyboard, and chapters, scenes, beats and references can be reordered with Move up and Move down
* **ui:** actions that fail tell you why instead of silently doing nothing
* **ui:** a locked scene still shows whether it is in Beats or Page view; the prose editor has an accessible name; export previews and draft comparisons stay on manuscript paper in dark mode
* **start-screen:** integrate the review action and align panels ([#293](https://github.com/smith-and-web/kindling/issues/293))
* **updater:** installation continues when saving your writing position fails

### Known Issues

These are scheduled for 1.3.1 (see `docs/release-triage-v1.3.md`):

* Snapshots don't capture custom fields, tags or saved filters ([#297](https://github.com/smith-and-web/kindling/issues/297)). The new pre-upgrade library backup does.
* Scrivener import skips Text documents nested under other Text documents ([#320](https://github.com/smith-and-web/kindling/issues/320)). Flatten them in Scrivener before importing.
* RTF export writes emoji and other characters outside the Basic Multilingual Plane incorrectly ([#301](https://github.com/smith-and-web/kindling/issues/301)). Use another export format.
* Longform beats are not included in sync ([#317](https://github.com/smith-and-web/kindling/issues/317)), Longform reference links are not imported ([#299](https://github.com/smith-and-web/kindling/issues/299)), and some documented Longform grouping layouts are rejected ([#332](https://github.com/smith-and-web/kindling/issues/332)).
* yWriter items are not imported ([#316](https://github.com/smith-and-web/kindling/issues/316)); Markdown files saved with a BOM parse differently ([#319](https://github.com/smith-and-web/kindling/issues/319)); Scrivener titles can lose entities and spaces ([#313](https://github.com/smith-and-web/kindling/issues/313)).
* Migrated custom fields can reappear after renaming or deleting them ([#307](https://github.com/smith-and-web/kindling/issues/307)); duplicating a scene or chapter doesn't copy notes or reference links ([#302](https://github.com/smith-and-web/kindling/issues/302)); AND tag filters can match too much ([#311](https://github.com/smith-and-web/kindling/issues/311)).
* Word manuscripts have a right indent on body paragraphs ([#310](https://github.com/smith-and-web/kindling/issues/310)); the one-page text treatment omits summaries of chapters outside a Part ([#309](https://github.com/smith-and-web/kindling/issues/309)).
* A chapter restored from the archive shows at the end of the sidebar until the project is reopened ([#303](https://github.com/smith-and-web/kindling/issues/303)).
* Screenplays: the time control removes slugline qualifiers after a second dash ([#326](https://github.com/smith-and-web/kindling/issues/326)), page estimates can double count ([#325](https://github.com/smith-and-web/kindling/issues/325)), and a new project ignores the chosen target length ([#300](https://github.com/smith-and-web/kindling/issues/300)).
* Going back to 1.2 after opening a library in 1.3 works, but novelWriter projects then show as Markdown projects.

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
