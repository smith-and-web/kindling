/** Match saved prose counting: block boundaries separate words; inline marks do not. */
export function countWordsInHtml(html: string | null | undefined): number {
  if (!html) return 0;
  const document = new DOMParser().parseFromString(html, "text/html");
  for (const node of document.querySelectorAll("p, div, br, hr, li, h1, h2, h3, blockquote")) {
    node.before(" ");
    node.after(" ");
  }
  const text = document.body.textContent?.trim() ?? "";
  return text ? text.split(/\s+/u).length : 0;
}
