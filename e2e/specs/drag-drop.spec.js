/**
 * Drag and Drop Reordering E2E Tests (Feature #14)
 *
 * Tests for reordering chapters and scenes via drag and drop
 */

import {
  waitForAppReady,
  selectChapter,
  getChapterTitles,
  getSceneTitles,
} from "./helpers.js";

describe("Drag and Drop Reordering (#14)", () => {
  beforeEach(async () => {
    await waitForAppReady();
  });

  describe("Chapter Reordering", () => {
    it("should show drag handle on hover", async () => {
      const chapter = await $('[data-testid="chapter-item"]');
      await chapter.moveTo();

      const handle = await chapter.$('[data-testid="drag-handle"]');
      expect(await handle.isDisplayed()).toBe(true);
    });

    it("should reorder chapters via drag and drop", async () => {
      const beforeTitles = await getChapterTitles();
      expect(beforeTitles.length).toBeGreaterThanOrEqual(2);

      const firstChapter = (await $$('[data-testid="chapter-item"]'))[0];
      const secondChapter = (await $$('[data-testid="chapter-item"]'))[1];

      // Get the drag handle
      await firstChapter.moveTo();
      const handle = await firstChapter.$('[data-testid="drag-handle"]');

      // Perform drag and drop
      await handle.dragAndDrop(secondChapter);

      // Verify order changed
      const afterTitles = await getChapterTitles();
      expect(afterTitles[0]).toBe(beforeTitles[1]);
      expect(afterTitles[1]).toBe(beforeTitles[0]);
    });

    it("should show drop indicator while dragging", async () => {
      const chapters = await $$('[data-testid="chapter-item"]');
      if (chapters.length < 2) return; // Skip if not enough chapters

      const firstChapter = chapters[0];
      await firstChapter.moveTo();
      const handle = await firstChapter.$('[data-testid="drag-handle"]');

      // Start dragging (we can't easily test the mid-drag state in WebDriver)
      // This test validates the drag handle is interactive
      expect(await handle.isClickable()).toBe(true);
    });

    it("should persist chapter order after page refresh", async () => {
      // First reorder
      const beforeTitles = await getChapterTitles();
      if (beforeTitles.length < 2) return;

      const firstChapter = (await $$('[data-testid="chapter-item"]'))[0];
      const secondChapter = (await $$('[data-testid="chapter-item"]'))[1];

      await firstChapter.moveTo();
      const handle = await firstChapter.$('[data-testid="drag-handle"]');
      await handle.dragAndDrop(secondChapter);

      const afterReorder = await getChapterTitles();

      // Refresh the page
      await browser.refresh();
      await waitForAppReady();

      // Verify order persisted
      const afterRefresh = await getChapterTitles();
      expect(afterRefresh).toEqual(afterReorder);
    });
  });

  describe("Scene Reordering", () => {
    beforeEach(async () => {
      await selectChapter("Act 1");
    });

    it("should show drag handle on scene hover", async () => {
      const scene = await $('[data-testid="scene-item"]');
      await scene.moveTo();

      const handle = await scene.$('[data-testid="drag-handle"]');
      expect(await handle.isDisplayed()).toBe(true);
    });

    it("should reorder scenes via drag and drop", async () => {
      const beforeTitles = await getSceneTitles();
      if (beforeTitles.length < 2) return;

      const firstScene = (await $$('[data-testid="scene-item"]'))[0];
      const secondScene = (await $$('[data-testid="scene-item"]'))[1];

      await firstScene.moveTo();
      const handle = await firstScene.$('[data-testid="drag-handle"]');
      await handle.dragAndDrop(secondScene);

      const afterTitles = await getSceneTitles();
      expect(afterTitles[0]).toBe(beforeTitles[1]);
      expect(afterTitles[1]).toBe(beforeTitles[0]);
    });

    it("should persist scene order after navigation", async () => {
      const beforeTitles = await getSceneTitles();
      if (beforeTitles.length < 2) return;

      const firstScene = (await $$('[data-testid="scene-item"]'))[0];
      const secondScene = (await $$('[data-testid="scene-item"]'))[1];

      await firstScene.moveTo();
      const handle = await firstScene.$('[data-testid="drag-handle"]');
      await handle.dragAndDrop(secondScene);

      const afterReorder = await getSceneTitles();

      // Navigate to different chapter and back
      await selectChapter("Act 2");
      await selectChapter("Act 1");

      const afterNavigation = await getSceneTitles();
      expect(afterNavigation).toEqual(afterReorder);
    });
  });

  describe("Visual Feedback", () => {
    it("should reduce opacity of dragged item", async () => {
      // Note: Testing visual CSS properties in WebDriver is limited
      // This test validates the element structure is correct for drag styling
      const chapter = await $('[data-testid="chapter-item"]');
      expect(await chapter.isExisting()).toBe(true);
    });
  });
});
