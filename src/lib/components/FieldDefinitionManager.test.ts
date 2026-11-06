import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import FieldDefinitionManager from "./FieldDefinitionManager.svelte";
import { setLocale } from "../i18n";
import type { FieldEntityType } from "../types";

afterEach(() => {
  cleanup();
  setLocale("en");
  vi.mocked(invoke).mockReset();
});

it.each([
  ["en", "character", "Characters", "Character fields"],
  ["de", "character", "Figuren", "Felder für Figuren"],
  ["ru", "location", "Места", "Поля мест"],
  ["es", "scene", "Escenas", "Campos de escenas"],
  ["pt-BR", "dragon", "Dragões", "Campos: Dragões"],
] as const)("titles the %s %s fields as a whole phrase", (locale, entityType, label, title) => {
  setLocale(locale);
  vi.mocked(invoke).mockResolvedValue([]);
  render(FieldDefinitionManager, {
    projectId: "p1",
    entityType: entityType as FieldEntityType,
    entityLabel: label,
  });
  expect(screen.getByRole("heading", { name: title })).toBeTruthy();
});
