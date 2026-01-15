import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";

const app = mount(App, {
  target: document.getElementById("app")!,
});

// Expose Tauri invoke for E2E testing
// This allows WebDriver tests to call Tauri commands directly
declare global {
  interface Window {
    __KINDLING_TEST__?: {
      invoke: typeof invoke;
    };
  }
}

// Always expose for E2E testing - the test helper checks for this
window.__KINDLING_TEST__ = { invoke };

export default app;
