import assert from "node:assert/strict";
import { test } from "node:test";
import { Runner } from "./run.mjs";

test("localize rewrites English attribute selectors in any quoting, and leaves the rest", async () => {
  const asked = [];
  const app = {
    evaluate: async (code) => {
      asked.push(code);
      return {
        "Close Find and Replace": ["Suchen und ersetzen schließen"],
        "QA Mara": ["QA Mara"],
        "Writer\\'s notes": ["Writer\\'s notes"],
      };
    },
  };
  const r = new Runner(app, "/tmp/qa-out");
  const code = `document.querySelector('[aria-label="Save profile"]')`;
  assert.equal(await r.localize(code), code, "no rewrite before the locale strings load");
  assert.equal(r.tr("Save profile"), "Save profile");

  r.strings = {
    "Save profile": ["Profil speichern"],
    "Line spacing": ["Zeilenabstand", "Zeilenabst."],
    "Writer's notes": ["Notizen „des Autors“", "l'auteur"],
  };
  assert.equal(r.tr("Save profile"), "Profil speichern");
  assert.equal(r.tr("QA Mara"), "QA Mara");
  assert.equal(
    await r.localize(`document.querySelector('[aria-label="Save profile"]')`),
    `document.querySelector('[aria-label="Profil speichern"]')`
  );
  assert.equal(
    await r.localize(JSON.stringify('nav [title="Line spacing"] button')),
    JSON.stringify('nav :is([title="Zeilenabstand"],[title="Zeilenabst."]) button')
  );
  assert.equal(
    await r.localize(`'[placeholder="Writer\\'s notes"]'`),
    `'[placeholder="Writer\\'s notes"]'`,
    "an escaped English apostrophe is a different string and is left alone"
  );
  assert.equal(
    (await r.localize(`x('[aria-label="Writer's notes"]')`)).includes(`l\\'auteur`),
    true,
    "apostrophes in translations are escaped for single-quoted JS strings"
  );
  assert.equal(
    await r.localize(`[aria-labelledby="Save profile"]`),
    `[aria-labelledby="Save profile"]`,
    "aria-labelledby is an id reference, not copy"
  );
  assert.equal(await r.localize(`[data-testid="Save profile"]`), `[data-testid="Save profile"]`);

  assert.equal(
    await r.localize(`[aria-label="Close Find and Replace"] [title="QA Mara"]`),
    `[aria-label="Suchen und ersetzen schließen"] [title="QA Mara"]`,
    "labels outside the catalog are resolved through the page once and cached"
  );
  const calls = asked.length;
  await r.localize(`[aria-label="Close Find and Replace"]`);
  assert.equal(asked.length, calls);
});
