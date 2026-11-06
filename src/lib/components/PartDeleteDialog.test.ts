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
  // Screenplays group sequences into acts; novels group chapters into parts.
  partKind: chapterNoun === "sequence" ? ("act" as const) : ("part" as const),
  onDeletePartOnly: vi.fn(),
  onDeletePartAndChapters: vi.fn(),
  onCancel: vi.fn(),
});

describe("PartDeleteDialog copy", () => {
  // The fixture holds the strings the dialog rendered with `{label}s` concatenation. The
  // delete-all copy now has one key per part kind, so only the shared lines are compared.
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
        });
      }
    }
  }

  it("defaults to a part holding chapters", () => {
    render(PartDeleteDialog, {
      props: { ...props(2), chapterNoun: undefined, partKind: undefined },
    });
    expect(screen.getByText("Delete part and all chapters")).toBeTruthy();
    expect(screen.getByText("This permanently deletes the part and its 2 chapters.")).toBeTruthy();
  });

  it.each([
    ["en", "part", 1, "Delete Part", "Delete part only", "Delete part and all chapters"],
    ["en", "act", 2, "Delete Act", "Delete act only", "Delete act and all sequences"],
    ["de", "part", 2, "Teil löschen", "Nur Teil löschen", "Teil und alle Kapitel löschen"],
    ["ru", "act", 5, "Удалить акт", "Удалить только акт", "Удалить акт и все секвенции"],
  ] as const)("agrees with the %s %s in every line", (locale, kind, count, title, only, all) => {
    setLocale(locale);
    const noun = kind === "act" ? "sequence" : "chapter";
    render(PartDeleteDialog, { props: { ...props(count, noun), partKind: kind } });
    expect(screen.getByRole("heading", { name: title })).toBeTruthy();
    expect(screen.getByTestId("delete-part-only").querySelector("span")?.textContent).toBe(only);
    const button = screen.getByTestId("delete-part-and-chapters");
    expect(button.querySelector("span")?.textContent).toBe(all);
    expect(button.querySelector("small")?.textContent).not.toMatch(/\{|\(/);
  });

  it("puts Russian chapter counts in the hint's own case", () => {
    setLocale("ru");
    render(PartDeleteDialog, { props: props(3, "chapter") });
    expect(
      screen.getByTestId("delete-part-and-chapters").querySelector("small")?.textContent?.trim()
    ).toBe("Часть и 3 главы в ней будут удалены безвозвратно.");
  });
});
