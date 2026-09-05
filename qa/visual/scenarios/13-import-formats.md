# 13 Imported outlines, parts and references

Adds visual coverage for Markdown, yWriter and Longform alongside the existing
Plottr scenarios (00, 05 and 12). Run `/visual-qa 13` as its own group.
Budget: 3 screenshots, about 20 tool calls, 3 minutes.

Precondition: scratch database, harness installed, no dialogs open. Each import
creates a new owned fixture; `q.cleanupFixtures()` removes only those captured
IDs. The test bridge bypasses native pickers and reference classification, so
this scenario checks the imported workspace, not the guided-import flow.

Use `<repo>` for the absolute repository path. After **each** `q.importFixture`,
wait for `#qa-done-fixture-created` attached, then the chapter text specified
below. The marker is cleared for each import and prevents a stale chapter from
satisfying the gate. On timeout inspect `q.last` and record the error; do not
continue with the previously loaded project.

## 13-01 Markdown outline

```js
const q = window.__qa;
q.importFixture("<repo>/src-tauri/tests/fixtures/hamlet.md", "markdown");
```

Wait for the creation marker, then text `Act I`. Click `Act I` only if no scene
rows are visible, wait for `Scene 1 - The Battlements`, then:

```js
const q = window.__qa;
q.click("scene-title", "Scene 1 - The Battlements");
```

Wait for `[data-testid="beat-header"]`, then:

```js
const q = window.__qa;
q.mark("13-01-outline", {
  sync: !!q.find("sync-button"),
  prompts: q.titles("beat-header"),
});
q.shot("13-01-markdown-outline");
JSON.stringify(q.preflight());
```

Wait for `#qa-settled-13-01-markdown-outline`, then screenshot.

**Assert**: five acts, four beats in the first scene, including `Francisco
stands guard at Elsinore`; Sync is shown. **Expect**: a usable outline, clear
chapter/scene hierarchy and readable prompts, without empty prose occupying
most of the workspace.

## 13-02 yWriter parts

```js
const q = window.__qa;
q.importFixture("<repo>/src-tauri/tests/fixtures/parts_example.yw7", "ywriter");
```

Wait for the creation marker, then `[data-testid="part-item"]`. Part headings
may be uppercased by CSS. If Chapter 1 is hidden, click Part 1 to expand it.
Click `Chapter 1` only if its scenes are hidden; wait for `Scene 1`, then select
it and wait for `[data-testid="scene-panel"]`.

```js
const q = window.__qa;
q.mark("13-02-parts", { parts: q.titles("part-item"), sync: !!q.find("sync-button") });
q.shot("13-02-ywriter-parts");
JSON.stringify(q.preflight());
```

Wait for `#qa-settled-13-02-ywriter-parts`, then screenshot.

**Assert**: two parts containing three chapters overall; Chapter 1 has Scene 1
and Scene 2; Sync is shown. **Expect**: part headings visually distinct from
chapters, scenes indented under their chapter, the selected scene's title and
metadata readable. Do not treat the capitalized Part labels as missing text.

## 13-03 Longform prose and references

```js
const q = window.__qa;
q.importFixture("<repo>/qa/visual/fixtures/longform/index.md", "longform");
```

Wait for the creation marker, then `Harbor Chapter`. Expand it if needed, select
`Arrival`, and wait for `[data-testid="beat-header"]`. Then:

```js
const q = window.__qa;
q.clickNth("beat-header", 0);
q.mark("13-03-longform", {
  sync: !!q.find("sync-button"),
  hasMara: document.body.textContent.includes("Mara"),
  hasSynopsis: document.body.textContent.includes("Mara returns to the harbor"),
});
q.shot("13-03-longform-references");
q.axe();
JSON.stringify(q.preflight());
```

Wait for `#qa-settled-13-03-longform-references`, then screenshot. Wait for
`#qa-axe-done` before reading `q.axeResult`.

**Assert**: Harbor Chapter and Arrival appear as scene rows; Arrival has two
beats and a synopsis, Mara appears in Characters and Old Harbor in
Locations (switch the References tab to inspect it), and Sync shown.
**Expect**: imported prose and prompts remain separate, reference names and
counts fit their tabs, and synopsis/status controls remain readable alongside
an open editor. Record `q.audit()` and the axe result.

## Finish

```js
const q = window.__qa;
q.key("Escape");
q.done("13");
JSON.stringify({ ...q.flush(), errors: q.takeErrors(), audit: q.audit(), axe: q.axeResult });
```

Before scenarios that expect Simple Story (including sweeps), restore that
workspace: close the project and click its exact `project-card`, or import a
fresh owned `test-data/simple-story.pltr` and wait for the new creation marker
and `Act 1`. Final cleanup removes all owned imports. No fixture files change.
