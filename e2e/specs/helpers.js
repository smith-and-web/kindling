/**
 * E2E Test Helpers for Kindling
 *
 * Common utilities for interacting with the Kindling app
 */

import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import fs from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
export const projectRoot = resolve(__dirname, "../..");
export const testDataDir = resolve(projectRoot, "test-data");

/**
 * Wait for the app to be ready (any content loaded)
 * This waits for either:
 * - Onboarding modal (first launch)
 * - Start screen (after onboarding, no project)
 * - Editor (project loaded)
 */
export async function waitForAppReady() {
  await browser.waitUntil(
    async () => {
      const main = await $("main");
      if (!(await main.isExisting())) return false;

      // Check for onboarding (first launch), start screen, or editor
      const onboarding = await $('[data-testid="onboarding"]');
      const importSection = await $('[data-testid="import-section"]');
      const sidebar = await $('[data-testid="sidebar"]');
      return (
        (await onboarding.isExisting()) ||
        (await importSection.isExisting()) ||
        (await sidebar.isExisting())
      );
    },
    { timeout: 15000, timeoutMsg: "App did not load within 15 seconds" }
  );
}

/**
 * Skip onboarding if it's showing
 */
export async function skipOnboardingIfPresent() {
  const skipButton = await $('[data-testid="skip-onboarding"]');
  if (await skipButton.isExisting()) {
    await skipButton.click();
    // Wait for onboarding to disappear and start screen to appear
    await browser.waitUntil(
      async () => {
        const onboarding = await $('[data-testid="onboarding"]');
        return !(await onboarding.isExisting());
      },
      { timeout: 5000, timeoutMsg: "Onboarding did not close" }
    );
  }
}

/**
 * Wait for the start screen to be visible (before loading a project)
 * Will skip onboarding first if needed
 */
export async function waitForStartScreen() {
  await waitForAppReady();
  await skipOnboardingIfPresent();
  await browser.waitUntil(
    async () => {
      const importSection = await $('[data-testid="import-section"]');
      return importSection.isExisting();
    },
    { timeout: 15000, timeoutMsg: "Start screen did not load within 15 seconds" }
  );
}

/**
 * Wait for the editor to be visible (after loading a project)
 */
export async function waitForEditor() {
  await browser.waitUntil(
    async () => {
      const sidebar = await $('[data-testid="sidebar"]');
      return sidebar.isExisting();
    },
    { timeout: 15000, timeoutMsg: "Editor did not load within 15 seconds" }
  );
}

/**
 * Import a Plottr file by invoking Tauri command directly
 * This bypasses the native file dialog which can't be controlled in E2E tests
 */
export async function importPlottrFile(filename) {
  const filePath = resolve(testDataDir, filename);

  // Use the app's importProject helper which handles:
  // 1. Calling invoke("import_plottr")
  // 2. Updating currentProject store
  // 3. Setting ui view to "editor"
  const result = await browser.executeAsync(async (path, done) => {
    try {
      // The app exposes importProject via __KINDLING_TEST__ for E2E testing
      if (!window.__KINDLING_TEST__?.importProject) {
        throw new Error("__KINDLING_TEST__.importProject not available");
      }
      const project = await window.__KINDLING_TEST__.importProject(path);
      done({ success: true, project });
    } catch (error) {
      done({ success: false, error: error.message || String(error) });
    }
  }, filePath);

  if (!result.success) {
    throw new Error(`Failed to import Plottr file: ${result.error}`);
  }

  // Wait for UI to update after import
  await browser.pause(500);
  await waitForEditor();

  // Wait for chapters to be visible in the sidebar
  await waitForProjectLoaded();

  return result.project;
}

/**
 * Wait for project chapters to be loaded in the UI
 */
export async function waitForProjectLoaded() {
  await browser.waitUntil(
    async () => {
      const chapters = await $$('[data-testid="chapter-item"]');
      return chapters.length > 0;
    },
    {
      timeout: 10000,
      timeoutMsg: "Expected project chapters to load",
    }
  );
}

/**
 * Replace a test file with another (for reimport testing)
 * Returns the backup path
 */
export async function replaceTestFile(originalName, replacementName) {
  const originalPath = resolve(testDataDir, originalName);
  const replacementPath = resolve(testDataDir, replacementName);
  const backupPath = resolve(testDataDir, `${originalName}.backup`);

  // Backup original
  await fs.promises.copyFile(originalPath, backupPath);
  // Replace with updated version
  await fs.promises.copyFile(replacementPath, originalPath);

  return backupPath;
}

/**
 * Restore a test file from backup
 */
export async function restoreTestFile(originalName) {
  const originalPath = resolve(testDataDir, originalName);
  const backupPath = resolve(testDataDir, `${originalName}.backup`);

  try {
    await fs.promises.access(backupPath);
    await fs.promises.copyFile(backupPath, originalPath);
    await fs.promises.unlink(backupPath);
  } catch {
    // Backup doesn't exist, nothing to restore
  }
}

