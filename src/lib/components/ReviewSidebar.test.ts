import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/svelte";
import ReviewSidebar from "./ReviewSidebar.svelte";
import { sliceHtml, editorialSchema } from "../utils/editorial";
import { Fragment, Slice } from "@tiptap/pm/model";

beforeEach(() => {
  HTMLElement.prototype.scrollIntoView = vi.fn();
});
afterEach(cleanup);

/** A reviewer's link mark, as a package can carry it: no target, or the app's own window. */
function linked(target: string | null) {
  const mark = editorialSchema.marks.link.create({ href: "https://example.com/elsewhere", target });
  return sliceHtml(new Slice(Fragment.from(editorialSchema.text("a link", [mark])), 0, 0).toJSON());
}

function click(element: Element) {
  const event = new MouseEvent("click", { bubbles: true, cancelable: true });
  element.dispatchEvent(event);
  return event;
}

describe("ReviewSidebar", () => {
  it.each([null, "_top", "_self"])(
    "never follows a suggested passage's link (target %s) out of the app",
    (target) => {
      const view = render(ReviewSidebar, {
        items: [
          {
            id: "s1",
            kind: "suggestion",
            state: "open",
            author: "Rowan",
            excerpt: "a link",
            messages: [],
            before: linked(target),
            after: linked(target),
          },
        ],
        selected: "s1",
        name: "Writer",
        onName: vi.fn(),
        onSelect: vi.fn(),
        onStep: vi.fn(),
        onReply: vi.fn().mockResolvedValue(true),
        onDecide: vi.fn(),
        onWithdraw: vi.fn(),
        onResolve: vi.fn(),
        onComment: vi.fn(),
        onCancelComment: vi.fn(),
      });
      const links = view.container.querySelectorAll(".original-prose a, .suggested-prose a");
      expect(links).toHaveLength(2);
      for (const link of links) expect(click(link).defaultPrevented).toBe(true);

      // Only the sidebar's own links are held; the rest of the app is unaffected.
      const outside = document.createElement("a");
      outside.href = "https://example.com";
      document.body.append(outside);
      expect(click(outside).defaultPrevented).toBe(false);
      outside.remove();
    }
  );
});
