# Kindling visual QA suite

> For documentation and website screenshots use `qa/demo/` instead. It seeds a
> rich, reproducible project into its own data dir. Do not put demo content in
> `qa/visual/data`: a QA run imports its fixture on top of it, and
> `q.cleanupFixtures()` only deletes ids the harness itself imported.

A screenshot-verified mirror of the WebDriverIO end-to-end suite in `e2e/`,
extended to the rest of the app and driven through the `kindling-mcp` tools
from Claude Code. It exists because the WebDriver suite cannot run on macOS
(no WKWebView driver) and because it asserts only DOM structure, never
appearance.

|         | `e2e/` (WebDriverIO)                                                | `qa/visual/` (kindling-mcp)                                                                       |
| ------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Runs on | Linux / CI / Docker                                                 | macOS dev machine, against `npm run tauri:qa`                                                     |
| Driver  | tauri-driver + WebKitWebDriver                                      | `tauri-plugin-mcp` socket at `/tmp/kindling-mcp.sock` (debug builds only)                         |
| Asserts | element exists, text, counts                                        | the same, plus pixel diff against baselines, Press audits, axe-core, and a judged screenshot      |
| Verdict | pass / fail                                                         | pass / DOM regression / visual regression / inconclusive, per checkpoint                          |
| Fixture | `test-data/simple-story.pltr` via `__KINDLING_TEST__.importProject` | same, plus `simple-story-v2.pltr` for sync updates and a scratch database via `KINDLING_DATA_DIR` |

## Layout

```
qa/visual/
  README.md            this runbook
  harness.js           window.__qa helpers, served by Vite, loaded with one execute_js
  rename.mjs           names screenshot files from the __qa.shot() log
  history.mjs          builds results/index.md from every report.md
  baselines/           one reference JPEG per screenshot checkpoint (committed)
  data/                scratch app data dir used by `npm run tauri:qa` (gitignored)
  fixtures/            small authored import fixtures (currently Longform)
  scenarios/           one file per e2e spec, then the areas e2e never covered
    00-app-launch.md
    01-create-content.md      ... 05-reimport.md   mirror the e2e specs
    06-structure-actions.md   parts, rename, duplicate, lock, archive, part delete
    07-page-view.md           page view and the switch-back confirmation
    08-references.md          add a character through the References panel
    09-settings-and-theme.md  settings dialogs and the real theme radio
    10-tools.md               export dialog, snapshots, command palette
    11-new-project.md         blank project, no sync button
    12-sync-updates.md        sync picks up a renamed chapter, new scene, new beat
    13-import-formats.md      Markdown outline, yWriter parts, Longform references
    14-novelwriter-round-trip.md  export options, selective prose sync, narrow/dark diff
    98-narrow-sweep.md        1100 x 700 overflow check
    99-dark-sweep.md          four dark screens
  results/             per-run output (gitignored)
    index.md           run history, from history.mjs
    <YYYY-MM-DD-HHMM>/
      report.md
      <scenario>-<nn>-<label>.jpg
      <checkpoint>-FAIL.jpg, <checkpoint>-FAIL.html   failure artifacts
```

## Budget

The machine has a mandatory screen lock. Targets for the core run (00–12) plus both sweeps:

| Tool calls | Screenshots | Wall clock  |
| ---------- | ----------- | ----------- |
| about 110  | about 40    | 8 to 10 min |

Run the import extension separately: `/visual-qa 13` (3 screenshots, about
3 minutes) and `/visual-qa 14` (5 screenshots, about 4–5 minutes). Scenario 14
includes dark and narrow checks of the prose diff, a surface the standard
sweeps do not show. A request for all scenarios needs multiple run groups.

If a run is going to exceed this, run scenarios in groups (`/visual-qa 06 07 08`)
and let each run write its own results folder. `history.mjs` merges them.

## Prerequisites

