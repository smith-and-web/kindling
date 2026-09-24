import { afterEach, describe, expect, it, vi } from "vitest";
import { open as pluginOpen, save as pluginSave } from "@tauri-apps/plugin-dialog";
import { open, save } from "./nativeDialog";
import { isModalOpen } from "./modalFocus";

afterEach(() => {
  vi.mocked(pluginOpen).mockReset();
  vi.mocked(pluginSave).mockReset();
});

describe("nativeDialog", () => {
  it("passes open through and counts it as a modal while it is showing", async () => {
    let pick!: (path: string) => void;
    vi.mocked(pluginOpen).mockReturnValue(new Promise((resolve) => (pick = resolve)) as never);
    const picked = open({ directory: true });
    expect(pluginOpen).toHaveBeenCalledWith({ directory: true });
    expect(isModalOpen()).toBe(true);
    pick("/tmp/vault");
    await expect(picked).resolves.toBe("/tmp/vault");
    expect(isModalOpen()).toBe(false);
  });

  it("passes save through and stops counting it when it is cancelled", async () => {
    vi.mocked(pluginSave).mockResolvedValue(null);
    const saving = save({ defaultPath: "draft.docx" });
    expect(isModalOpen()).toBe(true);
    await expect(saving).resolves.toBeNull();
    expect(pluginSave).toHaveBeenCalledWith({ defaultPath: "draft.docx" });
    expect(isModalOpen()).toBe(false);
  });
});
