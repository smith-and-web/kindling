# 02 Beat-level prose editing

Mirrors `e2e/specs/beat-editing.spec.js` (feature #38). Budget: 2
screenshots, about 8 tool calls.

Precondition: fixture imported, Act 1 expanded.

## Call 1: open the scene

```js
const q = window.__qa;
q.click("scene-title", "The Beginning");
("ok");
```

`wait_for` `[data-testid="beat-header"]`.

## Call 2: expand beat 0 (screenshot 02-01)

```js
const q = window.__qa;
q.clickNth("beat-header", 0);
q.shot("02-01-beat-expanded");
q.preflight();
```

Then `take_screenshot`.

e2e: _should expand a beat when clicking the header_

**Expect**: the first beat card opens beneath its header with a TipTap toolbar
and a prose area in Newsreader. The beat prompt stays visible above it. The
other beat remains a collapsed card. The prose column does not exceed the
`--measure` width. The sidebar highlight is on `The Beginning`; if the image
shows it elsewhere while `state.selected` says `The Beginning`, that is the
stale-paint finding: shoot again after `q.repaint()` and keep both.

## Call 3: collapse, reopen, autosave, navigate, one-open (DOM only, 02-02 to 02-07)

```js
const q = window.__qa;
window.__qaProse = "Unique prose for navigation test - " + Date.now();
q.run([
  ["02-02-escaped", () => q.key("Escape")],
  ["02-03-reopened", () => q.clickNth("beat-header", 0), 400],
  ["02-03-toggled", () => q.clickNth("beat-header", 0)],
  ["02-04-reopened", () => q.clickNth("beat-header", 0), 400],
  ["02-04-typed", () => q.setProse("The morning light filtered through the dusty window."), 900],
  ["02-04-saved", () => {}, 1500],
  ["02-06-typed", () => q.setProse(window.__qaProse), 2400],
  ["02-06-escaped", () => q.key("Escape")],
  ["02-06-away", () => q.click("scene-title", "Discovery"), 700],
  ["02-06-back", () => q.click("scene-title", "The Beginning"), 700],
  ["02-06-reopened", () => q.clickNth("beat-header", 0), 400],
  [
    "02-06-prose",
    () => q.mark("02-06-prose-text", { prose: q.getProse(), expected: window.__qaProse }),
    0,
  ],
  ["02-07-second", () => q.clickNth("beat-header", 1), 400],
  ["02-07-shot", () => q.shot("02-07-second-beat-open"), 0],
]);
```

`wait_for` text `Second beat text` (timeout 12000, the chain takes about 9 s),
then `take_screenshot`.

Asserts from the log:

- `02-02-escaped.editors === 0`, `02-03-reopened.editors === 1`,
  `02-03-toggled.editors === 0`.
- `02-04-typed.saving === true` (indicator showed) and `02-04-saved.saving === false`.
- `02-06-back.panel === "The Beginning"`; `02-06-prose-text.prose === expected`.
- `02-07-second.editors === 1`.

e2e: _should collapse a beat when pressing Escape_, _should collapse a beat
when clicking header again_, _should auto-save prose after typing_, _should
show saving indicator while saving_, _should preserve prose when navigating
away and back_, _should collapse current beat when expanding another_.

**Expect** (02-07): beat 2 is expanded with the editor; beat 1 is collapsed
above it showing its prose preview; the panel did not scroll the editor out
of view.

## Call 4: read the log

```js
JSON.stringify(window.__qa.flush());
```

Teardown: `q.key("Escape")` at the start of scenario 03's first call.
