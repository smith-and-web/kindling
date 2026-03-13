import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ command }) => {
  // Mock Tauri layer disabled - use real Tauri backend (npm run tauri dev).
  // To re-enable mocks for browser-only testing: set useMocks = command === "serve" && !host
  const useMocks = false;

  return {
    plugins: [svelte(), tailwindcss()],
    resolve: {
      alias: {
        $lib: path.resolve("./src/lib"),
        ...(useMocks
          ? {
              "@tauri-apps/api/core": path.resolve("./src/dev/mock-tauri.ts"),
              "@tauri-apps/api/event": path.resolve("./src/dev/mock-event.ts"),
              "@tauri-apps/plugin-dialog": path.resolve("./src/dev/mock-dialog.ts"),
              "@tauri-apps/plugin-opener": path.resolve("./src/dev/mock-opener.ts"),
            }
          : {}),
      },
    },
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
