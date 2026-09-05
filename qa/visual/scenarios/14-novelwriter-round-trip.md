# 14 novelWriter round trip and prose review

Mirrors `e2e/specs/novelwriter.spec.js`, with visual checks for export options,
long before/after text, independent selection, dark theme and narrow width.
Run `/visual-qa 14` separately. Budget: 5 screenshots, about 35 tool calls,
4–5 minutes. Baselines are added only after a judged native run.

Precondition: scratch database and harness installed. This uses a fresh owned
Longform import as the seed, so it does not depend on scenario 13. It exports
to a new temporary folder and edits only that export. The Kindling test bridge
bypasses native pickers; opening the export in novelWriter itself remains a
separate compatibility check.

For every `q.invoke`, wait for `#qa-done-invoke` attached and check
`q.last.status === "ok"` before using `q.last.result` or continuing. An error
also sets the marker: completion is not evidence of success.

## 1. Prepare an isolated round trip

Bash:

```bash
mktemp -d /tmp/kindling-qa-novelwriter.XXXXXX
```

Record the printed absolute path as `<scratch folder>` for this scenario only.

```js
const q = window.__qa;
q.importFixture("<repo>/qa/visual/fixtures/longform/index.md", "longform");
```

Wait for `#qa-done-fixture-created`, then `Harbor Chapter`. Expand the chapter
if needed, select `Arrival`, and wait for `[data-testid="beat-header"]`.

```js
const q = window.__qa;
window.__qaNW = {
  folder: "<scratch folder>",
  seedId: q.fixtureProject().id,
  local:
    "Mara watched the boats drift beneath the harbor bell. " +
    "She remembered every window along the quay, but none of the faces. ".repeat(6),
  retained: "She turned toward the empty pier.",
};
q.run([
  ["14-seed-open-first", () => q.clickNth("beat-header", 0), 400],
  ["14-seed-first", () => q.setProse(window.__qaNW.local), 2400],
  ["14-seed-open-second", () => q.clickNth("beat-header", 1), 400],
  ["14-seed-second", () => q.setProse(window.__qaNW.retained), 2400],
  [
    "14-seed-ready",
    () => {
      q.key("Escape");
      q.done("14-seed");
    },
    0,
  ],
]);
```

Wait for `#qa-done-14-seed`; verify no save is pending before exporting.

## 2. Export options (14-01)

Open `more-actions-button`, then in the next call `export-button`. Wait for
`#export-dialog-title`, then:

```js
const q = window.__qa;
q.click("export-format-novelwriter");
q.shot("14-01-novelwriter-export");
```

Wait for `#qa-settled-14-01-novelwriter-export`, then screenshot. In the next
call inspect the controls after Svelte has updated:

```js
const q = window.__qa;
q.mark("14-export-options", {
  beats: q.find("novelwriter-beat-comments").checked,
  notes: q.find("novelwriter-notes").checked,
  disabled: q.find("export-confirm").disabled,
  hasHelp: document.body.textContent.includes("disables beat-level sync"),
});
q.click("export-close");
q.invoke("export_to_novelwriter", {
  projectId: window.__qaNW.seedId,
  outputPath: window.__qaNW.folder,
  options: { include_beat_comments: true, include_notes: true, create_snapshot: false },
});
```

**Assert**: all four options assertions are true. After the invoke gate, check
`files_created > 1`. **Expect**: the selected novelWriter tile, both checked
options, readable beat-sync help and an empty destination field, with a
visibly disabled Export action. No clipped footer. The actual disk export
uses IPC here; this does not claim coverage of the native folder picker.

## 3. Re-import and establish a no-change baseline

```js
const q = window.__qa;
q.importFixture(window.__qaNW.folder, "novelwriter");
```

Wait for the new creation marker and `Harbor Chapter`, then:

```js
const q = window.__qa;
window.__qaNW.projectId = q.fixtureProject().id;
q.invoke("get_sync_preview", { projectId: window.__qaNW.projectId });
```

After the invoke gate, assert zero additions and zero changes. Expand Harbor
Chapter if needed, select Arrival and wait for two beats. Check Mara and Old
Harbor are still present in their References tabs. Check the synopsis survived
and Sync is visible. These are DOM checks; scenario 13 captures the workspace.

## 4. Edit only the exported source (Bash)

The header checksum deliberately remains stale, like an external editor that
has not regenerated it. Stop if the expected document or comments are missing.

