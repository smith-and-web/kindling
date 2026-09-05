# 07 Page view and beat view switching

No e2e counterpart. The Page view is the full-prose editor; switching back to
beats asks for confirmation because page prose is synced into beats.
Budget: 2 screenshots, about 6 tool calls.

Precondition: editor view, Act 1 expanded, scene `The Beginning` (has beats).

## Call 1: switch to Page view (screenshot 07-01)

```js
const q = window.__qa;
q.key("Escape");
if (q.state().scenes.length === 0) q.click("chapter-title", "Act 1");
q.click("scene-title", "The Beginning");
return "scene";
```

`wait_for` `[data-testid="beat-header"]`, then

```js
const q = window.__qa;
q.click("view-page");
q.shot("07-01-page-view");
return JSON.stringify(q.preflight());
```

`wait_for` the settle marker `#qa-settled-07-01-page-view`, then screenshot.
(The heading is uppercased by CSS, so a text wait for `Scene Prose` fails.)

**Expect**: the Beats list is replaced by a single full-page editor headed
Scene Prose, in Newsreader, containing the beat prose concatenated (the
`Unique prose` sentence from scenario 02 and the second beat's text). The
Page toggle is the accent-filled one. Beat cards are gone.

## Call 2: switch back asks for confirmation (screenshot 07-02)

```js
const q = window.__qa;
q.click("view-beats");
q.shot("07-02-switch-confirm");
return JSON.stringify(q.preflight());
```

`wait_for` `[data-testid="confirm-dialog"]`, then screenshot.

**Expect**: the confirm dialog titled Switch to Beat View explaining that
page changes sync back to beats, with a Switch primary and Cancel.

## Call 3: confirm and verify beats return (DOM only)

```js
const q = window.__qa;
return q.run([
  ["07-03-confirm", () => q.click("dialog-confirm"), 900],
  [
    "07-03-beats",
    () =>
      q.mark("07-03-beats-back", {
        beats: q.state().beats,
        pageHeader: !![...document.querySelectorAll("h2")].find(
          (h) => h.textContent.trim() === "Scene Prose"
        ),
      }),
    0,
  ],
  ["07-03-done", () => q.done("07"), 0],
]);
```

`wait_for` `#qa-done-07` attached. Assert `beats === 2` and
`pageHeader === false`.

## Call 4: read the log

```js
JSON.stringify(window.__qa.flush());
```
