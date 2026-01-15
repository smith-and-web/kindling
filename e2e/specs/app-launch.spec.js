/**
 * App Launch E2E Tests
 *
 * Basic smoke tests to verify the app launches correctly
 */

import { waitForAppReady } from "./helpers.js";

describe("Kindling App Launch", () => {
  it("should launch and show the welcome state", async () => {
    // Wait for app to be ready
    await waitForAppReady();

    // Check that the main layout is visible
    const main = await $("main");
    expect(await main.isExisting()).toBe(true);
  });

  it("should show the sidebar", async () => {
    const sidebar = await $('[data-testid="sidebar"]');
    expect(await sidebar.isExisting()).toBe(true);
  });

  it("should show recent projects section", async () => {
    const recentProjects = await $('[data-testid="recent-projects"]');
    expect(await recentProjects.isExisting()).toBe(true);
  });

  it("should show import options", async () => {
    // Check for import buttons (Plottr, Scrivener, etc.)
    const importSection = await $('[data-testid="import-section"]');
    expect(await importSection.isExisting()).toBe(true);
  });
});
