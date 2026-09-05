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
  const calls = [];
  let failDelete = false;
  w.__TAURI_INTERNALS__ = {
    invoke: async (command, args) => {
      calls.push({ command, args });
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
        id:
          command === "import_plottr"
            ? "created-fixture"
            : command.startsWith("import_")
              ? `created-${command}`
              : "created-blank",
        name: "QA Blank Project",
      };
    },
  };
  w.__KINDLING_TEST__ = {
    importCommands: Object.fromEntries(
      ["plottr", "markdown", "ywriter", "longform", "scrivener", "novelwriter"].map((format) => [
        format,
        `import_${format}`,
      ])
    ),
    invoke: (...args) => w.__TAURI_INTERNALS__.invoke(...args),
    disableGuidance() {},
    importProject: (path, format = "plottr") =>
      w.__TAURI_INTERNALS__.invoke(`import_${format}`, { path }),
  };
  w.eval(source);
  return {
    w,
    deleted,
    calls,
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

for (const format of ["markdown", "ywriter", "longform", "scrivener", "novelwriter"]) {
  test(`${format} imports are tracked by returned ID and cleaned up after reload`, async () => {
    const { w, calls, deleted, close } = setup();
    try {
      w.__qa.importFixture("/tmp/qa-import", format);
      await settle();
      assert.equal(calls[0].command, `import_${format}`);
      assert.equal(calls[0].args.path, "/tmp/qa-import");
      assert.equal(w.__qa.fixtureProject().id, `created-import_${format}`);
      assert.ok(w.document.getElementById("qa-done-fixture-created"));
      delete w.__qa;
      w.eval(source);
      w.__qa.cleanupFixtures();
      await settle();
      assert.deepEqual(deleted, [`created-import_${format}`]);
      assert.equal(w.__qa.fixtureProject(), null);
    } finally {
      close();
    }
  });
}

test("unknown formats and failed imports never acquire ownership", async () => {
  const { w, calls, deleted, close } = setup();
  try {
    assert.throws(
      () => w.__qa.importFixture("/tmp/qa", "delete_project"),
      /Unsupported QA import format/
    );
    assert.equal(calls.length, 0);
    w.__TAURI_INTERNALS__.invoke = async () => {
      throw new Error("Missing nwProject.nwx");
    };
    w.__qa.importFixture("/tmp/missing", "novelwriter");
    await settle();
    assert.equal(w.__qa.last.status, "error");
    assert.equal(w.__qa.fixtureProject(), null);
    assert.equal(w.document.getElementById("qa-done-fixture-created"), null);
    w.__qa.cleanupFixtures();
    await settle();
    assert.deepEqual(deleted, []);
  } finally {
    close();
  }
});

test("invoke completion markers distinguish fresh calls and surface errors", async () => {
  const { w, close } = setup();
  try {
    w.__qa.invoke("get_recent_projects");
    await settle();
    assert.equal(w.__qa.last.status, "ok");
    assert.ok(w.document.getElementById("qa-done-invoke"));
    w.__TAURI_INTERNALS__.invoke = async () => {
      throw new Error("export failed");
    };
    w.__qa.invoke("export_to_novelwriter");
    assert.equal(w.document.getElementById("qa-done-invoke"), null);
    await settle();
    assert.equal(w.__qa.last.status, "error");
    assert.match(w.__qa.last.error, /export failed/);
    assert.ok(w.document.getElementById("qa-done-invoke"));
  } finally {
    close();
  }
});
