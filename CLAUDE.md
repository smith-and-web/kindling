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
  app.css                Tailwind + Kindling brand tokens (@theme block)
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

cd src-tauri && cargo test --all-features                                  # Rust tests
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings   # Rust lint
cd src-tauri && cargo fmt --all -- --check                                 # Rust format

npm run check:all        # everything CI checks (types, format, lint, rust fmt+clippy)
```

## Conventions

- **Commits:** [Conventional Commits](https://www.conventionalcommits.org/), enforced by
  commitlint (`commitlint.config.js`) via the `.githooks/commit-msg` hook **and** the CI
  "Commit Messages" check. Format is `type(scope): subject` with a lowercase subject;
  valid types are `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`,
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

## Sensitive areas (touch only with explicit intent)

- `src-tauri/src/db/schema.rs` — the SQLite schema. Changes affect existing user files.
- `src/app.css` — Kindling brand tokens (Ember/Flame orange palette, theme variables).
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
