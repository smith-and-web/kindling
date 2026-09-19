# 05 Re-import (sync) from source

Mirrors `e2e/specs/reimport.spec.js` (feature #40). Budget: 2 screenshots,
about 7 tool calls.

Precondition: editor view, project imported from the fixture (has a source
path, so the sync button exists). No fresh import.

## Call 1: sync preview (screenshot 05-02)

```js
const q = window.__qa;
q.key("Escape");
const hasSync = !!document.querySelector('[data-testid="sync-button"]');
q.click("sync-button");
q.shot("05-02-sync-preview");
JSON.stringify({ hasSync, ...q.preflight() });
```

`wait_for` `[data-testid="sync-preview-dialog"]` (timeout 10000), then
`take_screenshot`.

e2e: _should show sync button for imported projects_, _should show sync
preview dialog after clicking sync_, _should show 'All synced' when no
changes detected_, _should show sync preview dialog with confirm or
all-synced state_. The case for a project without `source_path` is
unimplemented in e2e too; record _not covered_.

Assert in the next call: `hasSync === true`; dialog text matches
`/(All synced|New Items|Changes)/i`; either `[data-testid="sync-confirm"]`
exists or the text says "All synced".

**Expect**: a centred dialog with a Fraunces title, a close (X) control at
top right, and either an "All synced" state with a check icon or grouped
lists of New Items / Changes with checkboxes. No loading spinner left behind.
The sync button itself sits in the sidebar header beside More actions,
same size as its neighbours.

## Call 2: prose survives a sync (DOM only, 05-04)

```js
const q = window.__qa;
const d = document.querySelector('[data-testid="sync-preview-dialog"]');
window.__qaSync = {
  text: d?.textContent.trim().slice(0, 200),
  confirm: !!document.querySelector('[data-testid="sync-confirm"]'),
};
window.__qaProse2 = "User prose that must be preserved - QA sync " + Date.now();
q.run([
  ["05-04-close", () => q.click("sync-dialog-close"), 400],
  ["05-04-act1", () => q.click("chapter-title", "Act 1"), 700],
  ["05-04-scene", () => q.click("scene-title", q.state().scenes[0]), 700],
  ["05-04-beat", () => q.clickNth("beat-header", 0), 400],
  ["05-04-typed", () => q.setProse(window.__qaProse2), 2400],
  ["05-04-escaped", () => q.key("Escape")],
  ["05-04-sync", () => q.click("sync-button"), 2500],
  ["05-04-closed", () => q.click("sync-dialog-close"), 400],
  ["05-04-reopen", () => q.clickNth("beat-header", 0), 400],
  [
    "05-04-prose",
    () => q.mark("05-04-prose-text", { prose: q.getProse(), expected: window.__qaProse2 }),
    0,
  ],
]);
```

`wait_for` text `preserved - QA sync` (timeout 15000).

Asserts from the log: `05-04-typed.saving === true` at some point is not
required (debounce), but `05-04-prose-text.prose === expected` is the critical
assertion. `05-04-sync.dialogs` should include `sync-preview-dialog`.

e2e: _should preserve user-written prose after sync_ (critical).

## Call 3: read the log and the preview evidence

```js
JSON.stringify({ sync: window.__qaSync, ...window.__qa.flush() });
```

## Not covered

e2e _Content Updates_ (three `it.skip` cases) needs a
`test-data/simple-story-v2.pltr` fixture that does not exist. Do not modify
`test-data/`.
