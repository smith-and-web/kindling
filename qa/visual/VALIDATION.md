# Background visual QA validation — 2026-09-11

## Baseline refresh and preflight context — 2026-09-12 UTC

The accepted set now contains **147 reviewed PNG masters**, covering all 14
executable suites in light, dark and narrow (42 suite executions). The refresh
accepts the existing Home/sidebar relocation and export redesign and adds the
nine newer export checkpoints in all three variants. Every selected image was
inspected against Expect, including the corrected narrow custom-profile captures.
Source runs, checksums, dimensions, capture profile, revision and review reasons
are recorded in the manifest. Known References findings were preserved.

A fresh full comparison completed in **281 seconds (4m41s)**:

- **147/147 images matched**, zero changed, missing-baseline or incomplete images.
- **144 checkpoints passed; 3 failed** the existing critical References
  `aria-required-parent` audit. Exit **1** remains correct; image acceptance does
  not waive an accessibility finding.
- All 42 suite executions completed, owned fixtures were deleted, and preferences
  restored. No assertions, console errors, Press font/contrast/measure failures,
  document overflow, run errors or cleanup failures occurred.
- Overflow candidates are hidden collapsed-sidebar content and screen-reader-only
  export labels, previously visually reviewed. All captures remained hidden,
  unfocused and rendered at 2×.

[Full comparison](results/2026-09-12T00-29-36-037Z-socket/report.md) ·
[Preflight context](results/2026-09-12T00-29-36-037Z-socket/context.md) ·
[Source visual review](results/2026-09-11T21-54-41-332Z-socket/review.md).

The runner now collects Git and baseline context before connecting to the app:
revision/branch, commits and changed files since acceptance, working-tree changes,
selected baseline counts, optional operator notes and recorded findings. Unknown
provenance is reported explicitly. Before this refresh, the preflight correctly
identified suites 19, 22 and 23 as having no references in any variant. Context is
advisory and does not alter image thresholds or checkpoint verdicts. Reports now
show image-match and checkpoint counts separately. **32 harness tests pass**,
including real temporary Git repositories for change/provenance coverage.

Native/manual release checks remain separate and were not rerun for this update.
The historical migration/calibration evidence below describes the earlier
120-image set.

The socket runner now captures a hidden, isolated WKWebView as lossless 2× PNGs.
It was validated while macOS was **locked**, with the normal Kindling debug app
still running separately. No desktop screenshots, focus, pointer movement,
display-mode changes or DND changes were used.

## Original baselines and normal verification

**120 reviewed PNG masters** (23.7 MiB) use capture profile
`wkwebview-snapshot-2x-png-v1`. Their dimensions, SHA-256, Expect text, review
source and known findings are recorded in [the manifest](baselines/manifest.json).
All candidates were inspected against Expect; byte-identical PNGs reused a
previous visual review verified by checksum. Scrolled narrow states deliberately
show the controls/result named in Expect, not every part of the surrounding form.

A fresh hidden app process ran `npm run qa:visual` in **235 seconds (3m55s)**.
All 33 suite executions completed:

- **120/120 image comparisons matched**; 108 were exact.
- The other 12 contain changing project IDs or dates/times. The largest differing
  fraction was 0.1441%, below the unchanged 0.4% allowance at channel tolerance 24.
- **117 pass, 3 visual regression** from the existing References accessibility
  issue below. Exit **1** accurately reports those failures.
- No missing images, assertions, console errors, run exceptions or cleanup failures.
- All captures report `windowVisible:false`, `windowFocused:false`, and 2× pixels.

[Normal report](results/2026-09-11T02-57-50-579Z-socket/report.md) ·
[Full evidence](results/2026-09-11T02-57-50-579Z-socket/results.json).

The candidate run used `--calibrate`: **240 captures, all 120 repeat pairs exactly
identical**, in **379 seconds (6m19s)**. That run intentionally treated the former
capture profile as incompatible instead of comparing against desktop crops.
[Calibration report](results/2026-09-11T02-45-01-566Z-socket/report.md).

