import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

// Plain paths keep the alias working from a checkout whose folder name is not ASCII.
const fromHere = (relative: string) => fileURLToPath(new URL(relative, import.meta.url));

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  resolve: {
    alias: {
      $lib: fromHere("./src/lib"),
      $press: fromHere("./src/styles/press"),
    },
    conditions: ["browser"],
  },
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    coverage: {
      provider: "v8",
      include: ["src/lib/**/*.ts"],
      exclude: [
        "src/lib/**/*.test.ts",
        "src/lib/**/*.spec.ts",
        "src/lib/types.ts",
        "src/lib/utils/import.ts",
      ],
      reporter: ["text", "html", "json-summary"],
      thresholds: {
        statements: 95,
        branches: 65,
        functions: 98,
        lines: 95,
      },
    },
  },
});
