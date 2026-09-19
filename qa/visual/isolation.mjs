#!/usr/bin/env node
// Opt-in native proof: a hidden canvas larger than the desktop, fresh red/blue
// renders, exact lossless pixels, bad-request rejection, and viewport cleanup.
import { mkdirSync, writeFileSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { SocketClient } from "./socket.mjs";
import { validateSnapshot, CAPTURE_PROFILE } from "./capture.mjs";
import { acquireLock } from "./lock.mjs";
const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const app = new SocketClient();
const release = acquireLock(`${app.path}.run.lock`);
const output = join(
  repo,
  "qa/visual/results",
  `${new Date().toISOString().replace(/[:.]/g, "-")}-isolation`
);
mkdirSync(output, { recursive: true });
const evidence = { profile: CAPTURE_PROFILE, captures: [] };
let original,
  interrupted = false;
const interrupt = () => {
  interrupted = true;
};
for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, interrupt);
try {
  await app.connect();
  evidence.capabilities = await app.invoke("qa_capabilities");
  if (evidence.capabilities.profile !== CAPTURE_PROFILE || evidence.capabilities.checkout !== repo)
    throw new Error("Wrong capture profile");
  if ((await app.invoke("get_all_projects")).length)
    throw new Error("Isolation probe requires an empty QA library");
  original = await app.evaluate("return {width:innerWidth,height:innerHeight}");
  const viewport = { width: 2400, height: 1600 };
  await app.invoke("qa_set_viewport", viewport);
  await app.wait("innerWidth===2400 && innerHeight===1600");
  await app.evaluate(`const probe=document.createElement('div');probe.id='qa-isolation-probe';
    probe.style.cssText='position:fixed;inset:0;z-index:2147483647;pointer-events:none';document.body.append(probe);`);
  for (const [color, expected] of [
    ["red", [255, 0, 0, 255]],
    ["blue", [0, 0, 255, 255]],
  ]) {
    if (interrupted) throw new Error("Isolation probe interrupted");
    await app.evaluate(
      `document.getElementById('qa-isolation-probe').style.background=${JSON.stringify(color)};`
    );
    const result = await app.invoke("qa_snapshot", { ...viewport, scale: 2 });
    const path = join(output, `${color}.png`);
    writeFileSync(path, validateSnapshot(result, viewport));
    await app.evaluate(`window.__qaProbePixel={done:false};const im=new Image();
      im.onload=()=>{const c=document.createElement('canvas');c.width=c.height=1;
        const ctx=c.getContext('2d');ctx.drawImage(im,-100,-100);
        window.__qaProbePixel={done:true,rgba:[...ctx.getImageData(0,0,1,1).data]};};
      im.onerror=()=>window.__qaProbePixel={done:true,error:'PNG decode failed'};
      im.src=${JSON.stringify(`/@fs${path}`)};`);
    await app.wait("window.__qaProbePixel.done");
    const actual = await app.evaluate("return window.__qaProbePixel");
    if (JSON.stringify(actual.rgba) !== JSON.stringify(expected))
      throw new Error(`Stale/incorrect ${color} render: ${JSON.stringify(actual)}`);
    const { pngBase64, ...metadata } = result;
    evidence.captures.push({ ...metadata, file: `${color}.png`, rgba: actual.rgba });
  }
  for (const args of [
    { width: 1, height: 968, scale: 2 },
    { width: 1600, height: 968, scale: 1 },
  ]) {
    let rejected = false;
    try {
      await app.invoke("qa_snapshot", args);
    } catch {
      rejected = true;
    }
    if (!rejected) throw new Error("Invalid snapshot parameters were accepted");
  }
  evidence.invalidRequestsRejected = true;
  const session = execFileSync("/usr/sbin/ioreg", ["-n", "Root", "-d1"], { encoding: "utf8" });
  evidence.sessionLocked = /"CGSSessionScreenIsLocked"\s*=\s*Yes/.test(session);
  if (interrupted) throw new Error("Isolation probe interrupted");
  evidence.status = "passed";
} catch (error) {
  evidence.error = String(error.stack || error);
  process.exitCode = 1;
} finally {
  try {
    if (original) {
      await app.evaluate(
        "document.getElementById('qa-isolation-probe')?.remove();delete window.__qaProbePixel;"
      );
      await app.invoke("qa_set_viewport", original);
      evidence.cleanup = "probe removed; viewport restored";
    }
  } catch (error) {
    evidence.cleanup = String(error);
    process.exitCode = 1;
  }
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.off(signal, interrupt);
  app.close();
  release();
  writeFileSync(join(output, "results.json"), JSON.stringify(evidence, null, 2));
  console.log(JSON.stringify(evidence, null, 2));
  console.log(output);
}
