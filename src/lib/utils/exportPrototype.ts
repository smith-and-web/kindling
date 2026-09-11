/** Local-only UI prototype. This is deliberately not a public exporter contract. */
export type PreviewFormat =
  | "docx"
  | "epub"
  | "html"
  | "markdown"
  | "txt"
  | "longform"
  | "scrivener"
  | "novelwriter"
  | "treatment";
export const isExchange = (format: PreviewFormat) =>
  ["longform", "scrivener", "novelwriter", "treatment"].includes(format);
export const isPlain = (format: PreviewFormat) => ["markdown", "txt"].includes(format);
export interface PreviewScene {
  id: string;
  title: string;
  synopsis: string | null;
  blocks: { heading: string | null; html: string }[];
}
export interface PreviewChapter {
  id: string;
  title: string;
  part: string | null;
  scenes: PreviewScene[];
}
export interface ExportProfile {
  id: string;
  name: string;
  format: PreviewFormat;
  selection: "all" | "selected";
  chapterIds: string[];
  sceneId: string | null;
  chapterHeading: "number_title" | "number" | "title" | "none";
  numberStyle: "arabic" | "roman";
  startNumber: number;
  sceneTitles: boolean;
  partTitles: boolean;
  synopses: boolean;
  beatHeadings: boolean;
  separator: string;
  chapterBreaks: boolean;
  font: "Times New Roman" | "Courier New" | "Georgia" | "Arial";
  fontSize: number;
  lineSpacing: number;
  paragraphSpacing: number;
  indent: number;
  alignment: "left" | "justify";
  firstParagraphFlush: boolean;
  paper: "letter" | "a4";
  margin: number;
  titlePage: boolean;
  title: string;
  author: string;
  subtitle: string;
  wordCount: "exact" | "rounded" | "none";
  header: "none" | "author_title" | "title";
  contents: boolean;
  language: string;
  htmlMode: "document" | "fragment";
  styled: boolean;
  headingLevel: "h1" | "h2";
  fileName: string;
  description: string;
  coverPath: string;
  includeNotes: boolean;
  includeBeatComments: boolean;
  treatmentLevel: "one_page" | "five_page" | "full";
  treatmentFormat: "docx" | "txt";
}

export const formatLabels: Record<PreviewFormat, string> = {
  docx: "Submission manuscript",
  epub: "Ebook / reader copy",
  html: "Web / HTML",
  markdown: "Markdown manuscript",
  txt: "Plain text manuscript",
  longform: "Longform / Obsidian",
  scrivener: "Scrivener project",
  novelwriter: "novelWriter project",
  treatment: "Story treatment",
};

export function starterProfiles(title: string, author: string): ExportProfile[] {
  const base: ExportProfile = {
    id: "submission",
    name: "Agent submission",
    format: "docx",
    selection: "all",
    chapterIds: [],
    sceneId: null,
    chapterHeading: "number_title",
    numberStyle: "arabic",
    startNumber: 1,
    sceneTitles: false,
    partTitles: true,
    synopses: false,
    beatHeadings: false,
    separator: "#",
    chapterBreaks: true,
    font: "Times New Roman",
    fontSize: 12,
    lineSpacing: 2,
    paragraphSpacing: 0,
    indent: 0.5,
    alignment: "left",
    firstParagraphFlush: false,
    paper: "letter",
    margin: 1,
    titlePage: true,
    title,
    author,
    subtitle: "",
    wordCount: "rounded",
    header: "author_title",
    contents: false,
    language: "en",
    htmlMode: "document",
    styled: true,
    headingLevel: "h1",
    fileName: "{title}-{profile}",
    description: "",
    coverPath: "",
    includeNotes: true,
    includeBeatComments: true,
    treatmentLevel: "five_page",
    treatmentFormat: "docx",
  };
  return [
    base,
    {
      ...base,
      id: "readers",
      name: "Writing group",
      format: "epub",
      font: "Georgia",
      lineSpacing: 1.5,
      indent: 0,
      paragraphSpacing: 8,
      sceneTitles: true,
      titlePage: false,
      separator: "* * *",
      contents: true,
      wordCount: "none",
      header: "none",
    },
    {
      ...base,
      id: "website",
      name: "Website chapters",
      format: "html",
      font: "Georgia",
      titlePage: false,
      chapterBreaks: false,
      lineSpacing: 1.6,
      indent: 0,
      paragraphSpacing: 12,
      chapterHeading: "title",
      separator: "⁂",
      wordCount: "none",
      header: "none",
      contents: true,
      headingLevel: "h2",
    },
  ];
}

