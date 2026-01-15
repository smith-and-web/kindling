/**
 * Delete Chapters and Scenes E2E Tests (Feature #16)
 *
 * Tests for deleting chapters and scenes with confirmation
 */

import {
  waitForAppReady,
  selectChapter,
  selectScene,
  getChapterTitles,
  getSceneTitles,
} from "./helpers.js";

describe("Delete Chapters and Scenes (#16)", () => {
  beforeEach(async () => {
    await waitForAppReady();
  });

  describe("Delete Confirmation Dialog", () => {
    it("should show delete button on chapter hover", async () => {
      const chapter = await $('[data-testid="chapter-item"]');
      await chapter.moveTo();

      const deleteButton = await chapter.$('[data-testid="delete-button"]');
      expect(await deleteButton.isDisplayed()).toBe(true);
    });

    it("should show delete button on scene hover", async () => {
      await selectChapter("Act 1");

      const scene = await $('[data-testid="scene-item"]');
      await scene.moveTo();

      const deleteButton = await scene.$('[data-testid="delete-button"]');
      expect(await deleteButton.isDisplayed()).toBe(true);
    });

    it("should show confirmation dialog with content counts for chapter", async () => {
      const chapter = await $('[data-testid="chapter-item"]');
      await chapter.moveTo();

      const deleteButton = await chapter.$('[data-testid="delete-button"]');
      await deleteButton.click();

      // Check dialog appears
      const dialog = await $('[data-testid="confirm-dialog"]');
      expect(await dialog.isExisting()).toBe(true);

      // Check it shows counts
      const message = await dialog.$('[data-testid="dialog-message"]');
      const text = await message.getText();
      expect(text).toMatch(/\d+ scenes?/);
      expect(text).toMatch(/\d+ beats?/);
    });

    it("should show confirmation dialog for scene with beat count", async () => {
      await selectChapter("Act 1");

      const scene = await $('[data-testid="scene-item"]');
      await scene.moveTo();

      const deleteButton = await scene.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const dialog = await $('[data-testid="confirm-dialog"]');
      expect(await dialog.isExisting()).toBe(true);

      const message = await dialog.$('[data-testid="dialog-message"]');
      const text = await message.getText();
      expect(text).toMatch(/\d+ beats?/);
    });
  });

  describe("Delete Actions", () => {
    it("should close dialog without deleting when clicking Cancel", async () => {
      const beforeTitles = await getChapterTitles();

      const chapter = await $('[data-testid="chapter-item"]');
      await chapter.moveTo();

      const deleteButton = await chapter.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const cancelButton = await $('[data-testid="dialog-cancel"]');
      await cancelButton.click();

      // Dialog should close
      const dialog = await $('[data-testid="confirm-dialog"]');
      expect(await dialog.isExisting()).toBe(false);

      // Chapter should still exist
      const afterTitles = await getChapterTitles();
      expect(afterTitles).toEqual(beforeTitles);
    });

    it("should delete chapter when confirming", async () => {
      const beforeTitles = await getChapterTitles();
      const chapterToDelete = beforeTitles[beforeTitles.length - 1];

      // Click the last chapter's delete button
      const chapters = await $$('[data-testid="chapter-item"]');
      const lastChapter = chapters[chapters.length - 1];
      await lastChapter.moveTo();

      const deleteButton = await lastChapter.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Chapter should be removed
      const afterTitles = await getChapterTitles();
      expect(afterTitles).not.toContain(chapterToDelete);
      expect(afterTitles.length).toBe(beforeTitles.length - 1);
    });

    it("should delete scene when confirming", async () => {
      await selectChapter("Act 1");

      const beforeTitles = await getSceneTitles();
      const sceneToDelete = beforeTitles[beforeTitles.length - 1];

      const scenes = await $$('[data-testid="scene-item"]');
      const lastScene = scenes[scenes.length - 1];
      await lastScene.moveTo();

      const deleteButton = await lastScene.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      const afterTitles = await getSceneTitles();
      expect(afterTitles).not.toContain(sceneToDelete);
    });

    it("should clear selection if deleted item was selected", async () => {
      await selectChapter("Act 1");

      // Select a scene first
      const scene = await $('[data-testid="scene-item"]');
      await scene.click();

      // Verify it's selected
      const selectedBefore = await scene.getAttribute("data-selected");
      expect(selectedBefore).toBe("true");

      // Delete it
      await scene.moveTo();
      const deleteButton = await scene.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Scene panel should show empty/welcome state
      const scenePanel = await $('[data-testid="scene-panel"]');
      const emptyState = await scenePanel.$('[data-testid="empty-state"]');
      expect(await emptyState.isExisting()).toBe(true);
    });
  });

  describe("Delete Persistence", () => {
    it("should persist deletion after page refresh", async () => {
      const beforeTitles = await getChapterTitles();
      if (beforeTitles.length === 0) return;

      const chapterToDelete = beforeTitles[beforeTitles.length - 1];

      // Delete the last chapter
      const chapters = await $$('[data-testid="chapter-item"]');
      const lastChapter = chapters[chapters.length - 1];
      await lastChapter.moveTo();

      const deleteButton = await lastChapter.$('[data-testid="delete-button"]');
      await deleteButton.click();

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Refresh
      await browser.refresh();
      await waitForAppReady();

      // Verify still deleted
      const afterRefresh = await getChapterTitles();
      expect(afterRefresh).not.toContain(chapterToDelete);
    });
  });
});
