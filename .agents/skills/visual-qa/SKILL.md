---
name: visual-qa
description: Run Kindling desktop visual QA through its direct socket runner, inspect regressions, and review isolated 2× PNG baseline updates. Use for Kindling screenshot checks, visual regression passes, and baseline maintenance.
---

# Kindling visual QA

Use the existing Node runner for navigation, fixtures, captures, audits and pixel
comparison. App communication goes directly over the Unix socket; no MCP server,
per-checkpoint agent driving, browser substitute or website capture script is needed.

## Locate and select

Work from the Kindling checkout containing `qa/visual/run.mjs`. When invoked
outside that checkout, resolve this skill's real directory (following symlinks):
the repository is three parents above it. Read `qa/visual/README.md` for current
prerequisites, capture rules and recovery. Read `qa/visual/COVERAGE.md` when
mapping requested features to executable suites or manual checks. The repository
scripts and runbook are authoritative; do not recreate their implementation.

A plain request to run visual QA means `npm run qa:visual`, with all executable
suites and the default light/dark/narrow matrix. Preserve explicit scope:

```bash
npm run qa:visual -- --list
npm run qa:visual -- --only 09,15,17 --variants light
npm run qa:visual -- --only 07 --variants narrow --calibrate
```

Translate requested IDs such as `09 15 17` into `--only 09,15,17`. Unsupported IDs
remain manual/unexecuted unless actually performed. Use `--calibrate` only for
requested repeatability work or diagnosing capture instability, not every pass.
The last normal 120-checkpoint pass took 235 seconds; the 240-capture calibration
pass took 379 seconds. Report measured timing and allow the process to finish.

## Prepare and run

1. Start/reuse the hidden debug app with `npm run tauri:qa`. It has a separate
   authenticated socket `/tmp/kindling-qa.sock`, `background-profile` database,
   settings directory and browser storage. The normal app can stay open.
2. Run `npm run qa:baselines:check` and `npm run qa:visual -- --dry-run`.
   Dry-run checks socket snapshot capabilities/profile without app changes.
   Use `npm run test:qa-harness` when changing/validating QA tooling.
3. Run the selected command once. Keep its process/session ID and poll for
   completion with concise updates. Do not edit app, harness or scenario files
   during capture: Vite reload invalidates in-flight work. Do not overlap drivers;
   the runner's lock prevents concurrent passes against the same app.

Captures use `WKWebView.takeSnapshot` through socket IPC, rasterized at 2× and
encoded as lossless PNG. Do not use desktop `screencapture`, manipulate displays,
focus windows, park the pointer, enable DND, or require Screen Recording. The
user can use the computer normally. macOS 14+ and a running login session are
required; actual logout or forced sleep cannot keep a GUI process running.

The runner handles owned fixtures, deterministic panel preferences, unsaved
sample author drafts, capture, audits, diffs and cleanup. Never clear an existing
library to bypass its empty-profile guard. Normal runs do not accept baselines.

## Evaluate and report

Use the output directory printed by this invocation under `qa/visual/results/`.
Read its `report.md` and summarize `results.json` programmatically before loading
images. Check every requested suite completed and that cleanup succeeded.

- Exit **0**: all requested checkpoint evidence passed.
- Exit **1**: run, assertion, audit or cleanup failure. Read the evidence; it
  does not necessarily mean the PNGs differ.
- Exit **2**: review pending, including new/changed/incomparable images or
  incomplete evidence. A missing baseline is not a pass.

For a routine regression pass, inspect changed/new/incomparable previews against
their Expect text, using PNG masters when detail is needed. Also review overflow
candidates and audit findings. Do not load all unchanged screenshots into model
context unless the user requests a full visual review. Matching pixels cannot
waive accessibility, console, assertion or cleanup failures.

Consult `qa/visual/VALIDATION.md` and the manifest for existing findings. The
recorded References `aria-required-parent` issue affects three variants; report
it if present, but never hardcode an expected failure count or suppress it.

For snapshot/socket failures, preserve artifacts and ownership evidence. Diagnose and correct the cause before a bounded rerun of affected suites.
Follow the runbook's exact-ID ownership recovery; never adopt fixtures by name
or source path. Do not repeatedly rerun unchanged failures to obtain a pass.

Report image-match counts separately from overall checkpoint verdicts, executed
variants/suites, material findings, cleanup, elapsed time and
the report path. State relevant unexecuted manual coverage without implying full
v1.3 behavioral acceptance. `node qa/visual/history.mjs` builds the optional run
index; the runner also refreshes that index.

## Update baselines when requested

A routine QA pass does not rewrite accepted baselines. For an intentional update,
inspect each affected image before copying its lossless `-2x.png` master into
`qa/visual/baselines/`. Never promote previews, upscale images or bulk-accept an
uninspected run. Follow the README to update the entry in `manifest.json`,
including capture profile, checksum, dimensions, Expect text, source run and review reason.
Never mix the legacy desktop-crop and isolated WKWebView capture profiles.
Run `npm run qa:baselines:check`, then a fresh affected-suite comparison. Known
findings remain audit failures even when a visual reference is accepted.
