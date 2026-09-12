import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, renameSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { collectContext, contextMarkdown } from "./context.mjs";

const chosen = [
  { id: "08", title: "References" },
  { id: "23", title: "Export profiles" },
];

test("preflight explains committed, staged, unstaged and untracked changes without changing Git", (t) => {
  const dir = mkdtempSync(join(tmpdir(), "qa-context-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd: dir, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "QA Test");
  git("config", "user.email", "qa@example.test");
  git("config", "commit.gpgsign", "false");
  const save = (path, value) => writeFileSync(join(dir, path), value);
  mkdirSync(join(dir, "src"));
  save("src/app.css", "initial");
  save("old.txt", "rename");
  git("add", ".");
  git("commit", "-qm", "Initial reference");
  const revision = git("rev-parse", "HEAD");
  save("src/app.css", "new colors");
  git("commit", "-qam", "Change shared styles");
  save("src/app.css", "unstaged colors");
  save("staged file.txt", "staged");
  git("add", "staged file.txt");
  renameSync(join(dir, "old.txt"), join(dir, "renamed.txt"));
  git("add", "old.txt", "renamed.txt");
  save("untracked\nfile.txt", "draft");
  const before = git("status", "--porcelain=v1");
  const manifest = {
    revision,
    entries: [
      { name: "08-02-reference-listed-light-2x", knownFindings: ["Critical aria-required-parent"] },
      { name: "08-02-reference-listed-dark-2x", knownFindings: ["Critical aria-required-parent"] },
    ],
  };
  const c = collectContext(dir, manifest, chosen, ["light"], "Intentional palette update");
  assert.equal(c.baselineRevision, revision);
  assert.equal(c.commitCount, 1);
  assert.match(c.commits[0], /Change shared styles/);
  assert.deepEqual(c.workingTree.unstaged, ["src/app.css"]);
  assert.ok(c.workingTree.staged.includes("staged file.txt"));
  assert.ok(c.changedFiles.includes("old.txt"));
  assert.ok(c.changedFiles.includes("renamed.txt"));
  assert.deepEqual(c.workingTree.untracked, ["untracked\nfile.txt"]);
  assert.deepEqual(
    c.coverage.map((e) => e.accepted),
    [1, 0]
  );
  assert.equal(c.knownFindings.length, 1);
  assert.equal(c.areas["Shared styles/assets — potentially many screenshots"].length, 1);
  assert.deepEqual(c.warnings, []);
  const report = contextMarkdown(c).join("\n");
  assert.match(report, /NO BASELINES/);
  assert.match(report, /operator supplied.*Intentional palette update/);
  assert.match(report, /known audit failures still fail/);
  assert.equal(git("status", "--porcelain=v1"), before);
});

test("unknown Git provenance is reported as unknown rather than no changes", (t) => {
  const dir = mkdtempSync(join(tmpdir(), "qa-context-no-git-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const c = collectContext(dir, { revision: "--output=untrusted", entries: [] }, chosen, ["dark"]);
  assert.equal(c.revision, null);
  assert.equal(c.baselineRevision, null);
  assert.equal(c.changedFiles, null);
  assert.equal(c.workingTree.staged, null);
  assert.ok(c.warnings.length >= 5);
  assert.match(contextMarkdown(c).join("\n"), /changes since acceptance are unknown/);
});
