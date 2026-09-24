import { describe, expect, it } from "vitest";
import { escapeHtml, holdLinkClick, inertElement, safeProse } from "./safeHtml";
import { manuscript } from "./editorial";
import { proseText, replaceProse } from "./proseSearch";
import { parseReviewHtml } from "./revisions";

// The live document constructs a defined custom element even in a detached subtree, just
// as it fetches an <img> and runs its onerror. An inert parse must activate neither.
const activated: string[] = [];
customElements.define(
  "x-probe",
  class extends HTMLElement {
    constructor() {
      super();
      activated.push(this.textContent ?? "");
    }
  }
);
const hostile = '<p>Hello <x-probe>there</x-probe><img src="x" onerror="alert(1)"></p>';

describe("safeProse", () => {
  it("retains formatting but removes executable elements, attributes, and remote resources", () => {
    const safe = safeProse(
      '<p onclick="alert(1)">Hello <em>world</em><img src="https://example.com/tracker"><script>secret()</script><a href="javascript:alert(1)"> link</a><svg><text>bad</text></svg></p>'
    );
    expect(safe).toBe("<p>Hello <em>world</em> link</p>");
  });

  it("removes navigation, forms and overlays that a content security policy does not block", () => {
    const safe = safeProse(
      '<p><strong>Bio:</strong> Kind<meta http-equiv="refresh" content="0;url=https://example.com"><form action="https://example.com"><input name="password"><button>Sign in</button></form><a href="https://example.com" style="position:fixed;inset:0">Continue</a><br>and <em>brave</em></p>'
    );
    // A <form> closes the open paragraph while parsing; only the inert text and formatting remain.
    expect(safe).toBe("<p><strong>Bio:</strong> Kind</p>Continue<br>and <em>brave</em><p></p>");
  });

  it("escapes text so it cannot become markup", () => {
    expect(safeProse("<p>&lt;img src=x onerror=alert(1)&gt;</p>")).toBe(
      "<p>&lt;img src=x onerror=alert(1)&gt;</p>"
    );
    expect(escapeHtml(`<a href="x">'&'</a>`)).toBe(
      "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
    );
  });
});

describe("inert parsing of untrusted prose", () => {
  it.each([
    ["inertElement", () => inertElement(hostile).innerHTML],
    [
      "an editorial manuscript, before its package is validated",
      () =>
        manuscript([
          {
            id: "a",
            scene_id: "s",
            chapter_id: "c",
            chapter: "One",
            scene: "Letter",
            mode: "page" as const,
            html: hostile,
            locked: false,
          },
        ]).textContent,
    ],
    ["Find and Replace search", () => proseText(hostile)],
    ["Find and Replace replacement", () => replaceProse(hostile, [{ from: 0, to: 5 }], "Hi")],
    ["a scene review", () => parseReviewHtml(hostile).textContent],
  ])("does not activate markup in %s", (_label, parse) => {
    activated.length = 0;
    parse();
    expect(activated).toEqual([]);
  });

  it("keeps the markup and text exactly as a div would", () => {
    expect(inertElement(hostile).innerHTML).toBe(hostile);
    expect(inertElement(hostile).ownerDocument).not.toBe(document);
    expect(proseText(hostile)).toBe("Hello there\n");
    expect(replaceProse(hostile, [{ from: 0, to: 5 }], "Hi")).toBe(hostile.replace("Hello", "Hi"));
  });
});

describe("holdLinkClick", () => {
  it.each([
    ["a link", '<a href="https://example.com"><em>go</em></a>', true],
    ["a link without an address", "<a><em>go</em></a>", false],
    ["plain prose", "<p><em>go</em></p>", false],
  ])("follows the browser default only when the click is not on %s", (_label, html, held) => {
    const root = document.createElement("div");
    root.innerHTML = html;
    document.body.append(root);
    const event = new MouseEvent("click", { bubbles: true, cancelable: true });
    root.addEventListener("click", holdLinkClick);
    root.querySelector("em")!.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(held);
    root.remove();
  });
});
