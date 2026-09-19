import { expect, it } from "vitest";
import { countWordsInHtml } from "./wordCount";

it.each([
  [null, 0],
  [undefined, 0],
  ["", 0],
  ["<p>one</p><p>two</p>", 2],
  ["<p>&nbsp; &#160;</p>", 0],
  ["hel<strong>lo</strong>&nbsp;world", 2],
  ["one<br/>two<hr>three", 3],
  ["<p>one</p><!-- ignored --><p>two &amp; three</p>", 4],
])("counts visible words in %s", (html, count) => expect(countWordsInHtml(html)).toBe(count));
