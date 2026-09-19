import { describe, expect, it } from "vitest";
import { searchProse, replaceProse } from "./proseSearch";
const options = { caseSensitive: false, wholeWord: false };

describe("prose search", () => {
  it("finds literal text across inline formatting and decodes entities, excluding attributes", () => {
    const html = '<p title="Alice">Al<strong>ice</strong> &amp; Alice</p>';
    expect(searchProse(html, "alice", options).matches).toEqual([
      { from: 0, to: 5 },
      { from: 8, to: 13 },
    ]);
    expect(searchProse(html, "&", options).matches).toEqual([{ from: 6, to: 7 }]);
    expect(
      searchProse(
        '<p class="hidden">Text</p><script>hidden</script><style>hidden</style>',
        "hidden",
        options
      ).matches
    ).toEqual([]);
    expect(searchProse("<p>.* $1 [a]</p>", ".*", options).matches).toHaveLength(1);
  });
  it("supports case sensitivity, Unicode word boundaries, empty queries, and nonoverlapping matches", () => {
    expect(
      searchProse("<p>Alice alice</p>", "Alice", { ...options, caseSensitive: true }).matches
    ).toHaveLength(1);
    expect(
      searchProse("<p>cat cats écat caté _cat cat2 cat cat́</p>", "cat", {
        ...options,
        wholeWord: true,
      }).matches
    ).toHaveLength(2);
    expect(searchProse("<p>banana</p>", "ana", options).matches).toHaveLength(1);
    expect(searchProse("", "", options).matches).toEqual([]);
    expect(searchProse("<p>İ cat</p>", "cat", options).matches[0]).toEqual({ from: 2, to: 5 });
  });
  it("does not match across paragraph, hard break or scene separator boundaries", () => {
    const html = "<p>one</p><p>two<br>three</p><hr><p>four</p>";
    expect(searchProse(html, "onetwo", options).matches).toEqual([]);
    expect(searchProse(html, "two\nthree", options).matches).toEqual([]);
    expect(searchProse(html, "threefour", options).matches).toEqual([]);
  });
  it("replaces all matches backwards, retaining formatting and surrounding text", () => {
    const html =
      '<p style="text-align: center">Al<strong>ice</strong> met <em>Alice</em>.</p><hr><p>Alice!</p>';
    const { matches } = searchProse(html, "alice", options);
    const result = replaceProse(html, matches, "Bob");
    expect(result).toBe(
      '<p style="text-align: center">Bob<strong></strong> met <em>Bob</em>.</p><hr><p>Bob!</p>'
    );
    expect(
      replaceProse(
        "<p>aa aa aa</p>",
        searchProse("<p>aa aa aa</p>", "aa", options).matches,
        "longer"
      )
    ).toBe("<p>longer longer longer</p>");
  });
  it("replaces one match, allows deletion, and treats replacement as literal text", () => {
    const html = "<p>cat cat</p>";
    const { matches } = searchProse(html, "cat", options);
    expect(replaceProse(html, [matches[1]], "")).toBe("<p>cat </p>");
    expect(replaceProse(html, [matches[0]], "<$&> & $1")).toBe(
      "<p>&lt;$&amp;&gt; &amp; $1 cat</p>"
    );
    expect(replaceProse(html, [], "dog")).toBe(html);
  });
});
