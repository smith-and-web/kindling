# Previous scene summary — issue #66

Base: `36aab52a6fd63ab4c896f7e3241fb62caf24f4cd` (latest main at branch creation).

| Criterion                              | Implementation                                                                      | Validation                                                                                                |
| -------------------------------------- | ----------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Collapsible Previously at top of scene | `Previously.svelte`, mounted above scene header                                     | Component tests; native desktop inspection                                                                |
| Predecessor title and synopsis         | Position-ordered chapter/scene traversal; retains pending synopsis                  | Same/cross-chapter tests, including save completion and cleared synopsis; native cross-chapter inspection |
| Closing 2–3 prose sentences            | Last three sentence segments; active Page or ordered Beat prose; no outline prompts | HTML/entities/dialogue/empty/short prose tests; native Beat and Page fixtures                             |
| Remember collapsed state               | `kindling:previouslyCollapsed` localStorage preference                              | Navigation/remount tests; native navigation and webview reload                                            |
| Hide when no predecessor               | First scene/missing selection return no summary                                     | Component/helper tests; native first-scene inspection                                                     |

Additional regressions cover archived/empty chapters, stale navigation responses,
load failures/retry, pending prose saves, delayed viewport restoration, and
refresh after Find/Replace, editorial apply, or draft discard. No new IPC commands, dependencies, or schema changes.

## Automated validation

- Focused suite: 35 tests passed.
- Full frontend suite with coverage: 560 tests passed in 41 files; statements
  98.56%, branches 94.11%, functions 98.61%, lines 99.13% (all gates passed).
- `npm run check:all` passed: Press sync, types, formatting, ESLint, Rust formatting
  and all-target/all-feature Clippy. Existing Svelte/ESLint warnings remain.
- `npm run build` passed; existing bundle-size warning remains.

## Native inspection

Ran the real Tauri debug app on macOS with a disposable database at
`/tmp/kindling-issue66-data` and an isolated application identifier
`com.kindling.issue66qa`. Imported `test-data/simple-story.pltr` and seeded only
that disposable project through existing IPC commands.

Verified a Beat excerpt in Discovery, a Page excerpt from Discovery while viewing
Turning Point in the next chapter, synopsis-only context with no prose, collapse
across navigation, and collapse persistence after webview reload/reopening the
project. Inspected the rendered UI and screenshots in both themes. Computed
reading styles: Newsreader, 17px, 576px maximum width; light ink `rgb(35,29,24)`
and dark ink `rgb(232,224,212)`.

Read-failure/retry and delayed-response races are covered by automated tests;
those failure scenarios were not established in the native runtime. Native
WebDriver E2E is unsupported on macOS.

Light and dark screenshots were inspected inline before the machine locked.
Later attempts to save screenshots returned black frames, so no screenshot files
are included as evidence.

The fixture project was deleted through IPC after inspection, saved preferences
were restored, and the isolated debug app was stopped.

## Toolbar layout follow-up

Previously and Revisions now share one compact, left-aligned Scene tools row.
The summary expands beneath it; the separate rows and dividing rules are removed.
Revisions remains available for the first scene, during loading, and on errors.

Validated against the running native app in light/collapsed and dark/expanded
states. Both controls share the same y-coordinate and 32px height, use Inter,
and have no borders. Inspected both screenshots and restored the original theme
and collapse preference. The 19 existing Previously/SceneRevisions component
tests, type check, ESLint, and production build passed.
