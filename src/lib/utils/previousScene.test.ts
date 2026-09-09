import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { mockChapters, mockScenes } from "../../dev/mock-data";
import type { Beat, Scene } from "../types";
import { closingProse, loadPreviousScene } from "./previousScene";
import { proseSaves } from "./proseSaves";

const chapter = { ...mockChapters[0], id: "chapter", position: 0 };
const previous: Scene = {
  ...mockScenes[0],
  id: "previous",
  chapter_id: chapter.id,
  position: 0,
  prose: "<p>Page ending.</p>",
  editor_mode: "page",
};
const current: Scene = { ...previous, id: "current", position: 1 };
let scenes: Scene[];
let beats: Beat[];
beforeEach(() => {
  scenes = [current, previous];
  beats = [];
  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "get_scenes")
      return scenes.filter(
        (scene) => scene.chapter_id === (args as { chapterId: string }).chapterId
      );
    if (cmd === "get_beats") return beats;
    throw new Error(`Unexpected command: ${cmd}`);
  });
});
afterEach(() => vi.restoreAllMocks());

describe("closingProse", () => {
  it.each([
    ["", ""],
    ["<p><br></p>", ""],
    ["<p>Just a fragment</p>", "Just a fragment"],
    ["<p>One. Two.</p><p>Three! Four?</p>", "Two. Three! Four?"],
    [
      "<p>Old.</p><p>She <em>waited</em>. &ldquo;Go!&rdquo; He left</p>",
      "She waited. “Go!” He left",
    ],
    ["<script>Bad.</script><style>Bad.</style><p>A &amp; B.</p>", "A & B."],
    ["<p>First.</p><p>Next<br>line. Last.</p>", "First. Next line. Last."],
    ["<p>一。二。三。四。</p>", "二。三。四。"],
    ['<img src="https://invalid.test/pixel"><p>Only prose.</p>', "Only prose."],
  ])("extracts inert closing text from %s", (html, expected) =>
    expect(closingProse(html)).toBe(expected)
  );
  it("supports webviews without Intl.Segmenter", () => {
    vi.spyOn(Intl, "Segmenter", "get").mockReturnValue(undefined as never);
    expect(closingProse("One. Two! Three? Four")).toBe("Two! Three? Four");
  });
});

it("finds the previous scene in position order, without mutating input", async () => {
  expect(await loadPreviousScene("project", current, [chapter])).toEqual({
    scene: previous,
    excerpt: "Page ending.",
  });
  expect(scenes[0]).toBe(current);
});
it("has no previous scene for the first scene or a missing selection/chapter", async () => {
  expect(await loadPreviousScene("project", previous, [chapter])).toBeNull();
  expect(await loadPreviousScene("project", { ...current, id: "missing" }, [chapter])).toBeNull();
  expect(await loadPreviousScene("project", current, [])).toBeNull();
});
it("crosses empty chapters and skips archived chapters/scenes", async () => {
  const second = { ...chapter, id: "empty", position: 1 };
  const third = { ...chapter, id: "third", position: 3 };
  const archived = { ...chapter, id: "archived", position: 2, archived: true };
  const selected = { ...current, chapter_id: third.id, position: 0 };
  scenes = [previous, { ...previous, id: "hidden", position: 1, archived: true }, selected];
  expect(
    (await loadPreviousScene("project", selected, [third, archived, second, chapter]))?.scene.id
  ).toBe(previous.id);
  expect(vi.mocked(invoke).mock.calls).not.toContainEqual([
    "get_scenes",
    { chapterId: archived.id },
  ]);
});
it("reads ordered beat prose, ignoring cached page prose and outline prompts", async () => {
  scenes = [{ ...previous, editor_mode: "beat" }, current];
  beats = [
    {
      id: "last",
      scene_id: previous.id,
      content: "Not prose",
      prose: "<p>Three. Four.</p>",
      position: 2,
    },
    { id: "empty", scene_id: previous.id, content: "Not prose", prose: null, position: 1 },
    {
      id: "first",
      scene_id: previous.id,
      content: "Not prose",
      prose: "<p>One. Two.</p>",
      position: 0,
    },
  ];
  expect((await loadPreviousScene("project", current, [chapter]))?.excerpt).toBe(
    "Two. Three. Four."
  );
});
it("uses scene prose in beat mode only when there are no beats", async () => {
  scenes = [{ ...previous, editor_mode: "beat" }, current];
  expect((await loadPreviousScene("project", current, [chapter]))?.excerpt).toBe("Page ending.");
  beats = [{ id: "empty", scene_id: previous.id, content: "Prompt", prose: null, position: 0 }];
  expect((await loadPreviousScene("project", current, [chapter]))?.excerpt).toBe("");
});
it("uses pending page prose even when it finishes saving during the read", async () => {
  vi.spyOn(proseSaves, "draftsForRecovery")
    .mockReturnValueOnce([
      { projectId: "project", kind: "page", id: previous.id, prose: "Fresh page." },
    ])
    .mockReturnValue([]);
  expect((await loadPreviousScene("project", current, [chapter]))?.excerpt).toBe("Fresh page.");
});
it("uses latest pending beat prose after the read", async () => {
  scenes = [{ ...previous, editor_mode: "beat" }, current];
  beats = [{ id: "beat", scene_id: previous.id, content: "Prompt", prose: "Old.", position: 0 }];
  vi.spyOn(proseSaves, "draftsForRecovery")
    .mockReturnValueOnce([])
    .mockReturnValue([{ projectId: "project", kind: "beat", id: "beat", prose: "Latest." }]);
  expect((await loadPreviousScene("project", current, [chapter]))?.excerpt).toBe("Latest.");
});
it("propagates load failures for the retry UI", async () => {
  vi.mocked(invoke).mockRejectedValue(new Error("database unavailable"));
  await expect(loadPreviousScene("project", current, [chapter])).rejects.toThrow(
    "database unavailable"
  );
});
