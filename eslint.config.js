import js from "@eslint/js";
import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: {
      globals: {
        // Browser globals
        console: "readonly",
        alert: "readonly",
        window: "readonly",
        document: "readonly",
        fetch: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        // DOM types
        Event: "readonly",
        KeyboardEvent: "readonly",
        DragEvent: "readonly",
        HTMLElement: "readonly",
        MouseEvent: "readonly",
      },
      parserOptions: {
        extraFileExtensions: [".svelte"],
      },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },
  {
    ignores: [
      "dist/",
      "node_modules/",
      "src-tauri/",
      ".svelte-kit/",
      "build/",
    ],
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "svelte/no-at-html-tags": "warn",
      // Disabled: Svelte 5.53 keyed #each triggers $.validate_each_keys runtime error in dev
      "svelte/require-each-key": "off",
    },
  }
);
