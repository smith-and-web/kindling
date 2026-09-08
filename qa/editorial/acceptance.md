# Editorial workflow acceptance — #287

Base: `70cad8793772c3bc7dfd211745352744cccfe9f9`; branch `feat/editorial-packages`.

| Criterion | Implementation evidence | Validation | Remaining gap |
| --- | --- | --- | --- |
| Realistic revision demo | `sample_revisions.rs`; status synchronization in `db/revisions.rs` and `db/queries.rs`; scene guide in `docs/editorial-review.md` | Sample assertions cover all statuses, history, anchors, locks and both prose modes | None; fresh native sample created and exercised |
| Continuous manuscript | `EditorialManuscript.svelte`; block ownership, rich schema and source mapping in `editorial.ts` | Cross-scene replacement, source suffix preservation, search, formatting, keyboard focus and 60k-word tracking regression tests | No native pixel capture available; rendered native DOM checked at both sizes/themes |
| Suggestions and decisions | Natural TipTap edits/history; comment mapping and merged discussions; contextual comparisons and explicit reanchor; atomic backend decisions | Engine, workspace, and Rust acceptance/conflict/lock/history tests | None; both final independent reviews report No findings |
| Portable files and recovery | Versioned JSON packages; SQLite rounds, returns and sessions; CAS save queue and local recovery journal; reopenable recovery packages | Round-trip, repeated/partial import, fresh-install recovery, generation and failed-save tests | None; both final independent reviews report No findings |
| Writer/editor discussion | Writer response export merges replies/decisions into the editor's ongoing review | Backend preservation test and workspace export interaction test | None; native editor → writer → editor replies/acceptance passed |
| Native file opening | Tauri file associations, macOS document identities/icons, queued initial/open events and single instance routing | Packaged macOS cold/running open on separate app profile succeeded; repeated open kept one session | Windows/Linux installer and upgrade runs unavailable on this host; macOS upgrade from an older release not exercised |
| Continued writing and multiple rounds | Immutable exported baseline; current prose CAS; affected-passage conflicts; scoped bulk preparation | Disjoint writer edits, changed structure, independent reviewers, later rounds and repeat-import tests | None; both final independent reviews report No findings |

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

| Viewport | Light | Dark |
| --- | --- | --- |
| 1360 × 768 content (1360 × 800 OS window) | [Light](workspace-1360-light.png) | [Dark](workspace-1360-dark.png) |
| 1600 × 968 content | [Light](workspace-1600-light.png) | [Dark](workspace-1600-dark.png) |

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
