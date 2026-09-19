# Kindling — Project Context

Kindling is a **free, open-source, local-first desktop writing app** for plotters and
outliners. It bridges the gap between a story outline and a first draft: scene beats
from an imported outline appear as expandable prompts in a scaffolded writing view.

> This file is the conventions channel for automated agents (e.g. blacksmith) and for
> interactive Claude Code. Blacksmith does **not** read `.claude/settings.json`, so any
> guidance an agent needs must live here.

## Product invariants (do not violate)

- **No AI, no cloud, no subscription.** This is the core promise on the website and
  README. Never add AI features, telemetry, network calls for core features, cloud
  sync, accounts, or paid tiers to the product. It works fully offline.
- **Local-first.** Projects are local SQLite files. Don't introduce a server or remote
  storage dependency.
- **Privacy.** No analytics or phone-home in the app.
- **License:** MIT. Keep it that way.

## Tech stack

- **Frontend:** Svelte 5 (runes) + TypeScript + Tailwind CSS v4, built with Vite.
- **Backend:** Rust + Tauri 2.x.
- **Database:** SQLite via `rusqlite` (schema defined in code).
- **Editor:** TipTap.
- **Tests:** Vitest (frontend), `cargo test` (Rust), WebDriverIO (e2e).

## Layout

```
src/                      Svelte 5 + TS frontend
  lib/components/         UI components (*.svelte, co-located *.test.ts)
  lib/stores/            Runed stores (*.svelte.ts)
  lib/utils/             Helpers (theme, import, ...)
  app.css                Tailwind + Press imports, fonts, and app-wide styles
  styles/press/           Generated read-only mirror + Tailwind token bridge
src-tauri/src/           Rust backend
  commands/              Tauri IPC commands (import, export, crud, sync, ...)
  parsers/               Import parsers: plottr, ywriter, scrivener, longform, markdown
  models/                Domain structs (project, chapter, scene, beat, character, ...)
  db/                    SQLite layer; schema.rs is the source of truth for the schema
  lib.rs                 App entry; tauri::generate_context! (embeds frontend dist/)
e2e/                     WebDriverIO end-to-end suite (separate npm package)
```

## Commands

```bash
npm run tauri dev        # run the app in development
npm run tauri build      # production build

npm test                 # frontend unit tests (vitest run)
npm test -- --coverage   # frontend tests with coverage gate
npm run check            # svelte-check (types)
npm run lint             # eslint src/
npm run format:check     # prettier check
npm run sync:design-system   # sync Press from the sibling brand-assets repo
npm run check:design-system  # fail if a synced file or generated bridge drifted
# Add -- --with-app-icons when the canonical app-icon master changes.

cd src-tauri && cargo test --all-features                                  # Rust tests
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings   # Rust lint
cd src-tauri && cargo fmt --all -- --check                                 # Rust format

npm run check:all        # everything CI checks (types, format, lint, rust fmt+clippy)
```

## Press design system

`../brand-assets/design-system/` is the canonical source for the app's colours,
typography, spacing, states, elevation, and component foundations. Read
`DESIGN_GUIDE.md` before changing UI. Never define a token locally or hand-edit
anything under `src/styles/press/`; change Press upstream, regenerate
`tokens.json`, then run `npm run sync:design-system` here. Light is the default
theme and dark remains a supported app theme.

Use token-backed utilities from the generated Tailwind bridge. Never free-hand a
hex, font family, font size, z-index, or reduced text opacity in app UI. Reading
prose uses Newsreader, operational UI uses Inter, headings use Fraunces, and prose
is capped by `--measure`. Verify shared-style changes through computed styles in
both themes because component-scoped Svelte CSS can win the cascade.

## Conventions

