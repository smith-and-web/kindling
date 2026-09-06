# Demo fixtures — documentation and website screenshots

The user-facing **Sample Project** creates **The Letter** from the committed
fixture in `src-tauri/src/commands/sample_project.rs`: three chapters, nine scenes,
typed reference fields, hierarchical tags and an active-protagonist filter.
This is the preferred source for new documentation and website screenshots.

## Create the current fixtures

Start the debug app with an empty scratch data directory:

```bash
KINDLING_DATA_DIR="$(mktemp -d /tmp/kindling-demo.XXXXXX)" npm run tauri dev
```

Click **Sample Project**. For screenplay and sync screenshots, also run this in
the app's developer console (or through the local MCP `execute_js` command):

```js
await window.__KINDLING_TEST__.invoke("create_demo_fixture");
```

The command adds **The Letter — Screenplay** and **The Letter — Source Outline**.
Return to **All Projects** to open them. It is compiled and registered only in
debug builds. Every invocation creates new projects with independent IDs; use an
empty data directory for a clean screenshot run.

The outline project initially has an empty sync preview. Its `source_path` points
to a `demo-outline-<uuid>.md` file beside the active database. Append a scene to
that file and click **Sync** to photograph the diff:

```markdown
## Beneath the Tower

- Eleanor turns the key
```

From the sibling `kindling-splash` repository, capture without installing images:

```bash
npm run shots -- --keep
```

Keep screenshot data separate from `qa/visual/data`, whose minimal imported
fixtures drive the visual regression baselines. Theme, onboarding and panel widths
are local preferences; the fixture commands do not set them. Create a real
snapshot in the app if a screenshot needs one.

## Older SQL seed

`seed.json` and `seed.mjs` remain available to reproduce the older screenshot
content. They do not contain the current typed fields, tags or nine-scene spine.
The instructions below apply to that older seed only.

### Use the older seed

```bash
npm run tauri:demo
```

That sets `KINDLING_DATA_DIR=qa/demo/data`, which debug builds honour and release
builds ignore (`resolve_data_dir` in `src-tauri/src/lib.rs`). The first launch
creates `qa/demo/data/kindling.db` with an empty schema. Quit the app, then:

```bash
npm run seed:demo
```

Relaunch and **The Letter** is there. Re-run `seed:demo` any time — it deletes the
demo project and rewrites it, so it is safe to repeat and safe to run over an
edited copy.

Two flags: `--print` writes the SQL to stdout instead of applying it, and
`--db <path>` targets a database somewhere else.

### Editing the older content

Edit `seed.json` and reseed. Every id is a UUIDv5 derived from the `key` fields, so
the same manifest always produces the same database — a copy-edit shows up as a
small diff rather than a fresh set of UUIDs, and screenshots stay comparable
across reseeds. `created_at` / `modified_at` are pinned to a fixed
`SEEDED_AT` for the same reason.

The manifest carries the project, characters, locations, custom reference items,
chapters, scenes, beats, and the scene-to-reference links (including the panel
ordering the app reads back from `scene_reference_state`). Beat and scene prose is
TipTap HTML — `<p>`, `<strong>`, `<em>`, `<blockquote>` — because that is what the
editor stores and what `strip_html` in `src-tauri/src/commands/export.rs` expects.

Run the tests after editing:

```bash
npm run test:demo-seed
```

### What the older seed does not cover

- **Snapshots.** A `snapshots` row points at a real archive on disk with a real
  `file_size`. Faking the row would give a demo where restore fails, which is worse
  than having no snapshots. Create one in the app if a screenshot needs it.
- **Sync and reimport.** The project is seeded with `sourceType: "blank"` and no
  `source_path`, so it has no sync button. Screenshots of the sync preview and
  reimport summary need a genuinely imported project — import
  `test-data/simple-story.pltr` by hand for those.
- **Local preferences.** Theme, onboarding state and panel widths live in
  `localStorage`, not the database. Set them in the app; the seed will not touch
  them.

### Why the older script writes SQL directly

It bypasses the Tauri commands, which is a real trade-off: nothing stops the
manifest from encoding a state the app itself could not produce. Two things hold
it honest.

The database is never committed — it is always created by launching the app, so
the **schema** always comes from `src-tauri/src/db/schema.rs` and this script only
ever adds rows to it. And `seed.test.mjs` parses `schema.rs` for its `CREATE TABLE`
and `ALTER TABLE ADD COLUMN` statements and asserts that every column the seed
writes still exists. Rename or drop a column and that test fails, rather than the
next person to reseed discovering it as a `sqlite3` error.

The alternative — driving the UI to build the project and dumping the result —
avoids the trade-off entirely but makes a copy-edit a ten-minute session. The
guard is the cheaper half of that deal.

Node 20 compatibility is intentional: `node:sqlite` only exists from 22.5 and CI
runs 20, so rows go in through the `sqlite3` CLI rather than a driver. No new
dependency either way.