/**
 * Select a chapter in the sidebar by title
 * WebDriverIO doesn't support :has-text(), so we find all chapters and filter
 * Note: WebKit getText() can return empty, so we use textContent property instead
 */
export async function selectChapter(chapterTitle) {
  const chapters = await $$('[data-testid="chapter-item"]');
  for (const chapter of chapters) {
    const titleEl = await chapter.$('[data-testid="chapter-title"]');
    if (await titleEl.isExisting()) {
      // Use textContent property instead of getText() for WebKit compatibility
      const text = await browser.execute((el) => el.textContent, titleEl);
      if (text && text.trim() === chapterTitle) {
        await chapter.click();
        // Wait for scenes to load after clicking a chapter
        // The loadScenes function is async and fetches from backend
        await browser.waitUntil(
          async () => {
            const scenes = await $$('[data-testid="scene-item"]');
            return scenes.length > 0;
          },
          { timeout: 5000, timeoutMsg: "Scenes did not load after selecting chapter" }
        );
        return;
      }
    }
  }
  throw new Error(`Chapter "${chapterTitle}" not found`);
}

/**
 * Select a scene in the sidebar by title
 * WebDriverIO doesn't support :has-text(), so we find all scenes and filter
 * Note: WebKit getText() can return empty, so we use textContent property instead
 */
export async function selectScene(sceneTitle) {
  const scenes = await $$('[data-testid="scene-item"]');
  for (const scene of scenes) {
    const titleEl = await scene.$('[data-testid="scene-title"]');
    if (await titleEl.isExisting()) {
      // Use textContent property instead of getText() for WebKit compatibility
      const text = await browser.execute((el) => el.textContent, titleEl);
      if (text && text.trim() === sceneTitle) {
        await scene.click();
        // Wait for beats to load after clicking a scene
        // The selectScene function in Sidebar.svelte calls get_beats asynchronously
        await browser.waitUntil(
          async () => {
            const beats = await $$('[data-testid="beat-header"]');
            return beats.length > 0;
          },
          { timeout: 5000, timeoutMsg: "Beats did not load after selecting scene" }
        );
        return;
      }
    }
  }
  throw new Error(`Scene "${sceneTitle}" not found`);
}

/**
 * Expand a beat in the scene panel
 */
export async function expandBeat(beatIndex) {
  const beats = await $$('[data-testid="beat-header"]');
  if (beats[beatIndex]) {
    await beats[beatIndex].click();
  }
}

/**
 * Type prose into the expanded beat textarea
 */
export async function typeProse(text) {
  const textarea = await $('[data-testid="beat-prose-textarea"]');
  await textarea.setValue(text);
}

/**
 * Wait for save to complete
 * The UI shows "Saving..." indicator during save, then hides it when complete.
 * We wait for the indicator to appear (save started) then disappear (save complete).
 */
export async function waitForSaved() {
  // First, check if save indicator is visible (saving in progress)
  // Give it a moment for the save to potentially start
  await browser.pause(200);

  // Wait for save to complete by waiting for indicator to disappear
  // The indicator only shows during "saving" state, and goes to "idle" after ~1s
  await browser.waitUntil(
    async () => {
      const indicator = await $('[data-testid="save-indicator"]');
      // Save is complete when the indicator no longer exists
      return !(await indicator.isExisting());
    },
    { timeout: 5000, timeoutMsg: "Save did not complete" }
  );
}

/**
 * Click the new chapter button
 */
export async function clickNewChapter() {
  const button = await $('[data-testid="new-chapter-button"]');
  await button.click();
}

/**
 * Click the new scene button in an expanded chapter
 */
export async function clickNewScene() {
  const button = await $('[data-testid="new-scene-button"]');
  await button.click();
}

/**
 * Submit the inline title input
 */
export async function submitTitleInput(title) {
  const input = await $('[data-testid="title-input"]');
  await input.setValue(title);
  await browser.keys("Enter");
}

/**
 * Cancel the inline title input
 */
export async function cancelTitleInput() {
  await browser.keys("Escape");
}

/**
 * Get all chapter titles
 */
export async function getChapterTitles() {
  const chapters = await $$('[data-testid="chapter-title"]');
  // Use textContent property instead of getText() for WebKit compatibility
  // Iterate manually because browser.execute doesn't work well with .map() on element arrays
  const titles = [];
  for (const chapter of chapters) {
    const text = await browser.execute((el) => el.textContent?.trim() || "", chapter);
    titles.push(text);
  }
  return titles;
}

/**
 * Get all scene titles for the expanded chapter
 */
export async function getSceneTitles() {
  const scenes = await $$('[data-testid="scene-title"]');
  // Use textContent property instead of getText() for WebKit compatibility
  // Iterate manually because browser.execute doesn't work well with .map() on element arrays
  const titles = [];
  for (const scene of scenes) {
    const text = await browser.execute((el) => el.textContent?.trim() || "", scene);
    titles.push(text);
  }
  return titles;
}
