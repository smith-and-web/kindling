/**
 * E2E Test Helpers for Kindling
 *
 * Common utilities for interacting with the Kindling app
 */

import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
export const projectRoot = resolve(__dirname, "../..");
export const testDataDir = resolve(projectRoot, "test-data");

/**
 * Wait for the app to be ready (main content loaded)
 * On initial launch, the app shows StartScreen (import-section)
 * After loading a project, it shows the editor (sidebar)
 */
export async function waitForAppReady() {
  // Wait for the main element and either the start screen or editor to appear
  await browser.waitUntil(
    async () => {
      const main = await $("main");
      if (!(await main.isExisting())) return false;

      // Check if we're on the start screen (import-section) or editor (sidebar)
      const importSection = await $('[data-testid="import-section"]');
      const sidebar = await $('[data-testid="sidebar"]');
      return (await importSection.isExisting()) || (await sidebar.isExisting());
    },
    { timeout: 15000, timeoutMsg: "App did not load within 15 seconds" }
  );
}

/**
 * Wait for the start screen to be visible (before loading a project)
 */
export async function waitForStartScreen() {
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
 * Import a Plottr file via the file dialog
 * Note: This is tricky in e2e tests since native dialogs can't be controlled
 * We may need to use a test-specific import route
 */
export async function importPlottrFile(filename) {
  // For CI, we'll need to set up a test-specific import method
  // or use clipboard/drag-drop if supported
  console.log(`Would import: ${resolve(testDataDir, filename)}`);
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
