import { describe, expect, it } from "vitest";
import {
  IMPORT_COMMANDS,
  IMPORT_FORMATS,
  importTypeForCommand,
  isImportType,
  supportsSync,
} from "./importFormats";
import { invoke as mockInvoke } from "../dev/mock-tauri";
import type { Project } from "./types";

describe("import format registry", () => {
  it("keeps picker, commands, source types, and dev imports aligned", async () => {
    for (const [type, config] of Object.entries(IMPORT_FORMATS)) {
      expect(isImportType(type)).toBe(true);
      expect(IMPORT_COMMANDS[type as keyof typeof IMPORT_COMMANDS]).toBe(config.command);
      expect(importTypeForCommand(config.command)).toBe(
        type === "longformVault" ? "longform" : type
      );
      const project = await mockInvoke<Project>(config.command, { path: "/fixture" });
      expect(project.source_type).toBe(config.sourceType);
      expect(project.source_path).toBe("/fixture");
      expect(supportsSync(config.sourceType)).toBe(config.sync);
    }
    expect(isImportType("__proto__")).toBe(false);
    expect(isImportType("missing")).toBe(false);
    expect(importTypeForCommand("delete_project")).toBeUndefined();
    expect(supportsSync("Blank")).toBe(false);
  });
});
