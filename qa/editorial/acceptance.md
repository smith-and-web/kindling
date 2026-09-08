# Editorial workflow acceptance — #287

Base: `70cad8793772c3bc7dfd211745352744cccfe9f9`; branch `feat/editorial-packages`.

| Criterion                             | Implementation evidence                                                                                                                      | Validation                                                                                                                     | Remaining gap                                                                                                        |
| ------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Realistic revision demo               | `sample_revisions.rs`; status synchronization in `db/revisions.rs` and `db/queries.rs`; scene guide in `docs/editorial-review.md`            | Sample assertions cover all statuses, history, anchors, locks and both prose modes                                             | None; fresh native sample created and exercised                                                                      |
| Continuous manuscript                 | `EditorialManuscript.svelte`; block ownership, rich schema and source mapping in `editorial.ts`                                              | Cross-scene replacement, source suffix preservation, search, formatting, keyboard focus and 60k-word tracking regression tests | No native pixel capture available; rendered native DOM checked at both sizes/themes                                  |
| Suggestions and decisions             | Natural TipTap edits/history; comment mapping and merged discussions; contextual comparisons and explicit reanchor; atomic backend decisions | Engine, workspace, and Rust acceptance/conflict/lock/history tests                                                             | None; both final independent reviews report No findings                                                              |
| Portable files and recovery           | Versioned JSON packages; SQLite rounds, returns and sessions; CAS save queue and local recovery journal; reopenable recovery packages        | Round-trip, repeated/partial import, fresh-install recovery, generation and failed-save tests                                  | None; both final independent reviews report No findings                                                              |
| Writer/editor discussion              | Writer response export merges replies/decisions into the editor's ongoing review                                                             | Backend preservation test and workspace export interaction test                                                                | None; native editor → writer → editor replies/acceptance passed                                                      |
| Native file opening                   | Tauri file associations, macOS document identities/icons, queued initial/open events and single instance routing                             | Packaged macOS cold/running open on separate app profile succeeded; repeated open kept one session                             | Windows/Linux installer and upgrade runs unavailable on this host; macOS upgrade from an older release not exercised |
| Continued writing and multiple rounds | Immutable exported baseline; current prose CAS; affected-passage conflicts; scoped bulk preparation                                          | Disjoint writer edits, changed structure, independent reviewers, later rounds and repeat-import tests                          | None; both final independent reviews report No findings                                                              |

## Design decisions

- One ProseMirror manuscript supports ordinary selection and editing across scene,
  chapter and beat boundaries. Source identity stays internal to the data model.
- ChangeSet comparisons include formatting. Native editor history handles text
  undo/redo. Comments map through edit transactions; coalesced suggestions retain
  every existing discussion.
- Cross-source replacement text belongs to the first affected source. Consumed
  prose is removed from later sources while their untouched suffixes and outline
  records stay in place. No editor-side splitting is required.
- Outbound rounds retain their exact baseline. Responses import annotations
  before prose decisions. Acceptance checks current prose and locks atomically
  and preserves prior scene drafts.
- Scene locks protect the writer's prose. Suggestions in a portable review do
  not modify that prose and are permitted; acceptance names a locked scene and
  requires the writer to unlock it. The local locked scene Revisions panel is
  read-only. Attribution is not authorization.
- Scene status and revision status describe one workflow. Editor Review is a
  substage of Draft. Moving Revised/Final back to Draft starts First Draft;
  the Revisions panel can move it into Editor Review.
- A recovery copy is a review package containing unfinished work, with a newer
  portable generation and a separate local CAS generation on opening. It can
  resume on a fresh installation without the writer's project.
- Writer responses contain the editor's last returned document plus replies and
  decisions. Opening them retains newer local edits and merges discussion data.
- Reading viewport and cursor position save separately. Search preserves input
  focus, and writer decisions preserve manuscript context.

## Review record

The initial independent review-agent reported eight findings: search focus,
recoverability, writer reply transport, merged discussions, structural conflict
scope, writer decision viewport, scroll-only reading progress and Unicode search.
All received implementation fixes and regression coverage.

Claude `code-review:code-review` ran at `--effort high`, session
`79d448e9-b03e-4455-a44e-0117875f3cd6`. It confirmed search focus and a quit-save
short circuit; both were fixed. Locks and reverse status transitions were
intentional behaviors, clarified above and in the user guide for reassessment.
Its platform review was source inspection, not execution on those operating systems.

