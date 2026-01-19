/**
 * Beat Editing E2E Tests (Feature #38)
 *
 * Tests for beat-level prose editing functionality
 */

import {
  waitForAppReady,
  skipOnboardingIfPresent,
  importPlottrFile,
  selectChapter,
  selectScene,
  expandBeat,
  typeProse,
  getProseContent,
  clearProseContent,
  waitForSaved,
} from "./helpers.js";

describe("Beat-Level Prose Editing (#38)", () => {
  before(async () => {
    await waitForAppReady();
    await skipOnboardingIfPresent();
    await importPlottrFile("simple-story.pltr");
    // Navigate to first chapter and scene for all tests
    await selectChapter("Act 1");
    await selectScene("The Beginning");
  });

  // Ensure beats are collapsed before each test to avoid shared state issues
  beforeEach(async () => {
    // Press Escape to collapse any expanded beat
    await browser.keys("Escape");
    await browser.pause(200);
  });

  it("should expand a beat when clicking the header", async () => {
    // Click on the first beat header
    await expandBeat(0);

    // Check that prose editor is now visible (TipTap contenteditable)
    const editor = await $('[data-testid="beat-prose-editor"]');
    expect(await editor.isExisting()).toBe(true);
  });

  it("should collapse a beat when pressing Escape", async () => {
    // Expand a beat
    await expandBeat(0);
    const editor = await $('[data-testid="beat-prose-editor"]');
    expect(await editor.isExisting()).toBe(true);

    // Press Escape
    await browser.keys("Escape");

    // Editor should no longer be visible
    await browser.waitUntil(
      async () => {
        const ed = await $('[data-testid="beat-prose-editor"]');
        return !(await ed.isExisting());
      },
      { timeout: 2000 }
    );
  });

  it("should collapse a beat when clicking header again", async () => {
    // Expand and collapse by clicking
    await expandBeat(0);
    await expandBeat(0); // Click again

    // Editor should be gone
    await browser.waitUntil(
      async () => {
        const ed = await $('[data-testid="beat-prose-editor"]');
        return !(await ed.isExisting());
      },
      { timeout: 2000 }
    );
  });

  it("should auto-save prose after typing", async () => {
    await expandBeat(0);

    // Clear existing content first
    await clearProseContent();
    await typeProse("The morning light filtered through the dusty window.");

    // Wait for the save to complete (indicator disappears when save is done)
    await waitForSaved();

    // Verify the editor still has our content (save didn't clear it)
    const value = await getProseContent();
    expect(value).toContain("morning light");
  });

  it("should show saving indicator while saving", async () => {
    await expandBeat(0);

    // Start typing - should show "Saving..."
    await typeProse("Test content for saving");

    // Wait for saving indicator to appear (debounce is 500ms, then "Saving..." shows)
    await browser.waitUntil(
      async () => {
        // The save status is shown in the NovelEditor toolbar
        const savingText = await $(".save-status.saving");
        return await savingText.isExisting();
      },
      { timeout: 3000, timeoutMsg: "Save indicator did not appear" }
    );
  });

  it("should preserve prose when navigating away and back", async () => {
    const testProse = "Unique prose for navigation test - " + Date.now();

    // Write prose (clear first to avoid accumulation from previous tests)
    await expandBeat(0);
    await clearProseContent();
    await typeProse(testProse);
    await waitForSaved();

    // Collapse beat before navigating
    await browser.keys("Escape");
    await browser.pause(200);

    // Navigate to different scene
    await selectScene("Discovery");

    // Navigate back
    await selectScene("The Beginning");
    await expandBeat(0);

    // Wait for editor to appear after beat expansion
    await browser.waitUntil(
      async () => {
        const ed = await $('[data-testid="beat-prose-editor"]');
        return await ed.isExisting();
      },
      { timeout: 3000, timeoutMsg: "Editor did not appear after expanding beat" }
    );

    // Check prose is still there
    const value = await getProseContent();
    expect(value).toBe(testProse);
  });

  it("should collapse current beat when expanding another", async () => {
    // Expand first beat
    await expandBeat(0);
    let editors = await $$('[data-testid="beat-prose-editor"]');
    expect(editors.length).toBe(1);

    // Expand second beat
    await expandBeat(1);

    // Should still only have one editor visible
    editors = await $$('[data-testid="beat-prose-editor"]');
    expect(editors.length).toBe(1);
  });
});
