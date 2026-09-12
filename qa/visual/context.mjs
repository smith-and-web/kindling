import { spawnSync } from "node:child_process";

const splitPaths = (value) => value.split("\0").filter(Boolean);
const text = (value) => String(value ?? "unknown").replace(/[\r\n\t]/g, " ");

// These are review hints, not dependency analysis or grounds for waiving a diff.
function area(path) {
  if (path.startsWith("qa/visual/baselines/")) return "Accepted references";
  if (path.startsWith("qa/") || /(?:test|spec)\.[^.]+$/.test(path))
    return "Tests and capture tooling";
  if (/^(?:src\/.*\.css|src\/styles\/|public\/|brand-assets\/|tailwind)/.test(path))
    return "Shared styles/assets — potentially many screenshots";
  if (path.startsWith("src/")) return "App UI and behavior — review affected surfaces";
  if (path.startsWith("src-tauri/")) return "Native behavior and data — may affect rendered state";
  if (/^(?:docs\/|.*\.md$)/.test(path)) return "Documentation";
  return "Other/configuration — impact requires review";
}

export function collectContext(repo, manifest, chosen, variants, note = "") {
  const warnings = [];
  const git = (...args) => {
    const result = spawnSync("git", args, {
      cwd: repo,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    if (result.status !== 0) throw new Error(`git ${args[0]} unavailable`);
    return result.stdout;
  };
  const read = (label, fn) => {
    try {
      return fn();
    } catch {
      warnings.push(`${label} could not be determined; absence of evidence is not a clean diff.`);
      return null;
    }
  };
  const revision = read("Current revision", () => git("rev-parse", "HEAD").trim());
  const branch = read("Branch", () => git("rev-parse", "--abbrev-ref", "HEAD").trim());
  const workingTree = {
    staged: read("Staged files", () =>
      splitPaths(git("diff", "--cached", "--name-only", "--no-renames", "-z"))
    ),
    unstaged: read("Unstaged files", () =>
      splitPaths(git("diff", "--name-only", "--no-renames", "-z"))
    ),
    untracked: read("Untracked files", () =>
      splitPaths(git("ls-files", "--others", "--exclude-standard", "-z"))
    ),
  };
  // Accept old manifests with a hash followed by a working-tree qualification.
  // Never pass arbitrary manifest text as a revision or option to Git.
  const hash = /^(?:[0-9a-f]{40}|[0-9a-f]{64})(?=\s|$)/i.exec(manifest?.revision || "")?.[0];
  const baselineRevision = hash
    ? read("Baseline revision", () => git("rev-parse", "--verify", `${hash}^{commit}`).trim())
    : null;
  if (!hash)
    warnings.push("No usable baseline revision is recorded; changes since acceptance are unknown.");
  if (hash && manifest.revision !== hash)
    warnings.push(
      "The baseline revision includes a working-tree qualification; the Git comparison is approximate."
    );
  const changedFiles = baselineRevision
    ? read("Changes since baseline acceptance", () =>
        splitPaths(git("diff", "--name-only", "--no-renames", "-z", baselineRevision, "--"))
      )
    : null;
  const commits =
    baselineRevision && revision
      ? read("Commits since baseline acceptance", () =>
          git("log", "-20", "--format=%h %s", `${baselineRevision}..${revision}`, "--")
            .trim()
            .split("\n")
            .filter(Boolean)
        )
      : null;
  const commitCount =
    baselineRevision && revision
      ? read("Commit count", () =>
          Number(git("rev-list", "--count", `${baselineRevision}..${revision}`, "--").trim())
        )
      : null;
  if (baselineRevision && revision) {
    const ancestor = spawnSync("git", ["merge-base", "--is-ancestor", baselineRevision, revision], {
      cwd: repo,
    });
    if (ancestor.status !== 0)
      warnings.push(
        "The baseline is not a verified ancestor of HEAD; file changes compare the two states directly."
      );
  }
  const files = [
    ...new Set([
      ...(changedFiles || []),
      ...Object.values(workingTree).flatMap((paths) => paths || []),
    ]),
  ].sort();
  const areas = Object.groupBy(files, area);
  const coverage = chosen.flatMap((suite) =>
    variants.map((variant) => {
      const entries = (manifest?.entries || []).filter(
        (entry) => entry.name.startsWith(`${suite.id}-`) && entry.name.endsWith(`-${variant}-2x`)
      );
      return { suite: suite.id, title: suite.title, variant, accepted: entries.length };
    })
  );
  const selectedEntries = (manifest?.entries || []).filter(
    (entry) =>
      chosen.some((s) => entry.name.startsWith(`${s.id}-`)) &&
      variants.some((v) => entry.name.endsWith(`-${v}-2x`))
  );
  return {
    capturedAt: new Date().toISOString(),
    revision,
    branch,
    baselineRevision,
    baselineRecordedRevision: manifest?.revision ?? null,
    baselineAcceptedAt: manifest?.acceptedAt ?? null,
    note,
    workingTree,
    changedFiles,
    commits,
    commitCount,
    areas,
    coverage,
    knownFindings: selectedEntries
      .filter((e) => e.knownFindings?.length)
      .map((e) => ({ checkpoint: e.name, findings: e.knownFindings })),
    warnings,
  };
}

export function contextMarkdown(context) {
  if (!context) return [];
  const lines = [
    "## Context collected before capture",
    "",
    `- Branch/revision: ${text(context.branch)} / ${text(context.revision)}`,
    `- Baseline acceptance: ${text(context.baselineAcceptedAt)} at ${text(context.baselineRecordedRevision)}`,
    "- Context is advisory. Changed images still need review; known audit failures still fail. File groups are hints, not a complete dependency map.",
    "- Baseline counts cannot detect new checkpoints inside an existing suite; those are identified during capture.",
  ];
  if (context.note) lines.push(`- Run note (operator supplied): ${text(context.note)}`);
  for (const [kind, paths] of Object.entries(context.workingTree))
    lines.push(`- ${kind}: ${paths === null ? "unknown" : `${paths.length} file(s)`}`);
  lines.push(`- Commits since acceptance: ${context.commitCount ?? "unknown"} (up to 20 shown).`);
  for (const commit of context.commits || []) lines.push(`  - ${text(commit)}`);
  for (const warning of context.warnings) lines.push(`- Context limitation: ${text(warning)}`);
  for (const [group, paths] of Object.entries(context.areas)) {
    lines.push(`- ${group}: ${paths.length} file(s).`);
    for (const path of paths.slice(0, 12)) lines.push(`  - ${text(path)}`);
    if (paths.length > 12) lines.push(`  - ${paths.length - 12} more; full paths in context.json.`);
  }
  lines.push("", "Accepted references in the selected matrix:", "");
  for (const suite of [...new Set(context.coverage.map((c) => c.suite))]) {
    const entries = context.coverage.filter((c) => c.suite === suite);
    lines.push(
      `- ${suite} ${text(entries[0].title)}: ${entries.map((e) => `${e.variant} ${e.accepted}${e.accepted ? "" : " (NO BASELINES)"}`).join(", ")}`
    );
  }
  if (context.knownFindings.length) {
    lines.push("", "Recorded findings in the selected references (not waived):", "");
    for (const entry of context.knownFindings)
      lines.push(`- ${entry.checkpoint}: ${entry.findings.map(text).join("; ")}`);
  }
  return [...lines, ""];
}
