# 08 References panel

No e2e counterpart. Creates a character through the References panel and
checks the tab count. Budget: 2 screenshots, about 6 tool calls.

Precondition: editor view with a scene selected (the panel is always
visible; the Characters tab is active by default).

## Call 1: open the add dialog (screenshot 08-01)

```js
const q = window.__qa;
q.click("add-reference-button");
q.shot("08-01-reference-dialog");
return JSON.stringify(q.preflight());
```

`wait_for` selector `[aria-labelledby="reference-dialog-title"]`, then screenshot.

**Expect**: a centred dialog for a new Character with Name, optional
Description and Notes, an attributes section with an add control, Cancel and
a disabled Save until a name is entered.

## Call 2: create, then verify (DOM only, 08-02)

```js
const q = window.__qa;
return q.run([
  ["08-02-fill", () => q.fillPlaceholder("Enter name...", "QA Character")],
  ["08-02-save", () => q.click("reference-save"), 1200],
  [
    "08-02-listed",
    () =>
      q.mark("08-02-panel", {
        tab: [...document.querySelectorAll("button")]
          .find((b) => /^Characters/.test(b.textContent.trim()))
          ?.textContent.trim(),
        listed: !![...document.querySelectorAll("aside")]
          .find((a) => a.className.includes("border-l"))
          ?.textContent.includes("QA Character"),
      }),
    0,
  ],
  [
    "08-02-shot",
    () => {
      q.shot("08-02-reference-listed");
      q.done("08");
    },
    0,
  ],
]);
```

The dialog footer is Cancel and Save (verified 2026-09-05).

`wait_for` `#qa-done-08` attached, then screenshot.

Asserts: `08-02-panel.tab` is `Characters (1)` (or one more than before) and
`listed === true`.

**Expect**: the new character as a row in the References panel with its icon,
name, and hover actions (edit, delete); the Characters tab count incremented;
no dialog residue.

## Call 3: read the log

```js
JSON.stringify(window.__qa.flush());
```

Cleanup is implicit: the fixture project is deleted at the end of the run.
