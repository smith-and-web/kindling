# 11 New blank project

Covers the one e2e case that was never implemented: _should NOT show sync
button for projects without source_path_. Creates a blank project from the
start screen, checks the editor, then deletes it. Budget: 2 screenshots,
about 8 tool calls.

Precondition: start screen (close the fixture first with
`q.clickSel('[data-testid="sidebar-home"]')`).

## Call 1: new project dialog (screenshot 11-01)

```js
const q = window.__qa;
q.click("new-project-button");
q.shot("11-01-new-project-dialog");
return JSON.stringify(q.preflight());
```

`wait_for` selector `#new-project-dialog-title`, then screenshot.

**Expect**: dialog with a Novel / Screenplay project-type toggle, a name
field, a structure template picker with a Browse templates control, Cancel
and a disabled Create until a name is entered.

## Call 2: create and inspect (DOM only, then screenshot 11-02)

```js
const q = window.__qa;
return q.run([
  ["11-02-name", () => q.fillPlaceholder("Enter project name...", "QA Blank Project")],
  ["11-02-create", () => q.createBlankProject(), 1800],
  [
    "11-02-editor",
    () =>
      q.mark("11-02-state", {
        sync: !!document.querySelector('[data-testid="sync-button"]'),
        settings: !!document.querySelector('[data-testid="settings-button"]'),
        title: document
          .querySelector('[data-testid="sidebar"]')
          ?.textContent.includes("QA Blank Project"),
      }),
    0,
  ],
  [
    "11-02-shot",
    () => {
      q.shot("11-02-blank-project");
      q.done("11");
    },
    0,
  ],
]);
```

`wait_for` `#qa-done-11` attached (timeout 8000), then screenshot.

Asserts: `11-02-state.sync === false` (no source path, so no sync button),
`settings === false`, `title === true`, `view === "editor"`, `chapters`
equals `["Chapter 1"]` (a blank project starts with one default chapter).

**Expect**: the editor shell with the new project's name in the sidebar
header, no sync icon beside More actions, a single
`Chapter 1` expanded with the undefined-planning hint and a "Switch to
Flexible" affordance, the New Chapter button, and the scene panel empty state.

## Call 3: read the log, then delete the project

```js
const q = window.__qa;
const out = q.flush();
q.clickSel('[data-testid="sidebar-home"]');
q.cleanupNamed("QA Blank Project");
return JSON.stringify(out);
```

Next call: `JSON.stringify(window.__qa.last)` should show `deleted: 1`.
