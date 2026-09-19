# Visual coverage for v1.3

`npm run qa:visual -- --list` lists executable socket suites. The default runs
all of those suites in light (1600×968), dark (1600×968), and narrow light
(1100×668), rendered as lossless 2× PNGs from an isolated hidden WKWebView.
These are CSS viewport sizes, independent of desktop geometry. Each screenshot records a DOM assertion, Press audit, axe results,
overflow candidates, console errors, and baseline diff. These are **surface and
interaction checks**, not complete acceptance of every feature.

The numbered Markdown scenarios remain the release checklist. A scenario being
listed here does not mean its manual checks ran. Record them separately in the
run report, including skipped and inconclusive checks.

| Area                                                                            | Socket suite                | Additional release checks                                                                                                                       |
| ------------------------------------------------------------------------------- | --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Start, empty recent-project placeholder, six import formats, review entry point | 00                          | Populated recent-project cards, fresh onboarding, native cold launch; startup browser regression                                                |
| Outline and chapter creation                                                    | 01                          | 01–06: scene/beat CRUD, delete confirmations, locks, archives, parts, duplicates, drag order and reload                                         |
| Beat and Page editors                                                           | 07                          | Page-to-Beat warning with independently edited Page prose; formatting, beat reorder, autosave failures                                          |
| Reference creation and copy-source/filter UI                                    | 08                          | Copy commit, duplicates/Keep both, typed fields, nested tags, disabled categories, 500-item library, source independence, reload (08 checklist) |
| All unified settings areas and theme radios                                     | 09                          | Draft retention/discard guards, saving to another project, goals/tags/fields CRUD, empty library, failed-load/retry (09 checklist)              |
| Export dialog, empty/populated snapshots, palette                               | 10                          | Actual output files/options, snapshot restore/delete, native file chooser and OS shortcut delivery                                              |
| Custom export workspace, remembered profiles, generated files and exchange previews | 19, 22, 23              | Native save/cover choosers, deeper format options and output fidelity beyond the generated-file/structure checks                                 |
| Blank project and no sync                                                       | 00 previews creation dialog | 11: create novel/screenplay, templates, no-source behavior                                                                                      |
| Source sync and selective updates                                               | Not ported                  | 05 and 12; use temp copies of fixtures only                                                                                                     |
| Markdown, yWriter, Longform                                                     | Not ported                  | 13; Scrivener native import/classification remains additional manual work                                                                       |
| novelWriter import/export/prose diff                                            | Not ported                  | 14 including its dark/narrow diff and real round trip                                                                                           |
| Find/replace, results, empty search, replace-all confirmation, undo             | 15                          | Scene scope, match case/whole words, locked/draft exclusions, Open scene, formatted prose preservation                                          |
| Writing counts, statistics, collapsed sidebar, Previously                       | 16                          | Goals/streaks, session reset/net deletions, cross-chapter and Page excerpts; no import/restore/review writing credit                            |
| Editorial manuscript, menu, history/overview and package setup                  | 17                          | Suggest/comment/reply/resolve; simple/all markup; portable export/open/return/accept/reject; conflicted and failed saves                        |
| About and feedback form                                                         | 18                          | Offline/validation/failure/success component tests; do not send QA feedback to the live service                                                 |
| Planning, discovery notes and screenplay                                        | Not ported                  | 19, including screenplay element types, page targets and export                                                                                 |
| Shortcut filter, recorder and rejected binding                                  | 20                          | Save/clear/reset persistence, conflicts, native menus and hints, real key chords                                                                |
| Resume position                                                                 | Not ported                  | 21: scene/beat/cursor/scroll after project switch and native restart                                                                            |
| Narrow and dark                                                                 | Every socket checkpoint     | 98/99 supplement manual surfaces, including sync/prose diff and destructive confirmations                                                       |
| Startup loading/retry                                                           | `npm run test:startup`      | Native first visible frame and reload focus; browser test alone cannot prove OS presentation                                                    |

See [editorial acceptance](../editorial/acceptance.md),
[native editorial checks](../editorial/native-checks.md),
[settings acceptance](../settings/acceptance.md), and
[Previously acceptance](../previously/acceptance.md) for deeper feature-specific
checks and prior evidence. Prior evidence is not a new release run.

The socket runner deliberately uses real app state and disposable imported
projects. Seeding prose uses IPC; it does not test typing or award semantics.
Menu events use the same application handler as native menus; they do not test
OS accelerator registration. DOM clicks do not prove keyboard focus trapping.
A screenshot cannot establish data integrity, restart persistence or file-format
interoperability: run the corresponding behavioral/native checks as well.
