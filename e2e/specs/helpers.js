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

  // Invoke Tauri command directly via browser executeAsync
  // Using executeAsync because Tauri invoke returns a Promise
  // The app exposes window.__KINDLING_TEST__.invoke for E2E testing
  const result = await browser.executeAsync(async (path, done) => {
    try {
      // The app exposes invoke via __KINDLING_TEST__ for E2E testing
      if (!window.__KINDLING_TEST__?.invoke) {
        throw new Error("__KINDLING_TEST__.invoke not available");
      }
      const project = await window.__KINDLING_TEST__.invoke("import_plottr", {
        path,
      });
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
 * Select a chapter in the sidebar
 */
export async function selectChapter(chapterTitle) {
  const chapter = await $(
    `[data-testid="chapter-item"]:has-text("${chapterTitle}")`
  );
  await chapter.click();
}

/**
 * Select a scene in the sidebar
 */
export async function selectScene(sceneTitle) {
  const scene = await $(
    `[data-testid="scene-item"]:has-text("${sceneTitle}")`
  );
  await scene.click();
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
 * Wait for save indicator to show "Saved"
 */
export async function waitForSaved() {
  await browser.waitUntil(
    async () => {
      const indicator = await $('[data-testid="save-indicator"]');
      const text = await indicator.getText();
      return text.includes("Saved");
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
  return Promise.all(chapters.map((c) => c.getText()));
}

/**
 * Get all scene titles for the expanded chapter
 */
export async function getSceneTitles() {
  const scenes = await $$('[data-testid="scene-title"]');
  return Promise.all(scenes.map((s) => s.getText()));
}