Subsequent reviews found comment movement during withdrawal, journal discussion
loss, repeated historical threads after refinement, stale writer response decisions,
and selection remapping after acceptance. Fixes include precise schema-compatible
inverse edits, separate editor/writer version ordering, message-id unions, stable
historical discussion identities, and mapped reading anchors. Follow-up results
are recorded below when complete.

## Desktop and visual evidence

Executed on macOS using a disposable sample, a separate application identifier,
and isolated SQLite data. No original writer project is required on the editor
profile.

- Packaged application: cold-open and running-open through Launch Services;
  repeated opening resumed one session. Built Info.plist contains both custom
  UTIs, document names, icons, and association ranks.
- Native application: review export, ordinary insertion and replacement, local
  save/reopen, feedback preview/import, writer reply, acceptance, reply-package
  export, and editor reopening with the received decision and discussion.
- A second round replaced “dusk” with “sunset” while the writer independently
  changed it to “dawn.” The returned file preserved “dawn,” showed all three
  passages, disabled direct acceptance, and accepted only after explicit passage
  selection. SQLite inspection confirmed a pre-acceptance draft containing
  “dawn” and one accepted feedback entry. Reimport kept that single entry and its accepted decision.
- This replacement test found a cross-language JSON key-order mismatch. Slice
  comparison now uses structural equality, covered by a serde-style sorted-JSON
  round-trip regression.
- A 60,000-word fixture (60 sources; about 406,000 characters) retained separate
  suggestions. Native edit measured 26 ms; search index construction 58 ms and
  cached searches 2–5 ms. These are observations on this machine, not a hardware
  independent performance guarantee.
- Native DOM geometry and computed colors were inspected. The screenshots below
  are **browser renderings of that captured native DOM**, with the application's
  styles and fonts; they are not screenshots of native window pixels. The prose
  sheet remains light in both themes, with sufficient room for the editor and
  feedback sidebar at the supported minimum window size.

| Viewport                                  | Light                             | Dark                            |
| ----------------------------------------- | --------------------------------- | ------------------------------- |
| 1360 × 768 content (1360 × 800 OS window) | [Light](workspace-1360-light.png) | [Dark](workspace-1360-dark.png) |
| 1600 × 968 content                        | [Light](workspace-1600-light.png) | [Dark](workspace-1600-dark.png) |

Windows MSI/NSIS and Linux deb/RPM/AppImage installation, association/update paths,
and cross-platform runtime behavior remain unexecuted on this macOS host. macOS
registration was tested with a candidate bundle, not an upgrade from an older
release. Follow [native-checks.md](native-checks.md) before release sign-off.

## Final checks

- Frontend: 38 files, **501 tests passed**; coverage 98.53% statements, 94.10%
  branches, 98.79% functions, 99.17% lines (above repository gates).
- Rust: **455 tests passed** with all features.
- `npm run check:all` passed: design-system guard, Svelte/TypeScript, Prettier,
  ESLint, rustfmt and Clippy. Six existing Svelte warnings remain; rich comparison
  HTML is serialized through the restricted prose schema (four lint warnings).
- Production frontend build passed. Final macOS desktop bundle build passed.
- Review-agent's final read-only follow-up: **No findings**.
- Claude's final high-effort follow-up: **No findings** (same session above).

The final review regressions cover multiparagraph paste followed by withdrawal
and undo/redo, and reopening a substantial scene with 20 separate suggestions.
Both tests were observed failing with their respective fixes reverted, then
passing after restoring the fixes. Live editor and package documents now share a
comparison schema; restored tracking uses the same reactive source identity as
the mounted workspace.

First-open error handling is centralized in the action wrapper. A new regression
covers the visible alert, retry path and closing. The prior per-caller guards
already covered unsupported file opens; this is a uniformity improvement, not
evidence of a previously reproducible blank failure.

## Editor UI redesign follow-up — September 8, 2026

Follow-up base: `0bd0f083542bfac4d90bc47c88ddb9d499770f3c`.
This section supersedes the initial modal UI description and screenshots above.

