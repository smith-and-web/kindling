import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SluglineInput from "./SluglineInput.svelte";
import type { Location } from "../types";

afterEach(cleanup);

const locations = [{ name: "Lighthouse" }, { name: "Harbour" }] as Location[];

describe("SluglineInput", () => {
  it("parses the slugline into prefix, location and time", () => {
    render(SluglineInput, { value: "EXT. HARBOUR - NIGHT", onSave: vi.fn(), locations });
    expect(screen.getByRole("button", { name: "EXT." }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("button", { name: "INT." }).getAttribute("aria-pressed")).toBe("false");
    expect((screen.getByLabelText("Location") as HTMLInputElement).value).toBe("HARBOUR");
    expect((screen.getByLabelText("Time of day") as HTMLSelectElement).value).toBe("NIGHT");
  });

  it("saves when the prefix, time or location changes", async () => {
    const onSave = vi.fn();
    render(SluglineInput, { value: "INT. LIGHTHOUSE - DAY", onSave, locations });

    await fireEvent.click(screen.getByRole("button", { name: "EXT." }));
    expect(onSave).toHaveBeenLastCalledWith(expect.stringMatching(/^EXT\. LIGHTHOUSE - DAY$/));

    await fireEvent.change(screen.getByLabelText("Time of day"), { target: { value: "DUSK" } });
    expect(onSave).toHaveBeenLastCalledWith(expect.stringMatching(/DUSK$/));

    const location = screen.getByLabelText("Location");
    await fireEvent.input(location, { target: { value: "Harbour" } });
    await fireEvent.blur(location);
    expect(onSave).toHaveBeenLastCalledWith(expect.stringMatching(/HARBOUR|Harbour/));
  });

  it("keeps an unlisted time of day selectable", () => {
    render(SluglineInput, { value: "INT. LIGHTHOUSE - GOLDEN HOUR", onSave: vi.fn() });
    const time = screen.getByLabelText("Time of day") as HTMLSelectElement;
    expect(time.value).toBe("GOLDEN HOUR");
  });

  it("disables every control while saving", () => {
    render(SluglineInput, { value: "INT. LIGHTHOUSE - DAY", onSave: vi.fn(), disabled: true });
    for (const el of [
      screen.getByRole("button", { name: "INT." }),
      screen.getByLabelText("Location"),
      screen.getByLabelText("Time of day"),
    ]) {
      expect((el as HTMLButtonElement).disabled).toBe(true);
    }
  });
});
