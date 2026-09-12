#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn, spawnSync } from "node:child_process";
import { acquireLock } from "./lock.mjs";
import { SocketClient } from "./socket.mjs";
import { suites } from "./suites.mjs";
import { SocketCapture, CAPTURE_PROFILE } from "./capture.mjs";
import { loadBaselines, missingCheckpoints } from "./baselines.mjs";
import { collectContext, contextMarkdown } from "./context.mjs";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const quote = JSON.stringify;
export const variants = {
  light: { theme: "light", width: 1600, height: 968 },
  dark: { theme: "dark", width: 1600, height: 968 },
  narrow: { theme: "light", width: 1100, height: 668 },
};

export function verdict(row) {
  if (row.errors?.length || (row.error && row.errorKind === "assertion")) return "DOM regression";
  if (row.error) return "inconclusive";
  if (
    row.audit?.fonts.length ||
    row.audit?.contrast.length ||
    row.audit?.measure.length ||
    row.overflow?.documentOverflow ||
    row.axe?.violations?.some((v) => ["serious", "critical"].includes(v.impact))
  )
    return "visual regression";
  if (
    !row.audit?.scanned ||
    !row.overflow ||
    !Array.isArray(row.errors) ||
    row.axe?.status !== "ok" ||
    !row.capture ||
    row.diff?.status !== "compared" ||
    row.diff.changed ||
    (row.calibration && (row.calibration.status !== "compared" || row.calibration.changed))
  )
    return "inconclusive";
  return "pass";
}