export const profileStorageKey = (projectId: string) => `kindling:export-prototype:v1:${projectId}`;

const numericSettings = [
  ["fontSize", "Font size", 8, 24],
  ["lineSpacing", "Line spacing", 1, 3],
  ["paragraphSpacing", "Paragraph spacing", 0, 36],
  ["indent", "First-line indent", 0, 1],
  ["margin", "Margins", 0.25, 2],
  ["startNumber", "Chapter start number", 1, 999],
] as const;
type NumericSetting = (typeof numericSettings)[number][0];

function numericSettingVisible(profile: ExportProfile, key: NumericSetting): boolean {
  if (isExchange(profile.format)) return false;
  if (key === "startNumber") return profile.chapterHeading.startsWith("number");
  if (key === "margin") return profile.format === "docx";
  return !isPlain(profile.format);
}

export function profileValidation(profile: ExportProfile): string {
  if (!profile.name.trim()) return "Give this profile a name before saving.";
  for (const [key, label, min, max] of numericSettings) {
    if (!numericSettingVisible(profile, key)) continue;
    const value = profile[key];
    if (typeof value !== "number" || !Number.isFinite(value) || value < min || value > max)
      return `${label} must be between ${min} and ${max}.`;
    if (key === "startNumber" && !Number.isInteger(value))
      return "Chapter start number must be a whole number.";
  }
  return "";
}

/** Save a complete profile while allowing irrelevant, unfinished controls to be hidden. */
export function profileForStorage(profile: ExportProfile): ExportProfile {
  const error = profileValidation(profile);
  if (error) throw new Error(error);
  const saved = { ...profile, chapterIds: [...profile.chapterIds] };
  const defaults = starterProfiles(profile.title, profile.author)[0];
  for (const [key, , min, max] of numericSettings) {
    if (numericSettingVisible(profile, key)) continue;
    const value = profile[key];
    if (
      !Number.isFinite(value) ||
      value < min ||
      value > max ||
      (key === "startNumber" && !Number.isInteger(value))
    )
      saved[key] = defaults[key];
  }
  return saved;
}

