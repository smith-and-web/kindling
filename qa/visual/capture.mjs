import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const CAPTURE_PROFILE = "wkwebview-snapshot-2x-png-v1";
const sh = (command, args) =>
  execFileSync(command, args, { encoding: "utf8", timeout: 20000 }).trim();

export function pngSize(bytes) {
  if (
    bytes.length < 24 ||
    !bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])) ||
    bytes.toString("ascii", 12, 16) !== "IHDR"
  )
    throw new Error("Capture is not a PNG");
  return [bytes.readUInt32BE(16), bytes.readUInt32BE(20)];
}

export function validateDensity(size, viewport) {
  if (size[0] !== viewport.width * 2 || size[1] !== viewport.height * 2)
    throw new Error(
      `Capture is ${size.join("×")}; expected ${viewport.width * 2}×${viewport.height * 2} rendered pixels`
    );
}

export function validateSnapshot(result, viewport) {
  if (
    result?.profile !== CAPTURE_PROFILE ||
    result.engine !== "WKWebView.takeSnapshot" ||
    result.scale !== 2
  )
    throw new Error("Unsupported snapshot profile; no desktop/JPEG fallback");
  if (result.windowVisible !== false || result.windowFocused !== false)
    throw new Error("The isolated QA window must remain hidden and unfocused");
  if (result.viewport?.width !== viewport.width || result.viewport?.height !== viewport.height)
    throw new Error("Snapshot viewport changed");
  if (typeof result.pngBase64 !== "string" || result.pngBase64.length > 100_000_000)
    throw new Error("Missing or oversized PNG payload");
  const bytes = Buffer.from(result.pngBase64, "base64");
  const pixels = pngSize(bytes);
  validateDensity(pixels, viewport);
  if (JSON.stringify(pixels) !== JSON.stringify(result.pixels))
    throw new Error("PNG dimensions disagree with snapshot metadata");
  return bytes;
}

export class SocketCapture {
  constructor(app, output, command = sh) {
    this.app = app;
    this.output = output;
    this.command = command;
    this.metadata = new Map();
  }
  async preflight() {
    const caps = await this.app.invoke("qa_capabilities");
    if (caps?.profile !== CAPTURE_PROFILE || !caps.background || caps.format !== "png")
      throw new Error("Start the isolated debug app with npm run tauri:qa");
    this.command("/usr/bin/which", ["oxipng"]);
    if (!existsSync("/usr/bin/sips")) throw new Error("macOS sips is required for review previews");
    return caps;
  }
  async capture(name) {
    if (!/^[a-zA-Z0-9_-]+$/.test(name)) throw new Error("Invalid capture name");
    const viewport = await this.app.evaluate("return {width:innerWidth,height:innerHeight};");
    const result = await this.app.invoke("qa_snapshot", { ...viewport, scale: 2 });
    const bytes = validateSnapshot(result, viewport);
    const file = `${name}.png`,
      target = join(this.output, file);
    writeFileSync(target, bytes);
    this.command("oxipng", ["-o", "2", "--strip", "safe", "-q", target]);
    validateDensity(pngSize(readFileSync(target)), viewport);
    const preview = `${name}-preview.jpg`;
    // Only the review preview is resized/compressed; the PNG master is lossless.
    this.command("/usr/bin/sips", [
      "-s",
      "format",
      "jpeg",
      "-s",
      "formatOptions",
      "70",
      "--resampleWidth",
      "800",
      target,
      "--out",
      join(this.output, preview),
    ]);
    const { pngBase64: _png, ...metadata } = result;
    Object.assign(metadata, { file, preview });
    writeFileSync(join(this.output, `${name}.capture.json`), JSON.stringify(metadata, null, 2));
    this.metadata.set(name, metadata);
    return file;
  }
}
