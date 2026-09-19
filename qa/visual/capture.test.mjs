import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CAPTURE_PROFILE, SocketCapture, pngSize, validateSnapshot } from "./capture.mjs";
import { acquireLock } from "./lock.mjs";

const viewport = { width: 1600, height: 968 };
// A header fixture suffices for arithmetic tests; live validation decodes real PNGs.
function result() {
  const bytes = Buffer.alloc(24);
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(bytes);
  bytes.write("IHDR", 12);
  bytes.writeUInt32BE(3200, 16);
  bytes.writeUInt32BE(1936, 20);
  return {
    profile: CAPTURE_PROFILE,
    engine: "WKWebView.takeSnapshot",
    scale: 2,
    viewport: { ...viewport, dpr: 1 },
    pixels: [3200, 1936],
    windowVisible: false,
    windowFocused: false,
    pngBase64: bytes.toString("base64"),
  };
}
test("2x rendering accepts a hidden, unfocused webview with 1x backing density", () => {
  assert.deepEqual(pngSize(validateSnapshot(result(), viewport)), [3200, 1936]);
});
test("rejects wrong profile, lossy data, resized output and visible windows", () => {
  for (const change of [
    { profile: "macos-webview-2x-png" },
    { engine: "xcap" },
    { scale: 1 },
    { windowVisible: true },
    { windowFocused: true },
    { pngBase64: "not a png" },
    { pixels: [1600, 968] },
    { viewport: { width: 1100, height: 668 } },
  ])
    assert.throws(() => validateSnapshot({ ...result(), ...change }, viewport));
  const bad = result();
  const bytes = Buffer.from(bad.pngBase64, "base64");
  bytes.writeUInt32BE(1600, 16);
  bad.pngBase64 = bytes.toString("base64");
  assert.throws(() => validateSnapshot(bad, viewport), /expected/);
});
test("capture uses only socket snapshot and lossless PNG master, with no focus/desktop commands", async () => {
  const dir = mkdtempSync(join(tmpdir(), "qa-capture-"));
  const calls = [],
    commands = [];
  const app = {
    evaluate: async () => viewport,
    invoke: async (command, args) => {
      calls.push({ command, args });
      return result();
    },
  };
  try {
    const capture = new SocketCapture(app, dir, (command) => commands.push(command));
    assert.equal(await capture.capture("00-03-test"), "00-03-test.png");
    assert.deepEqual(calls, [{ command: "qa_snapshot", args: { ...viewport, scale: 2 } }]);
    assert.deepEqual(commands, ["oxipng", "/usr/bin/sips"]);
    assert.deepEqual(
      readFileSync(join(dir, "00-03-test.png")),
      Buffer.from(result().pngBase64, "base64")
    );
    assert.equal(capture.metadata.get("00-03-test").pngBase64, undefined);
    app.invoke = async () => {
      throw new Error("snapshot timeout");
    };
    await assert.rejects(capture.capture("failure"), /snapshot timeout/);
    assert.equal(commands.length, 2);
    await assert.rejects(capture.capture("../unsafe"), /Invalid/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
test("a concurrent runner cannot take a lock or remove its owner's record", () => {
  const dir = mkdtempSync(join(tmpdir(), "qa-lock-")),
    path = join(dir, "run.lock");
  try {
    const release = acquireLock(path);
    assert.throws(() => acquireLock(path), /QA lock exists/);
    release();
    const releaseNext = acquireLock(path);
    release();
    assert.throws(() => acquireLock(path), /QA lock exists/);
    releaseNext();
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
