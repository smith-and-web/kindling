/**
 * Delete Chapters and Scenes E2E Tests (Feature #16)
 *
 * Tests for deleting chapters and scenes with confirmation
 * Note: Delete is accessed via context menu (right-click or 3-dot menu button)
 */

import {
  waitForAppReady,
  skipOnboardingIfPresent,
  importPlottrFile,
  selectChapter,
  selectScene,
  getChapterTitles,
  getSceneTitles,
  openContextMenuFor,
  clickContextMenuItem,
} from "./helpers.js";

/**
 * Helper to close all open dialogs/menus
 */
async function closeAllOverlays() {
  // Press Escape to close any open overlay
  await browser.keys("Escape");
  await browser.pause(200);

  // Double-check and close confirm dialog if still open
  const confirmDialog = await $('[data-testid="confirm-dialog"]');
  if (await confirmDialog.isExisting()) {
    await browser.keys("Escape");
    await browser.pause(200);
  }
}

describe("Delete Chapters and Scenes (#16)", () => {
  before(async () => {
    await waitForAppReady();
    await skipOnboardingIfPresent();
    await importPlottrFile("simple-story.pltr");
  });

  // Close overlays before and after each test
  beforeEach(async () => {
    await closeAllOverlays();
  });

  afterEach(async () => {
    await closeAllOverlays();
  });

  describe("Delete Confirmation Dialog", () => {
    it("should show menu button on chapter hover", async () => {
      const chapter = await $('[data-testid="chapter-item"]');
      await chapter.moveTo();

      const menuButton = await chapter.$('[data-testid="menu-button"]');
      expect(await menuButton.isDisplayed()).toBe(true);
    });

    it("should show menu button on scene hover", async () => {
      await selectChapter("Act 1");

      const scene = await $('[data-testid="scene-item"]');
      await scene.moveTo();

      const menuButton = await scene.$('[data-testid="menu-button"]');
      expect(await menuButton.isDisplayed()).toBe(true);
    });

    it("should show confirmation dialog with content counts for chapter", async () => {
      const chapter = await $('[data-testid="chapter-item"]');
      await openContextMenuFor(chapter);
      await clickContextMenuItem("Delete");

      // Check dialog appears
      const dialog = await $('[data-testid="confirm-dialog"]');
      expect(await dialog.isExisting()).toBe(true);

      // Check it shows counts (use textContent for WebKit compatibility)
      const message = await dialog.$('[data-testid="dialog-message"]');
      const text = await browser.execute((el) => el.textContent, message);
      expect(text).toMatch(/\d+ scenes?/);
      expect(text).toMatch(/\d+ beats?/);
    });

    it("should show confirmation dialog for scene with beat count", async () => {
      await selectChapter("Act 1");

      const scene = await $('[data-testid="scene-item"]');
      await openContextMenuFor(scene);
      await clickContextMenuItem("Delete");

      const dialog = await $('[data-testid="confirm-dialog"]');
      expect(await dialog.isExisting()).toBe(true);

      const message = await dialog.$('[data-testid="dialog-message"]');
      const text = await browser.execute((el) => el.textContent, message);
      expect(text).toMatch(/\d+ beats?/);
    });
  });

  describe("Delete Actions", () => {
    it("should close dialog without deleting when clicking Cancel", async () => {
      const beforeTitles = await getChapterTitles();

      const chapter = await $('[data-testid="chapter-item"]');
      await openContextMenuFor(chapter);
      await clickContextMenuItem("Delete");

      const cancelButton = await $('[data-testid="dialog-cancel"]');
      await cancelButton.click();

      // Dialog should close
      await browser.waitUntil(
        async () => {
          const dialog = await $('[data-testid="confirm-dialog"]');
          return !(await dialog.isExisting());
        },
        { timeout: 2000 }
      );

      // Chapter should still exist
      const afterTitles = await getChapterTitles();
      expect(afterTitles).toEqual(beforeTitles);
    });

    it("should delete chapter when confirming", async () => {
      const beforeTitles = await getChapterTitles();
      const chapterToDelete = beforeTitles[beforeTitles.length - 1];

      // Click the last chapter's menu
      const chapters = await $$('[data-testid="chapter-item"]');
      const lastChapter = chapters[chapters.length - 1];
      await openContextMenuFor(lastChapter);
      await clickContextMenuItem("Delete");

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Wait for dialog to close and UI to update
      await browser.waitUntil(
        async () => {
          const dialog = await $('[data-testid="confirm-dialog"]');
          return !(await dialog.isExisting());
        },
        { timeout: 3000 }
      );

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
      await openContextMenuFor(lastScene);
      await clickContextMenuItem("Delete");

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Wait for dialog to close
      await browser.waitUntil(
        async () => {
          const dialog = await $('[data-testid="confirm-dialog"]');
          return !(await dialog.isExisting());
        },
        { timeout: 3000 }
      );

      const afterTitles = await getSceneTitles();
      expect(afterTitles).not.toContain(sceneToDelete);
    });

    it("should clear selection if deleted item was selected", async () => {
      await selectChapter("Act 1");

      // Select a scene first
      const scene = await $('[data-testid="scene-item"]');
      await scene.click();
      await browser.pause(300);

      // Delete it via context menu
      await openContextMenuFor(scene);
      await clickContextMenuItem("Delete");

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Wait for dialog to close
      await browser.waitUntil(
        async () => {
          const dialog = await $('[data-testid="confirm-dialog"]');
          return !(await dialog.isExisting());
        },
        { timeout: 3000 }
      );

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
      await openContextMenuFor(lastChapter);
      await clickContextMenuItem("Delete");

      const confirmButton = await $('[data-testid="dialog-confirm"]');
      await confirmButton.click();

      // Wait for dialog to close
      await browser.waitUntil(
        async () => {
          const dialog = await $('[data-testid="confirm-dialog"]');
          return !(await dialog.isExisting());
        },
        { timeout: 3000 }
      );

      // Refresh
      await browser.refresh();
      await waitForAppReady();

      // Verify still deleted
      const afterRefresh = await getChapterTitles();
      expect(afterRefresh).not.toContain(chapterToDelete);
    });
  });
});
