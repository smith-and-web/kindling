#!/usr/bin/env node
/**
 * Applies qa/demo/seed.json to a Kindling database so documentation and website
 * screenshots always start from the same project.
 *
 * The database itself is never committed: create it by launching the app once
 * (`npm run tauri:demo`), then run `npm run seed:demo`. That way the schema always
 * comes from `src-tauri/src/db/schema.rs` and this script only writes rows.
 *
 * Every id is a UUIDv5 derived from the manifest's `key` fields, so re-seeding
 * reproduces the same database and a re-seed after an edit is a small diff rather
 * than a churn of fresh UUIDs. Ids must parse as UUIDs: the Rust side calls
 * `Uuid::parse_str` on them (src-tauri/src/commands/crud.rs).
 *
 * Node 20 compatible on purpose — `node:sqlite` only exists from 22.5, and CI is on
 * 20 — so rows go in through the `sqlite3` CLI rather than a driver.
 */
import { createHash } from "node:crypto";
import { readFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const HERE = dirname(fileURLToPath(import.meta.url));

/** Fixed so a re-seed is byte-identical rather than differing only in timestamps. */
export const SEEDED_AT = "2026-01-15T09:00:00Z";

/** DNS namespace, per RFC 4122 appendix C. Any stable namespace would do. */
const NAMESPACE = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

/** RFC 4122 v5: SHA-1 of namespace bytes + name, with the version and variant bits set. */
export function uuidv5(name, namespace = NAMESPACE) {
  const ns = Buffer.from(namespace.replace(/-/g, ""), "hex");
  const hash = createHash("sha1").update(Buffer.concat([ns, Buffer.from(name, "utf8")])).digest();
  hash[6] = (hash[6] & 0x0f) | 0x50;
  hash[8] = (hash[8] & 0x3f) | 0x80;
  const hex = hash.subarray(0, 16).toString("hex");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

/** Ids are namespaced by kind so a chapter and a scene may share a key without colliding. */
export const idFor = (kind, key) => uuidv5(`kindling-demo:${kind}:${key}`);

/** SQLite literal. NULL for null/undefined, doubled single quotes otherwise. */
export function lit(value) {
  if (value === null || value === undefined) return "NULL";
  if (typeof value === "number") return String(value);
  if (typeof value === "boolean") return value ? "1" : "0";
  return `'${String(value).replace(/'/g, "''")}'`;
}

const insert = (table, row) =>
  `INSERT INTO ${table} (${Object.keys(row).join(", ")}) VALUES (${Object.values(row).map(lit).join(", ")});`;

/**
 * Turns a manifest into the SQL that reproduces it.
 * Pure: no filesystem, no clock, no database. `qa/demo/seed.test.mjs` covers it.
 */
export function buildSql(manifest, { seededAt = SEEDED_AT } = {}) {
  const p = manifest.project;
  const projectId = idFor("project", p.key);
  const out = ["PRAGMA foreign_keys = ON;", "BEGIN;"];

  // Idempotent: drop any previous seeding of this project first. Every child table
  // cascades from projects, so one delete clears chapters, scenes, beats and refs.
  out.push(`DELETE FROM projects WHERE id = ${lit(projectId)};`);

  out.push(
    insert("projects", {
      id: projectId,
      name: p.name,
      source_type: p.sourceType ?? "blank",
      source_path: p.sourcePath ?? null,
      created_at: seededAt,
      modified_at: seededAt,
      author_pen_name: p.authorPenName ?? null,
      genre: p.genre ?? null,
      description: p.description ?? null,
      word_target: p.wordTarget ?? null,
      reference_types: JSON.stringify(p.referenceTypes ?? ["characters", "locations"]),
      project_type: p.projectType ?? "novel",
      target_page_count: p.targetPageCount ?? null,
    })
  );

  const attributes = (table, fk, ownerId, attrs) =>
    Object.entries(attrs ?? {}).map(([key, value]) =>
      insert(table, { [fk]: ownerId, key, value })
    );

  for (const c of manifest.characters ?? []) {
    const id = idFor("character", c.key);
    out.push(
      insert("characters", {
        id,
        project_id: projectId,
        name: c.name,
        description: c.description ?? null,
        source_id: null,
      }),
      ...attributes("character_attributes", "character_id", id, c.attributes)
    );
  }

  for (const l of manifest.locations ?? []) {
    const id = idFor("location", l.key);
    out.push(
      insert("locations", {
        id,
        project_id: projectId,
        name: l.name,
        description: l.description ?? null,
        source_id: null,
      }),
      ...attributes("location_attributes", "location_id", id, l.attributes)
    );
  }

  for (const r of manifest.referenceItems ?? []) {
    const id = idFor("referenceItem", r.key);
    out.push(
      insert("reference_items", {
        id,
        project_id: projectId,
        reference_type: r.type,
        name: r.name,
        description: r.description ?? null,
        source_id: null,
      }),
      ...attributes("reference_item_attributes", "reference_item_id", id, r.attributes)
    );
  }

  manifest.chapters?.forEach((ch, chapterPos) => {
    const chapterId = idFor("chapter", ch.key);
    out.push(
      insert("chapters", {
        id: chapterId,
        project_id: projectId,
        title: ch.title,
        position: chapterPos,
        source_id: null,
        synopsis: ch.synopsis ?? null,
        planning_status: ch.planningStatus ?? "fixed",
        archived: ch.archived ?? 0,
        locked: ch.locked ?? 0,
        is_part: ch.isPart ?? 0,
      })
    );

    ch.scenes?.forEach((sc, scenePos) => {
      const sceneId = idFor("scene", sc.key);
      out.push(
        insert("scenes", {
          id: sceneId,
          chapter_id: chapterId,
          title: sc.title,
          synopsis: sc.synopsis ?? null,
          prose: sc.prose ?? null,
          position: scenePos,
          source_id: null,
          scene_type: sc.sceneType ?? "normal",
          scene_status: sc.sceneStatus ?? "draft",
          planning_status: sc.planningStatus ?? "fixed",
          editor_mode: sc.editorMode ?? "beat",
          archived: sc.archived ?? 0,
          locked: sc.locked ?? 0,
        })
      );

      sc.beats?.forEach((b, beatPos) =>
        out.push(
          insert("beats", {
            id: idFor("beat", `${sc.key}:${beatPos}`),
            scene_id: sceneId,
            content: b.content,
            prose: b.prose ?? null,
            position: beatPos,
            source_id: null,
          })
        )
      );

      // Links, plus the panel ordering the app reads back from scene_reference_state.
      let refPos = 0;
      const link = (kind, table, column, keys) => {
        for (const key of keys ?? []) {
          const refId = idFor(kind, key);
          out.push(
            insert(table, { scene_id: sceneId, [column]: refId }),
            insert("scene_reference_state", {
              scene_id: sceneId,
              reference_type: kind === "referenceItem" ? refTypeOf(manifest, key) : `${kind}s`,
              reference_id: refId,
              position: refPos++,
              expanded: 0,
            })
          );
        }
      };
      link("character", "scene_character_refs", "character_id", sc.characters);
      link("location", "scene_location_refs", "location_id", sc.locations);
      link("referenceItem", "scene_reference_item_refs", "reference_item_id", sc.items);
    });
  });

  out.push("COMMIT;");
  return out.join("\n") + "\n";
}

const refTypeOf = (manifest, key) =>
  manifest.referenceItems?.find((r) => r.key === key)?.type ?? "items";

// ---------------------------------------------------------------- CLI

function applySql(dbPath, sql) {
  const run = spawnSync("sqlite3", [dbPath], { input: sql, encoding: "utf8" });
  if (run.error) throw run.error;
  if (run.status !== 0) throw new Error(run.stderr.trim() || `sqlite3 exited ${run.status}`);
  return run.stdout;
}

function main(argv) {
  const args = argv.slice(2);
  const dbFlag = args.indexOf("--db");
  const dbPath = resolve(dbFlag === -1 ? join(HERE, "data", "kindling.db") : args[dbFlag + 1]);
  const manifest = JSON.parse(readFileSync(join(HERE, "seed.json"), "utf8"));
  const sql = buildSql(manifest);

  if (args.includes("--print")) {
    process.stdout.write(sql);
    return 0;
  }

  if (!existsSync(dbPath)) {
    console.error(
      `No database at ${dbPath}.\n` +
        `Launch the app once so it creates the schema, then re-run:\n` +
        `  npm run tauri:demo\n`
    );
    return 1;
  }

  applySql(dbPath, sql);
  const counts = applySql(
    dbPath,
    "SELECT (SELECT count(*) FROM projects), (SELECT count(*) FROM chapters), " +
      "(SELECT count(*) FROM scenes), (SELECT count(*) FROM beats), " +
      "(SELECT count(*) FROM characters), (SELECT count(*) FROM locations), " +
      "(SELECT count(*) FROM reference_items);"
  ).trim();
  const [projects, chapters, scenes, beats, characters, locations, items] = counts.split("|");
  console.log(
    `Seeded ${dbPath}\n` +
      `  projects ${projects}  chapters ${chapters}  scenes ${scenes}  beats ${beats}\n` +
      `  characters ${characters}  locations ${locations}  items ${items}`
  );
  return 0;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  try {
    process.exit(main(process.argv));
  } catch (error) {
    console.error(String(error.message ?? error));
    process.exit(1);
  }
}
