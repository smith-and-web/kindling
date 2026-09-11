# 98 Narrow window sweep

Legacy supplemental checklist. The socket runner now repeats every executable
checkpoint at 1100×700; these three screens alone are not complete narrow
coverage. Also run the new 15–21 checklists for manual surfaces and include
selective-sync/prose-diff and destructive confirmations.

Precondition: editor view with the fixture, a scene selected.

## Call 1: resize and audit the editor

`manage_window set_size` width 1100 height 700, then

```js
const q = window.__qa;
q.click("scene-title", "The Beginning");
q.shot("98-01-narrow-editor");
return JSON.stringify(q.overflow());
```

`wait_for` `#qa-settled-98-01-narrow-editor`, then screenshot.

Asserts: `documentOverflow === false`; `offenders` empty or only elements
that are intentionally scrollable.

**Expect**: sidebar, scene panel and References panel all fit; the References
panel may collapse or narrow, the scene title wraps rather than clips, the
select controls stay on one row or wrap cleanly.

## Call 2: a dialog at narrow width

```js
const q = window.__qa;
q.click("more-actions-button");
return "more";
```

Next call:

```js
const q = window.__qa;
q.click("export-button");
q.shot("98-02-narrow-export");
return JSON.stringify(q.overflow());
```

`wait_for` `#qa-settled-98-02-narrow-export`, then screenshot, then
`q.click("export-close")`.

**Expect**: the dialog fits inside the viewport with its footer visible,
scrolling internally if needed; the format grid reflows to fewer columns.

## Call 3: start screen

```js
const q = window.__qa;
q.clickSel('[aria-label="Close project"]');
q.shot("98-03-narrow-start");
return "start";
```

`wait_for` `#qa-settled-98-03-narrow-start`, then screenshot, then

```js
const q = window.__qa;
const o = q.overflow();
q.click("project-card", "Simple Story");
return JSON.stringify(o);
```

`wait_for` text `Act 1`.

**Expect**: import grid and recent projects stack or shrink without
horizontal scrolling; project cards truncate long names with an ellipsis.

## Call 4: restore the window

`manage_window set_size` 1600 x 1000.
