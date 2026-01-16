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

    // Check that textarea is now visible
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    expect(await textarea.isExisting()).toBe(true);
  });

  it("should collapse a beat when pressing Escape", async () => {
    // Expand a beat
    await expandBeat(0);
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    expect(await textarea.isExisting()).toBe(true);

    // Press Escape
    await browser.keys("Escape");

    // Textarea should no longer be visible
    await browser.waitUntil(
      async () => {
        const ta = await $('[data-testid="beat-prose-textarea"]');
        return !(await ta.isExisting());
      },
      { timeout: 2000 }
    );
  });

  it("should collapse a beat when clicking header again", async () => {
    // Expand and collapse by clicking
    await expandBeat(0);
    await expandBeat(0); // Click again

    // Textarea should be gone
    await browser.waitUntil(
      async () => {
        const ta = await $('[data-testid="beat-prose-textarea"]');
        return !(await ta.isExisting());
      },
      { timeout: 2000 }
    );
  });

  it("should auto-save prose after typing", async () => {
    await expandBeat(0);

    // Clear existing content first
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    await textarea.clearValue();
    await typeProse("The morning light filtered through the dusty window.");

    // Wait for the save to complete (indicator disappears when save is done)
    await waitForSaved();

    // Verify the textarea still has our content (save didn't clear it)
    const value = await textarea.getValue();
    expect(value).toContain("morning light");
  });

  it("should show saving indicator while saving", async () => {
    await expandBeat(0);

    // Start typing - should show "Saving..."
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    await textarea.setValue("Test content for saving");

    // Wait for saving indicator to appear (debounce is 1s, then "Saving..." shows)
    await browser.waitUntil(
      async () => {
        const indicator = await $('[data-testid="save-indicator"]');
        if (await indicator.isExisting()) {
          const text = await browser.execute((el) => el.textContent, indicator);
          return text && text.includes("Saving");
        }
        return false;
      },
      { timeout: 3000, timeoutMsg: "Save indicator did not appear" }
    );
  });

  it("should preserve prose when navigating away and back", async () => {
    const testProse = "Unique prose for navigation test - " + Date.now();

    // Write prose (clear first to avoid accumulation from previous tests)
    await expandBeat(0);
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    await textarea.clearValue();
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

    // Wait for textarea to appear after beat expansion
    await browser.waitUntil(
      async () => {
        const ta = await $('[data-testid="beat-prose-textarea"]');
        return await ta.isExisting();
      },
      { timeout: 3000, timeoutMsg: "Textarea did not appear after expanding beat" }
    );

    // Check prose is still there
    const resultTextarea = await $('[data-testid="beat-prose-textarea"]');
    const value = await resultTextarea.getValue();
    expect(value).toBe(testProse);
  });

  it("should collapse current beat when expanding another", async () => {
    // Expand first beat
    await expandBeat(0);
    let textareas = await $$('[data-testid="beat-prose-textarea"]');
    expect(textareas.length).toBe(1);

    // Expand second beat
    await expandBeat(1);

    // Should still only have one textarea visible
    textareas = await $$('[data-testid="beat-prose-textarea"]');
    expect(textareas.length).toBe(1);
  });
});
