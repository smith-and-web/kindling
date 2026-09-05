/**
 * Kindling visual QA harness.
 *
 * Loaded into the running dev app's webview from the Vite dev server (see
 * README "Install the harness"). Everything hangs off `window.__qa`.
 * Re-load after any page reload.
 *
 * Constraints learned from the MCP plugin (verified 2026-09-04):
 *  - `execute_js` does NOT resolve promises. Any async result comes back as
 *    `{}`. Helpers are synchronous or record into `__qa.log` for a later read.
 *  - Svelte 5 delegated `onclick` handlers fire for `.click()` on the owning
 *    <button>. `__qa.click` does that. Avoid the OS-level MCP mouse tools until
 *    their coordinate space is confirmed (README).
 *  - Changing `data-theme` does not repaint the <aside> panels in WebKit
 *    until a layout invalidation. `__qa.setTheme` forces one.
 */
(() => {
  const qa = (window.__qa = window.__qa || {});
  qa.log = qa.log || [];
  qa.shots = qa.shots || JSON.parse(sessionStorage.getItem("__qa_shots") || "[]");

  const all = (testid) => [...document.querySelectorAll(`[data-testid="${testid}"]`)];
  const $ = (sel) => document.querySelector(sel);

  // ---------------------------------------------------------------- finding
  qa.find = (testid, text) => {
    const els = all(testid);
    if (text == null) return els[0] || null;
    return els.find((e) => e.textContent.trim().includes(text)) || null;
  };
  qa.nth = (testid, n) => all(testid)[n] || null;
  qa.titles = (testid) => all(testid).map((e) => e.textContent.trim());

  // ---------------------------------------------------------------- actions
  const clickTarget = (el) => {
    if (!el) return null;
    if (/^(BUTTON|A|INPUT|LABEL)$/.test(el.tagName)) return el;
    return el.querySelector("button, a, [role=button]") || el;
  };
  const need = (el, what) => {
    if (!el) throw new Error(`__qa: ${what} not found`);
    return el;
  };

  qa.click = (testid, text) => {
    clickTarget(need(qa.find(testid, text), `[${testid}] "${text ?? ""}"`)).click();
    return true;
  };
  qa.clickNth = (testid, n) => {
    clickTarget(need(qa.nth(testid, n), `[${testid}] #${n}`)).click();
    return true;
  };
  qa.clickSel = (selector) => {
    clickTarget(need($(selector), selector)).click();
    return true;
  };
  qa.clickWithin = (parentTestid, parentText, childTestid) => {
    const parent = need(qa.find(parentTestid, parentText), `[${parentTestid}] "${parentText}"`);
    clickTarget(need(parent.querySelector(`[data-testid="${childTestid}"]`), childTestid)).click();
    return true;
  };
  /** Synthetic hover: mouseenter is attached directly, so this shows hover-only controls. */
  qa.hover = (testid, text) => {
    const el = need(qa.find(testid, text), `[${testid}] "${text ?? ""}"`);
    el.dispatchEvent(new MouseEvent("mouseenter", { bubbles: false }));
    el.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    return true;
  };
  qa.openMenu = (testid, text) => {
    qa.hover(testid, text);
    return qa.clickWithin(testid, text, "menu-button");
  };
  qa.menuItem = (label) => {
    need(
      $(`[data-testid="context-menu-item"][data-label="${label}"]`),
      `menu item ${label}`
    ).click();
    return true;
  };
  qa.key = (key, opts = {}) => {
    const target = document.activeElement || document.body;
    const init = { key, code: key, bubbles: true, cancelable: true, ...opts };
    target.dispatchEvent(new KeyboardEvent("keydown", init));
    target.dispatchEvent(new KeyboardEvent("keyup", init));
    return true;
  };
  /** Fill the inline title input and press Enter/Escape. Avoids type_text timeouts. */
  qa.fillTitle = (value, key = "Enter") => {
    const i = need($('[data-testid="title-input"]'), "title-input");
    i.focus();
    i.value = value;
    i.dispatchEvent(new Event("input", { bubbles: true }));
    if (key === "blur") {
      i.blur();
      i.dispatchEvent(new FocusEvent("blur"));
    } else qa.key(key);
    return true;
  };
  qa.setProse = (text) => {
    const ed = need(
      $('[data-testid="beat-prose-editor"] .novel-editor-content'),
      "open beat editor"
    );
    ed.focus();
    ed.innerHTML = `<p>${text}</p>`;
    ed.dispatchEvent(new InputEvent("input", { bubbles: true }));
    return true;
  };
  qa.getProse = () =>
    $('[data-testid="beat-prose-editor"] .novel-editor-content')?.innerText.trim() ?? null;

  /**
   * DOM-driven drag for the sidebar's pointer-based reorder. Mirrors the
   * app's own listeners: mousedown on the row's drag handle (delegated by
   * Svelte 5, so a bubbling MouseEvent reaches it), then document mousemove
   * with clientY inside the target row, then document mouseup.
   */
  qa.drag = (rowTestid, fromText, toText) => {
    const from = need(qa.find(rowTestid, fromText), `[${rowTestid}] "${fromText}"`);
    const to = need(qa.find(rowTestid, toText), `[${rowTestid}] "${toText}"`);
    const handle = need(from.querySelector('[data-testid="drag-handle"]'), "drag handle");
    const hr = handle.getBoundingClientRect();
    const tr = to.getBoundingClientRect();
    const ev = (type, x, y) =>
      new MouseEvent(type, { bubbles: true, cancelable: true, button: 0, clientX: x, clientY: y });
    handle.dispatchEvent(ev("mousedown", hr.x + hr.width / 2, hr.y + hr.height / 2));
    document.dispatchEvent(ev("mousemove", tr.x + tr.width / 2, tr.y + tr.height / 2));
    document.dispatchEvent(ev("mouseup", tr.x + tr.width / 2, tr.y + tr.height / 2));
    return true;
  };

  /** Click a button (or role=button) by its visible text. Exact match first, then prefix. */
  qa.clickText = (text, root = document) => {
    const els = [...root.querySelectorAll("button, [role=button], a")].filter((e) => !e.disabled);
    const norm = (e) => e.textContent.replace(/\s+/g, " ").trim();
    const el = els.find((e) => norm(e) === text) || els.find((e) => norm(e).startsWith(text));
    need(el, `button with text "${text}"`).click();
    return true;
  };
  /** Fill an input found by placeholder (dialogs without testids). */
  qa.fillPlaceholder = (placeholder, value) => {
    const i = need(
      document.querySelector(
        `input[placeholder="${placeholder}"], textarea[placeholder="${placeholder}"]`
      ),
      `input "${placeholder}"`
    );
    i.focus();
    i.value = value;
    i.dispatchEvent(new Event("input", { bubbles: true }));
    return true;
  };
  /** Close the dialog whose heading has this id (several dialogs share aria-label="Close"). */
  qa.closeDialog = (titleId) => {
    const h = need(document.getElementById(titleId), `dialog title #${titleId}`);
    const dlg = h.closest('[role="dialog"]') || h.parentElement.parentElement;
    need(dlg.querySelector('[aria-label="Close"]'), "dialog close button").click();
    return true;
  };
  /** Keyboard shortcut on the window (App.svelte listens for meta+k, meta+e). */
  qa.shortcut = (key, mods = { metaKey: true }) => {
    document.body.dispatchEvent(
      new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...mods })
    );
    return true;
  };

  qa.center = (testid, text) => {
    const r = need(qa.find(testid, text), `[${testid}] "${text ?? ""}"`).getBoundingClientRect();
    return { x: Math.round(r.x + r.width / 2), y: Math.round(r.y + r.height / 2) };
  };

  // ------------------------------------------------------------ async ops
  // Only creation calls explicitly initiated by this harness confer ownership.
  // Persist across the reload scenario; never infer ownership from a filename/name.
  const ownedKey = "__qa_created_projects";
  const owned = () => JSON.parse(sessionStorage.getItem(ownedKey) || "[]");
  qa.trackCreation = (command, action) => {
    if (!["import_plottr", "create_blank_project", "create_screenplay_project"].includes(command))
      throw new Error("Unsupported QA creation command");
    const ipc = window.__TAURI_INTERNALS__;
    if (!ipc?.invoke) throw new Error("Tauri IPC missing; cannot track QA project ownership");
    const original = ipc.invoke;
    ipc.invoke = function (cmd, ...args) {
      const result = original.call(this, cmd, ...args);
      if (cmd !== command) return result;
      return Promise.resolve(result).then((project) => {
        if (!project?.id) throw new Error("Creation returned no project ID");
        const projects = owned();
        if (!projects.some((p) => p.id === project.id)) {
          projects.push({ id: project.id, name: project.name, fixture: cmd === "import_plottr" });
          sessionStorage.setItem(ownedKey, JSON.stringify(projects));
        }
        return project;
      });
    };
    try {
      return action();
    } finally {
      ipc.invoke = original;
    }
  };
  qa.createBlankProject = () =>
    qa.trackCreation("create_blank_project", () => qa.click("new-project-create"));

  /** Import the fixture. Gate on wait_for text "Act 1"; the promise never settles reliably. */
  qa.importFixture = (absolutePath) => {
    const hook = window.__KINDLING_TEST__;
    if (!hook?.importProject)
      throw new Error("__KINDLING_TEST__.importProject missing (not a dev build?)");
    qa.last = { op: "import", status: "pending" };
    qa.clearMarkers();
    hook.disableGuidance();
    qa.trackCreation("import_plottr", () => hook.importProject(absolutePath))
      .then((p) => (qa.last = { op: "import", status: "ok", projectId: p.id }))
      .catch((e) => (qa.last = { op: "import", status: "error", error: String(e) }));
    return "started";
  };
  /** Any Tauri command; result lands in __qa.last. */
  qa.invoke = (cmd, args = {}) => {
    qa.last = { op: cmd, status: "pending" };
    window.__KINDLING_TEST__
      .invoke(cmd, args)
      .then((r) => (qa.last = { op: cmd, status: "ok", result: r }))
      .catch((e) => (qa.last = { op: cmd, status: "error", error: String(e) }));
    return "started";
  };
  /** Delete only IDs captured from successful QA creation calls. */
  const cleanupOwned = (op, predicate) => {
    const inv = window.__KINDLING_TEST__.invoke;
    qa.last = { op, status: "pending" };
    const mine = owned().filter(predicate);
    (async () => {
      let deleted = 0;
      try {
        for (const project of mine) {
          await inv("delete_project", { projectId: project.id });
          sessionStorage.setItem(
            ownedKey,
            JSON.stringify(owned().filter((p) => p.id !== project.id))
          );
          deleted++;
        }
        qa.last = { op, status: "ok", deleted };
      } catch (e) {
        qa.last = { op, status: "error", deleted, error: String(e) };
      }
    })();
    return "started";
  };
  qa.cleanupFixtures = () => cleanupOwned("cleanup", (p) => p.fixture);
  qa.cleanupNamed = (name) => cleanupOwned("cleanupNamed", (p) => !p.fixture && p.name === name);

  // ---------------------------------------------------------------- theme
  qa.repaint = () => {
    const h = document.documentElement;
    h.style.display = "none";
    void h.offsetHeight;
    h.style.display = "";
    void h.offsetHeight;
    return true;
  };
  qa.originalTheme =
    qa.originalTheme || document.documentElement.getAttribute("data-theme") || "light";
  qa.setTheme = (theme) => {
    document.documentElement.setAttribute("data-theme", theme);
    qa.repaint();
    qa.settle("theme");
    return theme;
  };
  /**
   * Append `#qa-settled-<id>` only after two animation frames have run, i.e.
   * after at least one frame has actually been presented. wait_for on it
   * (state attached) before a screenshot that follows a whole-window repaint.
   */
  qa.settle = (id = "x") => {
    document.getElementById(`qa-settled-${id}`)?.remove();
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        const d = document.createElement("div");
        d.id = `qa-settled-${id}`;
        d.hidden = true;
        document.body.append(d);
      })
    );
    return `#qa-settled-${id}`;
  };
  qa.restoreTheme = () => qa.setTheme(qa.originalTheme);

  // ---------------------------------------------------------------- state
  qa.state = () => ({
    theme: document.documentElement.getAttribute("data-theme"),
    view: $('[data-testid="sidebar"]')
      ? "editor"
      : $('[data-testid="onboarding"]')
        ? "onboarding"
        : $('[data-testid="import-section"]')
          ? "start"
          : "unknown",
    chapters: qa.titles("chapter-title"),
    scenes: [
      ...document.querySelectorAll('[data-testid="sidebar"] [data-testid="scene-title"]'),
    ].map((e) => e.textContent.trim()),
    panel: $('[data-testid="scene-panel"] [data-testid="scene-title"]')?.textContent.trim() ?? null,
    selected:
      [...document.querySelectorAll('[data-testid="scene-item"] button.bg-press-accent')].map((b) =>
        b.textContent.trim()
      )[0] ?? null,
    beats: all("beat-header").length,
    editors: all("beat-prose-editor").length,
    saving: !!$('[data-testid="save-indicator"]'),
    dialogs: [...document.querySelectorAll("[data-testid$='dialog']")].map((d) => d.dataset.testid),
    menu: !!$('[data-testid="context-menu"]'),
    input: !!$('[data-testid="title-input"]'),
    empty: !!$('[data-testid="scene-panel"] [data-testid="empty-state"]'),
    vis: document.visibilityState,
  });

  /** Cheap gate before a screenshot batch. Stop the run if `ok` is false. */
  qa.preflight = () => ({
    ok: document.visibilityState === "visible",
    vis: document.visibilityState,
    w: innerWidth,
    h: innerHeight,
  });

  // ------------------------------------------------------- logging + steps
  /** Record a named state snapshot (plus optional extra) into __qa.log. */
  qa.mark = (name, extra) => {
    qa.log.push({ name, t: Date.now(), ...qa.state(), ...(extra || {}) });
    return name;
  };
  /** Announce the screenshot you are about to take, so rename.mjs can name the file. */
  /** Disable CSS transitions and animations so a capture never lands mid-transition. Default on at load. */
  qa.noMotion = (on = true) => {
    let st = document.getElementById("qa-no-motion");
    if (on && !st) {
      st = document.createElement("style");
      st.id = "qa-no-motion";
      st.textContent =
        "*, *::before, *::after { transition: none !important; animation: none !important; }";
      document.head.append(st);
    } else if (!on && st) st.remove();
    return on;
  };
  qa.noMotion(true);

  qa.shot = (name) => {
    qa.repaint();
    qa.shots.push({ name, t: Date.now() });
    sessionStorage.setItem("__qa_shots", JSON.stringify(qa.shots));
    // Two frames later the action that preceded this call has been presented;
    // the runner waits for #qa-settled-<name> before take_screenshot.
    qa.settle(name);
    return name;
  };
  /** Append a hidden marker so wait_for `#qa-done-<id>` (state attached) can gate a q.run chain. */
  qa.done = (id) => {
    const d = document.createElement("div");
    d.id = `qa-done-${id}`;
    d.hidden = true;
    document.body.append(d);
    return d.id;
  };

  /** Drain and return the log (and shots) in one call. */
  qa.flush = () => {
    const out = { log: qa.log, shots: qa.shots, last: qa.last ?? null };
    qa.log = [];
    return out;
  };

  /**
   * Run DOM-only checkpoints without round trips. Each step is
   * `[name, fn, delayMs?]`; fn runs, then after the delay (default 250 ms)
   * the state is recorded under `name`. Steps run strictly in order.
   * Read the results later with `__qa.flush()` — never from this call.
   */
  /** Remove stale done/settle markers so a wait_for cannot match a previous attempt. */
  qa.clearMarkers = () => {
    document
      .querySelectorAll("[id^=qa-done-], [id^=qa-settled-], #qa-axe-done, #qa-diff-done")
      .forEach((d) => d.remove());
    return true;
  };
  qa.run = (steps) => {
    qa.clearMarkers();
    qa.running = true;
    const go = (i) => {
      if (i >= steps.length) {
        qa.running = false;
        return;
      }
      const [name, fn, delay = 250] = steps[i];
      let err = null;
      try {
        fn();
      } catch (e) {
        err = String(e);
      }
      setTimeout(() => {
        qa.mark(name, err ? { error: err } : undefined);
        go(i + 1);
      }, delay);
    };
    go(0);
    return `running ${steps.length} steps`;
  };

  // --------------------------------------------------------- diagnostics
  /** Why does an element not follow the theme? Computed vs variable value. */
  qa.diagTheme = () => {
    const probe = (label, el) => {
      if (!el) return { label, missing: true };
      const s = getComputedStyle(el);
      return {
        label,
        bg: s.backgroundColor,
        color: s.color,
        surfaceVar: s.getPropertyValue("--color-surface").trim(),
      };
    };
    return {
      theme: document.documentElement.dataset.theme,
      body: probe("body", document.body),
      sidebar: probe("sidebar", $('[data-testid="sidebar"]')),
      references: probe(
        "references",
        [...document.querySelectorAll("aside")].find((a) => a.className.includes("border-l"))
      ),
      select: probe("select", $('[data-testid="scene-panel"] select')),
      prose: probe("prose", $(".novel-editor-content")),
      chapterTitle: probe("chapterTitle", $('[data-testid="chapter-title"]')),
    };
  };

  // ------------------------------------------------------ error capture
  /** console.error, window.onerror and unhandled rejections land here; q.state() counts them. */
  qa.errors = qa.errors || [];
  if (!qa._errorsHooked) {
    qa._errorsHooked = true;
    const origError = console.error.bind(console);
    console.error = (...args) => {
      const text = args.map(String).join(" ");
      if (!text.startsWith("TAURI-PLUGIN-MCP: Error executing JavaScript"))
        qa.errors.push({
          kind: "console.error",
          t: Date.now(),
          msg: args.map(String).join(" ").slice(0, 300),
        });
      origError(...args);
    };
    window.addEventListener("error", (e) =>
      qa.errors.push({ kind: "error", t: Date.now(), msg: String(e.message).slice(0, 300) })
    );
    window.addEventListener("unhandledrejection", (e) =>
      qa.errors.push({
        kind: "rejection",
        t: Date.now(),
        msg: String(e.reason?.message || e.reason).slice(0, 300),
      })
    );
  }
  /** Return and clear captured errors. Call at the end of every scenario. */
  qa.takeErrors = () => qa.errors.splice(0, qa.errors.length);

  // ------------------------------------------------- localStorage guard
  /** Snapshot every kindling:* key so a run can restore the user's real preferences. */
  qa.saveLocal = () => {
    const snap = {};
    for (const k of Object.keys(localStorage))
      if (k.startsWith("kindling:")) snap[k] = localStorage.getItem(k);
    sessionStorage.setItem("__qa_local", JSON.stringify(snap));
    return Object.keys(snap).length;
  };
  qa.restoreLocal = () => {
    const snap = JSON.parse(sessionStorage.getItem("__qa_local") || "null");
    if (!snap) return "no snapshot";
    for (const k of Object.keys(localStorage))
      if (k.startsWith("kindling:") && !(k in snap)) localStorage.removeItem(k);
    for (const [k, v] of Object.entries(snap)) localStorage.setItem(k, v);
    return Object.keys(snap).length;
  };

  // ------------------------------------------------------- failure dump
  /** Trimmed outerHTML of a region, for failure artifacts. */
  qa.dom = (selector = "main", limit = 20000) => {
    const el = document.querySelector(selector);
    if (!el) return `no element for ${selector}`;
    return el.outerHTML.replace(/\s+/g, " ").slice(0, limit);
  };

  // --------------------------------------------------- press audits
  const visible = (el) => {
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return false;
    const s = getComputedStyle(el);
    return s.visibility !== "hidden" && s.display !== "none" && s.opacity !== "0";
  };
  const parseRgb = (c) => {
    const m = c.match(/rgba?\(([^)]+)\)/);
    if (!m) return null;
    const [r, g, b, a = "1"] = m[1].split(",").map((x) => parseFloat(x));
    return { r, g, b, a: isNaN(a) ? 1 : a };
  };
  const lum = ({ r, g, b }) => {
    const f = (v) => {
      v /= 255;
      return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    };
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  };
  const effectiveBg = (el) => {
    let node = el;
    while (node && node !== document.documentElement) {
      const c = parseRgb(getComputedStyle(node).backgroundColor);
      if (c && c.a > 0.9) return c;
      node = node.parentElement;
    }
    return (
      parseRgb(getComputedStyle(document.body).backgroundColor) || { r: 255, g: 255, b: 255, a: 1 }
    );
  };
  const contrast = (fg, bg) => {
    const l1 = lum(fg),
      l2 = lum(bg);
    return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
  };
  /** Every colour the Press tokens resolve to right now, as rgb strings. */
  const tokenColours = () => {
    const root = getComputedStyle(document.documentElement);
    const names = new Set();
    for (const sh of document.styleSheets) {
      let rules;
      try {
        rules = sh.cssRules;
      } catch {
        continue;
      }
      const walk = (rs) => {
        for (const r of rs) {
          if (r.cssRules && !r.selectorText) walk(r.cssRules);
          if (r.style)
            for (const p of r.style)
              if (
                p.startsWith("--color-") ||
                p.startsWith("--paper") ||
                p.startsWith("--ink") ||
                p.startsWith("--raised") ||
                p.startsWith("--sunken") ||
                p.startsWith("--flame") ||
                p.startsWith("--ember") ||
                p.startsWith("--hair") ||
                p.startsWith("--tag-") ||
                /^--(green|red|amber|blue)/.test(p)
              )
                names.add(p);
        }
      };
      walk(rules);
    }
    const probe = document.createElement("div");
    document.body.append(probe);
    const out = new Set();
    for (const n of names) {
      probe.style.color = `var(${n})`;
      const c = getComputedStyle(probe).color;
      if (c) out.add(c.replace(/\s+/g, ""));
    }
    probe.remove();
    return out;
  };
  /**
   * Press design-system audit of everything currently visible. Returns counts
   * and samples; treat a non-empty `fonts`, `measure` or `contrast` list as a
   * visual regression, and `untokened` as a lead to check by hand.
   */
  qa.audit = (root = document) => {
    const allowedFonts = /inter|newsreader|fraunces|ui-monospace|menlo|monaco|consolas|monospace/i;
    const fonts = [],
      contrastFails = [],
      untokened = new Map();
    const tokens = tokenColours();
    const els = [...root.querySelectorAll("body *")].filter(
      (el) =>
        ![
          "SCRIPT",
          "STYLE",
          "SVG",
          "PATH",
          "LINE",
          "CIRCLE",
          "RECT",
          "POLYLINE",
          "POLYGON",
        ].includes(el.tagName) && visible(el)
    );
    for (const el of els) {
      const s = getComputedStyle(el);
      const hasText = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
      if (hasText && !allowedFonts.test(s.fontFamily))
        fonts.push({
          tag: el.tagName,
          cls: (el.className || "").toString().slice(0, 60),
          font: s.fontFamily.slice(0, 60),
        });
      if (hasText) {
        const fg = parseRgb(s.color),
          bg = effectiveBg(el);
        if (fg && fg.a > 0.9) {
          const ratio = contrast(fg, bg);
          const large =
            parseFloat(s.fontSize) >= 18.66 ||
            (parseFloat(s.fontSize) >= 14 && parseInt(s.fontWeight) >= 700);
          if (ratio < (large ? 3 : 4.5))
            contrastFails.push({
              text: el.textContent.trim().slice(0, 40),
              ratio: +ratio.toFixed(2),
              fg: s.color,
              bg: `rgb(${bg.r}, ${bg.g}, ${bg.b})`,
              cls: (el.className || "").toString().slice(0, 60),
            });
        }
        const key = s.color.replace(/\s+/g, "");
        if (!tokens.has(key) && !/rgba\(0,0,0,0\)/.test(key))
          untokened.set(key, (untokened.get(key) || 0) + 1);
      }
      const bgKey = s.backgroundColor.replace(/\s+/g, "");
      if (bgKey !== "rgba(0,0,0,0)" && !tokens.has(bgKey) && !bgKey.startsWith("rgba"))
        untokened.set(bgKey, (untokened.get(bgKey) || 0) + 1);
    }
    const measure = [];
    for (const ed of root.querySelectorAll(
      ".novel-editor-content, .prose, [class*='max-w-measure']"
    )) {
      const max = parseFloat(getComputedStyle(ed).maxWidth);
      const w = ed.getBoundingClientRect().width;
      if (!isNaN(max) && w > max + 1)
        measure.push({
          cls: (ed.className || "").toString().slice(0, 60),
          width: Math.round(w),
          max: Math.round(max),
        });
    }
    const dedupe = (arr, key) => [...new Map(arr.map((x) => [key(x), x])).values()];
    return {
      theme: document.documentElement.dataset.theme,
      scanned: els.length,
      fonts: dedupe(fonts, (f) => f.font + f.cls).slice(0, 10),
      contrast: dedupe(contrastFails, (c) => c.fg + c.bg).slice(0, 10),
      measure,
      untokened: [...untokened.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, 8)
        .map(([c, n]) => `${c} x${n}`),
      tokenCount: tokens.size,
    };
  };
  /** Elements whose content overflows horizontally, plus document-level overflow. Use in the narrow sweep. */
  qa.overflow = () => {
    const doc = document.documentElement;
    const offenders = [...document.querySelectorAll("body *")]
      .filter(
        (el) =>
          visible(el) &&
          el.scrollWidth > el.clientWidth + 2 &&
          !/auto|scroll/.test(getComputedStyle(el).overflowX)
      )
      .map((el) => ({
        tag: el.tagName,
        cls: (el.className || "").toString().slice(0, 60),
        scroll: el.scrollWidth,
        client: el.clientWidth,
      }))
      .slice(0, 12);
    return {
      documentOverflow: doc.scrollWidth > doc.clientWidth,
      viewport: [innerWidth, innerHeight],
      offenders,
    };
  };

  // ----------------------------------------------------------- axe-core
  /**
   * Load axe-core from node_modules via Vite and run it. Async: result lands in
   * q.axeResult and #qa-axe-done is appended; wait_for that, then read.
   */
  qa.axe = (context = document, tags = ["wcag2a", "wcag2aa"]) => {
    document.getElementById("qa-axe-done")?.remove();
    qa.axeResult = { status: "pending" };
    const run = () =>
      window.axe
        .run(context, { runOnly: { type: "tag", values: tags } })
        .then((r) => {
          qa.axeResult = {
            status: "ok",
            violations: r.violations.map((v) => ({
              id: v.id,
              impact: v.impact,
              help: v.help,
              nodes: v.nodes.length,
              sample: v.nodes[0]?.target?.[0],
            })),
            passes: r.passes.length,
          };
        })
        .catch((e) => (qa.axeResult = { status: "error", error: String(e) }))
        .finally(
          () =>
            qa.done("axe".replace("qa-done-", "")) &&
            document.getElementById("qa-done-axe") &&
            (document.getElementById("qa-done-axe").id = "qa-axe-done")
        );
    if (window.axe) run();
    else {
      const s = document.createElement("script");
      s.src = "/@fs/" + qa.repoRoot + "/node_modules/axe-core/axe.min.js";
      s.onload = run;
      s.onerror = () => {
        qa.axeResult = { status: "error", error: "axe-core failed to load; run npm install" };
        qa.done("axe");
        document.getElementById("qa-done-axe").id = "qa-axe-done";
      };
      document.head.append(s);
    }
    return "axe started";
  };
  qa.repoRoot =
    qa.repoRoot ||
    (
      [...document.scripts]
        .map((s) => s.src)
        .find((src) => src.includes("/qa/visual/harness.js")) || ""
    )
      .replace(/^.*\/@fs\//, "/")
      .replace(/\/qa\/visual\/harness\.js.*$/, "");

  // ------------------------------------------------------- pixel diff
  /**
   * Compare screenshots in a run folder against qa/visual/baselines using the
   * webview as the image engine (both served by Vite via /@fs/). Async: result
   * in q.diffResult, #qa-diff-done appended when finished.
   *   q.diff("<abs run dir>", [{ name: "01-01-chapter-input", file: "screenshot_123.jpg" }, ...])
   * `mismatch` is the share of pixels differing by more than `tolerance` per
   * channel (JPEG noise is ~10-20); `changed` flags mismatch above `threshold`.
   */
  qa.diff = (runDir, pairs, { tolerance = 24, threshold = 0.004 } = {}) => {
    document.getElementById("qa-diff-done")?.remove();
    qa.diffResult = { status: "pending", results: [] };
    const load = (src) =>
      new Promise((res, rej) => {
        const img = new Image();
        img.onload = () => res(img);
        img.onerror = () => rej(new Error("load failed: " + src));
        img.src = src;
      });
    const draw = (img) => {
      const c = document.createElement("canvas");
      c.width = img.naturalWidth;
      c.height = img.naturalHeight;
      c.getContext("2d").drawImage(img, 0, 0);
      return c;
    };
    const one = async ({ name, file }) => {
      const base = "/@fs/" + qa.repoRoot + "/qa/visual/baselines/" + name + ".jpg";
      const cur = "/@fs" + runDir + "/" + file;
      let a, b;
      try {
        a = await load(base);
      } catch {
        return { name, status: "no-baseline" };
      }
      try {
        b = await load(cur);
      } catch {
        return { name, status: "missing-capture" };
      }
      if (a.naturalWidth !== b.naturalWidth || a.naturalHeight !== b.naturalHeight)
        return {
          name,
          status: "size-mismatch",
          baseline: [a.naturalWidth, a.naturalHeight],
          current: [b.naturalWidth, b.naturalHeight],
        };
      const A = draw(a).getContext("2d").getImageData(0, 0, a.naturalWidth, a.naturalHeight).data;
      const B = draw(b).getContext("2d").getImageData(0, 0, b.naturalWidth, b.naturalHeight).data;
      let bad = 0,
        minX = 1e9,
        minY = 1e9,
        maxX = -1,
        maxY = -1;
      const w = a.naturalWidth;
      for (let i = 0; i < A.length; i += 4) {
        if (
          Math.abs(A[i] - B[i]) > tolerance ||
          Math.abs(A[i + 1] - B[i + 1]) > tolerance ||
          Math.abs(A[i + 2] - B[i + 2]) > tolerance
        ) {
          bad++;
          const p = i / 4,
            x = p % w,
            y = (p - x) / w;
          if (x < minX) minX = x;
          if (x > maxX) maxX = x;
          if (y < minY) minY = y;
          if (y > maxY) maxY = y;
        }
      }
      const mismatch = bad / (A.length / 4);
      return {
        name,
        status: "compared",
        mismatch: +mismatch.toFixed(5),
        changed: mismatch > threshold,
        box: bad ? [minX, minY, maxX, maxY] : null,
      };
    };
    (async () => {
      for (const p of pairs) qa.diffResult.results.push(await one(p));
      qa.diffResult.status = "ok";
      qa.diffResult.changed = qa.diffResult.results
        .filter((r) => r.changed || (r.status !== "compared" && r.status !== "no-baseline"))
        .map((r) => r.name);
      const d = document.createElement("div");
      d.id = "qa-diff-done";
      d.hidden = true;
      document.body.append(d);
    })();
    return `diffing ${pairs.length}`;
  };

  return "harness loaded: " + Object.keys(qa).length + " helpers";
})();
