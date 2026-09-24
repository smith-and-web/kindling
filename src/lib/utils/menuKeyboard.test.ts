import { afterEach, describe, expect, it, vi } from "vitest";
import { menuKeyboard } from "./menuKeyboard";

function build(html: string) {
  const trigger = document.createElement("button");
  trigger.textContent = "Open";
  const popover = document.createElement("div");
  popover.innerHTML = html;
  document.body.append(trigger, popover);
  return { trigger, popover };
}

function press(target: Element, key: string) {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

const menu = `
  <button role="menuitem">Export</button>
  <button role="menuitem" disabled>Print</button>
  <button role="menuitem" aria-disabled="true">Share</button>
  <button role="menuitem">Snapshots</button>
  <button role="menuitem">Archive</button>`;

afterEach(() => {
  document.body.innerHTML = "";
});

describe("menuKeyboard (menu)", () => {
  it("focuses the first enabled item and moves with arrows, Home and End", () => {
    const { popover } = build(menu);
    menuKeyboard(popover, { onClose: vi.fn() });
    const focused = () => document.activeElement?.textContent;
    expect(focused()).toBe("Export");
    expect(press(popover, "ArrowDown").defaultPrevented).toBe(true);
    expect(focused()).toBe("Snapshots");
    press(popover, "ArrowDown");
    press(popover, "ArrowDown");
    expect(focused()).toBe("Export");
    press(popover, "ArrowUp");
    expect(focused()).toBe("Archive");
    press(popover, "Home");
    expect(focused()).toBe("Export");
    press(popover, "End");
    expect(focused()).toBe("Archive");
    expect(press(popover, "a").defaultPrevented).toBe(false);
    expect(focused()).toBe("Archive");
  });

  it("closes on Escape and Tab and returns focus to the trigger", () => {
    for (const key of ["Escape", "Tab"]) {
      const { trigger, popover } = build(menu);
      const onClose = vi.fn();
      const outer = vi.fn();
      document.body.addEventListener("keydown", outer);
      menuKeyboard(popover, { onClose, trigger });
      expect(press(popover, key).defaultPrevented).toBe(true);
      expect(onClose).toHaveBeenCalledOnce();
      expect(document.activeElement).toBe(trigger);
      // Escape stops at the menu so a parent dialog or window handler doesn't also react.
      expect(outer).toHaveBeenCalledTimes(key === "Escape" ? 0 : 1);
      document.body.removeEventListener("keydown", outer);
      document.body.innerHTML = "";
    }
  });

  it("uses the latest options after an update and stops listening when destroyed", () => {
    const { trigger, popover } = build(menu);
    const first = vi.fn();
    const second = vi.fn();
    const action = menuKeyboard(popover, { onClose: first });
    action.update({ onClose: second, trigger });
    press(popover, "Escape");
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledOnce();
    action.destroy();
    press(popover, "Escape");
    expect(second).toHaveBeenCalledOnce();
  });

  it("tolerates a menu with no enabled items", () => {
    const { popover } = build(`<button role="menuitem" disabled>Nothing</button>`);
    menuKeyboard(popover, { onClose: vi.fn() });
    expect(press(popover, "ArrowDown").defaultPrevented).toBe(false);
  });
});

describe("menuKeyboard (dialog popover)", () => {
  it("focuses the first control, leaves arrows and Tab alone, and closes on Escape", () => {
    const { trigger, popover } = build(`
      <span>Filters</span>
      <button disabled>Reset</button>
      <button>Notes</button>
      <select><option>All</option></select>`);
    const onClose = vi.fn();
    menuKeyboard(popover, { onClose, trigger, role: "dialog" });
    expect(document.activeElement?.textContent).toBe("Notes");
    expect(press(popover, "ArrowDown").defaultPrevented).toBe(false);
    expect(press(popover, "Tab").defaultPrevented).toBe(false);
    expect(onClose).not.toHaveBeenCalled();
    press(popover, "Escape");
    expect(onClose).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(trigger);
  });
});