1. The dev app running with a scratch database:
   ```bash
   npm run tauri:qa
   ```
   This sets `KINDLING_DATA_DIR=qa/visual/data`, which debug builds honour and
   release builds ignore (`resolve_data_dir` in `src-tauri/src/lib.rs`). The
   first launch shows onboarding. Use this isolated database for QA runs.
   `q.saveLocal` / `q.restoreLocal` protect your `kindling:*` preferences.
2. The `kindling-mcp` server connected in Claude Code.
3. `npm install` done (axe-core is a dev dependency, served to the webview by
   Vite).

## Run procedure

Run `npm run test:qa-harness` before using the harness. These regression tests
exercise cleanup with an in-memory IPC mock, including pre-existing projects
whose filenames or names match the fixtures, reload, and failed operations.

Ask Claude: `/visual-qa` for everything in bounded groups, or `/visual-qa 03 04`
for some. Each numbered item is one or two tool calls; do not add `wait_for`, state reads or
renames beyond what is listed.

1. **Preflight, one Bash call.**
   ```bash
   RUN="$PWD/qa/visual/results/$(date +%Y-%m-%d-%H%M)"; mkdir -p "$RUN"; (caffeinate -dimsu & echo $! > "$RUN/.caffeinate.pid"); cp test-data/simple-story.pltr /tmp/qa-simple-story.pltr; echo "$RUN"
   ```
2. **Readiness and window, three calls.** `query_page app_info` must show a
   `main` window and a non-empty `monitors` list. `manage_window set_size`
   1600 x 1000, then `manage_window focus`.
3. **Install the harness, one call.** Vite serves the repo at `/@fs/`:
   ```js
   (() => {
     delete window.__qa;
     const s = document.createElement("script");
     s.src = "/@fs/<abs repo>/qa/visual/harness.js?t=" + Date.now();
     document.head.append(s);
     return "loading";
   })();
   ```
   The script loads asynchronously; if the next call finds `window.__qa`
   undefined, retry that call once. Every `execute_js` from here on starts
   with `const q = window.__qa;`.
4. **Guard and fixture, two calls.** `q.saveLocal(); q.importFixture("<abs repo>/test-data/simple-story.pltr")`
   then `wait_for` text `Act 1`. Import once for the whole run; scenario 12
   imports its own temp copy.
5. **Scenarios.** Per-checkpoint pattern below. Each scenario also gets one
   `q.axe()` on its most complex dialog or panel and one `q.audit()`, and
   ends with `q.flush()` plus `q.takeErrors()`.
6. **Sweeps.** `98-narrow-sweep.md`, then `99-dark-sweep.md`.
7. **Cleanup, two calls.** `q.cleanupFixtures(); q.cleanupNamed("QA Blank Project"); q.restoreTheme(); q.restoreLocal()`
   then read `q.last`.
8. **Rename, diff, report.** Read `q.shots`; one Bash call runs
   `node qa/visual/rename.mjs "$RUN" '<shots json>'` and prints the folder.
   Then `q.diff("$RUN", [{ name, file }, ...])`, `wait_for` `#qa-diff-done`
   attached, read `q.diffResult`. Kill caffeinate. Write `report.md` with a
   "Diff" column, run `node qa/visual/history.mjs`.

### Per-checkpoint pattern

- **Screenshot checkpoint, three calls.** `execute_js`: return the previous
  checkpoint's state if not yet read, perform this checkpoint's action, then
  `q.shot("01-01-chapter-input")`. `q.shot` forces a repaint and schedules
  `#qa-settled-01-01-chapter-input`, which appears only after two animation
  frames. `wait_for` that selector with state `attached`, then
  `take_screenshot` with `output_dir: RUN`, `max_width: 800`, `quality: 55`,
  `audience: assistant`. Without the settle gate about one capture in five
  shows the frame before the action. For UI that arrives asynchronously (a
  dialog after an `invoke`), `wait_for` the dialog selector first, then
  `q.shot` in the next call.
- **DOM-only group, two calls.** `q.run([...])` with the step list from the
  scenario; the last step calls `q.done("02")`; `wait_for` `#qa-done-02`
  attached. Text that is already on the page returns immediately and races
  the chain; the marker does not.
