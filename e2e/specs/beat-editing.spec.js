/**
 * Beat Editing E2E Tests (Feature #38)
 *
 * Tests for beat-level prose editing functionality
 */

import {
  waitForAppReady,
  selectChapter,
  selectScene,
  expandBeat,
  typeProse,
  waitForSaved,
} from "./helpers.js";

describe("Beat-Level Prose Editing (#38)", () => {
  beforeEach(async () => {
    await waitForAppReady();
    // Tests assume a project is already imported
    // In CI, we'd set up test data beforehand
  });

  it("should expand a beat when clicking the header", async () => {
    // Select a chapter and scene first
    await selectChapter("Act 1");
    await selectScene("The Beginning");

    // Click on the first beat header
    await expandBeat(0);

    // Check that textarea is now visible
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    expect(await textarea.isExisting()).toBe(true);
  });

  it("should collapse a beat when pressing Escape", async () => {
    await selectChapter("Act 1");
    await selectScene("The Beginning");

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
    await selectChapter("Act 1");
    await selectScene("The Beginning");

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
    await selectChapter("Act 1");
    await selectScene("The Beginning");

    await expandBeat(0);
    await typeProse("The morning light filtered through the dusty window.");

    // Wait for the save indicator
    await waitForSaved();

    // Verify indicator shows "Saved"
    const indicator = await $('[data-testid="save-indicator"]');
    expect(await indicator.getText()).toContain("Saved");
  });

  it("should show saving indicator while saving", async () => {
    await selectChapter("Act 1");
    await selectScene("The Beginning");

    await expandBeat(0);

    // Start typing - should show "Saving..."
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    await textarea.setValue("Test");

    // Check for saving indicator (may be brief)
    const indicator = await $('[data-testid="save-indicator"]');
    const text = await indicator.getText();
    expect(text === "Saving..." || text === "Saved").toBe(true);
  });

  it("should preserve prose when navigating away and back", async () => {
    const testProse = "This is my test prose content.";

    await selectChapter("Act 1");
    await selectScene("The Beginning");

    // Write prose
    await expandBeat(0);
    await typeProse(testProse);
    await waitForSaved();

    // Navigate to different scene
    await selectScene("Discovery");

    // Navigate back
    await selectScene("The Beginning");
    await expandBeat(0);

    // Check prose is still there
    const textarea = await $('[data-testid="beat-prose-textarea"]');
    const value = await textarea.getValue();
    expect(value).toBe(testProse);
  });

  it("should collapse current beat when expanding another", async () => {
    await selectChapter("Act 1");
    await selectScene("The Beginning");

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
