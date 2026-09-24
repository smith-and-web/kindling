import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";

afterEach(cleanup);

function items(overrides: Partial<Record<string, () => void>> = {}): MenuItem[] {
  return [
    { label: "Rename", action: overrides.rename ?? vi.fn() },
    {
      label: "Planning",
      action: vi.fn(),
      children: [
        { label: "Fixed", action: overrides.fixed ?? vi.fn() },
        { label: "Flexible", action: vi.fn() },
      ],
    },
    { label: "", action: vi.fn(), divider: true },
    { label: "Archive", action: vi.fn(), disabled: true },
    { label: "Delete", action: overrides.remove ?? vi.fn(), danger: true },
  ];
}

describe("ContextMenu", () => {
  it("renders Press menu items, a separator and a danger item", () => {
    render(ContextMenu, { items: items(), x: 10, y: 10, onClose: vi.fn() });
    const menu = screen.getByTestId("context-menu");
    expect(menu.getAttribute("role")).toBe("menu");
    expect(menu.querySelector('[role="separator"]')).toBeTruthy();
    const remove = screen.getByRole("menuitem", { name: "Delete" });
    expect(remove.classList.contains("ka-menu-danger")).toBe(true);
    expect(screen.getByRole("menuitem", { name: "Planning" }).getAttribute("aria-haspopup")).toBe(
      "menu"
    );
  });

  it("focuses the first item and moves with arrows, Home and End, skipping disabled items", async () => {
    render(ContextMenu, { items: items(), x: 10, y: 10, onClose: vi.fn() });
    await tick();
    expect(document.activeElement?.getAttribute("data-label")).toBe("Rename");
    await fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Planning");
    await fireEvent.keyDown(window, { key: "End" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Delete");
    await fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Rename");
    await fireEvent.keyDown(window, { key: "ArrowUp" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Delete");
    await fireEvent.keyDown(window, { key: "Home" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Rename");
  });

  it("opens a submenu with ArrowRight, runs its item and closes", async () => {
    const fixed = vi.fn();
    const onClose = vi.fn();
    render(ContextMenu, { items: items({ fixed }), x: 10, y: 10, onClose });
    await tick();
    await fireEvent.keyDown(window, { key: "ArrowDown" });
    await fireEvent.keyDown(window, { key: "ArrowRight" });
    await tick();
    expect(screen.getByRole("menuitem", { name: "Planning" }).getAttribute("aria-expanded")).toBe(
      "true"
    );
    expect(document.activeElement?.textContent?.trim()).toBe("Fixed");
    await fireEvent.keyDown(window, { key: "ArrowLeft" });
    expect(document.activeElement?.getAttribute("data-label")).toBe("Planning");
    await fireEvent.keyDown(window, { key: "ArrowRight" });
    await tick();
    await fireEvent.click(document.activeElement as HTMLElement);
    expect(fixed).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("closes on Escape and returns focus to the trigger", async () => {
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    const onClose = vi.fn();
    const view = render(ContextMenu, { items: items(), x: 10, y: 10, onClose });
    await tick();
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
    view.unmount();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });

  it("closes on Tab instead of letting focus wander off behind the menu", async () => {
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    const onClose = vi.fn();
    const view = render(ContextMenu, { items: items(), x: 10, y: 10, onClose });
    await tick();
    const tab = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
    window.dispatchEvent(tab);
    expect(tab.defaultPrevented).toBe(true);
    expect(onClose).toHaveBeenCalled();
    view.unmount();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });

  it("runs a plain item on click and ignores disabled ones", async () => {
    const rename = vi.fn();
    const onClose = vi.fn();
    render(ContextMenu, { items: items({ rename }), x: 10, y: 10, onClose });
    await fireEvent.click(screen.getByRole("menuitem", { name: "Archive" }));
    expect(onClose).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("menuitem", { name: "Rename" }));
    expect(rename).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });
});
