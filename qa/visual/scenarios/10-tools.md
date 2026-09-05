# 10 Export, snapshots and the command palette

No e2e counterpart. Opens the three tools reachable from the sidebar's More
actions menu and the Cmd+K palette. Nothing is exported to disk.
Budget: 4 screenshots, about 9 tool calls.

Precondition: editor view.

## Call 1: export dialog (screenshot 10-01)

```js
const q = window.__qa;
q.key("Escape");
q.click("more-actions-button");
return "more";
```

Next call: `q.click("export-button"); q.shot("10-01-export-dialog"); q.preflight()`.
`wait_for` selector `#export-dialog-title`, then screenshot.

**Expect**: dialog with format choices (DOCX manuscript, Markdown, and the
others offered), a save-location field with a browse control, and a disabled
Export primary until a location is chosen. Then `q.click("export-close")`.

## Call 2: snapshots panel and create a snapshot (screenshots 10-02, 10-03)

```js
const q = window.__qa;
q.click("more-actions-button");
return "more";
```

Next call: `q.click("snapshots-button"); q.shot("10-02-snapshots-empty"); q.preflight()`.
`wait_for` selector `#snapshots-panel-title`, then screenshot.

**Expect**: panel titled Snapshots with a Create Snapshot button and the
"No snapshots yet" empty state.

```js
const q = window.__qa;
return q.run([
  ["10-03-open", () => q.click("snapshot-create-button"), 400],
  ["10-03-name", () => q.fillPlaceholder("Enter snapshot name...", "QA snapshot")],
  [
    "10-03-create",
    () =>
      q.clickText(
        "Create Snapshot",
        document.querySelector("h3")?.closest("div[role=dialog], div") ?? document
      ),
    1200,
  ],
  [
    "10-03-listed",
    () =>
      q.mark("10-03-list", {
        listed: document.body.textContent.includes("QA snapshot"),
        empty: document.body.textContent.includes("No snapshots yet"),
      }),
    0,
  ],
  [
    "10-03-shot",
    () => {
      q.shot("10-03-snapshot-created");
      q.done("10a");
    },
    0,
  ],
]);
```

The second `Create Snapshot` is the confirm button inside the create dialog;
if `clickText` picks the panel button instead, scope it with the dialog's
`h3` as above or use the disabled state to tell them apart.

`wait_for` `#qa-done-10a` attached, then screenshot. Assert `listed === true`,
`empty === false`. Then `q.click("snapshots-close")`.

**Expect**: one snapshot row with name, timestamp, Restore and Delete
actions.

## Call 3: command palette (screenshot 10-04)

```js
const q = window.__qa;
q.shortcut("k");
q.shot("10-04-command-palette");
return JSON.stringify(q.preflight());
```

`wait_for` selector `[aria-label="Command palette"]`, then screenshot.

**Expect**: a centred palette with a search input ("Type a command or
search...") and a list of commands in Inter, over a dimmed backdrop. Then
`q.key("Escape")` and assert the palette is gone.

## Call 4: read the log

```js
JSON.stringify(window.__qa.flush());
```