/** Reject malformed persisted data instead of silently losing the user's saved profiles. */
export function decodeProfiles(raw: string): { profiles: ExportProfile[]; activeId: string } {
  const value = JSON.parse(raw);
  if (![1, 2].includes(value?.version) || !Array.isArray(value.profiles) || !value.profiles.length)
    throw new Error("Saved prototype profiles have an unsupported format.");
  const shape = starterProfiles("", "")[0];
  const enums: Partial<Record<keyof ExportProfile, unknown[]>> = {
    format: Object.keys(formatLabels),
    selection: ["all", "selected"],
    chapterHeading: ["number_title", "number", "title", "none"],
    numberStyle: ["arabic", "roman"],
    font: ["Times New Roman", "Courier New", "Georgia", "Arial"],
    alignment: ["left", "justify"],
    paper: ["letter", "a4"],
    wordCount: ["exact", "rounded", "none"],
    header: ["none", "author_title", "title"],
    htmlMode: ["document", "fragment"],
    headingLevel: ["h1", "h2"],
    treatmentLevel: ["one_page", "five_page", "full"],
    treatmentFormat: ["docx", "txt"],
  };
  for (const profile of value.profiles) {
    // Upgrade the first prototype's profiles without replacing their existing settings.
    if (profile && value.version === 1)
      for (const key of [
        "description",
        "coverPath",
        "includeNotes",
        "includeBeatComments",
        "treatmentLevel",
        "treatmentFormat",
      ] as const) {
        if (!(key in profile)) Object.assign(profile, { [key]: shape[key] });
      }
    for (const [key, defaultValue] of Object.entries(shape)) {
      const v = profile?.[key];
      if (
        key === "sceneId"
          ? v !== null && typeof v !== "string"
          : key === "chapterIds"
            ? !Array.isArray(v) || !v.every((id: unknown) => typeof id === "string")
            : typeof v !== typeof defaultValue || (typeof v === "number" && !Number.isFinite(v))
      )
        throw new Error("A saved prototype profile is invalid.");
      if (enums[key as keyof ExportProfile] && !enums[key as keyof ExportProfile]!.includes(v))
        throw new Error("A saved prototype option is unsupported.");
    }
  }
  if (new Set(value.profiles.map((p: ExportProfile) => p.id)).size !== value.profiles.length)
    throw new Error("Saved prototype profiles have duplicate IDs.");
  return { profiles: value.profiles, activeId: value.activeId };
}

export function escapeHtml(text: string): string {
  return text.replace(
    /[&<>"']/g,
    (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char]!
  );
}

/** Rebuild an inert prose subset. Never put imported attributes or executable markup in a preview. */
export function safeProse(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const allowed = new Set([
    "P",
    "BR",
    "STRONG",
    "B",
    "EM",
    "I",
    "U",
    "S",
    "DEL",
    "BLOCKQUOTE",
    "UL",
    "OL",
    "LI",
    "H1",
    "H2",
    "H3",
    "H4",
    "HR",
  ]);
  const dropped = new Set([
    "SCRIPT",
    "STYLE",
    "IFRAME",
    "OBJECT",
    "EMBED",
    "SVG",
    "MATH",
    "FORM",
    "IMG",
    "VIDEO",
    "AUDIO",
    "TEMPLATE",
  ]);
  function walk(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) return escapeHtml(node.textContent ?? "");
    if (!(node instanceof Element) || dropped.has(node.tagName.toUpperCase())) return "";
    const content = [...node.childNodes].map(walk).join("");
    if (!allowed.has(node.tagName)) return content;
    const tag = node.tagName.toLowerCase();
    return ["br", "hr"].includes(tag) ? `<${tag}>` : `<${tag}>${content}</${tag}>`;
  }
  return [...doc.body.childNodes].map(walk).join("");
}

export function selectedChapters(
  chapters: PreviewChapter[],
  profile: ExportProfile
): PreviewChapter[] {
  return chapters
    .filter((c) => profile.selection === "all" || profile.chapterIds.includes(c.id))
    .map((c) => ({
      ...c,
      scenes: c.scenes.filter((s) => !profile.sceneId || s.id === profile.sceneId),
    }))
    .filter((c) => c.scenes.length > 0);
}

export function manuscriptWords(chapters: PreviewChapter[]): number {
  const text = chapters
    .flatMap((c) =>
      c.scenes.flatMap((s) =>
        s.blocks.map((b) => {
          const doc = new DOMParser().parseFromString(safeProse(b.html), "text/html");
          doc.querySelectorAll("p,br,li,h1,h2,h3,blockquote").forEach((el) => el.append(" "));
          return doc.body.textContent ?? "";
        })
      )
    )
    .join(" ");
  return text.trim().split(/\s+/).filter(Boolean).length;
}

