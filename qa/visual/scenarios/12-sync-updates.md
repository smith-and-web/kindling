# 12 Sync picks up source changes

Covers the three e2e cases that were always skipped (_should update chapter
titles from source_, _should add new chapters from source_, _should add new
scenes from source_) using the `simple-story-v2.pltr` fixture. Nothing in
`test-data/` is modified: the run imports a temp copy of v1 and then
overwrites that temp copy with v2, which is what a writer editing their
outline in Plottr looks like to Kindling. Budget: 2 screenshots, about 9
tool calls.

v2 differs from v1 in three ways: `Act 3` is renamed `Act 3: Resolution`,
the `Discovery` scene gains a third paragraph (a new beat), and `Act 3` gains
a new scene `Epilogue`.

Precondition: any view. The temp copy is a separate project from the run's
main fixture; `q.cleanupFixtures()` deletes both.

## Call 1: stage the temp copy (Bash)

```bash
cp test-data/simple-story.pltr /tmp/qa-simple-story.pltr && echo staged
```

## Call 2: import it and open Act 3

```js
const q = window.__qa;
q.importFixture("/tmp/qa-simple-story.pltr");
return "importing";
```

`wait_for` text `Act 3`, then

```js
const q = window.__qa;
q.mark("12-01-before", { chapters: q.state().chapters });
return JSON.stringify(q.state().chapters);
```

## Call 3: swap the source for v2 (Bash)

```bash
cp test-data/simple-story-v2.pltr /tmp/qa-simple-story.pltr && echo swapped
```

## Call 4: sync preview shows the changes (screenshot 12-01)

```js
const q = window.__qa;
q.click("sync-button");
return "sync";
```

`wait_for` `[data-testid="sync-preview-dialog"]` (timeout 10000), then

```js
const q = window.__qa;
const d = document.querySelector('[data-testid="sync-preview-dialog"]');
const text = d.textContent.replace(/\s+/g, " ");
q.shot("12-01-sync-preview-changes");
return JSON.stringify({
  hasRename: /Act 3: Resolution/.test(text),
  hasEpilogue: /Epilogue/.test(text),
  hasNewBeat: /new beat added in v2/.test(text),
  confirm: !!document.querySelector('[data-testid="sync-confirm"]'),
});
```

`wait_for` `#qa-settled-12-01-sync-preview-changes`, then screenshot.

Asserts: `hasRename`, `hasEpilogue`, `hasNewBeat` and `confirm` all true.

**Expect**: the preview lists the new scene and beat under New Items (all
selected) and the renamed chapter under Changes with its old and new title
and its checkbox unselected: changes to existing items are opt-in so a
writer's own edits are not overwritten by default. Apply Sync enabled.

## Call 5: apply and verify (DOM only, then screenshot 12-02)

```js
const q = window.__qa;
return q.run([
  [
    "12-02-select-changes",
    () => {
      // Changes to existing items default to unselected (by design, to protect edits); pick them all.
      const alls = [
        ...document.querySelectorAll('[data-testid="sync-preview-dialog"] button'),
      ].filter((b) => b.textContent.trim() === "All");
      alls.at(-1)?.click();
    },
    300,
  ],
  ["12-02-apply", () => q.click("sync-confirm"), 2500],
  [
    "12-02-summary",
    () =>
      q.mark("12-02-summary-dialog", {
        dialogs: q.state().dialogs,
        text: document
          .querySelector('[data-testid="sync-summary-dialog"]')
          ?.textContent.replace(/\s+/g, " ")
          .slice(0, 200),
      }),
    0,
  ],
  [
    "12-02-close",
    () => {
      if (document.querySelector('[data-testid="sync-summary-dialog"]')) q.click("dialog-close");
    },
    600,
  ],
  [
    "12-02-act3",
    () => {
      if (!q.state().scenes.includes("Epilogue")) q.click("chapter-title", "Act 3");
    },
    900,
  ],
  [
    "12-02-shot",
    () => {
      q.shot("12-02-after-sync");
      q.done("12");
    },
    0,
  ],
]);
```

`wait_for` `#qa-settled-12-02-after-sync` (timeout 12000), then screenshot.

Asserts from the log: `12-02-selection.footer` is `4 items selected`;
`12-02-summary-dialog.text` contains `Chapters: 0 added, 1 updated`,
`Scenes: 1 added` and `Beats: 2 added`; `12-02-shot.chapters` includes
`Act 3: Resolution` and not `Act 3`; `12-02-shot.scenes` includes `Epilogue`
(verified 2026-09-05).

**Expect**: sidebar shows the renamed chapter and the new scene under it;
the summary dialog (if still visible in the log) reported 1 change and 2
additions.

## Call 6: read the log

```js
JSON.stringify({ ...window.__qa.flush(), errors: window.__qa.takeErrors() });
```

The temp project is removed by the run's normal `q.cleanupFixtures()`; the
temp file can be left in `/tmp`.