- **Commits:** [Conventional Commits](https://www.conventionalcommits.org/), enforced by
  commitlint (`commitlint.config.js`) via the `.githooks/commit-msg` hook **and** the CI
  "Commit Messages" check. Format is `type(scope): subject` (subject case is not
  enforced); valid types are `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
  `build`,
  `ci`, `chore`, `revert` (e.g. `feat(export): ...`, `fix(scrivener): ...`). A bare
  `<word>: ...` (no valid type) is rejected.
- **Formatting:** Prettier (frontend) and `cargo fmt` (Rust) — run before committing.
- **Linting:** ESLint (frontend) and `clippy -D warnings` (Rust) must be clean.
- **Svelte 5:** use runes (`$state`, `$derived`, `$props`, etc.); stores live in
  `*.svelte.ts` files. Co-locate component tests as `*.test.ts`.
- **Tests are required for new code.** Coverage thresholds are CI-enforced
  (statements ≥95%, branches ≥65%, functions ≥98%, lines ≥95%). Add tests with any
  behavior change.
- **DOCX export** follows Standard Manuscript Format — don't change those rules casually.

### Commit type and scope feed the release notes

Release notes are generated from commit messages —
`npm run changelog` runs `conventional-changelog -p conventionalcommits -i
CHANGELOG.md -s`. The commit message _is_ the release note, so two things are
worth getting right at commit time because they are tedious to fix afterwards.

**Type decides whether users see it.** In practice this preset renders only
`feat` (Features) and `fix` (Bug Fixes) sections; `chore`, `docs`, `style`,
`refactor`, `test`, `ci` and `build` are hidden. So pick the type by asking
whether a _writer using Kindling_ would care. Work on the toolchain, CI, git
hooks or the blacksmith orchestrator is `chore` or `ci` even when it fixes
something — `fix(blacksmith): ...` puts an automation detail in front of end
users, and the 1.1.0 notes already carry `fix(dev)` and `fix(mock)` entries
that mean nothing to a novelist.

**Scope should name a product area, not a work item.** Good: `export`,
`scrivener`, `plottr`, `editor`, `import`, `db`, `ui`. Not a PRD id, work-unit
id, branch name or ticket number — `feat(wu-01): ...` renders as a headline
feature scoped to a label no reader can interpret. If a work unit spans one
product area, use that area; if it spans several, split the commit.

Neither rule is machine-enforced. Commitlint checks the type is in its
`type-enum` and that the scope is kebab-case — `wu-01` passes both, and scope
is optional. The CI "Commit Messages" check will not catch a mis-typed `fix` or
a meaningless scope.

**Two gotchas when actually cutting the notes.** `npm run changelog` emits only
the _unreleased_ section and takes the version from `package.json`, so while
`package.json` still matches the latest tag it outputs nothing — bump the
version first, then generate. And CHANGELOG.md is currently stale: its newest
entry is `0.2.0-alpha`, so 1.0.0-beta through 1.2.0 were never appended.
`npm run changelog:all` (`-r 0`) regenerates the whole file from history and
will overwrite any hand-edited prose in it — check the diff rather than
trusting it.

## The IPC boundary (Rust ↔ TypeScript)

Rust and TypeScript are maintained independently and **nothing checks their
agreement at build time**. Currently 138 `#[tauri::command]` functions, 138
registered. Three things must line
up for every command:

1. The function is annotated `#[tauri::command]` (in `src-tauri/src/commands/`).
2. It is listed in `tauri::generate_handler![...]` in `src-tauri/src/lib.rs`.
   **A command that exists but isn't registered compiles fine and fails only at
   runtime** — check the count matches after adding or renaming one.
3. The frontend `invoke("name", { args })` matches the command name and its
   argument names exactly.

Two conversions that hide mismatches:

- **Tauri maps Rust `snake_case` parameters to `camelCase` on the JS side.** A
  mismatch here arrives as a missing or `null` argument, not an error.
- **A Rust `Err` becomes a rejected promise.** With no `catch` at the call site
  the failure is silent and the UI simply does nothing. Every `invoke` needs
  error handling; a missing one is a real defect, not a style nit.

Also: a Tauri plugin or filesystem/shell capability the frontend calls must be
permitted in `src-tauri/capabilities/default.json`, or it fails at runtime with a
misleading message.

`npm run dev` runs Vite **without** the Rust backend, so every `invoke` fails
there. Verify anything touching the boundary with `npm run tauri dev`.

## Sensitive areas (touch only with explicit intent)

- `src-tauri/src/db/schema.rs` — the SQLite schema. Changes affect existing user files.
- `../brand-assets/design-system/` — canonical Press tokens and component styles.
  Synced files in this repo are generated mirrors and must not be hand-edited.
- `Cargo.lock` / `package-lock.json` — don't add or bump dependencies unsupervised.

## Automation: blacksmith

This repo is onboarded to [blacksmith](https://github.com/smith-and-web/blacksmith), a
PRD-driven agentic orchestrator. PRDs live in `prds/`; see `prds/TEMPLATE.prd.md`.
Toolchain gates are in `blacksmith.toml`; runtime config in `blacksmith.config.toml`.

```bash
blacksmith validate prds/<feature>.prd.md   # offline contract check (no spend)
blacksmith prds/<feature>.prd.md            # run a PRD (needs BLACKSMITH_ANTHROPIC_API_KEY in .env)
```

### Working under blacksmith (required for automated runs)

Before completing **any** work unit, run the formatters with their tools — do **not**
hand-format to match rustfmt/Prettier, and don't rely on the gate to format for you (it
verifies; it can't format-and-commit your work):

- Rust changes: `cd src-tauri && cargo fmt --all`
- Frontend changes: `npm run format`

Then run the relevant tests and linters before finishing: `cargo test --all-features` /
`cargo clippy --all-targets --all-features -- -D warnings` for Rust, and `npm test` /
`npm run check` / `npm run lint` for the frontend. CI checks formatting with `--check`
and will reject unformatted code even though the blacksmith gate no longer does.
