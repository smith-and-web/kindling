import { describe, expect, it } from "vitest";
import {
  chapterLabel,
  profileForStorage,
  profileStorageKey,
  proseText,
  compileWorkspaceDocument,
  decodeProfiles,
  exportFilename,
  manuscriptWords,
  previewFilename,
  profileValidation,
  renderPreview,
  safeProse,
  selectedChapters,
  starterProfiles,
  type PreviewChapter,
} from "./exportPrototype";

const chapters: PreviewChapter[] = [
  {
    id: "a",
    title: "Arrival & Departure",
    part: "Part One",
    scenes: [
      {
        id: "s1",
        title: "The station",
        synopsis: "A <strong>quiet</strong> arrival.",
        blocks: [
          { heading: "Arrival", html: "<p>One <em>quiet</em> morning.</p><p>Another train.</p>" },
        ],
      },
      {
        id: "s2",
        title: "The crossing",
        synopsis: null,
        blocks: [{ heading: null, html: "<p>She left.</p>" }],
      },
    ],
  },
  {
    id: "b",
    title: "Later",
    part: "Part One",
    scenes: [
      {
        id: "s3",
        title: "Home",
        synopsis: null,
        blocks: [{ heading: null, html: "<p>Home at last.</p>" }],
      },
    ],
  },
];

