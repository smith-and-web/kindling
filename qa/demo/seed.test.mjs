/**
 * Regression tests for the demo seed generator. Run with `npm run test:demo-seed`.
 *
 * Node's own runner rather than Vitest because `vitest.config.ts` only includes
 * `src/**`, and because this must run on the Node 20 that CI uses.
 */
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { buildSql, uuidv5, idFor, lit, SEEDED_AT } from "./seed.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(readFileSync(join(HERE, "seed.json"), "utf8"));

const minimal = {
  project: { key: "p", name: "P" },
  chapters: [{ key: "c", title: "C", scenes: [{ key: "s", title: "S", beats: [{ content: "b" }] }] }],
};

test("uuidv5 is stable and well formed", () => {
  assert.equal(uuidv5("abc"), uuidv5("abc"));
  assert.notEqual(uuidv5("abc"), uuidv5("abd"));
  assert.match(uuidv5("abc"), /^[0-9a-f]{8}-[0-9a-f]{4}-5[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
});

test("idFor namespaces by kind so a shared key does not collide", () => {
  assert.notEqual(idFor("chapter", "x"), idFor("scene", "x"));
});

test("lit escapes quotes and maps the empty cases", () => {
  assert.equal(lit("keeper's"), "'keeper''s'");
  assert.equal(lit(null), "NULL");
  assert.equal(lit(undefined), "NULL");
  assert.equal(lit(90000), "90000");
  assert.equal(lit(true), "1");
  assert.equal(lit(false), "0");
});

test("re-seeding is deterministic", () => {
  assert.equal(buildSql(manifest), buildSql(manifest));
});

test("the seed is idempotent: it deletes the project before inserting it", () => {
  const sql = buildSql(minimal);
  const del = sql.indexOf("DELETE FROM projects");
  const ins = sql.indexOf("INSERT INTO projects");
  assert.ok(del !== -1 && del < ins, "delete must precede insert");
  assert.ok(sql.includes(idFor("project", "p")));
});

test("the whole seed is one transaction", () => {
  const sql = buildSql(minimal);
  assert.ok(sql.includes("BEGIN;"));
  assert.ok(sql.trimEnd().endsWith("COMMIT;"));
});

test("timestamps are fixed, so an unchanged manifest produces an unchanged database", () => {
  assert.ok(buildSql(minimal, { seededAt: "2020-01-01T00:00:00Z" }).includes("'2020-01-01T00:00:00Z'"));
  // No clock may leak in: the only timestamps present are the two SEEDED_AT columns.
  const timestamps = buildSql(minimal).match(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z/g);
  assert.deepEqual(timestamps, [SEEDED_AT, SEEDED_AT]);
});

test("positions follow manifest order", () => {
  const sql = buildSql({
    project: { key: "p", name: "P" },
    chapters: [
      { key: "a", title: "A", scenes: [] },
      { key: "b", title: "B", scenes: [{ key: "s", title: "S", beats: [{ content: "one" }, { content: "two" }] }] },
    ],
  });
  assert.ok(sql.includes(`'A', 0,`), "first chapter at position 0");
  assert.ok(sql.includes(`'B', 1,`), "second chapter at position 1");
  assert.ok(sql.indexOf("'one'") < sql.indexOf("'two'"));
  assert.ok(sql.includes(idFor("beat", "s:0")) && sql.includes(idFor("beat", "s:1")));
});

test("a beat without prose stores NULL rather than an empty string", () => {
  const sql = buildSql(minimal);
  const beatRow = sql.split("\n").find((line) => line.startsWith("INSERT INTO beats"));
  assert.ok(beatRow.includes("NULL"), beatRow);
});

test("linked references also get a scene_reference_state row, numbered from zero", () => {
  const sql = buildSql({
    project: { key: "p", name: "P" },
    referenceItems: [{ key: "i", type: "items", name: "I" }],
    chapters: [
      {
        key: "c",
        title: "C",
        scenes: [{ key: "s", title: "S", characters: ["ch"], locations: ["lo"], items: ["i"], beats: [] }],
      },
    ],
  });
  const state = sql.split("\n").filter((l) => l.startsWith("INSERT INTO scene_reference_state"));
  assert.equal(state.length, 3);
  assert.ok(state[0].includes("'characters'") && state[0].includes(", 0, 0)"));
  assert.ok(state[1].includes("'locations'") && state[1].includes(", 1, 0)"));
  assert.ok(state[2].includes("'items'") && state[2].includes(", 2, 0)"));
});

test("an item linked without a matching declaration still gets a reference type", () => {
  const sql = buildSql({
    project: { key: "p", name: "P" },
    chapters: [{ key: "c", title: "C", scenes: [{ key: "s", title: "S", items: ["unknown"], beats: [] }] }],
  });
  assert.ok(sql.includes("'items'"));
});

test("the committed manifest carries the content the docs screenshots show", () => {
  const sql = buildSql(manifest);
  assert.ok(sql.includes("The Letter"));
  assert.ok(sql.includes("On the Cliff"));
  assert.ok(sql.includes("Eleanor discovers a mysterious letter"));
  assert.ok(sql.includes("<blockquote>"), "beat prose is TipTap HTML");
});

/**
 * The drift guard. This script writes rows directly rather than going through the
 * Tauri commands, so a column renamed or dropped in schema.rs would otherwise only
 * surface as a runtime sqlite3 error the next time somebody reseeded.
 */
test("every column the seed writes exists in schema.rs", () => {
  const schema = readFileSync(join(HERE, "..", "..", "src-tauri", "src", "db", "schema.rs"), "utf8");

  const known = new Map();
  for (const [, table, body] of schema.matchAll(/CREATE TABLE(?: IF NOT EXISTS)? (\w+)\s*\(([\s\S]*?)\n\s*\);/g)) {
    const columns = body
      .split("\n")
      .map((line) => line.trim().replace(/,$/, ""))
      .filter((line) => line && !/^(PRIMARY KEY|FOREIGN KEY|UNIQUE|CHECK)\b/i.test(line))
      .map((line) => line.split(/\s+/)[0]);
    known.set(table, new Set(columns));
  }
  for (const [, table, column] of schema.matchAll(/ALTER TABLE (\w+) ADD COLUMN (\w+)/g)) {
    known.get(table)?.add(column);
  }

  assert.ok(known.size > 10, `parsed only ${known.size} tables from schema.rs`);

  const missing = [];
  for (const [, table, columns] of buildSql(manifest).matchAll(/INSERT INTO (\w+) \(([^)]+)\)/g)) {
    const schemaColumns = known.get(table);
    assert.ok(schemaColumns, `seed writes to ${table}, which schema.rs does not create`);
    for (const column of columns.split(", ")) {
      if (!schemaColumns.has(column)) missing.push(`${table}.${column}`);
    }
  }
  assert.deepEqual(missing, [], `columns not in schema.rs: ${missing.join(", ")}`);
});