export class Runner {
  constructor(app, output) {
    this.app = app;
    this.output = output;
    this.rows = [];
    this.owned = [];
    this.native = new SocketCapture(app, output);
  }
  js(code) {
    return this.app.evaluate(`const q = window.__qa; ${code}`);
  }
  wait(expression) {
    return this.app.wait(expression);
  }
  async check(expression, message = expression) {
    if (this.interrupted) throw new Error("Run interrupted; cleaning up owned fixtures");
    if (!(await this.js(`return !!(${expression});`))) throw new Error(message);
  }
  async click(css) {
    await this.js(`const el = document.querySelector(${quote(css)});
      if (!el || el.disabled || !el.getClientRects().length) throw new Error('Unavailable control: ' + ${quote(css)});
      el.scrollIntoView({block:'nearest'}); el.click();`);
  }
  async text(text, root = "document") {
    await this.js(`const root = ${root};
      const el = [...root.querySelectorAll('button,[role=button]')].find(e =>
        e.textContent.replace(/\\s+/g,' ').trim() === ${quote(text)} && !e.disabled && e.getClientRects().length);
      if (!el) throw new Error('Missing button: ' + ${quote(text)});
      el.scrollIntoView({block:'nearest'}); el.click();`);
  }
  async fill(css, value) {
    await this.js(`const el = document.querySelector(${quote(css)});
      if (!el || el.disabled) throw new Error('Unavailable input: ' + ${quote(css)});
      el.focus(); el.value = ${quote(value)};
      el.dispatchEvent(new Event('input',{bubbles:true})); el.dispatchEvent(new Event('change',{bubbles:true}));`);
  }
  // Deliver the same event as a native menu selection. This does NOT verify OS accelerators.
  async command(id) {
    await this.app.invoke("plugin:event|emit", { event: "menu-event", payload: id });
  }
  async dismiss() {
    await this.js(`for (const dialog of [...document.querySelectorAll('dialog[open]')].reverse())
      dialog.dispatchEvent(new Event('cancel', {cancelable:true})); q.key('Escape');`);
    await this.wait(`!document.querySelector('dialog[open]')`);
  }
  async closeProject() {
    // In editorial mode the first Close command exits that workspace; a second
    // closes its underlying project. This also handles interruption in packages.
    await this.command("close_project");
    await this.wait(`!document.querySelector('[aria-label="Editorial workspace"]:not([hidden])')`);
    await this.command("close_project");
    await this.wait("document.querySelector('[data-testid=\"import-section\"]')");
  }
  async refreshEmptyStart() {
    // Deletion through IPC does not refresh StartScreen's cached recent list.
    // Its real View all action fetches the empty library again.
    if (await this.js("return !!document.querySelector('[data-testid=recent-projects]');")) {
      await this.click('[data-testid="recent-projects"] button');
      await this.wait("!document.querySelector('[data-testid=recent-projects]')");
    }
  }
  async mode(name) {
    const v = variants[name];
    await this.app.invoke("qa_set_viewport", { width: v.width, height: v.height });
    await this.wait(`innerWidth === ${v.width} && innerHeight === ${v.height}`);
    await this.js(
      `window.__KINDLING_TEST__.setVisualPreferences({guidanceEnabled:false,referencesPanelWidth:288,sidebarCollapsed:false,referencesPanelCollapsed:false});`
    );
    await this.js(`q.setTheme(${quote(v.theme)});`);
    await this.wait(`document.getElementById('qa-settled-theme')`);
  }
  async install() {
    await this.app.evaluate(`if(!window.__qa) { const s=document.createElement('script');
      s.src=${quote(`/@fs${repo}/qa/visual/harness.js`)} + '?t=' + Date.now();document.head.append(s); }`);
    await this.wait("window.__qa?.importFixture && window.__qa?.diff");
    await this.js(`q.repoRoot = ${quote(repo)};`);
  }
  async fixture() {
    await this.js(`q.importFixture(${quote(join(repo, "test-data/simple-story.pltr"))});`);
    await this.wait(
      `document.getElementById('qa-done-fixture-created') || window.__qa.last?.status === 'error'`
    );
    const project = await this.js(
      `if(q.last?.status === 'error') throw new Error(q.last.error); return q.fixtureProject();`
    );
    if (!project?.id) throw new Error("Import did not record ownership");
    this.owned.push(project.id);
    await this.wait(
      `document.querySelector('[data-testid="sidebar"]') && [...document.querySelectorAll('[data-testid="chapter-title"]')].some(e=>e.textContent.includes('Act 1'))`
    );
    this.project = project;
    return project;
  }
  async scene(title = "The Beginning") {
    await this.js(`if(!q.find('scene-title', ${quote(title)})) q.click('chapter-title','Act 1');`);
    await this.wait(
      `[...document.querySelectorAll('[data-testid="scene-title"]')].some(e=>e.textContent.trim()===${quote(title)})`
    );
    await this.js(`q.click('scene-title', ${quote(title)});`);
    await this.wait(
      `document.querySelector('[data-testid="scene-panel"] [data-testid="scene-title"]')?.textContent.trim() === ${quote(title)}`
    );
    await this.wait("document.querySelector('[data-testid=\"beat-header\"]')");
  }
  async seedProse() {
    if (!this.owned.includes(this.project.id))
      throw new Error("Refusing to seed an unowned project");
    const chapters = await this.app.invoke("get_chapters", { projectId: this.project.id });
    const chapter = chapters.find((c) => c.title === "Act 1");
    const scenes = await this.app.invoke("get_scenes", { chapterId: chapter.id });
    for (const scene of scenes.slice(0, 2)) {
      const beats = await this.app.invoke("get_beats", { sceneId: scene.id });
      if (!beats.length) throw new Error("Fixture scene has no beats");
      await this.app.invoke("save_beat_prose", {
        beatId: beats[0].id,
        prose:
          "<p>Mara reached the harbor. The lantern shone over the water. She carried the letter home.</p>",
      });
    }
    this.seededScenes = scenes;
    // Direct IPC seeding does not notify the frontend writing store. Refresh it
    // through its real UI action so captures cannot show stale zero counts.
    const stats = await this.app.invoke("get_writing_stats", { projectId: this.project.id });
    await this.click('[aria-controls="writing-statistics-panel"]');
    await this.wait(
      `document.querySelector('[aria-controls="writing-statistics-panel"]')?.closest('footer').textContent.includes(${quote(`Project: ${stats.project_words.toLocaleString("en-US")} words`)})`
    );
    await this.click('[aria-controls="writing-statistics-panel"]');
  }
  async capture(name) {
    await this.js(`q.shot(${quote(name)});`);
    await this.wait(`document.getElementById(${quote(`qa-settled-${name}`)})`);
    const file = await this.native.capture(name);
    // Reject failed/blank renders independently of PNG encoding success.
    await this.js(`window.__qaImage = {done:false}; const im = new Image();
      im.onload=()=>{ const c=document.createElement('canvas');c.width=32;c.height=32;
        const ctx=c.getContext('2d');ctx.drawImage(im,0,0,32,32);const p=ctx.getImageData(0,0,32,32).data;
        let min=255,max=0;for(let i=0;i<p.length;i+=4){const l=(p[i]+p[i+1]+p[i+2])/3;min=Math.min(min,l);max=Math.max(max,l);}
        window.__qaImage={done:true,ok:max-min>8};};
      im.onerror=()=>window.__qaImage={done:true,ok:false}; im.src=${quote(`/@fs${this.output}/${file}`)};`);
    await this.wait("window.__qaImage.done");
    await this.check(
      "window.__qaImage.ok",
      "Blank or unreadable WebKit snapshot; preserve evidence and diagnose rendering"
    );
    return file;
  }
  async shot(id, expect, assertion = "true") {
    const name = `${id}-${this.variant}-2x`;
    const row = { name, expect, assertion, variant: this.variant };
    this.rows.push(row);
    console.log(`  ${name}`);
    let phase = "assertion";
    try {
      await this.check(assertion);
      row.state = await this.js("return q.state();");
      phase = "capture";
      row.capture = await this.capture(name);
      row.captureInfo = this.native.metadata.get(name);
      if (this.calibrate) {
        const repeat = await this.capture(`${name}-repeat`);
        await this
          .js(`q.diff(${quote(this.output)}, [{name:${quote(name)},file:${quote(repeat)},baseline:${quote(row.capture)}}],
          {baselineDir:${quote(this.output)},tolerance:0,threshold:0});`);
        await this.wait("document.getElementById('qa-diff-done')");
        row.calibration = await this.js("return q.diffResult.results[0];");
      }
      phase = "audit";
      row.audit = await this.js("return q.audit();");
      row.overflow = await this.js("return q.overflow();");
      await this.js("q.axe();");
      await this.wait("document.getElementById('qa-axe-done')");
      row.axe = await this.js("return q.axeResult;");
      row.errors = await this.js("return q.takeErrors();");
      phase = "diff";
      if (this.baselineCompatible) {
        await this.js(
          `q.diff(${quote(this.output)},[{name:${quote(name)},file:${quote(row.capture)},baseline:${quote(row.capture)}}]);`
        );
        await this.wait("document.getElementById('qa-diff-done')");
        row.diff = await this.js("return q.diffResult.results[0];");
      } else row.diff = { status: "incompatible-capture-profile", changed: true };
    } catch (error) {
      row.errorKind = phase;
      row.error = String(error.stack || error);
      await this.failure(name);
      throw error;
    } finally {
      this.report();
    }
  }
  async failure(name) {
    await this.capture(`${name}-FAIL`).catch(() => {});
    const html = await this.js("return q.dom('body', 100000);").catch((e) => String(e));
    writeFileSync(join(this.output, `${name}-FAIL.html`), html);
    const state = await this.js(
      "return {state:q.state(),last:q.last,owned:JSON.parse(sessionStorage.getItem('__qa_created_projects') || '[]'),errors:q.takeErrors()};"
    ).catch((e) => ({ error: String(e) }));
    writeFileSync(join(this.output, `${name}-FAIL.json`), JSON.stringify(state, null, 2));
  }
  report() {
    const imageCounts = { matched: 0, changed: 0, missingBaseline: 0, incomplete: 0 };
    const checkpointCounts = {};
    for (const row of this.rows) {
      const status = verdict(row);
      checkpointCounts[status] = (checkpointCounts[status] || 0) + 1;
      if (!row.capture) imageCounts.incomplete++;
      else if (row.diff?.status === "compared")
        imageCounts[row.diff.changed ? "changed" : "matched"]++;
      else if (row.diff?.status === "no-baseline") imageCounts.missingBaseline++;
      else imageCounts.incomplete++;
    }
    writeFileSync(
      join(this.output, "results.json"),
      JSON.stringify(
        {
          profile: CAPTURE_PROFILE,
          fixtureProfile: "empty-library-default-panels-v1",
          capabilities: this.capabilities,
          context: this.context,
          summary: { images: imageCounts, checkpoints: checkpointCounts },
          rows: this.rows,
          suites: this.executions,
          fatal: this.fatal,
          cleanup: this.cleanup,
        },
        null,
        2
      )
    );
    const cell = (v) =>
      String(v ?? "")
        .replaceAll("|", "\\|")
        .replaceAll("\n", " ");
    const rows = this.rows.map(
      (r) =>
        `| ${r.name} | ${verdict(r)} | ${cell(r.diff?.status)}${r.diff?.changed ? " (changed)" : ""} | ${r.capture ? `[PNG](${r.capture})${r.captureInfo?.preview ? ` / [preview](${r.captureInfo.preview})` : ""}` : "missing"} | ${cell(r.expect)} |`
    );
    const accessibility = new Map();
    for (const row of this.rows) {
      for (const finding of row.axe?.violations || []) {
        const key = `${finding.id}:${finding.sample}`;
        if (!accessibility.has(key)) accessibility.set(key, { ...finding, checkpoints: [] });
        accessibility.get(key).checkpoints.push(row.name);
      }
    }
    writeFileSync(
      join(this.output, "report.md"),
      [
        "# Socket visual QA",
        "",
        `- Cost: 0 MCP tool calls; ${this.native.metadata.size} screenshots (${this.rows.filter((r) => r.capture).length} checkpoints); ${Math.round((Date.now() - this.started) / 1000)} seconds`,
        `- Revision: ${this.revision}`,
        `- Images: ${imageCounts.matched} matched; ${imageCounts.changed} changed; ${imageCounts.missingBaseline} missing baseline; ${imageCounts.incomplete} incomplete.`,
        `- Checkpoints: ${
          Object.entries(checkpointCounts)
            .map(([status, count]) => `${count} ${status}`)
            .join("; ") || "none captured"
        }.`,
        "- Capture profile: hidden WKWebView, rendered 2× lossless PNG. Pixel comparisons use PNG masters, never JPEG previews.",
        `- Cleanup: ${cell(this.cleanup || "pending")}`,
        "- See results.json for assertions, console errors, axe, Press audit, overflow candidates and pixel differences.",
        "- Inconclusive captures require inspection against Expect; never auto-accept a baseline. Overflow candidates and moderate axe findings also need review.",
        this.fatal ? `- Run failure: ${cell(this.fatal)}` : "",
        "",
        ...contextMarkdown(this.context),
        ...(this.executions || []).map((s) => `- Suite ${s.id} (${s.variant}): ${s.status}`),
        "",
        "| Checkpoint | Verdict | Diff | Evidence | Expect |",
        "| --- | --- | --- | --- | --- |",
        ...rows,
        "",
        ...this.rows
          .filter((r) => r.calibration)
          .map(
            (r) =>
              `- Repeat capture ${r.name}: ${r.calibration.status}, exact differing fraction ${r.calibration.mismatch ?? "unknown"}; baseline not accepted.`
          ),
        ...[...accessibility.values()].map(
          (f) =>
            `- Accessibility (${cell(f.impact)}): ${cell(f.id)} — ${cell(f.help)}. Selector: \`${cell(f.sample)}\`. Checkpoints: ${f.checkpoints.join(", ")}.`
        ),
        "",
      ].join("\n")
    );
  }
}

