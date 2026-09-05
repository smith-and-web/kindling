import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { JSDOM } from "jsdom";

const source = readFileSync(new URL("./harness.js", import.meta.url), "utf8");
const settle = () => new Promise((resolve) => setImmediate(resolve));

function setup() {
  const dom = new JSDOM("<!doctype html>", { url: "http://localhost", runScripts: "outside-only" });
  const w = dom.window;
  const deleted = [];
  let failDelete = false;
  w.__TAURI_INTERNALS__ = {
    invoke: async (command, args) => {
      if (command === "get_recent_projects")
        return [
          { id: "existing-backup", source_path: "my-simple-story.pltr.backup" },
          { id: "existing-fixture", source_path: "simple-story.pltr" },
          { id: "existing-name", name: "QA Blank Project" },
        ];
      if (command === "delete_project") {
        if (failDelete) throw new Error("database unavailable");
        deleted.push(args.projectId);
        return;
      }
      return {
        id: command === "import_plottr" ? "created-fixture" : "created-blank",
        name: "QA Blank Project",
      };
    },
  };
  w.__KINDLING_TEST__ = {
    invoke: (...args) => w.__TAURI_INTERNALS__.invoke(...args),
    disableGuidance() {},
    importProject: (path) => w.__TAURI_INTERNALS__.invoke("import_plottr", { path }),
  };
  w.eval(source);
  return {
    w,
    deleted,
    fail: (value) => {
      failDelete = value;
    },
    close: () => dom.window.close(),
  };
}

test("cleanup never deletes pre-existing projects matching fixture paths or names", async () => {
  const { w, deleted, close } = setup();
  try {
    w.__qa.cleanupFixtures();
    await settle();
    w.__qa.cleanupNamed("QA Blank Project");
    await settle();
    assert.deepEqual(deleted, []);
  } finally {
    close();
  }
});

test("successful QA creations are tracked across reload and removed only once", async () => {
  const { w, deleted, close } = setup();
  try {
    const original = w.__TAURI_INTERNALS__.invoke;
    w.__qa.importFixture("simple-story.pltr");
    assert.equal(w.__TAURI_INTERNALS__.invoke, original);
    await settle();
    await w.__qa.trackCreation("create_blank_project", () =>
      w.__KINDLING_TEST__.invoke("create_blank_project")
    );
    delete w.__qa;
    w.eval(source);
    w.__qa.cleanupFixtures();
    await settle();
    assert.deepEqual(deleted, ["created-fixture"]);
    w.__qa.cleanupNamed("QA Blank Project");
    await settle();
    assert.deepEqual(deleted, ["created-fixture", "created-blank"]);
    w.__qa.cleanupFixtures();
    w.__qa.cleanupNamed("QA Blank Project");
    await settle();
    assert.equal(deleted.length, 2);
  } finally {
    close();
  }
});

test("failed deletes retain ownership for retry; failed creations grant no ownership", async () => {
  const { w, deleted, fail, close } = setup();
  try {
    w.__qa.importFixture("simple-story.pltr");
    await settle();
    fail(true);
    w.__qa.cleanupFixtures();
    await settle();
    assert.equal(w.__qa.last.status, "error");
    fail(false);
    w.__qa.cleanupFixtures();
    await settle();
    assert.deepEqual(deleted, ["created-fixture"]);
    const reject = async () => {
      throw new Error("import failed");
    };
    w.__TAURI_INTERNALS__.invoke = reject;
    w.__qa.importFixture("missing.pltr");
    await settle();
    assert.equal(w.__qa.last.status, "error");
    assert.equal(w.__TAURI_INTERNALS__.invoke, reject);
    assert.equal(w.sessionStorage.getItem("__qa_created_projects"), "[]");
  } finally {
    close();
  }
});