function bounded(value: number, min: number, max: number, fallback: number) {
  return Number.isFinite(value) ? Math.max(min, Math.min(max, value)) : fallback;
}
function roman(value: number): string {
  let n = Math.round(bounded(value, 1, 3999, 1));
  let result = "";
  for (const [amount, token] of [
    [1000, "M"],
    [900, "CM"],
    [500, "D"],
    [400, "CD"],
    [100, "C"],
    [90, "XC"],
    [50, "L"],
    [40, "XL"],
    [10, "X"],
    [9, "IX"],
    [5, "V"],
    [4, "IV"],
    [1, "I"],
  ] as const) {
    while (n >= amount) {
      result += token;
      n -= amount;
    }
  }
  return result;
}
export function chapterLabel(chapter: PreviewChapter, index: number, p: ExportProfile): string {
  const n = Math.round(bounded(p.startNumber, 1, 999, 1)) + index;
  const number = `Chapter ${p.numberStyle === "roman" ? roman(n) : n}`;
  return p.chapterHeading === "none"
    ? ""
    : p.chapterHeading === "title"
      ? chapter.title
      : p.chapterHeading === "number"
        ? number
        : `${number}: ${chapter.title}`;
}

export function previewCss(p: ExportProfile): string {
  const paged = p.format === "docx";
  const fonts = ["Times New Roman", "Courier New", "Georgia", "Arial"];
  return `body{margin:0;background:#e9e4da;color:#231d18} .manuscript{box-sizing:border-box;background:#fffdf9;margin:24px auto;max-width:${paged ? (p.paper === "a4" ? "794" : "816") : "740"}px;padding:${paged ? bounded(p.margin, 0.25, 2, 1) * 60 : 36}px;font-family:"${fonts.includes(p.font) ? p.font : "Georgia"}",serif;font-size:${bounded(p.fontSize, 8, 24, 12)}pt;line-height:${bounded(p.lineSpacing, 1, 3, 2)};text-align:${p.alignment === "justify" ? "justify" : "left"}}
  h1,h2,h3{line-height:1.35;text-align:center;text-indent:0}h1,h2{font-size:1.4em;margin:2em 0}h3{font-size:1.05em}p{margin:0 0 ${bounded(p.paragraphSpacing, 0, 36, 0)}pt;text-indent:${bounded(p.indent, 0, 1, 0.5)}in}blockquote{margin:1em 2em} .scene p.flush{ text-indent:0 } .separator{text-align:center;text-indent:0;margin:1.6em 0} .synopsis{font-style:italic;color:#655e56;text-indent:0;margin-bottom:1em} .running-head{font-size:0.8em;text-align:right;border-bottom:1px solid #ddd5c9;padding-bottom:12px;margin-bottom:24px} .title-page{text-align:center;padding:4em 0 6em}.title-page p{text-indent:0}.title-page h1{margin:1em 0}.part{margin:3em 0;text-align:center;font-size:1.6em}.contents{margin:2em 0}.contents a{color:inherit}.chapter+.chapter{margin-top:3em;${paged && p.chapterBreaks ? "border-top:1px dashed #ccc;padding-top:3em" : ""}} @media(max-width:600px){.manuscript{margin:0;padding:28px;max-width:100%}} @media print{body{background:white}.manuscript{margin:0;max-width:none;padding:0}.chapter+.chapter{border:0;${paged && p.chapterBreaks ? "break-before:page" : ""}}.title-page{break-after:page}@page{size:${p.paper === "a4" ? "A4" : "letter"};margin:${bounded(p.margin, 0.25, 2, 1)}in}}`;
}

