/** HTML from imported files and review packages is data, not markup the app trusts. */

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