describe("export prototype content and persistence", () => {
  it("requires usable numeric settings before committing a profile or saving output", () => {
    const p = starterProfiles("Book", "")[0];
    expect(profileValidation(p)).toBe("");
    p.fontSize = Number.NaN;
    expect(profileValidation(p)).toContain("Font size");
    p.fontSize = 12;
    p.startNumber = 1.5;
    expect(profileValidation(p)).toContain("whole number");
  });
  it("retains formatting but removes executable elements, attributes, and remote resources", () => {
    const safe = safeProse(
      '<p onclick="alert(1)">Hello <em>world</em><img src="https://example.com/tracker"><script>secret()</script><a href="javascript:alert(1)"> link</a><svg><text>bad</text></svg></p>'
    );
    expect(safe).toBe("<p>Hello <em>world</em> link</p>");
  });
  it("preserves manuscript order while narrowing chapters and scenes, without whole-project fallback", () => {
    const p = starterProfiles("Book", "Author")[0];
    p.selection = "selected";
    p.chapterIds = ["b", "a"];
    expect(selectedChapters(chapters, p).map((c) => c.id)).toEqual(["a", "b"]);
    p.sceneId = "s2";
    expect(selectedChapters(chapters, p).flatMap((c) => c.scenes.map((s) => s.id))).toEqual(["s2"]);
    p.chapterIds = ["missing"];
    expect(selectedChapters(chapters, p)).toEqual([]);
  });
  it("counts prose only with paragraph boundaries, excluding beat prompts and synopses", () => {
    expect(manuscriptWords(chapters)).toBe(10);
  });
  it("escapes metadata and custom markers and isolates a selected preview chapter", () => {
    const p = starterProfiles('<img src=x onerror="bad()">', "A & B")[0];
    p.separator = "<script>bad()</script>";
    p.titlePage = true;
    const full = renderPreview(chapters, p).document;
    expect(full).toContain("&lt;img");
    expect(full).toContain("&lt;script&gt;");
    expect(full).not.toContain("<script>");
    expect(full).toContain("Content-Security-Policy");
    const sample = renderPreview(chapters, p, { chapterId: "b" }).body;
    expect(sample).toContain("Chapter 2: Later");
    expect(sample).not.toContain("One <em>quiet");
  });
  it("renders clean fragments with no styles when requested, while preserving semantic headings", () => {
    const p = starterProfiles("Book", "")[2];
    p.styled = false;
    p.htmlMode = "fragment";
    const result = renderPreview(chapters, p);
    expect(result.output).not.toContain("<style>");
    expect(result.output).not.toContain("<!doctype");
    expect(result.output).toContain("<h2>Arrival &amp; Departure</h2>");
    expect(result.output).toContain("<em>quiet</em>");
    expect(result.document).toContain("<!doctype html>");
  });
  it("round-trips saved profiles and rejects corrupted or unsupported settings", () => {
    const profiles = starterProfiles("Book", "Author");
    const raw = JSON.stringify({ version: 1, profiles, activeId: "website" });
    expect(decodeProfiles(raw)).toEqual({ profiles, activeId: "website" });
    expect(() => decodeProfiles(raw.replace('"version":1', '"version":99'))).toThrow();
    expect(() => decodeProfiles(raw.replace('"lineSpacing":2', '"lineSpacing":"2"'))).toThrow();
    expect(() => decodeProfiles(raw.replace('"format":"docx"', '"format":"script"'))).toThrow();
    expect(() => decodeProfiles('{"version":1,"profiles":[null]}')).toThrow();
  });
  it("resolves filename tokens as a filename, never as directory components", () => {
    const p = starterProfiles("../A/Book", "")[0];
    p.fileName = "{title}-{date}";
    expect(previewFilename(p, new Date("2026-09-11T12:00:00Z"))).toBe("-A-Book-2026-09-11.html");
  });
  it("upgrades existing prototype profiles without losing custom settings", () => {
    const old = { ...starterProfiles("My book", "Author")[0], font: "Georgia" };
    for (const key of [
      "description",
      "coverPath",
      "includeNotes",
      "includeBeatComments",
      "treatmentLevel",
      "treatmentFormat",
    ])
      delete (old as Record<string, unknown>)[key];
    const result = decodeProfiles(
      JSON.stringify({ version: 1, profiles: [old], activeId: old.id })
    );
    expect(result.profiles[0]).toMatchObject({
      title: "My book",
      font: "Georgia",
      coverPath: "",
      includeNotes: true,
      treatmentFormat: "docx",
    });
    expect(decodeProfiles(JSON.stringify({ version: 2, ...result }))).toEqual(result);
  });
  it("compiles selected Word content with resolved headings and list markers", () => {
    const p = starterProfiles("Book", "Author")[0];
    p.selection = "selected";
    p.chapterIds = ["a"];
    p.sceneTitles = true;
    p.beatHeadings = true;
    p.startNumber = 4;
    p.font = "Georgia";
    const input = structuredClone(chapters);
    input[0].scenes[0].blocks[0].html +=
      "<ul><li><p>Bullet</p></li></ul><ol><li><p>Number</p></li></ol>";
    const result = compileWorkspaceDocument(input, p);
    expect(result.chapters).toHaveLength(1);
    expect(result.chapters[0]).toMatchObject({
      title: "Chapter 4: Arrival & Departure",
      part: "Part One",
    });
    expect(result.chapters[0].scenes[0].blocks[0].html).toContain("<p>• Bullet</p>");
    expect(result.chapters[0].scenes[0].blocks[0].html).toContain("<p>1. Number</p>");
    expect(result.layout.font).toBe("Georgia");
    expect(input[0].scenes[0].blocks[0].html).not.toContain("• Bullet");
  });
  it("creates valid EPUB XHTML with selected layout and title matter", () => {
    const p = starterProfiles("Book & <friends>", "Author")[1];
    p.format = "epub";
    p.titlePage = true;
    p.font = "Georgia";
    const result = compileWorkspaceDocument(chapters, p);
    for (const xhtml of [result.frontMatter!, ...result.chapters.map((c) => c.xhtml)]) {
      const xml = new DOMParser().parseFromString(xhtml, "application/xhtml+xml");
      expect(xml.querySelector("parsererror")).toBeNull();
      expect(xml.documentElement.namespaceURI).toBe("http://www.w3.org/1999/xhtml");
      expect(xml.querySelector("style")?.textContent).toContain("Georgia");
    }
    expect(result.frontMatter).toContain("Book &amp; &lt;friends&gt;");
    expect(result.chapters[1].xhtml).not.toContain('<div class="part">');
  });
  it("exports readable Markdown and plain text with format-specific filenames", () => {
    const p = starterProfiles("Book <title>", "Author")[0];
    p.format = "markdown";
    p.titlePage = true;
    p.fileName = "{title}";
    let result = compileWorkspaceDocument(chapters, p);
    expect(result.text).toContain("# Book \\<title\\>");
    expect(result.text).toContain("One *quiet* morning\\.");
    expect(exportFilename(p)).toBe("Book -title-.md");
    p.format = "txt";
    // Hidden typography must not prevent plain-text output.
    p.fontSize = Number.NaN;
    expect(profileValidation(p)).toBe("");
    result = compileWorkspaceDocument(chapters, p);
    expect(result.text).toContain("One quiet morning.");
    expect(result.text).not.toContain("<em>");
    expect(Number.isFinite(result.layout.fontSize)).toBe(true);
    p.format = "scrivener";
    expect(exportFilename(p)).toBe("Book -title-.scriv");
    p.format = "longform";
    expect(exportFilename(p)).toBe("Book -title-");
    p.format = "treatment";
    p.treatmentFormat = "txt";
    expect(exportFilename(p)).toBe("Book -title-.txt");
  });
});