- **`wait_for` only for real async gates:** import complete, scene list after
  a chapter click, beat list after a scene click, save indicator hidden, sync
  preview dialog, settle and done markers.
- **Judging.** With a baseline present, trust `q.diff`: `changed === false`
  means the capture matches the last accepted image and needs no close look.
  Look at every capture flagged `changed`, `no-baseline`, `size-mismatch` or
  `missing-capture`, judge it against the Expect block, and if the change is
  intended copy the new file over the baseline
  (`cp "$RUN/<name>.jpg" qa/visual/baselines/`).
- **Failure artifacts.** On a failed assertion, before moving on: full-size
  capture (`max_width` 1600, `quality` 80) renamed `<checkpoint>-FAIL.jpg`,
  and `q.dom("<panel selector>")` written to `<checkpoint>-FAIL.html` via a
  Bash heredoc.
- **Preflight.** The action call before a screenshot batch returns
  `q.preflight()`. `ok: false` means the page is hidden; that is usually
  occlusion, and captures often still work. Stop only if a capture comes back
  uniformly blank after `manage_window focus`.

## What the harness checks beyond the DOM

- `q.audit()` scans every visible element and reports text set in a font
  other than Inter, Newsreader, Fraunces or a monospace stack; text below
  WCAG contrast (4.5, or 3 for large text) against its effective background;
  prose columns wider than their max width; and colours that do not resolve
  to any Press token. `fonts`, `contrast` and `measure` entries are visual
  regressions. `untokened` is a lead, not a verdict, because rgba washes and
  images legitimately produce non-token colours.
- `q.axe(context)` runs axe-core WCAG 2 A and AA rules on `document` or a
  dialog element. Result in `q.axeResult` after `wait_for` `#qa-axe-done`.
  Report every violation with `impact` serious or critical; list moderate
  ones once per run.
- `q.overflow()` lists elements whose content overflows horizontally without
  their own scrollbar, plus document-level overflow. Used by the narrow
  sweep.
- `q.errors` collects `console.error`, `window.onerror` and unhandled
  rejections from the moment the harness loads. `q.takeErrors()` at the end
  of a scenario; any entry is a finding even when the DOM looks right,
  because a rejected `invoke` otherwise fails silently.
- `q.diff(runDir, pairs)` compares captures with `baselines/` using the
  webview's canvas: per-channel tolerance 24 absorbs JPEG noise, and more
  than 0.4% differing pixels marks a capture `changed`, with the bounding
  box of the change.

## Harness rules (read before writing a scenario)

- **`execute_js` does not resolve promises.** Helpers are synchronous or
  record into `q.log`, `q.last`, `q.axeResult`, `q.diffResult` and signal
  with a marker element.
- **Drive the UI through the DOM.** `q.click(testid, text)` calls `.click()`
  on the owning `<button>`, which reaches Svelte 5 delegated handlers. Prefer
  test ids (the list is in `e2e/README.md`); `q.clickText`, `q.clickSel` and
  `q.fillPlaceholder` are for controls that have none yet. `q.fillTitle` and
  `q.setProse` cover the inline title input and the beat editor.
- **Do not use the OS-level `click`, `mouse_action hover` or `mouse_action drag`
  tools.** Their coordinate space is unconfirmed and a drag once hid the
  window. `q.drag` and `q.hover` are DOM equivalents and are verified.
- **Do not edit `harness.js` while a run is in progress.** Vite serves it via
  `/@fs/`, so a change triggers a full page reload: the open project closes,
  `window.__qa` disappears and any `q.run` chain dies.
- **Two things need their own step or call:** `q.menuItem` after
  `q.openMenu` (the menu renders on the next tick), and any click on a
  chapter title, which toggles rather than selects, so guard it with "only if
  no scene rows are visible".
- **Theme.** `q.setTheme("dark")` sets the attribute, forces a repaint and
  schedules `#qa-settled-theme`. Scenario 09 uses the real radio instead.
