/**
 * Keyboard behaviour for a popover anchored to a trigger button.
 *
 * As a Svelte action on the popover element it moves focus in when the popover
 * opens, and Escape closes it and returns focus to the trigger. With the default
 * `role: "menu"` it follows the WAI-ARIA menu button pattern: focus lands on the
 * first menu item, Up/Down wrap between items, Home/End jump to the ends, and Tab
 * closes the menu (focus returns to the trigger rather than escaping into the page).
 * With `role: "dialog"` (a popover of form controls, such as a filter panel) focus
 * lands on the first control and Tab moves between its controls as usual.
 *
 * ContextMenu.svelte implements the same menu keys for pointer-positioned menus.
 */
export interface MenuKeyboardOptions {
  /** Closes the popover (the owner removes it from the DOM). */
  onClose: () => void;
  /** The button that opened the popover; focus returns here on Escape or Tab. */
  trigger?: HTMLElement | null;
  role?: "menu" | "dialog";
}

const MENU_ITEM = "[role='menuitem']:not(:disabled):not([aria-disabled='true'])";
const CONTROL =
  "button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex='-1'])";

export function menuKeyboard(node: HTMLElement, options: MenuKeyboardOptions) {
  let current = options;
  const targets = () =>
    [...node.querySelectorAll<HTMLElement>(current.role === "dialog" ? CONTROL : MENU_ITEM)].filter(
      (element) => !element.closest("[inert]")
    );

  function close() {
    current.onClose();
    current.trigger?.focus({ preventScroll: true });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      // Parent dialogs and window-level Escape handlers must not also react.
      event.stopPropagation();
      close();
      return;
    }
    if (current.role === "dialog") return;
    if (event.key === "Tab") {
      event.preventDefault();
      close();
      return;
    }
    const items = targets();
    if (!items.length) return;
    const index = items.indexOf(document.activeElement as HTMLElement);
    let next: number;
    if (event.key === "ArrowDown") next = (index + 1) % items.length;
    else if (event.key === "ArrowUp") next = (index - 1 + items.length) % items.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = items.length - 1;
    else return;
    event.preventDefault();
    items[next].focus();
  }

  node.addEventListener("keydown", handleKeydown);
  targets()[0]?.focus({ preventScroll: true });

  return {
    update(next: MenuKeyboardOptions) {
      current = next;
    },
    destroy() {
      node.removeEventListener("keydown", handleKeydown);
    },
  };
}
