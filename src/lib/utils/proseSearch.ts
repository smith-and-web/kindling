/** Search visible text, never HTML tags/attributes; retain the original DOM when replacing. */
export interface SearchOptions {
  caseSensitive: boolean;
  wholeWord: boolean;
}
export interface TextMatch {
  from: number;
  to: number;
}
export interface ProseDocument {
  id: string;
  scene_id: string;
  chapter_id: string;
  chapter_title: string;
  scene_title: string;
  beat_title: string | null;
  prose: string;
  locked: boolean;
}
export interface ProseReplacement {
  id: string;
  expected_prose: string;
  prose: string;
}

function parseProse(html: string) {
  const root = document.createElement("div");
  root.innerHTML = html;
  let text = "";
  const nodes: { node: Text; start: number }[] = [];
  function walk(node: Node) {
    if (node.nodeType === Node.TEXT_NODE) {
      nodes.push({ node: node as Text, start: text.length });
      text += node.textContent;
    } else if (node instanceof Element) {
      if (["SCRIPT", "STYLE"].includes(node.tagName)) return;
      const boundary = /^(P|DIV|BLOCKQUOTE|H[1-6]|LI|BR|HR|PRE)$/.test(node.tagName);
      if (boundary && text && !text.endsWith("\n")) text += "\n";
      node.childNodes.forEach(walk);
      if (boundary && text && !text.endsWith("\n")) text += "\n";
    }
  }
  root.childNodes.forEach(walk);
  return { root, text, nodes };
}

export function searchProse(html: string, query: string, options: SearchOptions) {
  return searchText(proseText(html), query, options);
}

export function proseText(html: string): string {
  return parseProse(html).text;
}

export function searchText(text: string, query: string, options: SearchOptions) {
  const matches: TextMatch[] = [];
  if (!query) return { text, matches };
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(escaped, options.caseSensitive ? "gu" : "giu");
  const word = /[\p{L}\p{N}\p{M}_]/u;
  for (const match of text.matchAll(pattern)) {
    const from = match.index;
    const to = from + match[0].length;
    // Paragraph and line boundaries are never consumed by replacement.
    if (match[0].includes("\n")) continue;
    const before = Array.from(text.slice(Math.max(0, from - 2), from)).pop() ?? "";
    const after = Array.from(text.slice(to, to + 2))[0] ?? "";
    if (options.wholeWord && (word.test(before) || word.test(after))) continue;
    matches.push({ from, to });
  }
  return { text, matches };
}

export function replaceProse(html: string, matches: TextMatch[], replacement: string): string {
  if (!matches.length) return html;
  const { root, nodes } = parseProse(html);
  for (const match of [...matches].sort((a, b) => b.from - a.from)) {
    let inserted = false;
    for (const { node, start } of nodes) {
      const end = start + node.length;
      if (end <= match.from || start >= match.to) continue;
      const from = Math.max(0, match.from - start);
      const to = Math.min(node.length, match.to - start);
      node.replaceData(from, to - from, inserted ? "" : replacement);
      inserted = true;
    }
  }
  return root.innerHTML;
}
