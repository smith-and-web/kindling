// Keep this entry point small: no Svelte, app stores or editor imports before
// the loading screen has had a chance to paint.
import { isTauri } from "@tauri-apps/api/core";
import { backgroundQA, installBackgroundFrames } from "./lib/qaMode";

installBackgroundFrames();

function paintOpportunity(): Promise<void> {
  // rAF runs BEFORE paint. Two frames leave a rendering opportunity between
  // them; browsers do not expose a portable "pixels presented" event.
  return new Promise((resolve) => {
    window.requestAnimationFrame(() => window.requestAnimationFrame(() => resolve()));
  });
}

async function loadingPaint() {
  if (backgroundQA()) {
    await document.fonts.ready;
    return;
  }
  if (
    typeof PerformanceObserver !== "undefined" &&
    PerformanceObserver.supportedEntryTypes.includes("paint")
  ) {
    // Chromium reports the actual first contentful paint, including one that
    // happened before this small module evaluated.
    await new Promise<void>((resolve) => {
      const observer = new PerformanceObserver((entries) => {
        if (entries.getEntries().some((entry) => entry.name === "first-contentful-paint")) {
          observer.disconnect();
          resolve();
        }
      });
      observer.observe({ type: "paint", buffered: true });
    });
  } else {
    // Without Paint Timing, let initial layout request its fonts,
    // then give the complete shell another rendering opportunity.
    await paintOpportunity();
    await document.fonts.ready;
    await paintOpportunity();
  }
}

async function start() {
  try {
    // Native page-load handling reveals this shell independently of JS success.
    // No top-level await: the page-load event must be free to complete while
    // this promise waits for the first visible frame.
    await loadingPaint();

    // MCP guest listeners must precede Svelte, but must not delay the first paint.
    if (import.meta.env.DEV && isTauri()) {
      const { setupPluginListeners } = await import("tauri-plugin-mcp");
      await setupPluginListeners();
    }

    const { ready, focusAfterStartup } = await import("./main");
    await ready;
    await document.fonts.ready;
    await Promise.all(Array.from(document.images, (image) => image.decode().catch(() => {})));
    await paintOpportunity();

    const app = document.getElementById("app")!;
    app.inert = false;
    app.removeAttribute("aria-busy");
    document.getElementById("startup-loading")?.remove();
    focusAfterStartup();
  } catch (error) {
    console.error("Kindling could not start:", error);
    document.getElementById("startup-loading")?.setAttribute("role", "alert");
    const message = document.getElementById("startup-message");
    if (message) message.textContent = "Kindling couldn’t start. Please try again.";
    const spinner = document.querySelector<HTMLElement>(".startup-spinner");
    if (spinner) spinner.hidden = true;
    const retry = document.getElementById("startup-retry");
    if (retry) {
      retry.hidden = false;
      retry.onclick = () => window.location.reload();
    }
  }
}

export const startup = start();
