import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import PartDeleteDialog from "./PartDeleteDialog.svelte";
import { setLocale } from "../i18n";
import type { Locale } from "../i18n/types";
import pluralOutput from "../i18n/plural-output.fixture.json";

afterEach(() => setLocale("en"));

const props = (count: number, chapterNoun?: "chapter" | "sequence") => ({
  partTitle: "Act One",
  childChapterCount: count,
  chapterNoun,
  onDeletePartOnly: vi.fn(),
  onDeletePartAndChapters: vi.fn(),
  onCancel: vi.fn(),
});

describe("PartDeleteDialog copy", () => {
  // The fixture holds the strings the dialog rendered with `{label}s` concatenation.
  for (const [locale, byCount] of Object.entries(pluralOutput)) {
    for (const [count, rows] of Object.entries(byCount)) {
      for (const noun of ["chapter", "sequence"] as const) {
        it(`renders ${count} ${noun}(s) as before in ${locale}`, () => {
          setLocale(locale as Locale);
          const row = rows as Record<string, string>;
          render(PartDeleteDialog, { props: props(Number(count), noun) });
          const dialog = screen.getByTestId("part-delete-dialog");
          expect(dialog.querySelector(".part-delete-lede")?.textContent?.trim()).toBe(
            row[`partDelete.lede.${noun}`]
          );
          const only = screen.getByTestId("delete-part-only");
          expect(only.querySelector("small")?.textContent?.trim()).toBe(
            row[`partDelete.deleteOnlyHint.${noun}`]
          );
          const all = screen.getByTestId("delete-part-and-chapters");
          expect(all.querySelector("span")?.textContent?.trim()).toBe(
            row[`partDelete.deleteAll.${noun}`]
          );
          expect(all.querySelector("small")?.textContent?.trim()).toBe(
            row[`partDelete.deleteAllHint.${noun}`]
          );
        });
      }
    }
  }

  it("defaults to chapters", () => {
    render(PartDeleteDialog, { props: { ...props(2), chapterNoun: undefined } });
    expect(screen.getByText("Delete part and all chapters")).toBeTruthy();
  });
});
