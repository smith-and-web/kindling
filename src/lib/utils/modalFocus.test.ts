import { afterEach, describe, expect, it, vi } from "vitest";
import { modalFocus, ownsEnterKey, tabbableElements } from "./modalFocus";

const flush = () => new Promise<void>((resolve) => queueMicrotask(resolve));

function key(target: EventTarget, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

/** A scrim-style dialog: `<div tabindex=-1 aria-modal>` holding the given markup. */
function dialog(html: string) {
  const node = document.createElement("div");
  node.setAttribute("role", "dialog");
  node.setAttribute("aria-modal", "true");
  node.tabIndex = -1;
  node.innerHTML = html;
  document.body.append(node);
  return node;
}

function outsideButton(label = "Behind") {
  const button = document.createElement("button");
  button.textContent = label;
  document.body.append(button);
  return button;
}

const cleanups: Array<() => void> = [];
function open(node: HTMLElement, options?: Parameters<typeof modalFocus>[1]) {
  const action = modalFocus(node, options);
  cleanups.push(() => action.destroy?.());
  return action;
}

afterEach(() => {
  while (cleanups.length) cleanups.pop()?.();
  document.body.innerHTML = "";
});

describe("tabbableElements", () => {
  it("skips disabled, hidden, inert and negative-tabindex controls", () => {
    const node = dialog(`
      <button id="a">A</button>
      <button disabled>disabled</button>
      <fieldset disabled><input id="in-disabled-fieldset" /></fieldset>
      <div hidden><button>hidden</button></div>
      <div inert><button>inert</button></div>
      <button tabindex="-1">skipped</button>
      <input type="hidden" />
      <a href="#x" id="link">link</a>
      <a id="no-href">no href</a>
      <div tabindex="0" id="custom">custom</div>
    `);
    expect(tabbableElements(node).map((el) => el.id)).toEqual(["a", "link", "custom"]);
  });

  it("treats a radio group as a single stop at the checked radio", () => {
    const node = dialog(`
      <input type="radio" name="g" id="g1" />
      <input type="radio" name="g" id="g2" checked />
      <input type="radio" name="h" id="h1" />
      <input type="radio" name="h" id="h2" />
      <input type="radio" id="lone" />
    `);
    expect(tabbableElements(node).map((el) => el.id)).toEqual(["g2", "h1", "lone"]);
  });

  it("respects checkVisibility where the engine supports it", () => {
    const node = dialog(`<button id="shown">shown</button><button id="gone">gone</button>`);
    const gone = node.querySelector<HTMLElement>("#gone")!;
    Object.assign(gone, { checkVisibility: () => false });
    expect(tabbableElements(node).map((el) => el.id)).toEqual(["shown"]);
  });
});

describe("ownsEnterKey", () => {
  it("is true for controls that act on Enter themselves", () => {
    const node = dialog(`
      <button id="button">b</button><a href="#" id="link">l</a><select id="select"></select>
      <textarea id="textarea"></textarea><input type="submit" id="submit" />
      <div role="button" tabindex="0" id="role"></div>
      <input type="text" id="text" /><input type="radio" id="radio" />
    `);
    const owns = (id: string) => ownsEnterKey(node.querySelector(`#${id}`));
    expect(["button", "link", "select", "textarea", "submit", "role"].every(owns)).toBe(true);
    expect(owns("text")).toBe(false);
    expect(owns("radio")).toBe(false);
    expect(ownsEnterKey(null)).toBe(false);
    expect(ownsEnterKey(window)).toBe(false);
  });
});

describe("modalFocus", () => {
  it("focuses the initialFocus target, else the first tabbable, else the dialog", async () => {
    const withTarget = dialog(`<button id="first">1</button><input id="wanted" />`);
    open(withTarget, { initialFocus: "#wanted" });
    await flush();
    expect(document.activeElement?.id).toBe("wanted");
    cleanups.pop()?.();

    const plain = dialog(`<button id="first">1</button><button>2</button>`);
    open(plain, { initialFocus: "#missing" });
    await flush();
    expect(document.activeElement?.id).toBe("first");
    cleanups.pop()?.();

    const empty = dialog(`<p>Nothing to focus</p>`);
    open(empty);
    await flush();
    expect(document.activeElement).toBe(empty);
  });

  it("leaves focus alone if the dialog already focused something inside", async () => {
    const node = dialog(`<button id="first">1</button><input id="auto" />`);
    open(node);
    node.querySelector<HTMLElement>("#auto")!.focus();
    await flush();
    expect(document.activeElement?.id).toBe("auto");
  });

  it("wraps Tab and Shift+Tab at the edges and leaves the middle to the browser", async () => {
    const node = dialog(
      `<button id="a">a</button><button id="b">b</button><button id="c">c</button>`
    );
    open(node);
    await flush();
    const byId = (id: string) => node.querySelector<HTMLElement>(`#${id}`)!;

    byId("c").focus();
    expect(key(byId("c"), { key: "Tab" }).defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(byId("a"));

    expect(key(byId("a"), { key: "Tab", shiftKey: true }).defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(byId("c"));

    byId("b").focus();
    expect(key(byId("b"), { key: "Tab" }).defaultPrevented).toBe(false);
    expect(key(byId("b"), { key: "Tab", metaKey: true }).defaultPrevented).toBe(false);

    node.focus();
    key(node, { key: "Tab" });
    expect(document.activeElement).toBe(byId("a"));
  });

  it("keeps Tab on the dialog itself when it has nothing tabbable", async () => {
    const node = dialog(`<p>Read only</p>`);
    open(node);
    await flush();
    expect(key(node, { key: "Tab" }).defaultPrevented).toBe(true);
    expect(document.activeElement).toBe(node);
  });

  it("closes on Escape without the key reaching window listeners behind", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    const onEscape = vi.fn();
    const node = dialog(`<button>a</button>`);
    open(node, { onEscape });
    await flush();

    key(document.activeElement!, { key: "Escape" });
    key(document.activeElement!, { key: "x" });
    expect(onEscape).toHaveBeenCalledTimes(1);
    expect(behind).not.toHaveBeenCalled();
    window.removeEventListener("keydown", behind);
  });

  it("ignores Escape a control inside already handled, and Escape while composing", async () => {
    const onEscape = vi.fn();
    const node = dialog(`<input id="field" />`);
    open(node, { onEscape });
    await flush();
    const field = node.querySelector("#field")!;
    field.addEventListener("keydown", (event) => event.preventDefault());
    key(field, { key: "Escape" });
    key(node, { key: "Escape", isComposing: true });
    expect(onEscape).not.toHaveBeenCalled();
  });

  it("does nothing on Escape when no onEscape was given, and picks up updated options", async () => {
    const node = dialog(`<button>a</button>`);
    const action = open(node);
    await flush();
    expect(key(node, { key: "Escape" }).defaultPrevented).toBe(false);
    const onEscape = vi.fn();
    action.update?.({ onEscape });
    key(node, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(1);
    action.update?.(undefined as never);
    key(node, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(1);
  });

  it("runs the dialog's own onKeydown first and lets it veto Escape", async () => {
    const onEscape = vi.fn();
    const onKeydown = vi.fn((event: KeyboardEvent) => {
      if (event.key === "Escape") event.preventDefault();
    });
    const node = dialog(`<button>a</button>`);
    open(node, { onEscape, onKeydown });
    await flush();
    key(document.activeElement!, { key: "Enter" });
    key(document.activeElement!, { key: "Escape" });
    expect(onKeydown.mock.calls.map(([event]) => event.key)).toEqual(["Enter", "Escape"]);
    expect(onEscape).not.toHaveBeenCalled();
  });

  it("pulls focus back when something behind the dialog takes it", async () => {
    const behind = outsideButton();
    const node = dialog(`<button id="inside">in</button>`);
    open(node);
    await flush();
    behind.focus();
    expect(document.activeElement?.id).toBe("inside");
  });

  it("handles keys that land outside it, e.g. on body after the focused control was removed", async () => {
    const onEscape = vi.fn();
    const later = vi.fn();
    const node = dialog(`<button id="a">a</button><button id="b">b</button>`);
    open(node, { onEscape });
    window.addEventListener("keydown", later);
    await flush();
    (document.activeElement as HTMLElement).blur();

    key(document.body, { key: "Tab", shiftKey: true });
    expect(document.activeElement?.id).toBe("b");
    (document.activeElement as HTMLElement).blur();
    key(document.body, { key: "Tab" });
    expect(document.activeElement?.id).toBe("a");

    const typed = key(document.body, { key: "q" });
    expect(typed.defaultPrevented).toBe(false);
    key(window, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(1);
    expect(later).not.toHaveBeenCalled();
    window.removeEventListener("keydown", later);
  });

  it("falls back to the dialog itself when Tab lands outside and nothing is tabbable", async () => {
    const node = dialog(`<p>Read only</p>`);
    open(node);
    await flush();
    node.blur();
    key(document.body, { key: "Tab" });
    expect(document.activeElement).toBe(node);
  });

  it("stays out of the way of another open modal that does not use it", async () => {
    const onEscape = vi.fn();
    const node = dialog(`<button id="inside">in</button>`);
    open(node, { onEscape });
    await flush();

    const other = document.createElement("dialog");
    other.setAttribute("open", "");
    other.innerHTML = `<button id="other">other</button>`;
    document.body.append(other);
    const otherButton = other.querySelector<HTMLElement>("#other")!;
    otherButton.focus();
    expect(document.activeElement).toBe(otherButton);
    key(otherButton, { key: "Escape" });
    expect(onEscape).not.toHaveBeenCalled();
  });

  it("only lets the topmost dialog react, and hands focus back down the stack", async () => {
    const trigger = outsideButton("Open");
    trigger.focus();
    const lowerEscape = vi.fn();
    const upperEscape = vi.fn();
    const lower = dialog(`<button id="lower">lower</button>`);
    const lowerAction = open(lower, { onEscape: lowerEscape });
    await flush();
    expect(document.activeElement?.id).toBe("lower");

    const upper = dialog(`<button id="upper">upper</button>`);
    const upperAction = open(upper, { onEscape: upperEscape });
    await flush();
    expect(document.activeElement?.id).toBe("upper");

    // Focus inside the lower dialog is not "outside" the upper one's business...
    lower.querySelector<HTMLElement>("#lower")!.focus();
    expect(document.activeElement?.id).toBe("upper");
    key(window, { key: "Escape" });
    expect(upperEscape).toHaveBeenCalledTimes(1);
    expect(lowerEscape).not.toHaveBeenCalled();

    // Closing the upper dialog returns focus into the lower one.
    cleanups.splice(cleanups.length - 1, 1);
    upperAction.destroy?.();
    upper.remove();
    expect(document.activeElement?.id).toBe("lower");

    cleanups.splice(cleanups.length - 1, 1);
    lowerAction.destroy?.();
    lower.remove();
    expect(document.activeElement).toBe(trigger);
  });

  it("passes its return target to a dialog opened over it when it closes first", async () => {
    const trigger = outsideButton("Open");
    trigger.focus();
    const first = dialog(`<button id="first">first</button>`);
    const firstAction = modalFocus(first);
    await flush();
    const second = dialog(`<button id="second">second</button>`);
    const secondAction = modalFocus(second);
    await flush();

    // e.g. About → Send feedback: About closes after Feedback has opened.
    firstAction.destroy?.();
    first.remove();
    expect(document.activeElement?.id).toBe("second");
    secondAction.destroy?.();
    second.remove();
    expect(document.activeElement).toBe(trigger);
  });

  it("keeps the upper dialog's own return target when it was opened from outside the lower one", async () => {
    const first = dialog(`<button>first</button>`);
    const firstAction = modalFocus(first);
    await flush();
    (document.activeElement as HTMLElement).blur();
    const second = dialog(`<button id="second">second</button>`);
    const secondAction = modalFocus(second);
    await flush();
    firstAction.destroy?.();
    first.remove();
    expect(document.activeElement?.id).toBe("second");
    secondAction.destroy?.();
    second.remove();
    expect(document.activeElement).toBe(document.body);
  });

  it("does not pull focus back once it has moved on deliberately", async () => {
    const trigger = outsideButton("Open");
    trigger.focus();
    const node = dialog(`<button>in</button>`);
    const action = modalFocus(node);
    await flush();
    const other = document.createElement("dialog");
    other.setAttribute("open", "");
    other.innerHTML = `<button id="next">next</button>`;
    document.body.append(other);
    other.querySelector<HTMLElement>("#next")!.focus();
    action.destroy?.();
    node.remove();
    expect(document.activeElement?.id).toBe("next");
  });

  it("does not restore focus to an element that has been removed", async () => {
    const gone = outsideButton("Gone");
    gone.focus();
    const node = dialog(`<button>in</button>`);
    const action = modalFocus(node);
    await flush();
    gone.remove();
    action.destroy?.();
    node.remove();
    expect(document.activeElement).toBe(document.body);
  });

  it("does not restore focus into an inert region", async () => {
    const trigger = outsideButton("Open");
    trigger.focus();
    const node = dialog(`<button>in</button>`);
    const action = modalFocus(node);
    await flush();
    trigger.setAttribute("inert", "");
    action.destroy?.();
    node.remove();
    expect(document.activeElement).not.toBe(trigger);
  });

  it("skips initial focus if it was closed or covered before the microtask ran", async () => {
    const node = dialog(`<button id="in">in</button>`);
    const action = modalFocus(node);
    action.destroy?.();
    node.remove();
    await flush();
    expect(document.activeElement).toBe(document.body);
  });
});
