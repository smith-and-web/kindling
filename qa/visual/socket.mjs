import net from "node:net";
import { readFileSync, existsSync } from "node:fs";
import { randomUUID } from "node:crypto";

// Same newline-delimited protocol as kindling-splash/scripts/capture-screenshots.mjs.
// No MCP server, SDK, browser driver, or agent round trips are involved.
export class SocketClient {
  constructor(
    socketPath = process.env.KINDLING_QA_SOCK || "/tmp/kindling-qa.sock",
    timeout = 20000
  ) {
    this.path = socketPath;
    this.timeout = timeout;
    this.pending = new Map();
    this.buffer = "";
  }

  async connect() {
    const tokenPath = `${this.path}.token`;
    this.token = existsSync(tokenPath) ? readFileSync(tokenPath, "utf8").trim() : undefined;
    await new Promise((resolve, reject) => {
      this.socket = net.createConnection(this.path, resolve);
      this.socket.setEncoding("utf8");
      this.socket.on("error", (error) => {
        reject(error);
        this.fail(error);
      });
      this.socket.on("close", () => this.fail(new Error("Kindling socket disconnected")));
      this.socket.on("data", (chunk) => this.receive(chunk));
    });
    return this;
  }

  receive(chunk) {
    this.buffer += chunk;
    let end;
    while ((end = this.buffer.indexOf("\n")) !== -1) {
      const line = this.buffer.slice(0, end);
      this.buffer = this.buffer.slice(end + 1);
      let message;
      try {
        message = JSON.parse(line);
      } catch {
        this.fail(new Error("Invalid JSON from Kindling socket"));
        this.socket.destroy();
        return;
      }
      const pending = this.pending.get(message.id);
      if (!pending) continue; // Timed-out response; never assign it to a new request.
      clearTimeout(pending.timer);
      this.pending.delete(message.id);
      if (message.success) pending.resolve(message.data);
      else pending.reject(new Error(message.error || "Socket command failed"));
    }
  }

  fail(error) {
    for (const entry of this.pending.values()) {
      clearTimeout(entry.timer);
      entry.reject(error);
    }
    this.pending.clear();
  }

  rpc(command, payload = {}) {
    if (!this.socket || this.socket.destroyed)
      return Promise.reject(new Error("Socket is not connected"));
    const id = randomUUID();
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`${command} timed out after ${this.timeout}ms`));
      }, this.timeout);
      this.pending.set(id, { resolve, reject, timer });
      this.socket.write(
        JSON.stringify({ command, payload, id, ...(this.token ? { authToken: this.token } : {}) }) +
          "\n"
      );
    });
  }

  async js(code) {
    const data = await this.rpc("execute_js", {
      code,
      window_label: "main",
      timeout_ms: this.timeout,
    });
    return data && typeof data === "object" && "result" in data ? data.result : data;
  }

  // Explicit envelope: undefined, thrown errors and rejected promises cannot look like success.
  async evaluate(code) {
    const raw = await this.js(`(() => { try { const value = (() => { ${code}\n })();
      return JSON.stringify({ok:true,value:value ?? null});
      } catch(e) { return JSON.stringify({ok:false,error:String(e) + "\\n" + (e.stack || "")}); } })()`);
    const result = JSON.parse(raw);
    if (!result.ok) throw new Error(result.error);
    return result.value;
  }

  async wait(expression, timeout = 10000) {
    const started = Date.now();
    while (Date.now() - started < timeout) {
      if (await this.evaluate(`return !!(${expression});`)) return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error(`Timed out waiting for ${expression}`);
  }

  async invoke(command, args = {}) {
    const key = `__qa_rpc_${randomUUID().replaceAll("-", "")}`;
    await this.evaluate(`window[${JSON.stringify(key)}] = {done:false};
      window.__KINDLING_TEST__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)})
        .then(value => window[${JSON.stringify(key)}] = {done:true,ok:true,value:value ?? null})
        .catch(error => window[${JSON.stringify(key)}] = {done:true,ok:false,error:String(error)});`);
    let phase = "completion";
    try {
      await this.wait(`window[${JSON.stringify(key)}]?.done`, this.timeout);
      phase = "result transfer";
      const result = await this.evaluate(`return window[${JSON.stringify(key)}];`);
      if (!result.ok) throw new Error(`${command}: ${result.error}`);
      return result.value;
    } catch (error) {
      throw new Error(`${command} (${phase}): ${error.message}`, { cause: error });
    } finally {
      await this.evaluate(`delete window[${JSON.stringify(key)}];`).catch(() => {});
    }
  }

  close() {
    this.fail(new Error("Socket closed"));
    this.socket?.destroy();
  }
}
