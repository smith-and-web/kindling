# 03 Drag and drop reordering

Mirrors `e2e/specs/drag-drop.spec.js` (feature #14). Budget: 2 screenshots,
about 6 tool calls. Drag is DOM-driven through `q.drag`, which dispatches the
same mousedown / mousemove / mouseup sequence the sidebar listens for. The
OS-level `mouse_action drag` is off limits (README). `q.drag` has not yet been
exercised against the live app; the first run that uses it decides whether
03-02 and 03-06 are real checks or _inconclusive_.

Precondition: editor view. No fresh import; expectations are computed from
the current chapter order.

## Call 1: hover state and chapter reorder (screenshot 03-01)

```js
const q = window.__qa;
q.key("Escape");
const c = q.state().chapters;
window.__qaOrder = c;
q.hover("chapter-item", c[0]);
q.shot("03-01-chapter-hover");
q.preflight();
```

Then `take_screenshot`.

e2e: _should show drag handle on hover_, _should show drop indicator while dragging_

**Expect**: the first chapter row shows a grip icon at the left and a 3-dot
menu at the right in the muted colour. Other rows show neither. (Synthetic
hover does not change the row background, so do not expect the hover tone.)

## Call 2: reorder chapters, then scenes (DOM only, 03-02 to 03-07)

```js
const q = window.__qa;
const c = window.__qaOrder;
q.run([
  ["03-02-dragged", () => q.drag("chapter-item", c[0], c[1]), 700],
  ["03-04-open-act1", () => q.click("chapter-title", "Act 1"), 700],
  ["03-05-hover", () => q.hover("scene-item", q.state().scenes[0])],
  [
    "03-06-dragged",
    () => {
      const s = q.state().scenes;
      window.__qaScenes = s;
      q.drag("scene-item", s[0], s[1]);
    },
    700,
  ],
  ["03-07-away", () => q.click("chapter-title", "Act 2"), 700],
  ["03-07-back", () => q.click("chapter-title", "Act 1"), 700],
  ["03-07-shot", () => q.shot("03-07-order-persisted"), 0],
]);
```

`wait_for` `[data-testid="scene-item"]` (timeout 8000), then `take_screenshot`.

Asserts from the log:

- `03-02-dragged.chapters[0] === c[1]` and `[1] === c[0]`. If unchanged, the
  synthetic drag did not register: verdict _inconclusive_, note whether
  any row kept `style.opacity = 0.5` (a stuck drag).
- `03-06-dragged.scenes` is `window.__qaScenes` with the first two swapped.
- `03-07-back.scenes` equals `03-06-dragged.scenes`.

e2e: _should reorder chapters via drag and drop_, _should show drag handle on
scene hover_, _should reorder scenes via drag and drop_, _should persist scene
order after navigation_. The e2e _persist chapter order after reimport_ case
only asserts that reimport works; it is covered by the import step at the
start of the run and is not repeated here.

**Expect** (03-07): the two chapters and the two scenes appear swapped, no
row is left at half opacity, no accent outline lingers on any row.

## Call 3: read the log

```js
JSON.stringify(window.__qa.flush());
```
