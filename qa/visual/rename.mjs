#!/usr/bin/env node
/**
 * Rename take_screenshot output (screenshot_<epoch-ms>.jpg) to the names
 * announced with __qa.shot(name) during the run.
 *
 *   node qa/visual/rename.mjs <run-dir> '<json array of {name,t}>'
 *
 * Each file is matched to the most recent shot announced before it was taken.
 * Files with no preceding shot are left alone.
 */
import { readdirSync, renameSync } from "node:fs";
import { join } from "node:path";

const [dir, shotsJson] = process.argv.slice(2);
if (!dir || !shotsJson) {
  console.error("usage: rename.mjs <run-dir> '<shots json>'");
  process.exit(1);
}
const shots = JSON.parse(shotsJson).sort((a, b) => a.t - b.t);
const files = readdirSync(dir)
  .filter((f) => /^screenshot_(\d+)\.jpg$/.test(f))
  .map((f) => ({ f, t: Number(f.match(/(\d+)/)[1]) }))
  .sort((a, b) => a.t - b.t);

const used = new Map();
for (const { f, t } of files) {
  const shot = [...shots].reverse().find((s) => s.t <= t);
  if (!shot) continue;
  const n = (used.get(shot.name) || 0) + 1;
  used.set(shot.name, n);
  const target = `${shot.name}${n > 1 ? `-${n}` : ""}.jpg`;
  renameSync(join(dir, f), join(dir, target));
  console.log(`${f} -> ${target}`);
}
