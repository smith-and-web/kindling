/** HTML from imported files and review packages is data, not markup the app trusts. */

export function escapeHtml(text: string): string {
  return text.replace(
    /[&<>"']/g,
    (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char]!
  );
}

/** Parse untrusted HTML without activating it. Markup parsed into an element of the live
 * document, even a detached one, is live: an `<img onerror>` fetches and runs its handler.
 * The template contents document has no browsing context, so nothing loads or runs, and
 * `innerHTML` reads back exactly as it would from a div. Never mount the result. */
export function inertElement(html: string): HTMLDivElement {
  const root = document.createElement("template").content.ownerDocument.createElement("div");
  root.innerHTML = html;
  return root;
}

/** Links in imported or reviewed prose are data. One without a target (or targeting
 * `_top`) would navigate the app's own window away, so a click on one is never followed. */
export function holdLinkClick(event: MouseEvent): void {
  if (event.target instanceof Element && event.target.closest("a[href]")) event.preventDefault();
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
