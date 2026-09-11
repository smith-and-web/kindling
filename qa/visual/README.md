# Kindling visual QA suite

`npm run qa:visual` drives a **hidden, isolated Kindling desktop process through
its Unix socket**. Navigation, fixture creation, audits and lossless screenshots
use that socket. No MCP server or per-checkpoint model calls are involved.

Screenshots come from the app's own **WKWebView.takeSnapshot** renderer, encoded
as PNG. They do not use macOS desktop/window capture. You can use your normal
Kindling app and other applications while QA runs; pointer position, overlapping
windows, notifications and display arrangement are outside the captured document.
No focus, Screen Recording permission, DND changes or HiDPI display mode is needed.

See [COVERAGE.md](COVERAGE.md) for executable v1.3 coverage and remaining manual
checks, [VALIDATION.md](VALIDATION.md) for measured evidence and known findings,
and [the visual-qa skill](../../.agents/skills/visual-qa/SKILL.md) for the agent workflow.

## Run

Prerequisites: macOS 14+, the project's Node/Rust dependencies and Xcode Command
Line Tools, and `oxipng` on PATH. macOS `sips` creates review previews only.

```bash
npm install
npm run tauri:qa                    # terminal 1: build/start the hidden QA app
npm run qa:baselines:check          # terminal 2: validate reviewed PNG manifest
npm run qa:visual -- --dry-run      # socket/capture capabilities; no app changes
npm run qa:visual                  # all 120 checkpoints
npm run qa:visual -- --list
npm run qa:visual -- --only 09,15,17 --variants light
npm run qa:visual -- --only 07 --variants narrow --calibrate
npm run test:qa-harness             # when changing QA tooling
node qa/visual/isolation.mjs         # optional native rendering/isolation proof
```

The launcher reuses this checkout's Vite server on port 1420, or starts one. It
builds the debug binary and starts an invisible process with a separate app
identifier, settings directory, WebKit website datastore, authenticated socket
(`/tmp/kindling-qa.sock`) and database (`qa/visual/data/background-profile/`).
The normal app's `/tmp/kindling-mcp.sock` and personal library remain independent.
An existing compatible QA process is reused. Stop a QA process before rebuilding
its native code. Stopping the launcher stops only processes it started.

Run one QA driver at a time; `/tmp/kindling-qa.sock.run.lock` rejects concurrent passes. Never
edit application, harness or suite files during capture: Vite reload invalidates
in-flight actions. Using the normal app is supported; changing the shared source
code during a run is not. The runner requires an empty QA library and refuses
to delete pre-existing projects to meet that requirement.

The process needs a running macOS login session. It does not require an unlocked
or illuminated display. `caffeinate -i` prevents idle system sleep during a pass
without preventing display sleep or lock. Actual logout terminates GUI processes;
shutdown or forced system sleep cannot be made transparent by a screenshot API.
This is macOS WebKit coverage, not a headless Linux/browser replacement.

Exit codes: **0** all checkpoint evidence passes; **1** run, assertion, audit or
cleanup failure; **2** review pending (changed/new/incompatible images or incomplete
evidence). A matching image cannot waive a failed accessibility audit.

The last normal 120-checkpoint pass took **235 seconds**; the 240-capture
calibration pass took **379 seconds**. See VALIDATION.md for measured evidence.

## Capture and comparison

The fixed CSS viewports are light **1600×968**, dark **1600×968**, and narrow light
**1100×668**. Their PNG masters are exactly **3200×1936** or **2200×1336** pixels.
The hidden window's viewport can exceed the connected display's workspace.

The app adds `qa_snapshot` to the existing socket-to-IPC bridge. The upstream
`take_screenshot` JPEG/window-capture path is not used; no plugin fork or MCP
server is required.

`qa_snapshot` requests a fresh software render of the WKWebView document. It
compensates for WebKit's backing scale _before rasterization_: requested snapshot
width is `CSS width × 2 / devicePixelRatio`. Both 1× and 2× backing scales therefore
produce the required 2× pixels. The resulting bitmap is never enlarged to pass a
size check. Native dimensions, requested viewport, profile, hidden/unfocused state,
and decoded PNG dimensions must agree; there is no desktop or JPEG fallback.

The capture profile is `wkwebview-snapshot-2x-png-v1`. The former desktop-crop
profile is incompatible even where dimensions match. Manifest/profile checks
prevent silent comparisons across capture methods. Lossless `oxipng` optimization
preserves the master's pixels; only the separate review preview is resized/JPEG.

The runner waits for actual DOM state and two frame callbacks. In background QA, a timer-driven frame scheduler runs
application layout callbacks that hidden WebKit would otherwise suspend. Startup also waits for fonts
and assets without requiring a visible first paint. WebKit snapshotting flushes
layout/paint into the new bitmap; the foreground hide/show repaint workaround is
not used. QA CSS disables animation/transition and blinking carets while keeping
focus outlines. Editor checks explicitly position and assert the intended prose.

