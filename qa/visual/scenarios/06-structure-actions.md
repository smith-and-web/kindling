# 06 Parts and chapter actions

No e2e counterpart. Covers the sidebar features the WebDriver suite never
touched: parts, rename, duplicate, lock, archive and restore, and the
part-delete dialog. Budget: 4 screenshots, about 10 tool calls.

Precondition: editor view with the fixture, at least chapters `Act 1`,
`Act 2`, `Act 3`, `Test Chapter QA` present (run after 01 to 05).

## Call 1: create a part (screenshot 06-01)

```js
const q = window.__qa;
q.key("Escape");
q.click("new-dropdown-button");
q.click("dropdown-new-part");
return "part input";
```

Next call: `q.fillTitle("Part One QA", "Enter")`, then `wait_for` selector
`[data-testid="part-item"]` (the title is rendered uppercase by CSS, so a text
wait fails), then `q.shot("06-01-part-created")`, `wait_for` its settle marker,
and screenshot.

**Expect**: a `part-item` row above the chapters, visually distinct from a
chapter (heavier or eyebrow typography, no chevron), with the same hover
affordances. The dropdown closed cleanly.

## Call 2: rename, duplicate, lock, archive (DOM only, 06-02 to 06-05)

```js
const q = window.__qa;
return q.run([
  ["06-02-menu", () => q.openMenu("chapter-item", "Test Chapter QA"), 300],
  ["06-02-rename", () => q.menuItem("Rename"), 300],
  [
    "06-02-dialog",
    () => q.mark("06-02-rename-dialog", { open: !!document.getElementById("rename-dialog-title") }),
    0,
  ],
  ["06-02-fill", () => q.fillPlaceholder("Enter name...", "Renamed Chapter QA")],
  ["06-02-save", () => q.click("rename-save"), 900],
  ["06-03-menu", () => q.openMenu("chapter-item", "Renamed Chapter QA"), 300],
  ["06-03-duplicate", () => q.menuItem("Duplicate"), 1200],
  [
    "06-04-act1",
    () => {
      if (q.state().scenes.length === 0) q.click("chapter-title", "Act 1");
    },
    700,
  ],
  ["06-04-menu", () => q.openMenu("scene-item", q.state().scenes[0]), 300],
  ["06-04-lock", () => q.menuItem("Lock"), 700],
  [
    "06-04-locked",
    () =>
      q.mark("06-04-lock-state", {
        lockedTitles: [
          ...document.querySelectorAll(
            '[data-testid="sidebar"] [data-testid="scene-title"].text-press-disabled-text'
          ),
        ].map((e) => e.textContent.trim()),
      }),
    0,
  ],
  ["06-04-menu2", () => q.openMenu("scene-item", q.state().scenes[0]), 300],
  ["06-04-unlock", () => q.menuItem("Unlock"), 700],
  ["06-05-menu", () => q.openMenu("chapter-item", "Renamed Chapter QA"), 300],
  ["06-05-archive", () => q.menuItem("Archive"), 1200],
  ["06-05-done", () => q.done("06a"), 0],
]);
```

`wait_for` `#qa-done-06a` attached (timeout 12000).

Asserts from the log:

- `06-02-rename-dialog.open === true`; `06-02-save.chapters` contains
  `Renamed Chapter QA` and not `Test Chapter QA`.
- `06-03-duplicate.chapters.length === 06-02-save.chapters.length + 1`.
- `06-04-lock-state.lockedTitles` contains the first scene;
  `06-04-unlock` has no locked titles.
- `06-05-archive.chapters` is one shorter than `06-03-duplicate.chapters`.

## Call 3: archive panel and restore (screenshot 06-05)

```js
const q = window.__qa;
q.click("more-actions-button");
return "more";
```

Next call: `q.click("archive-button"); q.shot("06-05-archive-panel"); q.preflight()`.
`wait_for` text `Archive`, then screenshot.

**Expect**: a panel or dialog titled Archive listing `Renamed Chapter QA`
with a Restore action and a delete action per row; empty-state copy absent.

Next call: `q.click("archive-restore")`, then `q.click("archive-close")`
in the following call (several dialogs carry an `aria-label="Close"` button, so
always scope by title id). Assert `Renamed Chapter QA` is back in `chapters`.

## Call 4: part delete dialog (screenshot 06-06)

```js
const q = window.__qa;
q.openMenu("part-item", "Part One QA");
return "menu";
```

Next call: `q.menuItem("Delete"); q.shot("06-06-part-delete-dialog"); q.preflight()`.
`wait_for` the settle marker, then screenshot.

An empty part gets the ordinary `confirm-dialog` ("0 scenes and 0 beats");
only a part that owns chapters gets `part-delete-dialog` with the
`delete-part-only` / `delete-part-and-chapters` choice. Handle both: if
`q.state().dialogs` includes `part-delete-dialog`, click `delete-part-only`,
otherwise `dialog-confirm`. Allow about a second, then assert no `part-item`
remains and the chapter count is unchanged.

**Expect** (confirm): the standard delete dialog naming the part. **Expect**
(part-delete-dialog): two destructive choices, with the more destructive one
visually heavier or separated, plus Cancel.

## Call 5: read the log

```js
JSON.stringify(window.__qa.flush());
```