| Approved criterion                       | Implementation evidence                                                            | Validation                                                                       | Gap  |
| ---------------------------------------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- | ---- |
| Main editor, no review overlay           | App embeds EditorialWorkspace; ScenePanel hands off saved prose and selection      | Scene integration tests; native fresh sample                                     | None |
| Shared writing controls and paper        | ProseToolbar used by NovelEditor and EditorialManuscript                           | Formatting/undo regressions; native typing/undo; both themes                     | None |
| Contextual feedback and references       | ReviewSidebar threads and Review/References tabs; explicit reference scene context | Name typing, reply failure, reference context tests; native threads              | None |
| Natural local and portable suggestions   | Local immutable rounds reuse continuous engine and acknowledged saves              | Local mode/save/lock tests; Rust local-round resumption; native local acceptance | None |
| Existing scene feedback and history      | Non-destructive adapter; separate Draft history view; atomic batch decisions       | Anchor, inactive prose, history CAS and rollback tests; native retained drafts   | None |
| Readable markup and contextual conflicts | Paragraph markers; Simple/All markup; distinct unapplied legacy previews           | Caret/markup regressions; native render inspection                               | None |
| Occasional actions outside prose         | Menus for history, packages, exports, scoped bulk operations                       | UI tests and native menu flow                                                    | None |

Validation for the redesign: 512 frontend tests passed; coverage statements
98.58%, branches 94.17%, functions 98.84%, lines 99.20%. All 458 Rust tests passed.
`npm run check:all` passed (six existing Svelte warnings and two restricted-schema
HTML rendering lint warnings; no errors). Production frontend build passed with
the existing large-chunk warning. The full run includes all 29 workspace tests. A 500-reference-library test
timed out during a concurrent build run, passed individually, then passed in
the full coverage run with one worker; no assertions or timeouts were weakened.

Native macOS QA used an isolated `com.kindlingwriter.editorialuiqa` profile and
disposable database. It covered fresh demo review, local save/reopen/acceptance,
SQLite verification of changed prose and preserved pre-acceptance history,
closing the project to Home, opening a review file through the running-instance
route, and ordinary typing/undo in the portable editor. Computed styles in both
themes kept paper `rgb(244,239,230)` and prose ink `rgb(35,29,24)`; the dark sidebar
used `rgb(38,33,27)` with text `rgb(232,224,212)`. No horizontal overflow at a
1360×800 outer window (1360×768 content). The 1600×1000 outer window was also
inspected. Fonts were loaded from bundled local assets.

The PNGs are browser renderings of captured native DOM with current form values,
scroll positions, application styles, and local fonts; they are not native pixel
captures. No platform installation claims are added by this UI follow-up.

Review-agent found and verified fixes for inline caret selection, attribution
input lifetime, project closing, reference context, current locks, legacy markup,
unsupported controls, history CAS refresh, and a close-failure propagation path.
Its final follow-up including inactive feedback ordering and labeling: **No findings**.
Claude’s initial high-effort review (session `4b115be0-716e-4040-a9d4-08c11f999500`)
identified inactive-feedback discoverability, unnecessary widget rebuilding,
manuscript cache invalidation, missing bulk UI tests, and an undefined line-height
token. All five received fixes. Scene review loading now uses one batched IPC;
its native registration was caught and corrected in the independent follow-up.
The actual native command returned eight scene reviews with fifteen annotations.
Claude's final high-effort follow-up in the same session: **No findings**. It
rechecked the final staged code, including command registration and the last
inactive-feedback labeling change, and verified all five fixes. It withdrew the
sixth concern after confirming that actual prose replacements enforce locks and
an already-applied, reanchored suggestion can legitimately record a decision
without replacing prose. Both reviewers remained read-only; test, build, native,
and screenshot verification were performed by the primary agent.

## Review menu and compact controls follow-up

The review options popover now uses left-aligned icon actions grouped by scope,
with short labels and no empty bulk-action groups. Filtered or inactive feedback
cannot become a target through those groups. Name editing stays open; action
selection, outside clicks, and Escape dismiss the menu. Escape handling is local
to the menu, so native File → Close Project cannot leave a hidden workspace
intercepting the next keypress.

At the user's explicit request, compact desktop review fields use the existing
14px `--text-small` token, overriding the guide's default 16px form minimum for
these controls. Full package setup forms keep their scale. macOS WebKit ignored
padding on native select appearance; compact selects now retain their
native interaction with a styled surface and a non-interactive chevron. Native
computed styles for the status, mode, markup, and feedback filters in both themes
confirmed 14px text, 4px vertical padding, 8px
left padding, 32px right space for the chevron, and 32.39px control height.
Name inputs use 4px/8px padding, and comment/reply areas use 8px padding.