```bash
python3 - <<'PY'
from pathlib import Path
folder = Path("<scratch folder>")
files = [p for p in (folder / "content").glob("*.md") if "### Arrival\n" in p.read_text()]
assert len(files) == 1, "Expected exactly one Arrival document"
path = files[0]
text = path.read_text()
first = "% Beat: Mara recognizes the harbor."
second = "% Beat: A stranger calls her name."
assert text.count(first) == text.count(second) == 1, "Unexpected beat boundaries"
prefix = text.split(first, 1)[0]
prose = ("The harbor bell fell silent when Mara stepped onto the quay. "
         + "A stranger waited beside the last boat, holding a letter she had never sent. " * 6)
path.write_text(prefix + first + "\n" + prose + "\n\n" + second
                + "\nThis incoming sentence must remain unaccepted.\n")
print(path)
PY
```

## 5. Review long prose (14-02)

Click `sync-button`, wait for `[data-testid="sync-preview-dialog"]`, then:

```js
const q = window.__qa;
q.shot("14-02-novelwriter-prose-review");
q.axe(document.querySelector('[data-testid="sync-preview-dialog"]'));
JSON.stringify({
  rows: document.querySelectorAll('[data-testid="sync-prose-diff"]').length,
  selected: document.querySelectorAll('[data-testid="sync-change-checkbox"]:checked').length,
  disabled: q.find("sync-confirm").disabled,
  audit: q.audit(),
});
```

Wait for the screenshot's settle marker, capture, then wait for `#qa-axe-done`
and record `q.axeResult`. **Assert**: two prose rows, zero selected changes,
Apply Sync disabled. **Expect**: clearly labeled Current/Incoming paragraphs,
readable Newsreader text, retained paragraph wrapping, independently reachable
checkboxes and internal scrolling. Scroll the list to inspect the second row;
long prose must not be reduced to a truncated single line.

## 6. Narrow and dark prose review (14-03, 14-04)

This is a new reading surface that the existing sweeps do not cover. Keep the
same diff open and selections unchanged:

1. `manage_window set_size` 1100 x 700. Call `q.shot("14-03-novelwriter-narrow")`,
   wait for its settle marker, capture and record `q.overflow()` and `q.audit()`.
2. Restore 1600 x 1000. Call `q.setTheme("dark")`, wait for
   `#qa-settled-theme`, then `q.shot("14-04-novelwriter-dark")`, wait for its
   settle marker, capture and record `q.audit()`.
3. Restore the original theme with `q.restoreTheme()` and wait for
   `#qa-settled-theme` before continuing.

**Expect**: both prose columns wrap within the dialog; the footer and checkbox
controls remain accessible at narrow width. In dark mode, body text has clear
contrast against the dialog surfaces, and labels, selected controls and the
scrim all follow the theme. Record any clipping or horizontal overflow.

## 7. Accept one change, leave the other untouched (14-05)

```js
const q = window.__qa;
const first = document.querySelector('[data-testid="sync-prose-diff"]');
first.closest("label").querySelector('[data-testid="sync-change-checkbox"]').click();
```

Next call: assert exactly one checked `sync-change-checkbox`, then click
`sync-confirm`. Wait for `[data-testid="sync-summary-dialog"]`. Assert the
summary says one prose item updated and one preserved, then click
`dialog-close`.

Select Arrival if necessary; wait for its beats, then:

```js
const q = window.__qa;
q.run([
  ["14-retained-open", () => q.clickNth("beat-header", 1), 400],
  [
    "14-retained-check",
    () =>
      q.mark("14-retained", {
        prose: q.getProse(),
        expected: window.__qaNW.retained,
      }),
    0,
  ],
  ["14-updated-open", () => q.clickNth("beat-header", 0), 400],
  [
    "14-updated-check",
    () =>
      q.mark("14-updated", {
        prose: q.getProse(),
        expectedStart: "The harbor bell fell silent",
      }),
    0,
  ],
  [
    "14-updated-shot",
    () => {
      q.shot("14-05-novelwriter-updated-editor");
      q.done("14");
    },
    0,
  ],
]);
```

Wait for `#qa-done-14` and `#qa-settled-14-05-novelwriter-updated-editor`, then
capture. **Assert**: the second beat still equals `retained`; the first starts
with `expectedStart`. **Expect**: the accepted prose appears in the normal
editor, prompts remain intact, and the other beat is still present.

Run `get_sync_preview` again through `q.invoke`: exactly one prose change must
remain, for the unaccepted second beat. Record `q.flush()` and `q.takeErrors()`.

## Cleanup

Close any dialog, restore the original theme and 1600 x 1000 window. Restore
Simple Story before the usual sweeps (see scenario 13). At final cleanup,
`q.cleanupFixtures()` deletes the seed and novelWriter imports by their captured
IDs. Wait for successful cleanup before removing the exact temporary folder
created in step 1. Do not delete other `/tmp` projects or change checked-in
fixtures. A failed cleanup retains ownership for retry.
