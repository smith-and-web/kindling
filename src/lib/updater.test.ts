import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

const { check } = await import("@tauri-apps/plugin-updater");
const { relaunch } = await import("@tauri-apps/plugin-process");
const { updateState, checkForUpdate, installAndRelaunch, dismissUpdate } =
  await import("./updater");

const checkMock = vi.mocked(check);
const relaunchMock = vi.mocked(relaunch);

describe("updater", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    updateState.set(null);
  });

  describe("checkForUpdate", () => {
    it("skips in dev mode", async () => {
      const origDev = import.meta.env.DEV;
      import.meta.env.DEV = true;

      await checkForUpdate();

      expect(checkMock).not.toHaveBeenCalled();
      import.meta.env.DEV = origDev;
    });

    it("does nothing when no update is available", async () => {
      const origDev = import.meta.env.DEV;
      import.meta.env.DEV = false;

      checkMock.mockResolvedValue(null as never);

      await checkForUpdate();

      expect(checkMock).toHaveBeenCalled();
      expect(get(updateState)).toBeNull();
      import.meta.env.DEV = origDev;
    });

    it("downloads and sets state when update is available", async () => {
      const origDev = import.meta.env.DEV;
      import.meta.env.DEV = false;

      const mockUpdate = {
        version: "1.2.0",
        body: "Release notes",
        download: vi.fn().mockResolvedValue(undefined),
        install: vi.fn(),
      };
      checkMock.mockResolvedValue(mockUpdate as never);

      await checkForUpdate();

      expect(mockUpdate.download).toHaveBeenCalled();
      const state = get(updateState);
      expect(state).not.toBeNull();
      expect(state!.ready).toBe(true);
      expect(state!.version).toBe("1.2.0");
      expect(state!.body).toBe("Release notes");
      import.meta.env.DEV = origDev;
    });

    it("sets body to null when update has no body", async () => {
      const origDev = import.meta.env.DEV;
      import.meta.env.DEV = false;

      const mockUpdate = {
        version: "1.2.0",
        body: undefined,
        download: vi.fn().mockResolvedValue(undefined),
        install: vi.fn(),
      };
      checkMock.mockResolvedValue(mockUpdate as never);

      await checkForUpdate();

      const state = get(updateState);
      expect(state!.body).toBeNull();
      import.meta.env.DEV = origDev;
    });

    it("handles check failure gracefully", async () => {
      const origDev = import.meta.env.DEV;
      import.meta.env.DEV = false;

      const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
      checkMock.mockRejectedValue(new Error("Network error"));

      await checkForUpdate();

      expect(warnSpy).toHaveBeenCalledWith("Update check failed:", expect.any(Error));
      expect(get(updateState)).toBeNull();
      warnSpy.mockRestore();
      import.meta.env.DEV = origDev;
    });
  });

  describe("installAndRelaunch", () => {
    it("installs and relaunches", async () => {
      const mockUpdate = {
        install: vi.fn().mockResolvedValue(undefined),
      };
      const state = {
        ready: true,
        version: "1.2.0",
        body: null,
        update: mockUpdate as never,
      };

      await installAndRelaunch(state);

      expect(mockUpdate.install).toHaveBeenCalled();
      expect(relaunchMock).toHaveBeenCalled();
    });

    it("does nothing when update object is null", async () => {
      const state = {
        ready: true,
        version: "1.2.0",
        body: null,
        update: null as never,
      };

      await installAndRelaunch(state);

      expect(relaunchMock).not.toHaveBeenCalled();
    });

    it("handles install failure gracefully", async () => {
      const errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      const mockUpdate = {
        install: vi.fn().mockRejectedValue(new Error("Install failed")),
      };
      const state = {
        ready: true,
        version: "1.2.0",
        body: null,
        update: mockUpdate as never,
      };

      await installAndRelaunch(state);

      expect(errorSpy).toHaveBeenCalledWith("Failed to install update:", expect.any(Error));
      errorSpy.mockRestore();
    });
  });

  describe("dismissUpdate", () => {
    it("clears the update state", () => {
      updateState.set({
        ready: true,
        version: "1.2.0",
        body: null,
        update: null as never,
      });

      dismissUpdate();

      expect(get(updateState)).toBeNull();
    });
  });
});