export function renderPreview(
  chapters: PreviewChapter[],
  p: ExportProfile,
  options: { chapterId?: string; titleOnly?: boolean } = {}
): { body: string; document: string; output: string } {
  const selected = selectedChapters(chapters, p);
  const title = escapeHtml(p.title || "Untitled manuscript");
  let body = "";
  if (p.format === "docx" && p.header !== "none")
    body += `<header class="running-head">${p.header === "author_title" && p.author ? `${escapeHtml(p.author)} / ` : ""}${title}</header>`;
  if (p.titlePage && (!options.chapterId || options.titleOnly)) {
    const words = manuscriptWords(selected);
    const count =
      p.wordCount === "rounded" && words >= 1000
        ? `${(Math.round(words / 1000) * 1000).toLocaleString()} words (approx.)`
        : `${words.toLocaleString()} words`;
    body += `<section class="title-page"><h1>${title}</h1>${p.subtitle ? `<p>${escapeHtml(p.subtitle)}</p>` : ""}${p.author ? `<p>by ${escapeHtml(p.author)}</p>` : ""}${p.wordCount !== "none" ? `<p>${count}</p>` : ""}</section>`;
  }
  if (p.contents && !options.chapterId && !options.titleOnly)
    body += `<nav class="contents" aria-label="Contents"><h2>Contents</h2><ol>${selected.map((c, i) => `<li><a href="#chapter-${i}">${escapeHtml(chapterLabel(c, i, p) || c.title)}</a></li>`).join("")}</ol></nav>`;
  let lastPart: string | null = null;
  if (!options.titleOnly)
    for (const [index, chapter] of selected.entries()) {
      const newPart = chapter.part !== lastPart;
      lastPart = chapter.part;
      if (options.chapterId && chapter.id !== options.chapterId) continue;
      if (p.partTitles && chapter.part && (newPart || options.chapterId))
        body += `<div class="part">${escapeHtml(chapter.part)}</div>`;
      const tag = p.format === "html" && p.headingLevel === "h2" ? "h2" : "h1";
      const heading = chapterLabel(chapter, index, p);
      body += `<section class="chapter" id="chapter-${index}">${heading ? `<${tag}>${escapeHtml(heading)}</${tag}>` : ""}`;
      for (const [sceneIndex, scene] of chapter.scenes.entries()) {
        if (sceneIndex > 0)
          body += `<p class="separator">${escapeHtml(p.separator) || "&nbsp;"}</p>`;
        body += `<section class="scene">${p.sceneTitles ? `<h3>${escapeHtml(scene.title)}</h3>` : ""}`;
        if (p.synopses && scene.synopsis)
          body += `<p class="synopsis">${escapeHtml(new DOMParser().parseFromString(safeProse(scene.synopsis), "text/html").body.textContent ?? "")}</p>`;
        for (const block of scene.blocks)
          body += `${p.beatHeadings && block.heading ? `<h3>${escapeHtml(new DOMParser().parseFromString(safeProse(block.heading), "text/html").body.textContent ?? "")}</h3>` : ""}${safeProse(block.html)}`;
        body += "</section>";
      }
      body += "</section>";
    }
  if (p.firstParagraphFlush) {
    const markup = new DOMParser().parseFromString(body, "text/html");
    for (const scene of markup.querySelectorAll(".scene")) {
      let first = true;
      for (const element of scene.querySelectorAll("h1,h2,h3,h4,p")) {
        if (element.matches("h1,h2,h3,h4")) first = true;
        else if (!element.classList.contains("synopsis")) {
          if (first) element.classList.add("flush");
          first = false;
        }
      }
    }
    body = markup.body.innerHTML;
  }
  body = `<main class="manuscript">${body}</main>`;
  const css = p.format !== "html" || p.styled ? `<style>${previewCss(p)}</style>` : "";
  const document = `<!doctype html>\n<html lang="${escapeHtml(p.language || "en")}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'"><title>${title}</title>${css}</head><body>${body}</body></html>`;
  return {
    body,
    document,
    output: p.format === "html" && p.htmlMode === "fragment" ? `${css}${body}` : document,
  };
}

