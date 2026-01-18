/**
 * Create Chapters and Scenes E2E Tests (Feature #15)
 *
 * Tests for creating new chapters and scenes
 */

import {
  waitForAppReady,
  skipOnboardingIfPresent,
  importPlottrFile,
  selectChapter,
  clickNewChapter,
  clickNewScene,
  submitTitleInput,
  cancelTitleInput,
  getChapterTitles,
  getSceneTitles,
} from "./helpers.js";

/**
 * Helper to cancel any open title input
 */
async function closeAnyOpenInput() {
  const input = await $('[data-testid="title-input"]');
  if (await input.isExisting()) {
    await browser.keys("Escape");
    await browser.pause(200);
  }
}

describe("Create Chapters and Scenes (#15)", () => {
  before(async () => {
    await waitForAppReady();
    await skipOnboardingIfPresent();
    await importPlottrFile("simple-story.pltr");
  });

  // Clean up any open inputs between tests
  beforeEach(async () => {
    await closeAnyOpenInput();
  });

  afterEach(async () => {
    await closeAnyOpenInput();
  });

  describe("Creating Chapters", () => {
    it("should show inline input when clicking new chapter button", async () => {
      await clickNewChapter();

      const input = await $('[data-testid="title-input"]');
      expect(await input.isExisting()).toBe(true);
      // Focus check can be unreliable in WebKit, just verify input exists
    });

    it("should create a chapter when pressing Enter", async () => {
      const chapterTitle = "Test Chapter E2E";

      await clickNewChapter();
      await submitTitleInput(chapterTitle);

      // Verify chapter appears in list
      const titles = await getChapterTitles();
      expect(titles).toContain(chapterTitle);
    });

    it("should cancel chapter creation when pressing Escape", async () => {
      const beforeTitles = await getChapterTitles();

      await clickNewChapter();
      const input = await $('[data-testid="title-input"]');
      await input.setValue("Should Not Create");
      await cancelTitleInput();

      const afterTitles = await getChapterTitles();
      expect(afterTitles.length).toBe(beforeTitles.length);
      expect(afterTitles).not.toContain("Should Not Create");
    });

    it("should cancel when clicking elsewhere", async () => {
      const beforeTitles = await getChapterTitles();

      await clickNewChapter();
      const input = await $('[data-testid="title-input"]');
      await input.setValue("Should Not Create Click");

      // Click elsewhere to trigger blur
      const sidebar = await $('[data-testid="sidebar"]');
      await sidebar.click();
      await browser.pause(300);

      const afterTitles = await getChapterTitles();
      expect(afterTitles.length).toBe(beforeTitles.length);
    });

    it("should auto-expand newly created chapter", async () => {
      const chapterTitle = "Auto Expand Test";

      await clickNewChapter();
      await submitTitleInput(chapterTitle);

      // Wait for chapter to be expanded and show new scene button
      await browser.waitUntil(
        async () => {
          const btn = await $('[data-testid="new-scene-button"]');
          return await btn.isExisting();
        },
        { timeout: 3000, timeoutMsg: "New scene button did not appear" }
      );

      const newSceneButton = await $('[data-testid="new-scene-button"]');
      expect(await newSceneButton.isExisting()).toBe(true);
    });
  });

  describe("Creating Scenes", () => {
    beforeEach(async () => {
      await closeAnyOpenInput();
      // Make sure a chapter is expanded
      await selectChapter("Act 1");
    });

    it("should show inline input when clicking new scene button", async () => {
      await clickNewScene();

      const input = await $('[data-testid="title-input"]');
      expect(await input.isExisting()).toBe(true);
    });

    it("should create a scene when pressing Enter", async () => {
      const sceneTitle = "Test Scene E2E";

      await clickNewScene();
      await submitTitleInput(sceneTitle);

      const titles = await getSceneTitles();
      expect(titles).toContain(sceneTitle);
    });

    it("should cancel scene creation when pressing Escape", async () => {
      const beforeTitles = await getSceneTitles();

      await clickNewScene();
      const input = await $('[data-testid="title-input"]');
      await input.setValue("Should Not Create");
      await cancelTitleInput();

      const afterTitles = await getSceneTitles();
      expect(afterTitles.length).toBe(beforeTitles.length);
    });

    it("should auto-select newly created scene", async () => {
      const sceneTitle = "Auto Select Test " + Date.now();

      await clickNewScene();
      await submitTitleInput(sceneTitle);

      // Wait for scene panel to update
      await browser.pause(500);

      // Check that the scene panel shows this scene (use textContent for WebKit)
      const scenePanel = await $('[data-testid="scene-panel"]');
      const panelTitle = await scenePanel.$('[data-testid="scene-title"]');
      const text = await browser.execute((el) => el.textContent?.trim() || "", panelTitle);
      expect(text).toBe(sceneTitle);
    });
  });
});