All 31 workspace tests passed, including filtered bulk groups, identity editing,
menu dismissal, focus return, and the native-close Escape regression.
`npm run check:all` and production frontend build passed with the same existing
warnings as the prior follow-up. Native Escape dismissed the menu and restored
focus to its trigger. The temporary theme change was restored. Screenshot
artifacts `review-menu-light.png` and `review-menu-dark.png` are cropped browser
renderings of captured native DOM with current input values, styles, and fonts.

Both review-agent and Claude high-effort follow-up (session
`4b115be0-716e-4040-a9d4-08c11f999500`) finished with **No findings**. Claude
verified the Escape fix and withdrew a screenshot concern after native menu and
input colors both measured background `rgb(38,33,27)` and text
`rgb(232,224,212)`; direct PNG samples also confirmed the panel and input were
dark. All five compact selects use the same wrapper and chevron treatment.

## Package, history, and manuscript polish follow-up

Follow-up base: `82d09c7bf592116639064ec00230e10f1936e7f0`.

| Requested improvement                  | Implementation evidence                                                                                                 | Validation                                                                                               | Gap  |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ---- |
| Package setup polish                   | Separate setup and continuing-review columns; padded compact fields; explicit entire-manuscript/selected-chapters scope | Native layout and scope checks; selected scope cannot export without a chapter; workspace regression     | None |
| Clear draft history                    | Newest-first saved-draft list; separate original and comparison panes; active prose only; visible text-change legend    | Three history tests, including latest selection and unchanged prose; both-theme visual inspection        | None |
| Consistent dismissible manuscript menu | Existing ContextMenu component, icons and divider; outside pointer and Escape dismissal; focus return                   | Native outside dismissal and history opening; workspace menu regression                                  | None |
| Green/red decisions                    | Success/error foreground, border and wash tokens for enabled individual and bulk decisions                              | Native computed colors match tokens in both themes                                                       | None |
| Search navigation                      | Explicit pane scrolling and persistent match decoration; refresh matches after edits and on search reopening            | Native distant search with input focus retained; count, highlight, edit, reopen and scrolling regression | None |
| Selected feedback navigation           | Scroll selected passage inside manuscript pane after rendering; saved reading position cancels queued cursor reveals    | Native cross-scene sidebar selection; read-only scroll and resume-position regressions                   | None |
| Readable paragraph spacing             | Writing-editor paragraph margins and indentation; matching blockquote treatment                                         | Native computed margins zero and subsequent paragraph indentation 1.5em; content unchanged               | None |

All 37 focused workspace/history tests passed. `npm run check:all` passed with
the same six existing Svelte warnings and two restricted-schema HTML lint
warnings; production frontend build passed with the existing large-chunk
warning. No Rust code changed in this follow-up.

Native macOS verification used the running demo without editing manuscript data,
saving drafts, deciding suggestions, or exporting packages. Searching for
`register` scrolled the manuscript to 1991px and placed the highlighted match
inside the viewport while keeping focus in the search input. Selecting feedback
for The Seventh Step scrolled from 0 to 1207px and revealed `The step`. Enabled
Accept/Reject colors measured #356B4D/#9E3E36 in light and #5FA37E/#E58B7F in dark,
matching the semantic tokens. Package setup and history had no horizontal
overflow at a 1360×800 outer window. Theme and window size were restored.

New `package-setup-light.png`, `package-setup-dark.png`,
`draft-history-light.png`, and `draft-history-dark.png` show the updated screens.
They are browser renderings of captured native DOM with current field values,
checked states, styles, and fonts, rather than native pixel screenshots.

The independent review-agent found queued cursor scrolling overriding restored
reading position and stale search offsets after edits. Claude's high-effort
code-review session `9b013d4c-3675-4833-8665-be790861a556` found missing native
radio grouping, stale results after reopening search, and a mismatched visible
and accessible comparison label. All were fixed and covered by focused tests.
Both reviewers checked the final application code and returned **No findings**.
The platform installation and upgrade verification gaps above remain unchanged.

## Menu interaction and pinned search follow-up

Follow-up base: `37dc05a9d9565c609d08a6de9f353ef8204f5870`.

