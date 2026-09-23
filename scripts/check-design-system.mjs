#!/usr/bin/env node
/* Verify the vendored Press snapshot against its recorded content hashes.
 *
 *   node scripts/check-design-system.mjs          verify (read-only)
 *   node scripts/check-design-system.mjs --write  regenerate the manifest
 *
 * Verification needs nothing but this repository — no ../press checkout, no
 * network — so CI can prove the committed copies are the ones that were synced.
 * `--write` is only reachable from scripts/sync-design-system.sh, which has
 * already refreshed the files from ../press.
 *
 * A Press version label, or even a commit, does not identify a snapshot taken
 * from a dirty working tree. When the source tree is dirty the manifest says so
 * and the hashes are the only identity the snapshot has.
 */
import { readFile, writeFile, stat } from "node:fs/promises";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { resolve, dirname } from "node:path";
import { globSync } from "node:fs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(root, "../press");
const manifestPath = resolve(root, "src/styles/press/MANIFEST.json");
const write = process.argv.includes("--write");

/* Everything scripts/sync-design-system.sh writes. Generated files (fonts.css,
   tailwind.css) are recorded too: a hand edit to either is just as much drift. */
const PATTERNS = [
  "src/styles/press/*.css",
  "src/styles/press/*.json",
  "src/styles/press/svelte/**/*",
  "static/fonts/**/*",
  "static/brand/*.svg",
  "static/favicon.svg",
  "static/favicon.png",
  "static/app-icon.svg",
  "DESIGN_GUIDE.md",
];

const files = [];
for (const pattern of PATTERNS) {
  const matches = globSync(pattern, { cwd: root })
    .filter((path) => !path.endsWith("MANIFEST.json"))
    .sort();
  if (!matches.length) {
    throw new Error(`No vendored file matched "${pattern}". Run npm run sync:design-system.`);
  }
  for (const path of matches) {
    const info = await stat(resolve(root, path));
    if (!info.isFile()) continue;
    const bytes = await readFile(resolve(root, path));
    files.push({
      path,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
}

/* The Tailwind bridge is generated here from the mirrored tokens.json and must
   agree with it. (tokens.json itself is a byte copy of Press's, checked by hash;
   Press verifies it against tokens.css at sync time.) */
{
  const result = spawnSync(
    process.execPath,
    [resolve(root, "scripts/generate-press-theme.mjs"), "--check"],
    { cwd: root, stdio: "inherit" }
  );
  if (result.status !== 0) process.exit(result.status ?? 1);
}

if (write) {
  const git = (...args) =>
    execFileSync("git", ["-C", source, ...args], { encoding: "utf8" }).trim();
  let version = "unknown";
  let commit = "unknown";
  let dirty = true;
  try {
    version = JSON.parse(await readFile(resolve(source, "package.json"), "utf8")).version;
    commit = git("rev-parse", "HEAD");
    dirty = git("status", "--porcelain").length > 0;
  } catch {
    /* recorded as unknown */
  }
  await writeFile(
    manifestPath,
    JSON.stringify(
      {
        package: "@kindling/design-system",
        version,
        commit,
        snapshot: dirty ? "working-tree" : "commit",
        note: dirty
          ? `Taken from a dirty ../press working tree at ${commit}. The version and commit do NOT identify these bytes; the sha256 values below are the only identity this snapshot has. Re-sync from a committed release before shipping.`
          : `Taken from ../press at ${commit} with a clean working tree.`,
        syncedAt: new Date().toISOString().replace(/\.\d+Z$/, "Z"),
        generator: "scripts/sync-design-system.sh -> scripts/check-design-system.mjs --write",
        verify: "npm run check:design-system",
        files,
      },
      null,
      2
    ) + "\n"
  );
  console.log(
    `recorded ${files.length} files from @kindling/design-system ${version} @ ${commit}${dirty ? " (working tree dirty)" : ""}`
  );
} else {
  const manifest = JSON.parse(
    await readFile(manifestPath, "utf8").catch(() => {
      throw new Error("src/styles/press/MANIFEST.json is missing. Run npm run sync:design-system.");
    })
  );
  const recorded = new Map(manifest.files.map((file) => [file.path, file]));
  const problems = [];
  for (const file of files) {
    const expected = recorded.get(file.path);
    if (!expected) problems.push(`${file.path}: present but not recorded in the manifest`);
    else if (expected.sha256 !== file.sha256)
      problems.push(`${file.path}: modified since sync (hand-edited mirror?)`);
    recorded.delete(file.path);
  }
  for (const path of recorded.keys()) {
    await stat(resolve(root, path)).catch(() =>
      problems.push(`${path}: recorded in the manifest but missing`)
    );
  }
  if (problems.length) {
    console.error(
      `Vendored Press copies do not match src/styles/press/MANIFEST.json:\n  ${problems.join("\n  ")}\n\nThese are read-only copies. Make the change in ../press and run npm run sync:design-system.`
    );
    process.exit(1);
  }
  console.log(
    `${files.length} vendored files match @kindling/design-system ${manifest.version} @ ${manifest.commit} (${manifest.snapshot} snapshot).`
  );
}
