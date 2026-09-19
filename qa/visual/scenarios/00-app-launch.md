# 00 App launch

Mirrors `e2e/specs/app-launch.spec.js`. Smoke test: the shell renders, the
start screen shows import options, onboarding can be dismissed.

Precondition: harness installed. No fixture needed. If a project is open,
`__qa.click` the "Home — all projects" button (`data-testid="sidebar-home"`) via
`__qa.clickSel('[data-testid="sidebar-home"]')` to return to the start screen.

## Steps

### 00-01 shell renders

e2e: _should launch and show the main element_

- `wait_for` selector `main`, state visible.
- Screenshot `00-01-shell`.
- Assert: `__qa.state().view` is `start`, `onboarding`, or `editor`.

**Expect**: a full window with no white flash or unstyled region. Background is
the Press paper tone in light, the warm dark surface in dark. The window
chrome title reads "Kindling".

### 00-02 onboarding dismissal

e2e: _skipOnboardingIfPresent_

- If `[data-testid="onboarding"]` exists: screenshot `00-02-onboarding` first,
  then `__qa.click("skip-onboarding")`, `wait_for` onboarding detached.
- If absent, record "onboarding already completed" and skip the screenshot.

**Expect** (when shown): a centred modal over a dimmed backdrop, Fraunces
heading, a clearly labelled skip control, no scrollbars inside the modal at
1600 x 1000.

### 00-03 start screen

e2e: _should show the start screen on initial launch_, _should show import options_

- `wait_for` selector `[data-testid="import-section"]`.
- Screenshot `00-03-start-screen`.
- Assert: `__qa.state().view === "start"`.

**Expect**: the "Import an Outline" card with six import options
(Plottr, Markdown, yWriter, Longform, Scrivener and novelWriter), each with an icon and label,
none truncated. Brand mark and product name top-left. New Project, Sample Project and Open Review Package entry points align. Text uses Inter; the heading uses Fraunces.

### 00-04 recent projects

e2e: _should not show recent projects on fresh install_

- Check `[data-testid="recent-projects"]` existence. Either outcome passes, as
  in e2e. Screenshot `00-04-recent` only if present.

**Expect** (when present): project cards in a single column or grid with title,
source type, and last-opened date aligned; no card overflows the panel; the
list scrolls rather than pushing the import card off screen.
