# 04 Delete chapters and scenes

Mirrors `e2e/specs/delete-content.spec.js` (feature #16). Budget: 3
screenshots, about 9 tool calls.

Precondition: editor view, no dialogs open. No fresh import; the chapter list
is whatever scenarios 01 to 03 left behind, which is fine because every
assertion is relative.

## Call 1: context menu on the last chapter (screenshot 04-03a)

```js
const q = window.__qa;
q.key("Escape");
const c = q.state().chapters;
window.__qaDel = c.at(-1);
q.openMenu("chapter-item", window.__qaDel);
q.shot("04-03a-context-menu");
q.preflight();
```

Then `take_screenshot`.

e2e: _should show menu button on chapter hover_ (the menu button was clicked,
so it rendered).

**Expect**: context menu anchored beside the row's 3-dot button, above other
content, items in Inter, Delete styled as destructive and separated from safe
actions.

## Call 2: confirm dialog (screenshot 04-03b)

```js
const q = window.__qa;
q.menuItem("Delete");
q.shot("04-03b-confirm-chapter");
q.preflight();
```

Then `take_screenshot`.

e2e: _should show confirmation dialog with content counts for chapter_

**Expect**: centred modal over a dimmed backdrop, Fraunces title, message
quoting the scene and beat counts, Cancel as secondary and Delete as the
danger primary, both fully inside the dialog.

## Call 3: cancel, confirm, scene delete, selected-scene delete (DOM only, 04-04 to 04-08)

```js
const q = window.__qa;
q.run([
  [
    "04-03-message",
    () =>
      q.mark("04-03-message-text", {
        message: document.querySelector('[data-testid="dialog-message"]')?.textContent.trim(),
      }),
    0,
  ],
  ["04-05-cancelled", () => q.click("dialog-cancel"), 300],
  ["04-06-menu", () => q.openMenu("chapter-item", window.__qaDel), 300],
  ["04-06-delete", () => q.menuItem("Delete"), 300],
  ["04-06-confirmed", () => q.click("dialog-confirm"), 900],
  ["04-07-act1", () => q.click("chapter-title", "Act 1"), 700],
  [
    "04-04-menu",
    () => {
      window.__qaScene = q.state().scenes.at(-1);
      q.openMenu("scene-item", window.__qaScene);
    },
    300,
  ],
  ["04-04-delete", () => q.menuItem("Delete"), 300],
  [
    "04-04-message",
    () =>
      q.mark("04-04-message-text", {
        message: document.querySelector('[data-testid="dialog-message"]')?.textContent.trim(),
      }),
    0,
  ],
  ["04-07-confirmed", () => q.click("dialog-confirm"), 900],
  ["04-08-select", () => q.click("scene-title", q.state().scenes[0]), 700],
  ["04-08-menu", () => q.openMenu("scene-item", q.state().selected), 300],
  ["04-08-delete", () => q.menuItem("Delete"), 300],
  ["04-08-confirmed", () => q.click("dialog-confirm"), 900],
  ["04-08-shot", () => q.shot("04-08-empty-state"), 0],
]);
```

`wait_for` `[data-testid="empty-state"]` (timeout 12000), then `take_screenshot`.

Asserts from the log:

- `04-03-message-text.message` matches `/\d+ scenes?/` and `/\d+ beats?/`;
  `04-04-message-text.message` matches `/\d+ beats?/`.
- `04-05-cancelled.chapters` still contains `window.__qaDel`, `dialogs` empty.
- `04-06-confirmed.chapters` lacks `window.__qaDel` and is one shorter.
- `04-07-confirmed.scenes` is one shorter than `04-07-act1.scenes`.
- `04-08-confirmed.empty === true`, `selected === null`.

e2e: _should show confirmation dialog for scene with beat count_, _should
close dialog without deleting when clicking Cancel_, _should delete chapter
when confirming_, _should delete scene when confirming_, _should clear
selection if deleted item was selected_.

**Expect** (04-08): the scene panel shows the empty/welcome state centred,
with no leftover beat cards or header from the deleted scene; the sidebar
closed the gap where the scene was; no backdrop residue.

## Call 4: read the log

```js
JSON.stringify(window.__qa.flush());
```

## Call 5: deletion survives reload (04-09, 3 calls)

e2e: _should persist deletion after page refresh_

- `navigate reload`, then re-run the harness loader snippet from the README
  (page state is gone).
- Next call: if `q.state().view === "start"`, `q.click("project-card")`; then
  `wait_for` text `Act 1`.
- Next call: `JSON.stringify({ chapters: window.__qa.state().chapters, deleted: window.__qaDel })`
  (`__qaDel` is gone after reload; use the value you recorded from the log).
  Assert the deleted chapter is absent. No screenshot unless it fails.