| Requested improvement                  | Implementation evidence                                                                                                                               | Validation                                                                                               | Gap                                                                                        |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Review options polish                  | Flat full-width action rows, consistent insets, separated identity field; green/red text with hover washes                                            | Updated light/dark menu captures; native computed styles                                                 | None                                                                                       |
| Working header menu                    | Opening click does not reach the newly mounted menu's outside handler; outside capture coordinates Review options for pointer and keyboard activation | Event-flush timing, toggle, dismissal, action and menu-coordination regressions; native DOM click checks | Physical pointer tool did not deliver events; pointer behavior covered by regression tests |
| Search stays visible during navigation | Search is a fixed flex row outside the manuscript scroller; formatting toolbar remains sticky inside it                                               | Search Next/reopen regression; native distant navigation and smaller-window geometry                     | None                                                                                       |

All 34 workspace tests passed. `check:all` passed with the existing warnings;
the production frontend build passed with the existing large-chunk warning.
The native search bar stayed at y78–143 while Next navigated to the third
`letter` match and scrolled the manuscript to 2000px. The toolbar remained at
y143–198 and the match was visible at y469–486. At a 1360×800 outer window,
search retained its position and there was no horizontal overflow.

Native menu rows measured 8px/12px padding with transparent resting backgrounds
and semantic decision text in both themes. The name input remained 14px with
normal ink (#231D18 light, #E8E0D4 dark), confirmed against Press's direct control
color declaration. Updated `review-menu-light.png` and `review-menu-dark.png`
are browser renderings of captured native DOM. Theme and window size were
restored; no manuscript data was edited for these checks.

Review-agent identified coordination of the two menus for pointer and keyboard
activation; both paths now dismiss the previously open menu. Its final result:
**No findings**. Claude high-effort session
`067fdf32-16fb-42b4-8f6c-2d9181cebb50` identified duplicate hover declarations,
which were removed. It withdrew the input-color concern after checking the
direct Press control declaration and its cascade above Tailwind preflight.
Claude's final result: **No findings**.

The first normal push run passed 517 tests but the existing 500-reference-library
test exceeded its 20-second timeout (24.8 seconds under full-suite contention).
That test passed in isolation in 4.97 seconds. The retry uses Vitest's supported
`VITEST_MAX_WORKERS=1` setting through the normal push hook; assertions, timeouts,
and hook checks are unchanged.

## Package return navigation and round timestamps

Follow-up base: `7e79af8d61ae96e887fd28a0b651adfd1ae2c476`.

| Requested improvement                 | Implementation evidence                                                                                                     | Validation                                                                                                                                              | Gap  |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| Back returns to originating revisions | Package setup retains active local mode, review/save state and captured manuscript view; direct opens retain close behavior | Reviewing/Suggesting roundtrip tests include pending edits, filter, cursor, scroll, repeated setup and close/reopen; native Reviewing returned to 750px | None |
| Readable manuscript menu              | Workspace-scoped 22rem width with viewport cap; icons do not shrink; shared menu unchanged                                  | Native computed 352px width and 40px single-line rows; light/dark browser-rendered menu details                                                         | None |
| Distinct same-day rounds              | Localized date and time including seconds; existing newest-first order retained                                             | Timestamp precision regression; native rounds showed 4:27:31 PM, 4:27:12 PM and 4:27:01 PM on the same day                                              | None |

All 37 workspace tests and `check:all` passed with the existing warnings.
Production frontend build passed with the existing large-chunk warning.
`package-setup-light.png` and `package-setup-dark.png` now include timestamps;
`manuscript-menu-light.png` and `manuscript-menu-dark.png` show the wider menu.
These are browser renderings of captured native DOM; the menu detail is positioned
inside the browser viewport for the crop. The native webview was hidden during
inspection, so its animation-frame viewport adjustment was not a runtime visual
check. Window size and theme were restored; no manuscript data was changed.

Review-agent's final result: **No findings**. Its review covered package return
state, save-queue lifetime, failure paths, closing/replacement, menu scope and
timestamp formatting.

Claude high-effort session `bf6565dc-e36b-47b8-80e1-024f355a3ba3` identified
package messages surviving Back and an unrelated change to local sidebar state
on direct opens. Back now uses the shared action lifecycle to clear messages;
the sidebar-state change was removed. Roundtrip tests cover both a package export
notice and a failed round-open alert, plus direct reopening after local review.
Review-agent rechecked these fixes and again returned **No findings**.
Claude's final focused follow-up also returned **No findings**.
