# 99 Dark theme sweep

Runs once at the end of a full run instead of a dark pass per scenario. Four
screens cover every surface the light pass exercised: sidebar with an inline
input, an open prose editor, a modal dialog, and the start screen.

Precondition: editor view, fixture loaded, no dialogs open.
`q.setTheme("dark")` (forces a repaint). Restore with `q.restoreTheme()` at
the end of the sweep, before cleanup.

Capture lag: the first screenshot after a whole-window recolour is one paint
behind even after the forced flush (run 2026-09-05-1207, `99-01`).
`q.setTheme` now also calls `q.settle("theme")`, which appends
`#qa-settled-theme` only after two animation frames. Put the theme flip in
its own call, `wait_for` selector `#qa-settled-theme` state `attached`, then
take the 99-01 shot.

### 99-01 sidebar and editor, input open

- `q.click("chapter-title", "Act 1")`; `q.click("new-scene-button")`;
  `q.shot("99-01-dark-sidebar-input")`. Screenshot.
- Next call: `q.fillTitle("", "Escape")`.

**Expect**: sidebar, main panel and References panel all on the dark surface
(`#26211B` family), chapter titles legible in light ink, the accent row still
readable, the inline input dark with a visible focus ring, the three select
controls dark with legible labels. Any panel still cream is the WebKit
invalidation bug from run 2026-09-04-1729 and a regression.

### 99-02 open beat editor

- `q.click("scene-title", "The Beginning")`, `wait_for` `[data-testid="beat-header"]`,
  `q.clickNth("beat-header", 0)`, `q.shot("99-02-dark-editor")`. Screenshot.
- Next call: `q.key("Escape")`.

**Expect**: the prose card is the paper reading surface with dark ink
(intentional; `--color-prose-bg` has no dark override), the toolbar icons are
legible, the beat header and card border follow the dark tokens.

### 99-03 confirm dialog

- `q.openMenu("chapter-item", q.state().chapters.at(-1))` in one call, then
  `q.menuItem("Delete"); q.shot("99-03-dark-confirm-dialog")` in the next (the
  menu renders on the next tick, so the two cannot share a call). Screenshot.
- Next call: `q.click("dialog-cancel")`.

**Expect**: dark modal on the dark overlay scrim, Fraunces title, Delete as
the danger action with legible contrast, Cancel as secondary.

### 99-04 start screen

- `q.clickSel('[aria-label="Close project"]')`, `q.shot("99-04-dark-start")`.
  Screenshot.
- Next call: `q.click("project-card")` to reopen the fixture, `wait_for` text
  `Act 1`.

**Expect**: the import card and recent-project cards on dark surfaces, brand
mark visible, no light-on-light text in the card metadata.
