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

## Independent copies between books

Use an isolated `KINDLING_DATA_DIR` as described in the QA runbook. Create two books and add references to the first: a character with notes, typed fields, and a nested tag; a location; an organization; a timeline; and a Notes reference. Disable at least one populated category. Give the second book a same-name character, an incompatible same-name custom field, and a same-name tag under a different parent.

In Book Two, choose **Copy references from project…**. Check that Book One is selectable, Book Two is excluded, all source categories appear, and the existing character defaults to Skip. Search by name and verify that hidden selections remain counted. Choose Keep both and verify the proposed name. Expand the fields/tags summary and check renames and the explanation that new fields also apply to existing references.

Capture the dialog in light and dark themes at 1100 × 700. Check no horizontal overflow, readable descriptions, visible focus, Tab/Shift+Tab containment, Escape dismissal, and focus restoration. With a large library (at least 500 references), check scrolling and filtered selection; the copy count must remain accurate.

Copy, verify the success counts, and close the dialog. Inspect the transferred fields, tags, and notes; the new references should be unlinked. Check the original scene selection and editor content remain intact. Edit and link a copied reference, then verify Book One remains unchanged. Restart the app and verify the copied data persists with existing typed values intact; newly added legacy keys should still migrate under the existing per-key behavior. Repeat the default copy and confirm possible duplicates are skipped.

Also check empty source/no-other-project states and a destination with no enabled categories. Inject a preview failure and a stale-preview response using the test harness: selections should remain, and retry must require a new preview. After a successful commit, simulate a refresh failure and verify the UI offers **Refresh references**, never another copy action.