describe("export edge cases", () => {
  it("normalizes unfinished hidden controls without accepting unfinished visible controls", () => {
    const p = starterProfiles("Book", "Author")[0];
    p.fontSize = undefined as unknown as number;
    expect(() => profileForStorage(p)).toThrow("Font size");
    p.format = "markdown";
    const saved = profileForStorage(p);
    expect(saved.fontSize).toBe(12);
    expect(p.fontSize).toBeUndefined();
    expect(
      decodeProfiles(JSON.stringify({ version: 2, profiles: [saved], activeId: saved.id }))
        .profiles[0]
    ).toEqual(saved);
    p.startNumber = NaN;
    expect(() => profileForStorage(p)).toThrow("Chapter start number");
    p.format = "longform";
    expect(profileForStorage(p).startNumber).toBe(1);
    p.name = " ";
    expect(() => profileForStorage(p)).toThrow("name");
    expect(profileStorageKey("one")).not.toBe(profileStorageKey("two"));
  });
  it("rejects duplicate identities and malformed persisted selection fields", () => {
    const p = starterProfiles("Book", "")[0];
    const decode = (profiles: unknown[]) =>
      decodeProfiles(JSON.stringify({ version: 2, profiles, activeId: p.id }));
    expect(() => decode([p, p])).toThrow("duplicate IDs");
    expect(() => decode([{ ...p, chapterIds: [1] }])).toThrow("invalid");
    expect(() => decode([{ ...p, sceneId: 42 }])).toThrow("invalid");
    expect(() => decode([{ ...p, sceneTitles: "true" }])).toThrow("invalid");
    expect(() => decode([])).toThrow("unsupported");
  });
  it("renders Roman numbering, heading-free contents and rounded word counts", () => {
    const p = starterProfiles("Book", "Author")[0];
    p.numberStyle = "roman";
    p.startNumber = 944;
    p.chapterHeading = "number";
    expect(chapterLabel(chapters[0], 0, p)).toBe("Chapter CMXLIV");
    p.chapterHeading = "none";
    p.contents = true;
    p.subtitle = "A subtitle";
    const input = structuredClone(chapters);
    input[0].scenes[0].blocks[0].html = `<p>${"word ".repeat(1234)}</p>`;
    expect(compileWorkspaceDocument(input, p).wordCount).toBe("1,000 words (approx.)");
    const result = renderPreview(input, p);
    expect(result.body).toContain("1,000 words (approx.)");
    expect(result.body).toContain("Arrival &amp; Departure</a>");
    expect(result.body).not.toContain("Chapter ");
    expect(compileWorkspaceDocument(input, p).chapters[0].navigationTitle).toBe(chapters[0].title);
    p.title = "";
    p.fileName = "../";
    expect(renderPreview(input, p).document).toContain("Untitled manuscript");
    expect(previewFilename(p)).toBe("-.html");
    p.fileName = "...";
    expect(previewFilename(p)).toBe("manuscript-preview.html");
  });
  it("preserves Markdown structure and literal punctuation without executable markup", () => {
    const p = starterProfiles("Book", "")[0];
    p.format = "markdown";
    p.titlePage = false;
    p.synopses = true;
    p.sceneTitles = true;
    p.beatHeadings = true;
    const input = structuredClone(chapters);
    input[0].scenes[0].blocks[0].html =
      "<h1>One</h1><h2>Two</h2><h3>Three</h3><h4>Four</h4><blockquote><p>Quoted<br>line</p></blockquote><ul><li><p>Bullet <b>bold</b></p></li></ul><ol><li><i>italic</i></li><li><del>old</del> <u>plain</u></li></ol><hr><!-- ignored -->";
    const text = compileWorkspaceDocument(input, p).text;
    for (const expected of [
      "# One",
      "## Two",
      "### Three",
      "#### Four",
      "> Quoted",
      "> line",
      "- Bullet **bold**",
      "1. *italic*",
      "2. ~~old~~ plain",
      "---",
      "#### Arrival",
    ])
      expect(text).toContain(expected);
    expect(text).not.toContain("ignored");
    expect(proseText("<div>A<br>B</div><script>hidden</script>")).toBe("A\nB");
  });
  it.each(["html", "epub"] as const)(
    "flushes prose after headings while skipping synopses in %s",
    (format) => {
      const p = starterProfiles("Book", "")[0];
      Object.assign(p, { format, firstParagraphFlush: true, synopses: true, beatHeadings: true });
      const input = structuredClone(chapters);
      input[0].scenes[0].blocks.push({
        heading: "Second beat",
        html: "<p>Next beat.</p><p>Indented.</p><h2>Embedded heading</h2><p>After embedded heading.</p>",
      });
      const html = new DOMParser().parseFromString(renderPreview(input, p).document, "text/html");
      const scene = html.querySelector(".scene")!;
      expect(scene.querySelector(".synopsis")!.classList.contains("flush")).toBe(false);
      expect([...scene.querySelectorAll("p.flush")].map((p) => p.textContent)).toEqual([
        "One quiet morning.",
        "Next beat.",
        "After embedded heading.",
      ]);
      p.firstParagraphFlush = false;
      expect(renderPreview(input, p).body).not.toContain('class="flush"');
    }
  );
});

it.each(["epub", "html"] as const)(
  "allows hidden unfinished margins in %s and emits a valid layout",
  (format) => {
    const p = starterProfiles("Book", "")[0];
    p.margin = undefined as unknown as number;
    expect(profileValidation(p)).toContain("Margins");
    p.format = format;
    expect(profileValidation(p)).toBe("");
    expect(profileForStorage(p).margin).toBe(1);
    expect(compileWorkspaceDocument(chapters, p).layout.margin).toBe(1);
  }
);
it.each(["title", "none"] as const)("ignores unfinished numbering with %s headings", (heading) => {
  const p = starterProfiles("Book", "")[0];
  p.startNumber = undefined as unknown as number;
  expect(profileValidation(p)).toContain("Chapter start number");
  p.chapterHeading = heading;
  expect(profileValidation(p)).toBe("");
  expect(profileForStorage(p).startNumber).toBe(1);
  p.format = "markdown";
  expect(profileValidation(p)).toBe("");
  expect(profileForStorage(p).startNumber).toBe(1);
  expect(compileWorkspaceDocument(chapters, p).text).not.toContain("NaN");
});
