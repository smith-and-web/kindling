# Unified settings acceptance

Branch: `feat/unified-settings`

Base: `e7962ec8dc40585acad5409523d8900e2c2e74a5` (updated `main`)

Verified: 2026-09-09

| Criterion | Implementation | Evidence |
| --- | --- | --- |
| One settings destination | One App-owned SettingsDialog; File → Settings and its shortcut/palette command; separate start/sidebar/reference gears and dialogs removed | App integration test, command-definition test, desktop inspection |
| Left navigation, right controls | Appearance & Guidance, Author & Contact, Project Details, Reference Types, Tags, Custom Fields | Inspected desktop screenshots in light/dark and at a 1066 CSS-pixel viewport |
| Settings for individual projects | All-project selector; keyed project draft; writes carry selected ID; open-project store updated only for matching ID | Component regression tests plus real SQLite IPC checks with two disposable projects |
| New branch from updated main | `git switch main`, `git pull --ff-only origin main`, `git switch -c feat/unified-settings` | Git reflog; independent reviewer verified |

## Behavior and regression checks

- Drafts survive area navigation; switching projects and closing prompt before discarding.
- Pending saves block project navigation and closing. Rejected saves retain inputs.
- Failed project/author loads expose retry; an unloaded author form cannot overwrite contact data.
- Goals remain untouched if statistics loading fails or is pending.
- Reference types, tags, and custom fields target the selected project.
- Disabling a reference type with an unsaved custom-field draft is blocked until that draft is saved or cancelled.
- Project settings preserve an existing screenplay page target under the backend's replacement semantics.
- Shared settings remain available with no projects.

## Automated validation

- `npm run check:all`: passed (design-system integrity, Svelte/TypeScript, formatting, ESLint, Rust formatting and Clippy across all targets/features). Existing Svelte accessibility/initial-value warnings and review-sidebar HTML lint warnings remain.
- `npm test -- --maxWorkers=2`: 42 files, 570 tests passed.
- `npm run test:coverage -- --maxWorkers=2`: passed; statements 98.56%, branches 94.11%, functions 98.61%, lines 99.13%. This preceded the added field-draft regression; the measured TypeScript implementation did not subsequently change.
- After the final screenplay-target preservation addition, the affected settings/goal suites passed all 12 tests.
- `npm run test:rust`: 458 tests passed.
- `npm run build`: passed; existing large-chunk advisory remains.
- Final Svelte check, lint, formatting and diff whitespace checks passed.

## Desktop verification

Used the running development desktop app with disposable projects in its demo database. Saved another project's genre (`QA Mystery`), word target (81,000), and daily goal (750), then read them back through real IPC. The original project's metadata and open-project ID remained unchanged. Saved reference types, a tag, and a location custom field were also read back from that second project.

Inspected screenshots of light Appearance, dark Tags, and light Project Details at 150% webview zoom (1066 CSS pixels). Navigation remains on the left; the controls scroll vertically. The dialog stayed within the viewport. Dark dialog/sidebar backgrounds were `rgb(38, 33, 27)` / `rgb(24, 20, 16)`; the light dialog was `rgb(251, 248, 241)`.

The screenshot tool returned a response-format error for two captures after saving the files; their saved images were opened and visually inspected. Captures are local artifacts in `/tmp/unified-settings-qa/`.

Restored light theme and 100% zoom, restored preferences, and deleted both owned fixture IDs. A subsequent IPC read confirmed only the four original projects remained. Failure injection was covered by automated component tests; no real filesystem failure was forced.

## Reviews

- Independent read-only reviewer: named `review-agent` skill was unavailable in the configured catalogs/skill locations, so a standard independent review was used explicitly. Found one field-draft-loss defect; fixed, regression-tested, and reassessed with **No findings**. Final screenplay-target addition also cleared.
- Claude `code-review:code-review`, CLI `--effort high`: **No actionable findings** in the full review and a focused final-state follow-up covering both fixes. Session: `bbdf5524-3ee7-4cad-b758-70f94491ac01`. Read-only; no tests/builds or external writes by reviewers.


## Navigation follow-up

User-requested refinement after `634211d`:

- Added a pinned Settings footer in the project sidebar that opens the shared window. It also works while the sidebar is used for local editorial review.
- Grouped settings navigation into Kindling → Preferences and Projects → Manuscript / Reference Library using labelled sections and nested lists.
- Moved the persistent Project selector under Projects in the settings sidebar. Choosing another project from a shared preferences area opens Project Details; existing draft/save guards still apply.
- Added a scope/project/subsection location label above the right-hand controls.

Validation: `check:all` and production build passed. All 572 frontend tests passed with coverage unchanged above thresholds. The 9 settings tests were rerun after correcting breadcrumb separator spacing. Inspected native screenshots of the pinned footer, light Project Details and dark Tags; confirmed the selector lives in navigation and selecting another project preserves the open-project ID. The footer's bottom matched the sidebar bottom at 920 CSS pixels, with a 53-pixel footer height. Screenshots are local artifacts under `/tmp/settings-navigation-qa/`. Removed the single owned fixture and restored preferences/light theme; confirmed the original four projects remained.

Final validation: the App and Settings suites passed all 60 tests after adding an integration test that opens the real local editorial workspace and checks Settings visibility from the footer, native menu event, and global shortcut. Final type, lint, format and production-build checks passed. The final breadcrumb was also checked in the desktop app as `Projects / The Letter / Manuscript`.

Follow-up reviews: the independent reviewer reported **No findings** on the navigation and final additions. Claude `code-review:code-review` at high effort identified the editorial test gap; it was addressed, then the final focused review reported **No actionable findings**. Same read-only review session: `bbdf5524-3ee7-4cad-b758-70f94491ac01`.