- **CSS-uppercased headings** (`Part One QA`, `Scene Prose`) defeat
  `wait_for` text; wait on selectors or markers.
- **Several dialogs share `aria-label="Close"`.** Use the per-dialog test ids
  (`rename-close`, `export-close`, ...) or `q.closeDialog("<title id>")`.
- **Async settle times.** Deletes, duplicates and archive need about a second
  before the sidebar reflects them; use 900 to 1200 ms step delays.
- **Import promise never settles.** Gate on `wait_for` text `Act 1`.
  For additional formats use `q.importFixture(path, format)`, where format is
  `plottr` (default), `markdown`, `ywriter`, `longform`, `scrivener` or
  `novelwriter`. Wait for the fresh `#qa-done-fixture-created` marker, then the
  expected chapter text. `q.fixtureProject()` returns the last owned fixture's
  captured ID, including after reload. It never searches existing projects.
  `q.invoke(command, args)` signals `#qa-done-invoke` on success **or failure**;
  check `q.last.status` before continuing. These helpers bypass native file
  pickers and reference classification; record that coverage boundary.
  `q.cleanupFixtures()` deletes only IDs returned by this harness's fixture
  imports, across all supported formats. `q.createBlankProject()` captures the
  ID from the scenario 11 creation click; `q.cleanupNamed(name)` can delete only those owned IDs.
  Ownership survives page reload in sessionStorage. With no ownership record,
  cleanup deletes nothing. Never adopt existing projects by filename or name.
  Finish cleanup before starting a new run; closing the window loses the
  record, leaving any remaining QA projects for manual inspection.
- **Screenshot naming.** `take_screenshot` ignores names. `q.shot(name)`
  records name and time (persisted in `sessionStorage`, so it survives the
  reload test); `rename.mjs` maps files to names at the end.
- **Never** hand-edit the fixtures in `test-data/`. Scenario 12 copies them to
  `/tmp`.
- **Shell.** `ls` is aliased to a tool that prints nothing when piped; use
  `command ls`.

## Verdicts

- **pass**: structural assertion holds, diff unchanged or the changed capture
  matches the Expect block, no audit or axe finding at serious or above.
- **DOM regression**: structural assertion fails, or `q.takeErrors()` is
  non-empty for the scenario.
- **visual regression**: structure is right but the capture or `q.audit()`
  shows clipping, overflow, wrong theme, illegible contrast, wrong typeface,
  or anything the Expect block rules out.
- **inconclusive**: the step could not be performed reliably. Say why.

Be specific in evidence. "Sidebar project title is 2.1:1 against the dark
surface" is a finding. "Looks slightly off" is not.

## Adding a scenario

Copy the structure of an existing file. Put DOM-only checkpoints into one
`q.run([...])` block ending in `q.done(id)`. Name every screenshot with
`q.shot` and add its baseline after the first accepted run. Dark theme is
covered once by `99-dark-sweep.md` and narrow widths by `98-narrow-sweep.md`;
add a screen there only if it introduces a surface the sweeps do not show.
When a control has no test id, add one in the component and list it in
`e2e/README.md` rather than matching on text.

## Startup overlay regression

Run `npm run build && npm run test:startup` with a Playwright Chromium browser
installed (`npx playwright install chromium`). To use an existing Chrome binary,
set `PLAYWRIGHT_CHROMIUM_EXECUTABLE` to its executable path.

The test serves production assets, withholds the application bundle and initial
project response, and checks first paint before the bundle request, overlay
stacking, disabled background interaction, light/dark/system themes, reduced
motion, ready-state handoff, and retry after a failed bundle request. It uses an
isolated browser context and a minimal IPC fixture; no project data is changed.
Screenshots are saved under `qa/visual/results/startup/`.

Also smoke-test a native cold launch: the first visible window should contain the
loading overlay and then uncover the start screen (or a document opened at launch).
Reload should not steal focus. Browsers provide rendering opportunities through
animation frames, not a portable guarantee that pixels reached the display.
