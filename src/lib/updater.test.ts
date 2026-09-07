import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { synopsisSaves } from "./stores/synopsisSaves.svelte";
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
  afterEach(async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await synopsisSaves.flush();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(undefined);
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
    it("saves a debounced synopsis before entering the installer", async () => {
      vi.useFakeTimers();
      synopsisSaves.stage({
        projectId: "project",
        sceneId: "scene",
        synopsis: "Last synopsis edit",
      });
      let finish!: () => void;
      vi.mocked(invoke).mockImplementation(async (cmd) => {
        if (cmd === "save_scene_synopsis")
          await new Promise<void>((resolve) => {
            finish = resolve;
          });
      });
      const install = vi.fn().mockResolvedValue(undefined);
      const installing = installAndRelaunch({
        ready: true,
        version: "1.2.0",
        body: null,
        update: { install } as never,
      });
      await vi.advanceTimersByTimeAsync(0);
      try {
        expect(invoke).toHaveBeenCalledWith("save_scene_synopsis", {
          sceneId: "scene",
          synopsis: "Last synopsis edit",
        });
        expect(install).not.toHaveBeenCalled();
        expect(relaunchMock).not.toHaveBeenCalled();
      } finally {
        finish?.();
      }
      await installing;
      expect(install).toHaveBeenCalledOnce();
      expect(relaunchMock).toHaveBeenCalledOnce();
    });

    it.each([false, true])(
      "blocks installation until a failed synopsis can be saved (already retained: %s)",
      async (retained) => {
        vi.useFakeTimers();
        const draft = { projectId: "project", sceneId: "scene", synopsis: "Do not lose this" };
        synopsisSaves.stage(draft);
        vi.mocked(invoke).mockRejectedValue("disk full");
        if (retained) await expect(synopsisSaves.flush()).rejects.toThrow("disk full");
        const install = vi.fn().mockResolvedValue(undefined);
        const state = { ready: true, version: "1.2.0", body: null, update: { install } as never };
        await expect(installAndRelaunch(state)).rejects.toThrow("disk full");
        expect(install).not.toHaveBeenCalled();
        expect(relaunchMock).not.toHaveBeenCalled();
        expect(synopsisSaves.getState("project", "scene").draft).toEqual(draft);
        vi.mocked(invoke).mockResolvedValue(undefined);
        await installAndRelaunch(state);
        expect(install).toHaveBeenCalledOnce();
        expect(relaunchMock).toHaveBeenCalledOnce();
      }
    );

    it.each([false, true])(
      "continues update installation after a failed position flush (installer fails: %s)",
      async (installFails) => {
        const saveError = new Error("Database is locked");
        const installError = new Error("Installer failed");
        const errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
        const flush = vi.spyOn(session, "flush").mockRejectedValue(saveError);
        const install = vi.fn(async () => {
          expect(flush).toHaveBeenCalledOnce();
          expect(errorSpy).toHaveBeenCalledWith(
            "Failed to save writing position before updating:",
            saveError
          );
          if (installFails) throw installError;
        });

        const installing = installAndRelaunch({
          ready: true,
          version: "1.2.0",
          body: null,
          update: { install } as never,
        });
        if (installFails) await expect(installing).rejects.toBe(installError);
        else await expect(installing).resolves.toBeUndefined();

        expect(install).toHaveBeenCalledOnce();
        if (installFails) {
          expect(errorSpy).toHaveBeenCalledWith("Failed to install update:", installError);
          expect(relaunchMock).not.toHaveBeenCalled();
        } else {
          expect(errorSpy).toHaveBeenCalledTimes(1);
          expect(relaunchMock).toHaveBeenCalledOnce();
        }
      }
    );

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
      await expect(
        installAndRelaunch({
          ready: true,
          version: "1.2.0",
          body: null,
          update: { install } as never,
        })
      ).rejects.toThrow("install failed");
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

    it("propagates install failure to the banner", async () => {
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

      await expect(installAndRelaunch(state)).rejects.toThrow("Install failed");

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
