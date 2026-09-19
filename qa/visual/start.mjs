#!/usr/bin/env node
import { release as osRelease } from "node:os";
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, realpathSync } from "node:fs";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { SocketClient } from "./socket.mjs";
import { CAPTURE_PROFILE } from "./capture.mjs";
import { acquireLock } from "./lock.mjs";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const children = new Set();
let interrupted = false;
function child(command, args, options = {}) {
  const p = spawn(command, args, { cwd: repo, stdio: "inherit", ...options });
  children.add(p);
  const done = new Promise((resolve, reject) => {
    p.once("error", reject);
    p.once("exit", (code, signal) => {
      children.delete(p);
      if (code === 0 || interrupted) resolve();
      else reject(new Error(`${command} exited with ${signal || code}`));
    });
  });
  return { p, done };
}
async function devAvailable() {
  try {
    const response = await fetch(`http://localhost:1420/@fs${repo}/src/startup.ts`, {
      signal: AbortSignal.timeout(1000),
    });
    return response.ok && (await response.text()).includes("startup");
  } catch {
    return false;
  }
}
async function main() {
  if (process.platform !== "darwin" || Number(osRelease().split(".")[0]) < 23)
    throw new Error("Background WKWebView QA currently requires macOS 14+");
  let release = () => {};
  const stop = () => {
    interrupted = true;
    for (const p of children) p.kill("SIGTERM");
  };
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, stop);
  try {
    const existing = new SocketClient();
    let connected = false;
    try {
      await existing.connect();
      connected = true;
      await existing.wait(
        "window.__KINDLING_TEST__ && !document.getElementById('startup-loading')",
        20000
      );
      const caps = await existing.invoke("qa_capabilities");
      if (caps.profile !== CAPTURE_PROFILE || caps.checkout !== repo)
        throw new Error("Existing QA app has an incompatible capture profile; restart it");
      console.log(`Reusing hidden QA app at ${existing.path}. Run npm run qa:visual.`);
      return;
    } catch (error) {
      if (connected || !["ENOENT", "ECONNREFUSED"].includes(error.code)) throw error;
    } finally {
      existing.close();
    }
    release = acquireLock(`${new SocketClient().path}.start.lock`);
    if (!(await devAvailable())) {
      const dev = child(process.execPath, [
        join(repo, "node_modules/vite/bin/vite.js"),
        "--host",
        "localhost",
        "--port",
        "1420",
        "--strictPort",
      ]);
      let failed;
      dev.done.catch((error) => {
        failed = error;
      });
      const deadline = Date.now() + 30000;
      while (!(await devAvailable())) {
        if (failed) throw failed;
        if (interrupted) return;
        if (Date.now() > deadline) throw new Error("Vite did not become ready on port 1420");
        await new Promise((resolve) => setTimeout(resolve, 200));
      }
    } else {
      const inspect = (args) =>
        execFileSync("/usr/sbin/lsof", args, {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
          timeout: 5000,
        });
      const pids = inspect(["-nP", "-iTCP:1420", "-sTCP:LISTEN", "-t"]).trim().split(/\s+/);
      const roots = pids.flatMap((pid) =>
        inspect(["-a", "-p", pid, "-d", "cwd", "-Fn"])
          .split("\n")
          .filter((line) => line.startsWith("n"))
          .map((line) => realpathSync(line.slice(1)))
      );
      if (!roots.length || roots.some((root) => root !== realpathSync(repo)))
        throw new Error("Port 1420 belongs to another checkout; use this checkout's Vite server");
      console.log("Reusing Vite on port 1420; the normal Kindling app can stay open.");
    }
    await child("cargo", ["build", "--manifest-path", "src-tauri/Cargo.toml"]).done;
    if (interrupted) return;
    const data = join(repo, "qa/visual/data/background-profile");
    mkdirSync(data, { recursive: true });
    console.log(
      "Starting hidden QA app at /tmp/kindling-qa.sock. Run npm run qa:visual in another terminal."
    );
    await child(join(repo, "src-tauri/target/debug/kindling"), [], {
      env: { ...process.env, KINDLING_QA_BACKGROUND: "1", KINDLING_DATA_DIR: data },
    }).done;
  } finally {
    stop();
    for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.off(signal, stop);
    release();
  }
}
main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