The former desktop PNG masters and manifest were archived before migration in
`results/legacy-desktop-baselines-2026-09-11/`. That historical process took
405 seconds per normal pass and required an unobstructed desktop; it is no longer
the default. The display wrapper and Swift display-switching helper were removed.

## Native isolation and recovery checks

`node qa/visual/isolation.mjs` provides a repeatable native probe. On this machine,
a **2400×1600 CSS canvas** rendered into **4800×3200 PNGs** while hidden, unfocused,
at **1× backing DPR**, and with macOS locked. Consecutive red/blue renders decoded
to the exact requested RGBA values, proving fresh content rather than a stale
window-server image. Invalid sizes and scale were rejected; the probe removed
its temporary element and restored the viewport.
[Probe evidence](results/2026-09-11T02-44-36-460Z-isolation/results.json).

A live SIGINT delivered during the review-package checkpoint stopped the run,
closed the editorial workspace and its underlying project, deleted owned
fixtures, and restored preferences/viewport. Exit 1 preserved the interruption
as a failure. [Interruption evidence](results/socket-probe/interrupt.json).

The launcher successfully reused this checkout's existing Vite server and later
reused an already-running QA app. A separate socket, app identifier, settings
directory, WebKit datastore and database isolate it from the normal app. Socket
authentication rejects missing tokens; the token file is owner-only (0600).
Starting background mode without an explicit data directory fails before opening
the normal app. [Auth](results/socket-probe/auth.json) ·
[Invalid profile](results/socket-probe/fail-closed.json).

The original four demo project IDs remain intact in their separate database.
[Data preservation](results/socket-probe/data-preservation.json). Fixture ownership
is recorded by returned ID; no projects were adopted by name or source path.

## Fixes found during implementation

Hidden WebKit suspends native animation-frame callbacks. This initially left the
manuscript menu offscreen despite identical repeat screenshots. Debug-only QA
frame scheduling now runs layout callbacks in timer-driven batches, with normal
cancellation and next-frame semantics. The menu checkpoint explicitly asserts
its viewport bounds. The clipped candidates were not accepted.

Interruption in editorial mode initially closed only the workspace, leaving the
project open. Final cleanup now closes both. The captured ownership record was
used to recover that one remaining fixture, and the corrected live SIGINT test
passed. An early socket-response timeout also preserved failure evidence and
cleaned up; IPC now allows 20 seconds for transfer with command/phase diagnostics.
The final complete runs had no timeouts.

## Automated checks

- QA harness/client/capture/manifest tests: **30 passed**.
- Frontend startup, test bridge and background frame scheduler: **11 passed**.
- Rust snapshot allocation/cancellation tests: **2 passed**.
- Rust Clippy (`--all-targets --all-features -D warnings`) and release check passed.
- Svelte check, ESLint and production build passed, with existing warnings only.
- Production startup browser checks passed for five theme/motion combinations,
  document-open keyboard focus and failed-bundle recovery. The test now uses
  `ControlOrMeta+f` so macOS Chrome and Linux Chromium exercise the same action.
  This machine used `PLAYWRIGHT_CHROMIUM_EXECUTABLE` pointing to installed Chrome;
  the default Playwright browser download was absent.
- Production assets contain no background-QA marker or frame-scheduler override.
- Baseline integrity and socket dry-run passed for the new capture profile.
- The shared `$visual-qa` skill was updated and its skill validator passed.

## Remaining findings and scope

The populated References list still has a **critical `aria-required-parent`**
finding on `ReferencesPanel.svelte`'s `div[role="listitem"]` without its required
list parent. It affects `08-02-reference-listed` in all three variants. The
baseline entries document it; image acceptance does not waive the failing audit.

This process covers web content. Native menus, OS dialogs, foreground focus
appearance, real key delivery, hover/caret behavior and the other manual checks
in [COVERAGE.md](COVERAGE.md) remain separate. OS/WebKit/font changes may require
reviewed baseline updates. A running macOS login session is required: locking is
supported and was tested; actual logout, shutdown or forced system sleep cannot
keep a GUI process executing.
