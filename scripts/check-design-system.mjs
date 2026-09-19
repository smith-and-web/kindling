#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const checks = [
  [resolve(repoRoot, "src/styles/press/generate-tokens.mjs"), "--check"],
  [resolve(repoRoot, "scripts/generate-press-theme.mjs"), "--check"],
];

for (const args of checks) {
  const result = spawnSync(process.execPath, args, { cwd: repoRoot, stdio: "inherit" });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
