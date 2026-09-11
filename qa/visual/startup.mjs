// Production startup regression: first paint precedes the app bundle, and the
// overlay survives slow content loading. Native window visibility is checked in
// the desktop smoke test; this harness supplies only the IPC needed by launch.
import assert from "node:assert/strict";
import { readFile, mkdir } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve, extname } from "node:path";
import { chromium } from "playwright";

const root = resolve("dist");
const output = resolve("qa/visual/results/startup");
await mkdir(output, { recursive: true });
const mime = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
};
const server = createServer(async (request, response) => {
  const pathname = new URL(request.url, "http://localhost").pathname;
  const file = resolve(root, `.${pathname === "/" ? "/index.html" : pathname}`);
  if (!file.startsWith(`${root}/`)) {
    response.writeHead(403).end();
    return;
  }
  try {
    const body = await readFile(file);
    response
      .writeHead(200, { "Content-Type": mime[extname(file)] || "application/octet-stream" })
      .end(body);
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
const url = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
  browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
  async function pageFor(context, preference, holdProjects = true, review = null) {
    const page = await context.newPage();
    await page.addInitScript(
      ({ preference, holdProjects, review }) => {
        let filesTaken = false;
        if (preference) localStorage.setItem("kindling:theme", preference);
        localStorage.setItem("kindling:onboardingCompleted", "true");
        localStorage.setItem("kindling:guidanceEnabled", "false");
        window.isTauri = false;
        window.__TAURI_INTERNALS__ = {
          metadata: {
            currentWindow: { label: "main" },
            currentWebview: { label: "main", windowLabel: "main" },
          },
          transformCallback: () => 1,
          invoke: async (command) => {
            if (command === "get_recent_projects" && holdProjects)
              return new Promise((resolve) => {
                window.__releaseProjects = () => resolve([]);
              });
            if (command === "take_editorial_open_files" && review && !filesTaken) {
              filesTaken = true;
              return ["/startup.kindling-review"];
            }
            if (command === "open_editorial_package") return review;
            if (command === "plugin:event|listen") return 1;
            if (command === "plugin:os|platform") return "macos";
            return [];
          },
        };
      },
      { preference, holdProjects, review }
    );
    return page;
  }
  for (const [preference, scheme, motion] of [
    [null, "light", "no-preference"],
    ["dark", "light", "no-preference"],
    ["system", "dark", "no-preference"],
    ["system", "light", "no-preference"],
    ["dark", "dark", "reduce"],
  ]) {
    const context = await browser.newContext({
      viewport: { width: 1000, height: 700 },
      colorScheme: scheme,
      reducedMotion: motion,
    });
    const page = await pageFor(context, preference);
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    let releaseBundle;
    const bundleGate = new Promise((done) => {
      releaseBundle = done;
    });
    let mainRequestTime;
    const cdp = await context.newCDPSession(page);
    await cdp.send("Network.enable");
    cdp.on("Network.requestWillBeSent", (event) => {
      if (/\/assets\/main-.*\.js$/.test(event.request.url)) mainRequestTime = event.wallTime * 1000;
    });
    await page.route("**/assets/main-*.js", async (route) => {
      await bundleGate;
      await route.continue();
    });
    await page.goto(url, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(
      () => performance.getEntriesByName("first-contentful-paint").length > 0
    );
    assert(await page.locator("#startup-loading").isVisible());
    assert.equal(await page.locator("#app").innerHTML(), "");
    const before = await page.evaluate(() => ({
      theme: document.documentElement.dataset.theme,
      bg: getComputedStyle(document.querySelector("#startup-loading")).backgroundColor,
      animation: getComputedStyle(document.querySelector(".startup-spinner")).animationName,
      transform: getComputedStyle(document.querySelector(".startup-spinner")).transform,
      firstPaint: performance.getEntriesByName("first-contentful-paint")[0].startTime,
      origin: performance.timeOrigin,
    }));
    const expected = preference === "system" ? scheme : preference || "light";
    assert.equal(before.theme, expected);
    assert.equal(before.bg, expected === "dark" ? "rgb(30, 26, 22)" : "rgb(244, 239, 230)");
    await page.waitForTimeout(150);
    const transform = await page
      .locator(".startup-spinner")
      .evaluate((el) => getComputedStyle(el).transform);
    if (motion === "reduce") assert.equal(before.animation, "none");
    else {
      assert.equal(before.animation, "startup-spin");
      assert.notEqual(before.transform, transform);
    }
    await page.screenshot({ path: `${output}/${preference || "default"}-${scheme}-${motion}.png` });
    releaseBundle();
    await page.waitForFunction(() => typeof window.__releaseProjects === "function");
    assert(await page.locator("#startup-loading").isVisible());
    assert(await page.locator("#app").evaluate((el) => el.inert));
    assert(
      await page.evaluate(
        () =>
          !!document.elementFromPoint(innerWidth / 2, innerHeight / 2).closest("#startup-loading")
      )
    );
    await page.evaluate(() => window.__releaseProjects());
    await page.locator("#startup-loading").waitFor({ state: "detached" });
    assert.equal(await page.locator("#app").evaluate((el) => el.inert), false);
    assert.equal(await page.locator("#app main").count(), 1);
    assert.deepEqual(errors, []);
    // Capture request creation, not the later network send after our route gate.
    const bundleStart = mainRequestTime - before.origin;
    assert(
      before.firstPaint <= bundleStart,
      `First paint ${before.firstPaint}ms must precede app request ${bundleStart}ms`
    );
    console.log(
      JSON.stringify({
        preference,
        scheme,
        motion,
        firstPaint: before.firstPaint,
        bundleStart,
        pass: true,
      })
    );
    await context.close();
  }
  const reviewContext = await browser.newContext();
  const reviewPage = await pageFor(reviewContext, "light", false, {
    format: "kindling-editorial",
    version: 1,
    kind: "review",
    session: null,
    round: {
      id: "startup-round",
      project_id: "startup-project",
      title: "Startup review",
      name: "First review",
      brief: "Check keyboard focus",
      created_at: "2026-09-10",
      sources: [
        {
          id: "source",
          scene_id: "scene",
          chapter_id: "chapter",
          chapter: "Chapter One",
          scene: "The Letter",
          mode: "page",
          html: "<p>Eleanor opened the letter.</p>",
          locked: false,
        },
      ],
    },
  });
  await reviewPage.goto(url, { waitUntil: "domcontentloaded" });
  await reviewPage.locator("#startup-loading").waitFor({ state: "detached" });
  const workspace = reviewPage.getByRole("region", { name: "Editorial workspace" });
  assert(await workspace.isVisible());
  assert(await workspace.evaluate((el) => document.activeElement === el));
  await reviewPage.keyboard.press("ControlOrMeta+f");
  const search = reviewPage.getByRole("searchbox", { name: "Find in manuscript" });
  await search.waitFor();
  assert(await search.evaluate((el) => document.activeElement === el));
  console.log("Document-open startup keyboard focus: pass");
  await reviewContext.close();

  const context = await browser.newContext();
  const page = await pageFor(context, "light", false);
  let fail = true;
  await page.route("**/assets/main-*.js", (route) => {
    if (fail) {
      fail = false;
      return route.abort();
    }
    return route.continue();
  });
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await page.getByText("Kindling couldn’t start. Please try again.").waitFor();
  assert.equal(await page.locator(".startup-spinner").isVisible(), false);
  await page.getByRole("button", { name: "Try again" }).click();
  await page.locator("#startup-loading").waitFor({ state: "detached" });
  assert.equal(await page.locator("#app main").count(), 1);
  console.log("Failed-bundle recovery: pass");
  await context.close();
} finally {
  await browser?.close();
  await new Promise((done) => server.close(done));
}
