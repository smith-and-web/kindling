import assert from "node:assert/strict";
import { test } from "node:test";
import net from "node:net";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { SocketClient } from "./socket.mjs";
import { main, verdict } from "./run.mjs";

async function server(t, respond, timeout = 200) {
  const dir = mkdtempSync(join(tmpdir(), "qa-sock-"));
  const socket = join(dir, "app.sock");
  writeFileSync(socket + ".token", "test-only-token\n");
  const connections = new Set();
  const srv = net.createServer((connection) => {
    connections.add(connection);
    let buffer = "";
    connection.on("data", (data) => {
      buffer += data.toString();
      let end;
      while ((end = buffer.indexOf("\n")) !== -1) {
        const msg = JSON.parse(buffer.slice(0, end));
        buffer = buffer.slice(end + 1);
        respond(msg, connection);
      }
    });
  });
  await new Promise((resolve) => srv.listen(socket, resolve));
  const client = await new SocketClient(socket, timeout).connect();
  t.after(async () => {
    client.close();
    for (const c of connections) c.destroy();
    await new Promise((resolve) => srv.close(resolve));
    rmSync(dir, { recursive: true, force: true });
  });
  return client;
}

test("correlates split and coalesced out-of-order replies and sends the token", async (t) => {
  const requests = [];
  const client = await server(t, (msg, c) => {
    assert.equal(msg.authToken, "test-only-token");
    requests.push(msg);
    if (requests.length === 2) {
      const reply = requests
        .toReversed()
        .map((r) => JSON.stringify({ id: r.id, success: true, data: r.command }) + "\n")
        .join("");
      c.write(reply.slice(0, 13));
      setImmediate(() => c.write(reply.slice(13)));
    }
  });
  assert.deepEqual(await Promise.all([client.rpc("first"), client.rpc("second")]), [
    "first",
    "second",
  ]);
  assert.equal(client.pending.size, 0);
});

test("rejects command failure without treating an error payload as data", async (t) => {
  const client = await server(t, (msg, c) =>
    c.write(JSON.stringify({ id: msg.id, success: false, error: "denied" }) + "\n")
  );
  await assert.rejects(client.rpc("save"), /denied/);
});

test("disconnect rejects all outstanding requests immediately", async (t) => {
  const client = await server(t, (_msg, c) => c.destroy(), 5000);
  const responses = await Promise.allSettled([client.rpc("one"), client.rpc("two")]);
  assert.ok(
    responses.every((r) => r.status === "rejected" && /disconnected/.test(r.reason.message))
  );
  assert.equal(client.pending.size, 0);
});

test("timeout removes pending requests and a late reply cannot satisfy another call", async (t) => {
  let previous;
  const client = await server(
    t,
    (msg, c) => {
      if (!previous) {
        previous = msg;
        return;
      }
      c.write(JSON.stringify({ id: previous.id, success: true, data: "stale" }) + "\n");
      c.write(JSON.stringify({ id: msg.id, success: true, data: "fresh" }) + "\n");
    },
    30
  );
  await assert.rejects(client.rpc("slow"), /timed out/);
  assert.equal(await client.rpc("next"), "fresh");
});

test("malformed protocol fails outstanding work rather than silently timing out", async (t) => {
  const client = await server(t, (_msg, c) => c.write("invalid JSON\n"));
  await assert.rejects(client.rpc("read"), /Invalid JSON/);
});

test("rejects unsupported selection before connecting or creating a run", async () => {
  await assert.rejects(main(["--only", "14"]), /no socket implementation/);
  await assert.rejects(main(["--variants", "tiny"]), /Unknown variant/);
  await assert.rejects(main(["--only"]), /incomplete argument/);
});

test("missing, changed and unaudited captures cannot pass", () => {
  const good = {
    capture: "shot.jpg",
    audit: { scanned: 10, fonts: [], contrast: [], measure: [] },
    overflow: { documentOverflow: false },
    errors: [],
    axe: { status: "ok", violations: [] },
    diff: { status: "compared", changed: false },
  };
  assert.equal(verdict(good), "pass");
  assert.equal(verdict({ ...good, calibration: { status: "compared", changed: false } }), "pass");
  assert.equal(
    verdict({ ...good, calibration: { status: "compared", changed: true } }),
    "inconclusive"
  );
  assert.equal(verdict({ ...good, calibration: { status: "missing-capture" } }), "inconclusive");
  for (const status of ["no-baseline", "missing-capture", "size-mismatch"]) {
    assert.equal(verdict({ ...good, diff: { status } }), "inconclusive");
  }
  assert.equal(verdict({ ...good, diff: { status: "compared", changed: true } }), "inconclusive");
  assert.equal(verdict({ ...good, axe: { status: "error" } }), "inconclusive");
  assert.equal(verdict({ ...good, audit: undefined }), "inconclusive");
  assert.equal(
    verdict({ ...good, error: "blank screenshot", errorKind: "capture" }),
    "inconclusive"
  );
  assert.equal(
    verdict({ ...good, error: "missing control", errorKind: "assertion" }),
    "DOM regression"
  );
  assert.equal(verdict({ ...good, errors: [{ msg: "rejected save" }] }), "DOM regression");
  assert.equal(verdict({ ...good, audit: { ...good.audit, contrast: [{}] } }), "visual regression");
  assert.equal(
    verdict({ ...good, axe: { status: "ok", violations: [{ impact: "serious" }] } }),
    "visual regression"
  );
});
