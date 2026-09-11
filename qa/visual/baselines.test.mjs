import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createHash } from "node:crypto";
import { loadBaselines, missingCheckpoints } from "./baselines.mjs";

test("accepted image changes require a matching reviewed manifest", (t) => {
  const dir = mkdtempSync(join(tmpdir(), "qa-baselines-test-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  assert.equal(loadBaselines(dir), null);
  assert.throws(() => loadBaselines(dir, true), /No accepted/);
  const bytes = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=",
    "base64"
  );
  const entry = {
    name: "00-03-start-light-2x",
    file: "00-03-start-light-2x.png",
    pixels: [1, 1],
    sha256: createHash("sha256").update(bytes).digest("hex"),
    expect: "Start screen",
    review: "Reviewed",
  };
  const save = (entries) =>
    writeFileSync(join(dir, "manifest.json"), JSON.stringify({ version: 1, entries }));
  writeFileSync(join(dir, entry.file), bytes);
  save([entry]);
  assert.equal(loadBaselines(dir).entries.length, 1);
  assert.throws(() => loadBaselines(dir, true, "wkwebview-snapshot-2x-png-v1"), /Incompatible/);
  save([entry, entry]);
  assert.throws(() => loadBaselines(dir), /Invalid/);
  save([{ ...entry, file: "../outside.png" }]);
  assert.throws(() => loadBaselines(dir), /Invalid/);
  save([{ ...entry, pixels: [2, 2] }]);
  assert.throws(() => loadBaselines(dir), /dimensions/);
  save([entry]);
  writeFileSync(join(dir, entry.file), Buffer.concat([bytes, Buffer.from("changed")]));
  assert.throws(() => loadBaselines(dir), /checksum/);
});

test("a conditional checkpoint cannot disappear from a selected suite unnoticed", () => {
  const entries = [
    "07-00-beat-prose-light-2x",
    "07-02-switch-confirm-light-2x",
    "07-02-switch-confirm-dark-2x",
    "08-01-reference-dialog-light-2x",
  ].map((name) => ({ name }));
  const rows = [
    { name: entries[0].name, capture: "beat.png" },
    { name: entries[1].name, error: "capture failed" },
  ];
  assert.deepEqual(missingCheckpoints({ entries }, "07", "light", rows), [entries[1].name]);
  rows[1].capture = "confirmation.png";
  assert.deepEqual(missingCheckpoints({ entries }, "07", "light", rows), []);
});