export function previewFilename(p: ExportProfile, date = new Date()): string {
  return (
    (p.fileName
      .replaceAll("{title}", p.title)
      .replaceAll("{profile}", p.name)
      .replaceAll("{date}", date.toISOString().slice(0, 10))
      .replace(/[<>:"/\\|?*\p{Cc}]/gu, "-")
      .replace(/^[.\s]+|[.\s]+$/g, "")
      .slice(0, 160) || "manuscript-preview") + ".html"
  );
}

export function outputExtension(p: ExportProfile): string {
  return p.format === "markdown"
    ? "md"
    : p.format === "treatment"
      ? p.treatmentFormat
      : p.format === "scrivener"
        ? "scriv"
        : p.format;
}
export function exportFilename(p: ExportProfile): string {
  const name = previewFilename(p).slice(0, -5);
  return ["longform", "novelwriter"].includes(p.format) ? name : `${name}.${outputExtension(p)}`;
}
export function proseText(html: string): string {
  const doc = new DOMParser().parseFromString(safeProse(html), "text/html");
  doc.querySelectorAll("p,br,li,h1,h2,h3,h4,blockquote").forEach((el) => el.append("\n"));
  return (doc.body.textContent ?? "").trim();
}
function markdownLiteral(text: string): string {
  return text.replace(/([\\`*_{}[\]<>#+.!|~-])/g, "\\$1");
}
function markdownProse(html: string): string {
  const doc = new DOMParser().parseFromString(safeProse(html), "text/html");
  function walk(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) return markdownLiteral(node.textContent ?? "");
    if (!(node instanceof Element)) return "";
    const content = [...node.childNodes].map(walk).join("");
    switch (node.tagName) {
      case "STRONG":
      case "B":
        return `**${content}**`;
      case "EM":
      case "I":
        return `*${content}*`;
      case "S":
      case "DEL":
        return `~~${content}~~`;
      case "BR":
        return "  \n";
      case "P":
        return `${content}\n\n`;
      case "H1":
      case "H2":
      case "H3":
      case "H4":
        return `${"#".repeat(Number(node.tagName[1]))} ${content}\n\n`;
      case "BLOCKQUOTE":
        return `${content
          .trim()
          .split("\n")
          .map((line) => `> ${line}`)
          .join("\n")}\n\n`;
      case "LI":
        return `${node.parentElement?.tagName === "OL" ? `${[...node.parentElement.children].indexOf(node) + 1}.` : "-"} ${content.trim()}\n`;
      case "UL":
      case "OL":
        return `${content}\n`;
      case "HR":
        return "\n---\n\n";
      default:
        return content;
    }
  }
  return [...doc.body.childNodes].map(walk).join("").trim();
}

function docxProse(html: string): string {
  const doc = new DOMParser().parseFromString(safeProse(html), "text/html");
  doc.querySelectorAll("li").forEach((li) => {
    const marker =
      li.parentElement?.tagName === "OL"
        ? `${[...li.parentElement.children].indexOf(li) + 1}. `
        : "• ";
    const paragraph = li.firstElementChild?.tagName === "P" ? li.firstElementChild : li;
    paragraph.prepend(marker);
  });
  return doc.body.innerHTML;
}
export interface WorkspaceDocument {
  title: string;
  author: string;
  subtitle: string;
  language: string;
  description: string;
  wordCount: string;
  titlePage: boolean;
  contents: boolean;
  coverPath: string;
  layout: Pick<
    ExportProfile,
    | "font"
    | "fontSize"
    | "lineSpacing"
    | "paragraphSpacing"
    | "indent"
    | "alignment"
    | "firstParagraphFlush"
    | "paper"
    | "margin"
    | "chapterBreaks"
    | "header"
  >;
  chapters: {
    title: string;
    navigationTitle: string;
    part: string | null;
    xhtml: string;
    scenes: {
      title: string;
      synopsis: string;
      separator: string | null;
      blocks: { heading: string; html: string }[];
    }[];
  }[];
  frontMatter: string | null;
  text: string;
}
function xhtmlDocument(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  return (
    '<?xml version="1.0" encoding="UTF-8"?>\n' +
    new XMLSerializer().serializeToString(doc.documentElement)
  );
}
export function compileWorkspaceDocument(
  chapters: PreviewChapter[],
  p: ExportProfile
): WorkspaceDocument {
  // Plain-text previews remain editable during validation; their native payload
  // carries neutral typography because those formats do not use page layout.
  const layout = isPlain(p.format) ? starterProfiles(p.title, p.author)[0] : profileForStorage(p);
  const selected = selectedChapters(chapters, p);
  const words = manuscriptWords(selected);
  const wordCount =
    p.wordCount === "none"
      ? ""
      : p.wordCount === "rounded" && words >= 1000
        ? `${(Math.round(words / 1000) * 1000).toLocaleString()} words (approx.)`
        : `${words.toLocaleString()} words`;
  let lastPart: string | null = null;
  const compiled = selected.map((c, i) => {
    const part = p.partTitles && c.part !== lastPart ? c.part : null;
    lastPart = c.part;
    // EPUB bodies share the same renderer as their previews; metadata/layout are not discarded.
    const chapterProfile = { ...p, partTitles: !!part, contents: false };
    return {
      title: chapterLabel(c, i, p),
      navigationTitle: chapterLabel(c, i, p) || c.title,
      part,
      xhtml: xhtmlDocument(renderPreview(chapters, chapterProfile, { chapterId: c.id }).document),
      scenes: c.scenes.map((s, n) => ({
        title: p.sceneTitles ? s.title : "",
        synopsis: p.synopses && s.synopsis ? proseText(s.synopsis) : "",
        separator: n > 0 ? p.separator : null,
        blocks: s.blocks.map((b) => ({
          heading: p.beatHeadings && b.heading ? proseText(b.heading) : "",
          html: p.format === "docx" ? docxProse(b.html) : safeProse(b.html),
        })),
      })),
    };
  });
  const md = p.format === "markdown";
  const literal = (text: string) => (md ? markdownLiteral(text) : text);
  const text: string[] = [];
  if (p.titlePage)
    text.push(
      `${md ? "# " : ""}${literal(p.title)}`,
      literal(p.subtitle),
      p.author ? `by ${literal(p.author)}` : "",
      wordCount,
      ""
    );
  if (p.contents)
    text.push(
      `${md ? "## " : ""}Contents`,
      ...compiled.map((c) => `${md ? "- " : ""}${literal(c.navigationTitle)}`),
      ""
    );
  for (const c of compiled) {
    if (c.part) text.push(`${md ? "# " : ""}${literal(c.part)}`, "");
    if (c.title) text.push(`${md ? "## " : ""}${literal(c.title)}`, "");
    for (const s of c.scenes) {
      if (s.separator !== null) text.push(literal(s.separator), "");
      if (s.title) text.push(`${md ? "### " : ""}${literal(s.title)}`, "");
      if (s.synopsis) text.push(`${md ? "> " : ""}${literal(s.synopsis)}`, "");
      for (const b of s.blocks) {
        if (b.heading) text.push(`${md ? "#### " : ""}${literal(b.heading)}`, "");
        if (b.html) text.push(md ? markdownProse(b.html) : proseText(b.html), "");
      }
    }
  }
  return {
    title: p.title || "Untitled manuscript",
    author: p.author,
    subtitle: p.subtitle,
    language: p.language,
    description: p.description,
    wordCount,
    titlePage: p.titlePage,
    contents: p.contents,
    coverPath: p.coverPath,
    layout: {
      font: layout.font,
      fontSize: layout.fontSize,
      lineSpacing: layout.lineSpacing,
      paragraphSpacing: layout.paragraphSpacing,
      indent: layout.indent,
      alignment: layout.alignment,
      firstParagraphFlush: layout.firstParagraphFlush,
      paper: layout.paper,
      margin: layout.margin,
      chapterBreaks: layout.chapterBreaks,
      header: layout.header,
    },
    chapters: compiled,
    frontMatter: p.titlePage
      ? xhtmlDocument(renderPreview(chapters, p, { titleOnly: true }).document)
      : null,
    text: text.join("\n").trim() + "\n",
  };
}
