/**
 * Mock @tauri-apps/plugin-opener for browser-only dev.
 * revealItemInDir would open Finder/Explorer - no-op in browser.
 */

export async function revealItemInDir(_path: string | string[]): Promise<void> {
  console.debug("[mock-opener] revealItemInDir() (no-op in browser)");
}

export async function openPath(_path: string): Promise<void> {
  console.debug("[mock-opener] openPath() (no-op in browser)");
}

export async function openUrl(_url: string): Promise<void> {
  console.debug("[mock-opener] openUrl() (no-op in browser)");
}