This captures the app's web content, including HTML dialogs and popovers. Native
menus, title bars, OS file dialogs, global shortcuts, foreground focus appearance, caret blinking, real pointer
hover and foreground activation still need manual checks. GPU-backed video/WebGL
would need separate validation if introduced. OS/WebKit/font changes can alter
rasterization; isolation removes desktop interference, not every source of render
variation. DOM content, dates and generated identifiers can also vary.

`q.diff()` decodes both PNGs and compares RGB pixels deterministically. A pixel
counts as different if any channel differs by **more than 24**. A fraction of
**more than 0.004** flags a change. The same arrays and settings give the same
answer without model calls/tokens. `--calibrate` additionally takes a second PNG
and compares it at zero tolerance and zero allowance; this checks repeatability
without accepting baselines. A nonidentical repeat is inconclusive.

## Artifacts and baseline review

Each run writes `qa/visual/results/<timestamp>-socket/`:

- `report.md` and `results.json`: verdicts, assertions, errors, Press/axe/overflow
  audits, comparisons, suite completion, elapsed time and cleanup.
- `<checkpoint>-<variant>-2x.png`: lossless comparison master.
- `*-preview.jpg`: 800-pixel-wide review preview, never compared or promoted.
- `*.capture.json`: engine/profile, CSS viewport, backing DPR, scale, pixel size,
  and hidden/focus state. No desktop coordinates or display-mode recovery state.
- `*-FAIL.png`, `.html`, `.json`: failure and ownership evidence.
- `*-repeat.png`: exact repeatability evidence with `--calibrate`.

Inspect changed, new or incomparable images against their Expect text. Review
overflow candidates and moderate axe findings too. Serious/critical axe, console
errors and structural failures remain findings regardless of pixels. The runner
also rebuilds the local history index.

`baselines/manifest.json` records reviewed PNGs, dimensions, SHA-256, Expect text,
source run and review reason. Manifest integrity is checked before fixture work;
accepted checkpoints cannot silently disappear from selected suites. A routine
run never rewrites baselines. On an intentional update, inspect each affected
image, copy its exact PNG master and update that manifest entry. Never promote
previews, upscale images or bulk-accept uninspected captures.

```bash
cp qa/visual/results/<run>/15-01-project-results-dark-2x.png qa/visual/baselines/
shasum -a 256 qa/visual/baselines/15-01-project-results-dark-2x.png
# Update the reviewed entry in manifest.json, then:
npm run qa:baselines:check
npm run qa:visual -- --only 15 --variants dark
```

## Cleanup and recovery

The runner records each returned creation ID before asynchronous frontend work.
It deletes only owned fixture IDs, after each suite and in final cleanup. It
restores local/live panel preferences, theme, no-motion CSS and viewport size,
leaving the hidden app at its empty start screen. Sample author details remain
unsaved drafts. SIGINT/SIGTERM/SIGHUP request orderly cleanup at the next safe
checkpoint. There are no desktop settings to restore.

Ownership survives a webview reload in sessionStorage; a new run refuses to
replace an unfinished record. Keep the QA process alive, inspect its failed
report, reload the harness if needed, then invoke `q.cleanupFixtures()` through
the socket. Wait for `q.last.op === 'cleanup' && q.last.status !== 'pending'` and
verify `status === 'ok'` before calling `q.restoreLocal(); q.restoreTheme()`.
Do not overlap cleanup calls. Do not match projects by name/source to claim
ownership. Do not empty a database or reset the data directory to bypass guards.

A force-quit, logout or socket loss may prevent cleanup. The run lock records its
PID; remove `/tmp/kindling-qa.sock.run.lock` only after verifying that PID has stopped and owned
fixture recovery is complete. The launcher has a separate `/tmp/kindling-qa.sock.start.lock` with
the same stopped-PID rule. A dead process's exact owned IDs may be recovered from
its result artifacts; leave any project without ownership evidence for inspection.

## Layout

- `start.mjs`: hidden process launcher, existing Vite reuse and child cleanup.
- `socket.mjs`: authenticated direct socket client and async IPC bridge.
- `run.mjs`, `suites.mjs`: CLI, fixture lifecycle, matrix, assertions and reports.
- `capture.mjs`: socket snapshot, PNG validation, lossless optimization and preview.
- `src-tauri/src/qa.rs`, `qa_snapshot.m`: debug-only IPC and native WebKit rendering.
- `lock.mjs`: exclusive driver/launcher ownership.
- `harness.js`: DOM actions, fixture ownership, audits and pixel arithmetic.
- `baselines.mjs`: manifest integrity/profile and missing-checkpoint guards.
- `*.test.mjs`: protocol, ownership, arithmetic, capture and concurrent-run tests.
- `scenarios/`: manual behavioral checklists, including v1.3 extensions 15–21.
- `baselines/`: reviewed PNG masters; historical JPEGs use separate names.
- `results/`, `data/`: ignored evidence and isolated app data.

