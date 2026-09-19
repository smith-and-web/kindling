import { existsSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { pngSize, CAPTURE_PROFILE } from "./capture.mjs";

export function loadBaselines(directory, required = false, expectedProfile) {
  const path = join(directory, "manifest.json");
  if (!existsSync(path)) {
    if (required) throw new Error("No accepted baseline manifest");
    return null;
  }
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  if (manifest.version !== 1 || !manifest.entries?.length)
    throw new Error("Unsupported or empty baseline manifest");
  if (expectedProfile && manifest.profile !== expectedProfile)
    throw new Error(
      `Incompatible baseline capture profile: ${manifest.profile}; expected ${expectedProfile}`
    );
  const names = new Set();
  for (const entry of manifest.entries) {
    if (
      !/^[0-9]{2}-[a-z0-9-]+-(light|dark|narrow)-2x$/.test(entry.name) ||
      entry.file !== `${entry.name}.png` ||
      names.has(entry.name) ||
      !entry.expect ||
      !entry.review
    )
      throw new Error(`Invalid/unreviewed baseline entry: ${entry.name}`);
    names.add(entry.name);
    const bytes = readFileSync(join(directory, entry.file));
    if (createHash("sha256").update(bytes).digest("hex") !== entry.sha256)
      throw new Error(
        `Baseline checksum mismatch: ${entry.file}. Review the change and update its manifest entry.`
      );
    if (JSON.stringify(pngSize(bytes)) !== JSON.stringify(entry.pixels))
      throw new Error(`Baseline dimensions mismatch: ${entry.file}`);
  }
  return manifest;
}

export function missingCheckpoints(manifest, suite, variant, rows) {
  const captured = new Set(rows.filter((r) => r.capture).map((r) => r.name));
  return (manifest?.entries || [])
    .filter((entry) => entry.name.startsWith(`${suite}-`) && entry.name.endsWith(`-${variant}-2x`))
    .map((entry) => entry.name)
    .filter((name) => !captured.has(name));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const manifest = loadBaselines(
    join(dirname(fileURLToPath(import.meta.url)), "baselines"),
    true,
    CAPTURE_PROFILE
  );
  console.log(
    `${manifest.entries.length} reviewed PNG baselines: names, checksums and dimensions verified`
  );
}
