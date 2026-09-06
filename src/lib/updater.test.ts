import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";
import { session } from "./stores/session.svelte";

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
  afterEach(() => vi.restoreAllMocks());
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
    it("waits for the position flush before entering an installer that may exit the process", async () => {
      let finishFlush!: () => void;
      vi.spyOn(session, "flush").mockReturnValue(
        new Promise<void>((resolve) => {
          finishFlush = resolve;
        })
      );
      let finishInstall!: () => void;
      const install = vi.fn(
        () =>
          new Promise<void>((resolve) => {
            finishInstall = resolve;
          })
      );
      const installing = installAndRelaunch({
        ready: true,
        version: "1.2.0",
        body: null,
        update: { install } as never,
      });
      expect(install).not.toHaveBeenCalled();
      finishFlush();
      await vi.waitFor(() => expect(install).toHaveBeenCalledOnce());
      expect(relaunchMock).not.toHaveBeenCalled();
      finishInstall();
      await installing;
    });

    it("flushes even if installation fails", async () => {
      vi.spyOn(console, "error").mockImplementation(() => {});
      const flushed = vi.spyOn(session, "flush").mockResolvedValue(undefined);
      const install = vi.fn(async () => {
        expect(flushed).toHaveBeenCalledOnce();
        throw new Error("install failed");
      });
      await installAndRelaunch({
        ready: true,
        version: "1.2.0",
        body: null,
        update: { install } as never,
      });
      expect(install).toHaveBeenCalledOnce();
      expect(relaunchMock).not.toHaveBeenCalled();
    });
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