## Harness rules and manual extensions

The numbered legacy scenarios contain JavaScript snippets and historical MCP
call counts. They are manual checklists, not executable input to `run.mjs`.
Unsupported `--only` IDs fail explicitly. Use `SocketClient` and `Runner` when
porting them; do not silently claim those scenarios ran with the default suite.
No MCP connection is required for a custom driver:

```js
import { SocketClient } from "./qa/visual/socket.mjs";
const app = await new SocketClient().connect();
try {
  const state = await app.evaluate("return window.__qa?.state();");
  console.log(state);
} finally {
  app.close();
}
```

- `execute_js` does not await promises. Use `app.invoke(command, args)` or launch
  harness work and wait for its unique marker/status with `app.wait(expression)`.
  Check errors after completion; a marker alone does not mean success.
- Use DOM test IDs or accessible labels, scoped to the owning dialog/panel.
  Svelte delegated handlers require bubbling events or the owning button's `.click()`.
  Never estimate OS coordinates from screenshots. Use `q.drag`/`q.hover` for
  existing DOM drag scenarios; native keyboard/drag checks remain separate.
- Chapter title clicks toggle expansion. Guard them before clicking; wait for
  scene rows, then the selected scene's asynchronously loaded beats.
- Menus render on the next tick. Wait for them before selecting an item. Several
  dialogs share Close labels; scope them or use their specific test IDs.
- Hidden editorial workspace markup remains mounted. Select its non-hidden
  state when testing visibility, and never treat hidden text as visible evidence.
- Do not edit the harness or app during a run: Vite reload destroys running
  chains. `vite.config.ts` excludes generated QA artifacts/data from watching.
- `q.setTheme` changes the rendered attribute and forces repaint; scenario 09 also
  exercises real theme radios. Restore local preferences and the original theme.
- Imports use `q.importFixture(path, format)` for Plottr, Markdown, yWriter,
  Longform, Scrivener and novelWriter. Wait for `#qa-done-fixture-created`, then
  expected UI content. File pickers/reference classification are not exercised.
- Never edit `test-data/` for sync tests. Copy fixtures into a run/temp directory.

`q.audit()` checks Inter/Newsreader/Fraunces/monospace, effective contrast (4.5:1;
3:1 at 24 CSS px or 18.66 CSS px bold), prose measure and token color candidates.
Font, contrast and measure findings require review; untokened colors are leads.
The contrast approximation cannot fully model gradients/images/translucency.
`q.axe()` runs WCAG 2 A/AA. `q.overflow()` reports document overflow and candidates
that may be intentional ellipsis or clipped controls. `q.takeErrors()` drains
console errors, window errors and unhandled rejections. `q.diff()` uses channel
tolerance 24 and a 0.4% differing-pixel threshold; it reports differing/total pixel
counts and bounding boxes, and checks capture existence before baseline availability.
For each decoded pixel, a difference greater than 24 in any RGB channel counts
once; a differing fraction greater than 0.004 flags a change. Both inequalities
are strict. The same decoded pixel arrays and settings always produce the same
result, using local JavaScript arithmetic with no model calls or tokens.
The runner explicitly selects PNG baselines; legacy `q.diff()` callers retain
their JPEG default. Thresholds remain unchanged: exact repeated captures do not
eliminate dynamic dates/IDs or OS/font differences across separate runs.

## Adding coverage

Add executable steps with unique numbered IDs, an explicit Expect description,
and meaningful assertions to `suites.mjs`. Wait for async state before asserting.
Exercise failures/populated states as well as empty dialogs. All checkpoints get
the standard variant matrix; add an explicit dark+narrow combination if a
particular surface needs it. Update `COVERAGE.md`, the related scenario and the
command's guidance. Accept baselines only after visual inspection.

## Startup overlay regression

Run `npm run build && npm run test:startup` with Playwright Chromium installed
(`npx playwright install chromium`). An existing Chrome binary can be selected
with `PLAYWRIGHT_CHROMIUM_EXECUTABLE`.

This production-asset test withholds the app bundle and initial project response,
checking first paint, overlay stacking, disabled background interaction,
light/dark/system themes, reduced motion, ready-state handoff and failed-bundle
retry. It uses a separate browser context and minimal IPC fixture. Captures go to
`qa/visual/results/startup/`.

Also check a native cold launch: the first visible window contains the loading
overlay, then reveals the start screen/opened document. Reload must not steal
focus. Browser animation frames cannot prove pixels reached the physical display.
