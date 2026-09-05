#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tokensPath = resolve(repoRoot, "src/styles/press/tokens.json");
const themePath = resolve(repoRoot, "src/styles/press/tailwind.css");
const checkOnly = process.argv.includes("--check");

const colors = {
  "press-bg": "--color-bg",
  "press-surface": "--color-surface",
  "press-sunken": "--color-surface-sunken",
  "press-text": "--color-text",
  "press-muted": "--color-text-muted",
  "press-accent": "--color-accent",
  "press-accent-text": "--color-accent-text",
  "press-flame": "--color-flame-inner",
  "press-on-accent": "--color-on-accent",
  "press-border": "--color-border",
  "press-focus": "--color-focus",
  "press-accent-wash": "--color-accent-wash",
  "press-success": "--color-success",
  "press-success-wash": "--color-success-wash",
  "press-error": "--color-error",
  "press-error-wash": "--color-error-wash",
  "press-warning": "--color-warning",
  "press-warning-wash": "--color-warning-wash",
  "press-info": "--color-info",
  "press-info-wash": "--color-info-wash",
  "press-overlay": "--color-overlay-scrim",
  "press-disabled-bg": "--color-disabled-bg",
  "press-disabled-text": "--color-disabled-text",
  "press-disabled-border": "--color-disabled-border",
  "press-prose-bg": "--color-prose-bg",
  "press-prose-text": "--color-prose-text",
  "press-prose-placeholder": "--color-prose-placeholder",
  "press-prose-border": "--color-prose-border",
  "press-prose-quote": "--color-prose-blockquote-border",
  "press-prose-code": "--color-prose-code-bg",
  "press-tag-red": "--color-tag-red",
  "press-tag-orange": "--color-tag-orange",
  "press-tag-yellow": "--color-tag-yellow",
  "press-tag-green": "--color-tag-green",
  "press-tag-teal": "--color-tag-teal",
  "press-tag-blue": "--color-tag-blue",
  "press-tag-purple": "--color-tag-purple",
  "press-tag-pink": "--color-tag-pink",
  "press-tag-slate": "--color-tag-slate",
  "press-tag-cyan": "--color-tag-cyan",
};

const fonts = {
  "press-display": "--font-display",
  "press-body": "--font-body",
  "press-ui": "--font-ui",
  "press-mono": "--font-mono",
};

const text = {
  "press-hero": "--text-hero",
  "press-h1": "--text-h1",
  "press-h2": "--text-h2",
  "press-h3": "--text-h3",
  "press-body-lg": "--text-body-lg",
  "press-body": "--text-body",
  "press-base": "--text-base",
  "press-ui": "--text-ui",
  "press-small": "--text-small",
  "press-eyebrow": "--text-eyebrow",
};

const shadows = {
  "press-sm": "--shadow-sm",
  "press-mounted": "--shadow-md",
  "press-overlay": "--shadow-overlay",
  "press-prose": "--shadow-prose",
};

const zIndexes = {
  "press-base": "--z-base",
  "press-raised": "--z-raised",
  "press-sticky": "--z-sticky",
  "press-dropdown": "--z-dropdown",
  "press-overlay": "--z-overlay",
  "press-modal": "--z-modal",
  "press-popover": "--z-popover",
  "press-toast": "--z-toast",
  "press-command-backdrop": "--z-command-backdrop",
  "press-command": "--z-command",
  "press-guidance-backdrop": "--z-guidance-backdrop",
  "press-guidance": "--z-guidance",
};

function declarations(namespace, values) {
  return Object.entries(values).map(
    ([name, token]) => `  --${namespace}-${name}: var(${token});`
  );
}

function render(tokens) {
  const required = [
    ...Object.values(colors),
    ...Object.values(fonts),
    ...Object.values(text),
    ...Object.values(shadows),
    ...Object.values(zIndexes),
    "--measure",
  ];
  for (const token of required) {
    if (!(token in tokens.light) || !(token in tokens.dark)) {
      throw new Error(`Press token mirror is missing ${token}`);
    }
  }

  return `/* Generated from tokens.json by scripts/generate-press-theme.mjs.\n   Do not edit: change tokens.css upstream, sync, and regenerate. */\n@theme inline {\n${[
    ...declarations("color", colors),
    ...declarations("font", fonts),
    ...declarations("text", text),
    ...declarations("shadow", shadows),
    ...declarations("z-index", zIndexes),
    "  --max-width-press-measure: var(--measure);",
  ].join("\n")}\n}\n`;
}

const tokens = JSON.parse(await readFile(tokensPath, "utf8"));
const expected = render(tokens);

if (checkOnly) {
  const actual = await readFile(themePath, "utf8").catch(() => "");
  if (actual !== expected) {
    console.error("Press Tailwind bridge is stale; run: npm run sync:design-system");
    process.exitCode = 1;
  } else {
    console.log("Press Tailwind bridge agrees with tokens.json");
  }
} else {
  await writeFile(themePath, expected);
  console.log("generated src/styles/press/tailwind.css from tokens.json");
}
