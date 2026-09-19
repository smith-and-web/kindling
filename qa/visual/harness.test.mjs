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

test("capture repaint preserves nested scroll positions and waits for two frames", () => {
  const { w, close } = setup();
  try {
    const frames = [];
    w.requestAnimationFrame = (callback) => frames.push(callback);
    const pane = w.document.createElement("div");
    w.document.body.append(pane);
    pane.scrollTop = 420;
    pane.scrollLeft = 12;
    pane.scrollTo = ({ top, left, behavior }) => {
      assert.equal(behavior, "instant");
      pane.scrollTop = top;
      pane.scrollLeft = left;
    };
    Object.defineProperty(w.document.documentElement, "offsetHeight", {
      get() {
        if (w.document.documentElement.style.display === "none") {
          pane.scrollTop = 0;
          pane.scrollLeft = 0;
        }
        return 100;
      },
    });
    w.__qa.shot("scrolled-editor");
    assert.equal(pane.scrollTop, 420);
    assert.equal(pane.scrollLeft, 12);
    assert.equal(w.document.documentElement.style.display, "");
    assert.equal(w.document.getElementById("qa-settled-scrolled-editor"), null);
    frames.shift()();
    assert.equal(w.document.getElementById("qa-settled-scrolled-editor"), null);
    frames.shift()();
    assert.ok(w.document.getElementById("qa-settled-scrolled-editor"));
  } finally {
    close();
  }
});

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

test("tracks fixture creation with a frozen Tauri IPC entry point before UI completion", async () => {
  const { w, deleted, close } = setup();
  try {
    const ipc = w.__TAURI_INTERNALS__.invoke;
    Object.defineProperty(w.__TAURI_INTERNALS__, "invoke", { value: ipc, writable: false });
    w.__KINDLING_TEST__.creationObserverVersion = 1;
    w.__KINDLING_TEST__.importProject = async () => {
      const onCreated = w.__KINDLING_TEST__.onProjectCreated;
      const project = await ipc("import_plottr", {});
      onCreated?.("import_plottr", project);
      return new Promise(() => {}); // Later frontend work never completes.
    };
    w.__qa.importFixture("fixture.pltr");
    await settle();
    assert.equal(w.__qa.fixtureProject()?.id, "created-fixture");
    w.__qa.cleanupFixtures();
    await settle();
    assert.deepEqual(deleted, ["created-fixture"]);
  } finally {
    close();
  }
});

test("creation observer is scoped to its action and restored even when the action throws", async () => {
  const { w, close } = setup();
  try {
    w.__KINDLING_TEST__.creationObserverVersion = 1;
    const previous = () => {};
    w.__KINDLING_TEST__.onProjectCreated = previous;
    let captured;
    w.__qa.trackCreation("create_blank_project", () => {
      captured = w.__KINDLING_TEST__.onProjectCreated;
    });
    assert.equal(w.__KINDLING_TEST__.onProjectCreated, previous);
    captured("create_blank_project", { id: "owned-blank", name: "QA Blank Project" });
    assert.equal(
      JSON.parse(w.sessionStorage.getItem("__qa_created_projects"))[0].id,
      "owned-blank"
    );
    assert.throws(() =>
      w.__qa.trackCreation("create_blank_project", () => {
        throw Error("cancelled");
      })
    );
    assert.equal(w.__KINDLING_TEST__.onProjectCreated, previous);
  } finally {
    close();
  }
});

test("frozen older bridge fails before creating an untracked project", () => {
  const { w, calls, close } = setup();
  try {
    Object.defineProperty(w.__TAURI_INTERNALS__, "invoke", { writable: false });
    assert.throws(() => w.__qa.importFixture("fixture.pltr"), /creation observer/);
    assert.equal(calls.length, 0);
  } finally {
    close();
  }
});

test("scoped audits scan dialog descendants and use CSS-pixel large-text thresholds", () => {
  const { w, close } = setup();
  try {
    w.document.body.innerHTML =
      '<section><p style="font-family:Inter;font-size:20px;font-weight:400;color:rgb(140,140,140);background-color:rgb(255,255,255)">Normal 20px text</p></section>';
    const p = w.document.querySelector("p");
    p.getBoundingClientRect = () => ({ width: 200, height: 24 });
    const root = w.document.querySelector("section");
    const audit = w.__qa.audit(root);
    assert.equal(audit.scanned, 1);
    assert.equal(audit.contrast.length, 1); // ~3.36:1 is insufficient below 24px regular.
    p.style.fontSize = "24px";
    assert.equal(w.__qa.audit(root).contrast.length, 0);
  } finally {
    close();
  }
});

test("missing capture is reported even when there is no baseline", async () => {
  const { w, close } = setup();
  try {
    w.Image = class {
      set src(value) {
        queueMicrotask(() => this.onerror());
      }
    };
    w.__qa.diff("/tmp/not-a-run", [{ name: "missing", file: "missing.jpg" }]);
    await settle();
    assert.equal(w.__qa.diffResult.results[0].status, "missing-capture");
  } finally {
    close();
  }
});

test("PNG masters use explicit baseline paths and deterministic pixel thresholds", async () => {
  const { w, close } = setup();
  try {
    const urls = [];
    let differences = 0,
      delta = 25;
    w.Image = class {
      naturalWidth = 1000;
      naturalHeight = 1;
      set src(value) {
        this.path = value;
        urls.push(value);
        queueMicrotask(() => this.onload());
      }
    };
    w.HTMLCanvasElement.prototype.getContext = function () {
      let image;
      return {
        drawImage(im) {
          image = im;
        },
        getImageData() {
          const data = new Uint8ClampedArray(4000).fill(100);
          if (image.path.endsWith("capture.png"))
            for (let n = 0; n < differences; n++) data[n * 4] += delta;
          return { data };
        },
      };
    };
    // draw() calls getContext twice, as browsers return the same context.
    const context = w.HTMLCanvasElement.prototype.getContext;
    w.HTMLCanvasElement.prototype.getContext = function () {
      return (this._context ||= context.call(this));
    };
    const compare = async (n, d) => {
      differences = n;
      delta = d;
      w.__qa.diff("/run", [{ name: "test-2x", file: "capture.png", baseline: "approved-2x.png" }], {
        baselineDir: "/baseline",
      });
      await settle();
      return w.__qa.diffResult.results[0];
    };
    assert.equal((await compare(100, 24)).changed, false);
    const boundary = await compare(4, 25);
    assert.equal(boundary.differingPixels, 4);
    assert.equal(boundary.changed, false);
    const changed = await compare(5, 25);
    assert.equal(changed.totalPixels, 1000);
    assert.equal(changed.changed, true);
    assert.deepEqual(Array.from(changed.box), [0, 0, 4, 0]);
    assert.equal(changed.mismatch, (await compare(5, 25)).mismatch);
    assert.ok(urls.includes("/@fs/baseline/approved-2x.png"));
    assert.ok(!urls.some((u) => u.endsWith(".jpg")));
  } finally {
    close();
  }
});
