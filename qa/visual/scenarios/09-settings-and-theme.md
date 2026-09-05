# 09 Settings dialogs and the real theme toggle

No e2e counterpart. Exercises the Project Settings dialog and the Kindling
Settings dialog, and switches theme through the actual radio control so the
`theme.ts` path is what gets verified, not the harness shortcut.
Budget: 4 screenshots, about 9 tool calls.

Precondition: editor view.

## Call 1: project settings (screenshot 09-01)

```js
const q = window.__qa;
q.key("Escape");
q.click("settings-button");
q.shot("09-01-project-settings");
return JSON.stringify(q.preflight());
```

`wait_for` selector `#settings-dialog-title`, then screenshot.

**Expect**: dialog with project name, author, genre, description and word
target fields, Fraunces or Inter heading per Press, Save primary and Cancel.
Then `q.click("project-settings-close")`.

## Call 2: Kindling settings from the start screen (screenshot 09-02)

```js
const q = window.__qa;
q.clickSel('[aria-label="Close project"]');
return "start";
```

`wait_for` `[data-testid="import-section"]`, then

```js
const q = window.__qa;
q.click("kindling-settings-button");
q.shot("09-02-kindling-settings");
return JSON.stringify(q.preflight());
```

`wait_for` text `Appearance`, then screenshot.

**Expect**: dialog with an Appearance section showing Dark, Light and System
as three equal segments, the current one outlined in accent, plus author
details fields below.

## Call 3: switch to dark through the radio (screenshot 09-03)

```js
const q = window.__qa;
q.click("theme-option-dark");
return q.settle("theme-ui");
```

`wait_for` `#qa-settled-theme-ui` attached, then

```js
const q = window.__qa;
q.shot("09-03-dark-via-settings");
return JSON.stringify({ ...q.diagTheme(), pf: q.preflight() });
```

Then screenshot.

**Expect**: the whole window including the start screen behind the dialog is
dark on the first capture, the Dark segment is the outlined one, and
`diagTheme` reports dark surfaces. A light frame here means the
`theme-switching` flush in `theme.ts` is not doing its job.

## Call 4: back to light, close, reopen the fixture

```js
const q = window.__qa;
q.click("theme-option-light");
q.click("kindling-settings-close");
q.shot("09-04-light-restored");
return q.settle("theme-ui2");
```

`wait_for` `#qa-settled-theme-ui2` attached, then screenshot, then
`q.click("project-card", "Simple Story")` and `wait_for` text `Act 1`.

**Expect**: start screen back in light, no dialog residue. The stored
preference is whatever it was before; this scenario toggles twice.
