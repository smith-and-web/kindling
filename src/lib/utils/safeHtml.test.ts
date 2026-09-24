import { describe, expect, it } from "vitest";
import { escapeHtml, safeProse } from "./safeHtml";

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