export async function main(args = process.argv.slice(2)) {
  const selected = [],
    modes = [];
  let calibrate = false,
    dryRun = false,
    note = "";
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--list" || args[i] === "--help") {
      console.log(
        "npm run qa:visual -- [--only 00,09,15] [--variants light,dark,narrow] [--calibrate] [--context 'Intentional changes']\nPreflight (no app changes): --dry-run\n" +
          suites.map((s) => `${s.id}  ${s.title}`).join("\n")
      );
      return 0;
    }
    if (args[i] === "--only" && args[i + 1]) selected.push(...args[++i].split(","));
    else if (args[i] === "--variants" && args[i + 1]) modes.push(...args[++i].split(","));
    else if (args[i] === "--calibrate") calibrate = true;
    else if (args[i] === "--dry-run") dryRun = true;
    else if (args[i] === "--context" && args[i + 1]) note = args[++i];
    else throw new Error(`Unknown or incomplete argument: ${args[i]}`);
  }
  for (const id of selected)
    if (!suites.some((s) => s.id === id))
      throw new Error(`Scenario ${id} has no socket implementation; see COVERAGE.md`);
  for (const mode of modes) if (!variants[mode]) throw new Error(`Unknown variant: ${mode}`);
  const chosen = suites.filter((s) => !selected.length || selected.includes(s.id));
  const matrix = modes.length ? [...new Set(modes)] : Object.keys(variants);
  const baselines = loadBaselines(join(repo, "qa/visual/baselines"));
  const output = join(
    repo,
    "qa/visual/results",
    new Date().toISOString().replace(/[:.]/g, "-") + "-socket"
  );
  mkdirSync(output, { recursive: true });
  const app = new SocketClient();
  const releaseLock = acquireLock(`${app.path}.run.lock`);
  const run = new Runner(app, output);
  run.calibrate = calibrate;
  run.baselineCompatible = baselines?.profile === CAPTURE_PROFILE;
  run.executions = matrix.flatMap((variant) =>
    chosen.map((s) => ({ id: s.id, variant, status: "not run" }))
  );
  run.started = Date.now();
  run.revision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repo,
    encoding: "utf8",
  }).stdout.trim();
  run.context = collectContext(repo, baselines, chosen, matrix, note);
  writeFileSync(join(output, "context.json"), JSON.stringify(run.context, null, 2));
  writeFileSync(join(output, "context.md"), contextMarkdown(run.context).join("\n"));
  console.log(contextMarkdown(run.context).join("\n"));
  const keepAwake =
    process.platform === "darwin" ? spawn("caffeinate", ["-i"], { stdio: "ignore" }) : null;
  keepAwake?.on("error", () => {});
  let guarded = false,
    original;
  const interrupt = () => {
    run.interrupted = true;
  };
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", interrupt);
  process.on("SIGHUP", interrupt);
  try {
    await app.connect();
    await app.wait(
      "window.__KINDLING_TEST__ && document.querySelector('main') && !document.getElementById('startup-loading')"
    );
    run.capabilities = await run.native.preflight();
    if (resolve(run.capabilities.checkout || "") !== repo)
      throw new Error("QA app was built from another checkout; restart it here");
    if (dryRun) {
      console.log(
        JSON.stringify(
          { socket: app.path, ...run.capabilities, baselineCompatible: run.baselineCompatible },
          null,
          2
        )
      );
      return run.baselineCompatible ? 0 : 2;
    }
    // A stale ownership record is recovery evidence; do not overwrite it or adopt its projects.
    await app.evaluate(`if(JSON.parse(sessionStorage.getItem('__qa_created_projects') || '[]').length)
      throw new Error('Unfinished QA cleanup: recover with the runbook before starting another run');
      if(document.querySelector('dialog[open], [aria-label="Editorial workspace"]:not([hidden])'))
      throw new Error('Close the current dialog/review before running visual QA');`);
    if ((await app.invoke("get_all_projects")).length)
      throw new Error(
        "Baseline profile requires an empty project library. Use npm run tauri:qa with its dedicated background-profile database; existing projects will not be deleted."
      );
    original = await app.evaluate(
      `return {width:innerWidth,height:innerHeight,theme:document.documentElement.dataset.theme,visual:window.__KINDLING_TEST__.visualPreferences()};`
    );
    await run.install();
    await run.js(
      "q.saveLocal(); q.noMotion(true); q.shots=[]; sessionStorage.removeItem('__qa_shots');"
    );
    guarded = true;
    await run.js("if(q.find('skip-onboarding')) q.click('skip-onboarding');");
    for (const variant of matrix) {
      run.variant = variant;
      await run.mode(variant);
      for (const suite of chosen) {
        await run.check("true");
        const execution = run.executions.find((s) => s.id === suite.id && s.variant === variant);
        execution.status = "in progress";
        console.log(`${suite.id} ${suite.title} (${variant})`);
        run.active = `${suite.id}-00-setup-${variant}`;
        await run.closeProject();
        if (suite.fixture === false) await run.refreshEmptyStart();
        if (suite.fixture !== false) await run.fixture();
        await suite.run(run);
        // Keep later start/settings captures independent of earlier suite choices.
        await run.closeProject();
        await run.js("q.cleanupFixtures();");
        await run.wait(
          "window.__qa.last?.op === 'cleanup' && window.__qa.last.status !== 'pending'"
        );
        await run.check("q.last.status === 'ok'", "Per-suite fixture cleanup failed");
        await run.refreshEmptyStart();
        const missing = missingCheckpoints(baselines, suite.id, variant, run.rows);
        if (missing.length)
          throw new Error(`Accepted baseline checkpoints were not captured: ${missing.join(", ")}`);
        execution.status = "completed (see checkpoint verdicts)";
        run.report();
      }
    }
  } catch (error) {
    run.fatal = String(error.stack || error);
    const execution = run.executions.find((s) => s.status === "in progress");
    if (execution) execution.status = "inconclusive: interrupted by run failure";
    console.error(run.fatal);
    if (guarded) await run.failure(run.active || "00-00-run");
  } finally {
    if (guarded) {
      run.interrupted = false; // Cleanup must still be allowed after a signal.
      try {
        await run.dismiss();
        await run.closeProject();
        await run.js("q.cleanupFixtures();");
        await run.wait(
          "window.__qa.last?.op === 'cleanup' && window.__qa.last.status !== 'pending'"
        );
        await run.check(
          "q.last.status === 'ok'",
          "Fixture cleanup failed; retain session ownership for recovery"
        );
        run.cleanup = "owned fixture IDs deleted";
        await run.refreshEmptyStart();
      } catch (error) {
        run.cleanup = `FAILED: ${error}`;
      }
      try {
        await app.invoke("qa_set_viewport", { width: original.width, height: original.height });
        await run.js(
          `window.__KINDLING_TEST__.setVisualPreferences(${quote(original.visual)}); q.restoreLocal(); q.setTheme(${quote(original.theme || "light")}); q.noMotion(false);`
        );
      } catch (error) {
        run.cleanup += `; preference/window restore FAILED: ${error}`;
      }
    } else run.cleanup = "no fixture or preference changes";
    run.report();
    keepAwake?.kill();
    app.close();
    releaseLock();
    process.off("SIGINT", interrupt);
    process.off("SIGTERM", interrupt);
    process.off("SIGHUP", interrupt);
    spawnSync(process.execPath, [join(repo, "qa/visual/history.mjs")], { stdio: "ignore" });
    console.log(output);
  }
  // 2 means review pending, distinct from a failing assertion/run (1).
  if (
    run.fatal ||
    run.cleanup.includes("FAILED") ||
    run.rows.some((r) => /regression/.test(verdict(r)))
  )
    return 1;
  return run.rows.some((r) => verdict(r) === "inconclusive") ? 2 : 0;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main()
    .then((code) => {
      process.exitCode = code;
    })
    .catch((error) => {
      console.error(error.message);
      process.exitCode = 1;
    });
}
