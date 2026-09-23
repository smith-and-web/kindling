#!/usr/bin/env node
// Records the README demo by driving a hidden, isolated kindling over the QA socket.
// Each checkpoint is serialised from the live DOM into vector SVG (dom2svg.js); the
// frames are then assembled into one looping SMIL animation with subset fonts.
//
//   npm run demo:readme                 # writes docs/assets/kindling-demo.svg
//   npm run demo:readme -- --png        # also keeps WKWebView PNGs for comparison
//   npm run demo:readme -- --keep-app   # leave the hidden app running for inspection
//
// See README.md in this directory for prerequisites.
import { spawn, execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { SocketClient } from "../../visual/socket.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, "../../..");
const SOCKET = "/tmp/kindling-qa.sock";
const OUT = join(repo, "docs/assets/kindling-demo.svg");
const WIDTH = 1280;
const HEIGHT = 800;
const CAPTION = 64;
const FADE = 0.3;
const keepPng = process.argv.includes("--png");
// Debugging aid: leave the hidden app (and its scratch library) running afterwards.
const keepApp = process.argv.includes("--keep-app");
const pngDir = join(repo, "qa/demo/readme/results");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const FONT_FILES = {
  "Inter|normal": "Inter-Variable.woff2",
  "Inter|italic": "Inter-Variable-Italic.woff2",
  "Fraunces|normal": "Fraunces-Variable.woff2",
  "Newsreader|normal": "Newsreader-Variable.woff2",
  "Newsreader|italic": "Newsreader-Variable-Italic.woff2",
};

function pyftsubset() {
  const candidates = [process.env.PYFTSUBSET, "pyftsubset"].filter(Boolean);
  for (const bin of candidates) {
    try {
      execFileSync(bin, ["--help"], { stdio: "ignore" });
      return bin;
    } catch {
      /* try the next candidate */
    }
  }
  throw new Error(
    "pyftsubset (fonttools with brotli) is required; set PYFTSUBSET or add it to PATH"
  );
}

// ------------------------------------------------------------------ app session
async function devServerReady() {
  try {
    const res = await fetch("http://localhost:1420/", { signal: AbortSignal.timeout(1000) });
    return res.ok;
  } catch {
    return false;
  }
}

async function launch() {
  if (existsSync(SOCKET)) {
    const probe = new SocketClient(SOCKET, 3000);
    try {
      await probe.connect();
      probe.close();
      throw new Error(
        `${SOCKET} is already served by a QA app. Stop it first: the demo needs a fresh library.`
      );
    } catch (error) {
      if (!["ENOENT", "ECONNREFUSED"].includes(error.code)) throw error;
    }
  }
  if (!(await devServerReady()))
    throw new Error("Start Vite for this checkout first (npm run dev, or npm run tauri dev)");
  execFileSync("cargo", ["build", "--manifest-path", join(repo, "src-tauri/Cargo.toml")], {
    stdio: "inherit",
  });
  const data = mkdtempSync(join(tmpdir(), "kindling-readme-demo-"));
  const app = spawn(join(repo, "src-tauri/target/debug/kindling"), [], {
    env: { ...process.env, KINDLING_QA_BACKGROUND: "1", KINDLING_DATA_DIR: data },
    stdio: "ignore",
  });
  const deadline = Date.now() + 60000;
  for (;;) {
    if (Date.now() > deadline || app.exitCode !== null) {
      app.kill("SIGTERM");
      rmSync(data, { recursive: true, force: true });
      throw new Error("The hidden demo app did not start");
    }
    try {
      const c = await new SocketClient(SOCKET, 30000).connect();
      await c.wait(
        "window.__KINDLING_TEST__ && !document.getElementById('startup-loading')",
        30000
      );
      return { c, app, data };
    } catch {
      await sleep(500);
    }
  }
}

// Large strings stall execute_js, so results are pulled in slices.
async function pull(c, expr, size = 150000) {
  const length = await c.evaluate(`return (${expr}).length`);
  let out = "";
  for (let i = 0; i < length; i += size)
    out += await c.evaluate(`return (${expr}).slice(${i},${i + size})`);
  return out;
}

async function invokeLarge(c, command, args, field) {
  await c.evaluate(`window.__demoRpc = {done:false};
    window.__KINDLING_TEST__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)})
      .then(v => window.__demoRpc = {done:true, v})
      .catch(e => window.__demoRpc = {done:true, err:String(e)});`);
  await c.wait("window.__demoRpc.done", 30000);
  const err = await c.evaluate("return window.__demoRpc.err || null");
  if (err) throw new Error(`${command}: ${err}`);
  return pull(c, `window.__demoRpc.v.${field}`);
}

// ------------------------------------------------------------------ driving helpers
const js = (c, body) => c.evaluate(body);
const settle = async (c, ms = 400) => {
  await c.evaluate(`window.__demoSettled = false;
    requestAnimationFrame(() => requestAnimationFrame(() => (window.__demoSettled = true)));`);
  await c.wait("window.__demoSettled", 5000);
  await sleep(ms);
};
const byText = (selector, text) =>
  `[...document.querySelectorAll(${JSON.stringify(selector)})].find(e => e.textContent.trim().includes(${JSON.stringify(text)}))`;
const byExact = (selector, text) =>
  `[...document.querySelectorAll(${JSON.stringify(selector)})].find(e => e.textContent.trim() === ${JSON.stringify(text)})`;
const clickText = async (c, selector, text) => {
  await c.wait(byText(selector, text), 10000);
  await js(c, `${byText(selector, text)}.click();`);
};
const center = (c, expr) =>
  js(
    c,
    `const r = (${expr}).getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2];`
  );
const shortcut = (c, key, code, extra = "") =>
  js(
    c,
    `document.activeElement?.blur?.(); window.dispatchEvent(new KeyboardEvent('keydown', {key: ${JSON.stringify(key)}, code: ${JSON.stringify(code)}, metaKey: true, bubbles: true${extra}}));`
  );
const setField = (c, expr, value) =>
  js(
    c,
    `const el = ${expr}; el.focus(); el.value = ${JSON.stringify(value)}; el.dispatchEvent(new Event('input', {bubbles: true}));`
  );
const setSelect = (c, expr, value) =>
  js(
    c,
    `const el = ${expr}; el.value = ${JSON.stringify(value)}; el.dispatchEvent(new Event('change', {bubbles: true}));`
  );

async function capture(c, name) {
  await settle(c);
  await js(c, "window.__dom2svgPrepare();");
  await c.wait("window.__dom2svgReady()", 10000);
  await settle(c, 200);
  const frame = JSON.parse(
    await (async () => {
      await js(
        c,
        `window.__demoFrame = JSON.stringify(window.__dom2svg(${JSON.stringify(name)}));`
      );
      return pull(c, "window.__demoFrame");
    })()
  );
  await js(c, "window.__dom2svgCleanup();");
  if (keepPng) {
    mkdirSync(pngDir, { recursive: true });
    const png = await invokeLarge(
      c,
      "qa_snapshot",
      { width: WIDTH, height: HEIGHT, scale: 2 },
      "pngBase64"
    );
    writeFileSync(join(pngDir, `${name}.png`), Buffer.from(png, "base64"));
  }
  console.log(`captured ${name} (${Math.round(frame.body.length / 1024)} KB)`);
  return frame;
}

// ------------------------------------------------------------------ the story
const TYPED =
  "Ninety-nine steps. Eleanor had counted them every night since she was six, the lamp oil sloshing in its can. Tonight her boots found a hundredth.";

async function record(c) {
  const scenes = [];
  const scene = (name, frame, seconds, caption, extra = {}) =>
    scenes.push({ name, frame, seconds, caption, ...extra });

  await c.invoke("qa_set_viewport", { width: WIDTH, height: HEIGHT });
  await js(c, readFileSync(join(here, "dom2svg.js"), "utf8"));
  await js(
    c,
    `const st = document.createElement('style'); st.id = 'qa-demo-motion';
     st.textContent = '*, *::before, *::after { transition: none !important; animation: none !important; caret-color: transparent !important; }';
     document.head.append(st);`
  );
  const palette = await js(
    c,
    `const probe = document.createElement('div'); document.querySelector('.press-app').append(probe);
     const read = (v) => { probe.style.color = 'var(' + v + ')'; return getComputedStyle(probe).color; };
     const p = { surface: read('--color-surface'), text: read('--color-text'), muted: read('--color-text-muted'),
       border: read('--color-border'), wash: read('--color-accent-wash'), accent: read('--color-accent-text') };
     probe.remove(); return p;`
  );

  // 1. Start screen.
  await c.wait(byText("button", "Sample project"), 20000);
  const sample = await center(c, byText("button", "Sample project"));
  scene(
    "home",
    await capture(c, "home"),
    3.2,
    "Start from the sample, a blank project, or an outline you already have",
    {
      pointer: { from: [980, 640], to: sample, at: 0.5, click: 2.2 },
    }
  );

  // 2. The sample project opens on its first scene.
  await clickText(c, "button", "Sample project");
  await c.wait(byText('[data-testid="scene-item"] button', "On the Cliff"), 20000);
  await clickText(c, '[data-testid="scene-item"] button', "On the Cliff");
  await c.wait(byText("h1, h2", "On the Cliff"), 10000);
  await settle(c, 900);
  scene(
    "scene",
    await capture(c, "scene"),
    3.2,
    "Every scene keeps its synopsis, status, tags and linked references"
  );

  // 3. Beats become prompts; write under one of them.
  await js(c, `${byText("h2, h3", "Beats")}.scrollIntoView({block: 'start'});`);
  await settle(c);
  const beat = `[...document.querySelectorAll('[data-testid="beat-header"]')][2]`;
  await js(c, `${beat}.click();`);
  await c.wait("document.querySelector('.ProseMirror')", 10000);
  await js(
    c,
    `const ed = [...document.querySelectorAll('.ProseMirror')].pop(); ed.focus();
     document.execCommand('insertText', false, ${JSON.stringify(TYPED)});
     ed.parentElement.setAttribute('data-demo-type', '');`
  );
  await c.wait(`!${byText("*", "Saving…")}`, 15000).catch(() => {});
  await settle(c, 1200);
  await js(c, `document.activeElement?.blur?.();`);
  // Park the expanded beat high in the panel so its whole sheet is in view.
  await js(
    c,
    `const h = ${beat}; let s = h.parentElement;
     while (s && !/(auto|scroll)/.test(getComputedStyle(s).overflowY)) s = s.parentElement;
     s.scrollTop += h.getBoundingClientRect().top - 230;`
  );
  await settle(c);
  const beatAt = await center(c, beat);
  scene(
    "beats",
    await capture(c, "beats"),
    6.4,
    "Outline beats become prompts — draft the prose right beneath them",
    {
      pointer: { from: [700, 620], to: beatAt, at: 0.2, click: 0.8 },
      typing: { start: 1.1, end: 5.2 },
    }
  );

  // 4. Find and replace across the manuscript.
  await shortcut(c, "f", "KeyF", ", shiftKey: true");
  await c.wait("document.querySelector('dialog[open] input')", 10000);
  const fields = `[...document.querySelectorAll('dialog[open] input')].filter(i => i.type !== 'checkbox')`;
  await setField(c, `${fields}[0]`, "lighthouse");
  await setField(c, `${fields}[1]`, "beacon");
  await c.wait(byText("dialog[open] *", "4 matches"), 10000);
  await js(c, `document.activeElement?.blur?.();`);
  scene(
    "find",
    await capture(c, "find"),
    3.6,
    "Find and replace in a scene or across the whole project",
    { badge: true }
  );
  await js(c, `document.querySelector('dialog[open] button[aria-label*="lose" i]').click();`);
  await settle(c);

  // 5. Editorial review with tracked suggestions.
  await js(c, `${byText("h1, h2", "On the Cliff")}.scrollIntoView({block: 'center'});`);
  await clickText(c, "button", "Revisions");
  await c.wait(`document.querySelector('select[aria-label="Markup view"]')`, 10000);
  await setSelect(c, `document.querySelector('select[aria-label="Markup view"]')`, "all");
  await clickText(c, "button", "Ground the opening in time");
  await c.wait(byText("button", "Accept"), 10000);
  await settle(c, 700);
  const accept = await center(
    c,
    `[...document.querySelectorAll('button')].filter(b => b.textContent.trim() === 'Accept').pop()`
  );
  scene(
    "review",
    await capture(c, "review"),
    4.2,
    "Editorial review: comments and tracked suggestions, offline",
    {
      badge: true,
      pointer: { from: [760, 420], to: accept, at: 0.6 },
    }
  );
  await js(c, `document.querySelector('button[aria-label="Return to writing"]').click();`);
  await settle(c);

  // 6. Export workspace with profiles and a live preview.
  await shortcut(c, "e", "KeyE");
  await c.wait(`document.querySelector('.dialog-scrim select')`, 10000);
  await setSelect(c, `document.querySelector('.dialog-scrim select')`, "submission");
  await clickText(c, ".dialog-scrim button", "Open workspace");
  await c.wait(byText("h1, h2, h3", "Export workspace"), 10000);
  await settle(c, 1200);
  scene(
    "export",
    await capture(c, "export"),
    3.8,
    "Export profiles with a live manuscript preview",
    { badge: true }
  );
  await js(
    c,
    `[...document.querySelectorAll('button[aria-label]')].filter(b => /close/i.test(b.getAttribute('aria-label'))).pop().click();`
  );
  await settle(c);

  // 7. Settings → Appearance → Dark.
  await shortcut(c, ",", "Comma");
  await c.wait(`document.querySelector('[data-testid="theme-option-dark"]')`, 10000);
  const dark = await center(
    c,
    `document.querySelector('[data-testid="theme-option-dark"]').closest('label')`
  );
  scene(
    "settings",
    await capture(c, "settings"),
    2.6,
    "One Settings window for preferences, shortcuts and project details",
    {
      badge: true,
      pointer: { from: [900, 560], to: dark, at: 0.3, click: 2.0 },
    }
  );
  await js(c, `document.querySelector('[data-testid="theme-option-dark"]').click();`);
  await js(
    c,
    `[...document.querySelectorAll('button[aria-label]')].filter(b => /close/i.test(b.getAttribute('aria-label'))).pop().click();`
  );
  await settle(c);

  // 8. Dark chrome, paper manuscript, Page view.
  await clickText(c, '[data-testid="scene-item"] button', "On the Cliff");
  await settle(c, 600);
  await js(c, `${byExact("button", "Page")}.click();`);
  await c.wait(byText("h2, h3", "Scene prose"), 10000);
  await js(c, `${byText("h2, h3", "Scene prose")}.scrollIntoView({block: 'start'});`);
  scene("dark", await capture(c, "dark"), 3.8, "Dark chrome keeps the manuscript on light paper");

  return { scenes, palette };
}

// ------------------------------------------------------------------ assembly
function fontCss(chars) {
  const bin = pyftsubset();
  const work = mkdtempSync(join(tmpdir(), "kindling-demo-fonts-"));
  let css = "";
  try {
    for (const [key, text] of Object.entries(chars)) {
      const file = FONT_FILES[key];
      if (!file) throw new Error(`No bundled font for ${key}`);
      const [family, style] = key.split("|");
      const txt = join(work, `${family}-${style}.txt`);
      const out = join(work, `${family}-${style}.woff2`);
      writeFileSync(txt, [...new Set([...text, " "])].join(""));
      execFileSync(bin, [
        join(repo, "static/fonts", file),
        `--text-file=${txt}`,
        "--flavor=woff2",
        "--layout-features=*",
        `--output-file=${out}`,
      ]);
      css += `@font-face{font-family:'${family}';font-style:${style};font-weight:100 900;src:url(data:font/woff2;base64,${readFileSync(out).toString("base64")}) format('woff2')}`;
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
  return css;
}

const f = (v) => Math.round(v * 1000) / 1000;
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function assemble({ scenes, palette }) {
  const total = scenes.reduce((sum, s) => sum + s.seconds, 0);
  const H = HEIGHT + CAPTION;
  const chars = {};
  const styles = new Map();
  const cls = (style) => {
    if (!styles.has(style)) styles.set(style, `t${styles.size}`);
    return styles.get(style);
  };
  const addChars = (key, text) => (chars[key] = (chars[key] || "") + text);
  const anim = (attr, values, times, mode = "linear") =>
    `<animate attributeName="${attr}" dur="${f(total)}s" repeatCount="indefinite" calcMode="${mode}" values="${values.join(";")}" keyTimes="${times.map((t) => f(Math.min(1, Math.max(0, t / total)))).join(";")}"/>`;

  let t = 0;
  const groups = [];
  const overlays = [];
  scenes.forEach((s, i) => {
    const start = t;
    const end = t + s.seconds;
    t = end;
    s.frame.chars && Object.entries(s.frame.chars).forEach(([k, v]) => addChars(k, v));
    let body = s.frame.body.replace(/ style="([^"]*)"/g, (_, style) => ` class="${cls(style)}"`);
    // Typed words appear one by one, followed by a caret.
    if (s.typing && s.frame.typed.length) {
      const words = s.frame.typed;
      const step = (s.typing.end - s.typing.start) / words.length;
      body = body.replace(/<tspan x="([^"]+)" data-i="(\d+)">/g, (_, x, index) => {
        const at = start + s.typing.start + step * (Number(index) + 1);
        return `<tspan x="${x}" fill-opacity="0">${anim("fill-opacity", [0, 1], [0, at], "discrete")}`;
      });
      const xs = [words[0].x, ...words.map((w) => w.right + 1)];
      const ys = [words[0].top, ...words.map((w) => w.top)];
      const times = [
        0,
        start + s.typing.start,
        ...words.map((_, k) => start + s.typing.start + step * (k + 1)),
      ];
      overlays.push(
        `<rect width="1.6" height="${f(words[0].height)}" fill="${palette.text}" opacity="0">${anim("x", [xs[0], ...xs], times, "discrete")}${anim("y", [ys[0], ...ys], times, "discrete")}${anim("opacity", [0, 1, 0, 1, 0, 1, 0], [0, start + s.typing.start - 0.6, start + s.typing.start - 0.3, start + s.typing.start, s.typing.end + start + 0.4, s.typing.end + start + 0.8, end + FADE], "discrete")}</rect>`
      );
    }
    // Scene visibility: fade in over the previous scene; the last fades out to the first.
    let values, times;
    if (i === 0) {
      values = [1, 1, 0, 0, 1, 1];
      times = [0, end + FADE, end + FADE + 0.01, total - FADE, total - FADE + 0.01, total];
    } else if (i === scenes.length - 1) {
      values = [0, 0, 1, 1, 0];
      times = [0, start - FADE, start, total - FADE, total];
    } else {
      values = [0, 0, 1, 1, 0, 0];
      times = [0, start - FADE, start, end + FADE, end + FADE + 0.01, total];
    }
    const defs = s.frame.defs ? `<defs>${s.frame.defs}</defs>` : "";
    groups.push(`<g opacity="${values[0]}">${anim("opacity", values, times)}${defs}${body}</g>`);

    // Caption for this scene.
    addChars("Inter|normal", s.caption + "New in 1.3");
    const cy = HEIGHT + CAPTION / 2 + 6;
    const badge = s.badge
      ? `<rect x="24" y="${HEIGHT + 19}" width="92" height="26" rx="13" fill="${palette.wash}"/><text x="70" y="${HEIGHT + 37}" text-anchor="middle" class="${cls("font-family:'Inter';font-size:13px;font-weight:600")}" fill="${palette.accent}">New in 1.3</text>`
      : "";
    const shown = i === 0 ? [1, 0, 1] : i === scenes.length - 1 ? [0, 1] : [0, 1, 0];
    const at =
      i === 0 ? [0, end, total - FADE / 2] : i === scenes.length - 1 ? [0, start] : [0, start, end];
    overlays.push(
      `<g opacity="${shown[0]}">${anim("opacity", shown, at, "discrete")}<rect y="${HEIGHT}" width="${WIDTH}" height="${CAPTION}" fill="${palette.surface}"/>${badge}<text x="${s.badge ? 132 : 24}" y="${cy}" class="${cls("font-family:'Inter';font-size:17px;font-weight:500")}" fill="${palette.text}">${esc(s.caption)}</text><text x="${WIDTH - 24}" y="${cy}" text-anchor="end" class="${cls("font-family:'Inter';font-size:14px;font-weight:400")}" fill="${palette.muted}">${i + 1} / ${scenes.length}</text></g>`
    );

    // Pointer: glide to the target, then optionally press.
    if (s.pointer) {
      const p = s.pointer;
      const a = start + p.at;
      const b = start + p.at + 0.9;
      const k = (x, y) => `${f(x)},${f(y)}`;
      overlays.push(
        `<g opacity="0">${anim("opacity", [0, 0, 1, 1, 0, 0], [0, start, start + 0.15, end - 0.1, end, total])}<g>${`<animateTransform attributeName="transform" type="translate" dur="${f(total)}s" repeatCount="indefinite" calcMode="spline" keySplines="0 0 1 1;0.4 0 0.2 1;0 0 1 1" values="${k(...p.from)};${k(...p.from)};${k(...p.to)};${k(...p.to)}" keyTimes="0;${f(a / total)};${f(b / total)};1"/>`}${p.click != null ? `<circle r="14" fill="${palette.accent}" opacity="0">${anim("opacity", [0, 0, 0.35, 0, 0], [0, start + p.click, start + p.click + 0.05, start + p.click + 0.45, total])}</circle>` : ""}<path d="M0 0 L0 17.5 L4.6 13.4 L7.6 20.2 L10.4 19 L7.5 12.3 L13.6 12.3 Z" fill="#111" stroke="#fff" stroke-width="1.3" stroke-linejoin="round"/></g></g>`
      );
    }
  });

  const css =
    fontCss(chars) +
    [...styles].map(([style, name]) => `.${name}{${style}}`).join("") +
    "text{white-space:pre}";
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${H}" width="${WIDTH}" height="${H}" role="img" aria-label="kindling demo: the sample project, writing under outline beats, find and replace, editorial review, export profiles, and dark mode"><title>kindling demo</title><style>${css}</style><clipPath id="window"><rect width="${WIDTH}" height="${H}" rx="12"/></clipPath><g clip-path="url(#window)">${groups.join("")}${overlays.join("")}<rect y="${HEIGHT}" width="${WIDTH}" height="1" fill="${palette.border}"/></g><rect x="0.5" y="0.5" width="${WIDTH - 1}" height="${H - 1}" rx="11.5" fill="none" stroke="${palette.border}"/></svg>`;
}

// ------------------------------------------------------------------ main
async function main() {
  pyftsubset();
  const { c, app, data } = await launch();
  let saved;
  try {
    saved = await js(c, "return JSON.stringify({...localStorage});");
    await js(
      c,
      `localStorage.setItem('kindling:theme', 'light'); document.documentElement.setAttribute('data-theme', 'light');
       localStorage.setItem('kindling:onboardingCompleted', 'true'); localStorage.setItem('kindling:guidanceEnabled', 'false');`
    );
    const recording = await record(c);
    const svg = assemble(recording);
    writeFileSync(OUT, svg);
    console.log(`wrote ${OUT} (${Math.round(svg.length / 1024)} KB)`);
  } finally {
    // The QA profile's WebKit storage is shared with visual QA; put it back exactly.
    if (saved)
      await js(
        c,
        `localStorage.clear(); for (const [k, v] of Object.entries(${saved})) localStorage.setItem(k, v);`
      ).catch(() => {});
    c.close();
    if (keepApp) {
      app.unref();
      console.log(`app left running (pid ${app.pid}, data ${data})`);
    } else {
      app.kill("SIGTERM");
      rmSync(data, { recursive: true, force: true });
    }
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
