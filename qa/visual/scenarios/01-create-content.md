# 01 Create chapters and scenes

Mirrors `e2e/specs/create-content.spec.js` (feature #15). Budget: 3
screenshots, about 9 tool calls.

Precondition: fixture imported, `q.state().view === "editor"`, chapters
`Act 1`, `Act 2`, `Act 3`.

## Call 1: chapter input (screenshot 01-01)

```js
const q = window.__qa;
q.click("new-chapter-button");
q.shot("01-01-chapter-input");
q.preflight();
```

Then `take_screenshot`.

e2e: _should show inline input when clicking new chapter button_

**Expect**: a text input rendered in place at the bottom of the chapter list,
full sidebar width minus padding, visible focus ring in the Press focus
colour, placeholder legible, nothing pushed out of the sidebar column.

## Call 2: create, cancel, blur, auto-expand (DOM only, 01-02 to 01-05)

```js
const q = window.__qa;
q.run([
  ["01-02-created", () => q.fillTitle("Test Chapter QA", "Enter"), 600],
  ["01-03-open", () => q.click("new-chapter-button")],
  ["01-03-escaped", () => q.fillTitle("Should Not Create", "Escape")],
  ["01-04-open", () => q.click("new-chapter-button")],
  ["01-04-blurred", () => q.fillTitle("Should Not Create Click", "blur")],
  ["01-05-open", () => q.click("new-chapter-button")],
  ["01-05-created", () => q.fillTitle("Auto Expand QA", "Enter"), 600],
]);
```

Then `wait_for` text `Auto Expand QA`.

Asserts (read from `q.flush().log` at the end of the scenario):

- `01-02-created.chapters` includes `Test Chapter QA`, `input === false`.
- `01-03-escaped.chapters` and `01-04-blurred.chapters` have the same length as
  `01-02-created.chapters` and no `Should Not Create*`.
- `01-05-created.input === false`; the expansion is visible in call 3's shot.

e2e: _should create a chapter when pressing Enter_, _should cancel chapter
creation when pressing Escape_, _should cancel when clicking elsewhere_,
_should auto-expand newly created chapter_.

## Call 3: expanded chapter and scene input (screenshot 01-06)

```js
const q = window.__qa;
q.click("chapter-title", "Act 1");
("ok");
```

`wait_for` `[data-testid="scene-item"]`, then

```js
const q = window.__qa;
q.click("new-scene-button");
q.shot("01-06-scene-input");
q.preflight();
```

Then `take_screenshot`.

e2e: _should show inline input when clicking new scene button_

**Expect**: `Auto Expand QA` shows an expanded state with its own New Scene
button; under Act 1 the scene input is indented at scene level, aligned with
the scene rows above it, focus ring visible.

## Call 4: create scene, cancel scene, auto-select (01-07 to 01-09)

```js
const q = window.__qa;
q.run([
  ["01-07-created", () => q.fillTitle("Test Scene QA", "Enter"), 700],
  ["01-08-open", () => q.click("new-scene-button")],
  ["01-08-escaped", () => q.fillTitle("Should Not Create", "Escape")],
  ["01-09-open", () => q.click("new-scene-button")],
  ["01-09-created", () => q.fillTitle("Auto Select QA", "Enter"), 700],
  ["01-09-shot", () => q.shot("01-09-scene-selected"), 0],
]);
```

`wait_for` text `Auto Select QA`, then `take_screenshot`.

Asserts from the log:

- `01-07-created.scenes` includes `Test Scene QA`.
- `01-08-escaped.scenes.length === 01-07-created.scenes.length`.
- `01-09-created.panel === "Auto Select QA"` and `selected === "Auto Select QA"`.

e2e: _should create a scene when pressing Enter_, _should cancel scene creation
when pressing Escape_, _should auto-select newly created scene_.

**Expect** (01-09): the sidebar row is filled with the accent colour and
on-accent text; the scene panel header shows the same title in Fraunces with
the chapter breadcrumb beneath; the Beats tab is active and the beat list is
empty with an "Add Your First Beat" affordance.

## Call 5: read the log

```js
JSON.stringify(window.__qa.flush());
```

Record verdicts, then continue to scenario 02 (which starts by selecting
`The Beginning`, so no teardown is needed).
