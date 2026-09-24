/**
 * modalFocus - keyboard and focus containment for div-based modal dialogs.
 *
 * Put `use:modalFocus={{ onEscape: onClose }}` on the dialog's outermost element
 * (the scrim, which should carry `tabindex="-1"`). While mounted it:
 *
 * - moves focus inside on open (`initialFocus` selector, else the first tabbable
 *   control, else the element itself);
 * - wraps Tab / Shift+Tab at the edges and pulls focus back if anything behind the
 *   dialog takes it (e.g. the editor refocusing itself);
 * - keeps key presses from propagating to window listeners behind the dialog;
 * - closes on Escape via `onEscape`, for the topmost modalFocus dialog only;
 * - restores focus to the element that was focused before it opened.
 *
 * Dialogs stack: only the most recently opened one reacts. A key or focus event
 * inside some other open modal (a native `<dialog>` or an `aria-modal` element
 * that does not use this action) is left alone.
 *
 * Pass the dialog's own key handling as `onKeydown`, not as an `onkeydown`
 * attribute on the same element: Svelte delegates that attribute to the app root,
 * which the stopped event never reaches. Handlers on descendants work as usual.
 */
import type { ActionReturn } from "svelte/action";
import { on } from "svelte/events";

export interface ModalFocusOptions {
  /** Called on Escape while this is the topmost dialog. Omit to ignore Escape. */
  onEscape?: () => void;
  /** Selector, within the dialog, for the control to focus on open. */
  initialFocus?: string;
  /** Runs first for keys pressed inside; preventDefault() skips the Escape/Tab handling. */
  onKeydown?: (event: KeyboardEvent) => void;
}

interface Trap {
  node: HTMLElement;
  previous: Element | null;
  options: ModalFocusOptions;
}

const FOCUSABLE = [
  "a[href]",
  "area[href]",
  "button",
  "input:not([type='hidden'])",
  "select",
  "textarea",
  "summary",
  "iframe",
  "[contenteditable]:not([contenteditable='false'])",
  "[tabindex]",
].join(",");

const MODAL = "dialog[open], [aria-modal='true']";

/** Elements that act on Enter themselves, so Enter must not also trigger a dialog default. */
const OWNS_ENTER = [
  "button",
  "a[href]",
  "select",
  "textarea",
  "summary",
  "[contenteditable]:not([contenteditable='false'])",
  "[role='button']",
  "[role='link']",
  "[role='menuitem']",
  "[role='option']",
  "[role='combobox']",
  "input[type='button']",
  "input[type='submit']",
  "input[type='reset']",
  "input[type='file']",
].join(",");

const stack: Trap[] = [];
let nativeDialogs = 0;

/**
 * Count an OS-level dialog (file picker, save panel) as a modal while `show` is pending,
 * so isModalOpen() is true for it. Use the wrappers in nativeDialog.ts.
 */
export async function trackNativeDialog<T>(show: () => Promise<T>): Promise<T> {
  nativeDialogs++;
  try {
    return await show();
  } finally {
    nativeDialogs--;
  }
}

/**
 * True while any modal is open: a native file dialog, a modalFocus dialog, an open
 * `<dialog>` or an `aria-modal` element. `ignore` excludes particular modal elements
 * (e.g. the Find dialog when a Find command should reach it).
 */
export function isModalOpen(ignore: (modal: Element) => boolean = () => false): boolean {
  if (nativeDialogs > 0) return true;
  const modals = [...stack.map(({ node }) => node), ...document.querySelectorAll(MODAL)];
  return modals.some((modal) => !ignore(modal));
}

function isRadioTabStop(root: HTMLElement, radio: HTMLInputElement): boolean {
  if (!radio.name || radio.checked) return true;
  // A radio group is one tab stop: the checked radio, or the first when none is checked.
  const group = Array.from(root.querySelectorAll<HTMLInputElement>("input[type='radio']")).filter(
    (other) => other.name === radio.name && other.form === radio.form
  );
  return !group.some((other) => other.checked) && group[0] === radio;
}

/** The elements Tab would visit inside `root`, in document order. */
export function tabbableElements(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter((el) => {
    if (el.tabIndex < 0 || el.matches(":disabled")) return false;
    if (el.closest("[inert], [hidden]")) return false;
    if (typeof el.checkVisibility === "function" && !el.checkVisibility()) return false;
    if (el instanceof HTMLInputElement && el.type === "radio") return isRadioTabStop(root, el);
    return true;
  });
}

/** True when Enter on `target` already does something (activates a button, opens a select...). */
export function ownsEnterKey(target: EventTarget | null): boolean {
  return target instanceof Element && target.matches(OWNS_ENTER);
}

function isTop(trap: Trap): boolean {
  return stack[stack.length - 1] === trap;
}

function isInside(trap: Trap, target: EventTarget | null): boolean {
  return target instanceof Node && trap.node.contains(target);
}

/** Inside an open modal that is not managed here (e.g. a native `<dialog>` opened on top). */
function inForeignModal(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  const modal = target.closest(MODAL);
  return !!modal && !stack.some(({ node }) => node.contains(modal) || modal.contains(node));
}

function focusInitial(trap: Trap) {
  const { node, options } = trap;
  const preferred = options.initialFocus
    ? node.querySelector<HTMLElement>(options.initialFocus)
    : null;
  (preferred ?? tabbableElements(node)[0] ?? node).focus();
}

function focusEdge(trap: Trap, last: boolean) {
  const items = tabbableElements(trap.node);
  (last ? items[items.length - 1] : items[0])?.focus();
  if (!isInside(trap, document.activeElement)) trap.node.focus();
}

function handleTab(trap: Trap, event: KeyboardEvent) {
  const items = tabbableElements(trap.node);
  const active = document.activeElement;
  const edge = event.shiftKey ? items[0] : items[items.length - 1];
  if (items.length === 0 || active === trap.node || active === edge) {
    event.preventDefault();
    focusEdge(trap, event.shiftKey);
  }
}

function restoreFocus(trap: Trap) {
  const previous = trap.previous;
  if (!(previous instanceof HTMLElement) || !previous.isConnected) return;
  if (previous.closest("[inert]")) return;
  const active = document.activeElement;
  // Focus already moved on somewhere deliberate (another dialog, a new view): leave it.
  if (active && active !== document.body && !trap.node.contains(active)) return;
  previous.focus({ preventScroll: true });
}

export function modalFocus(
  node: HTMLElement,
  options: ModalFocusOptions = {}
): ActionReturn<ModalFocusOptions> {
  const trap: Trap = { node, previous: document.activeElement, options };
  stack.push(trap);

  // Bubble phase on the dialog itself, after the dialog's own (delegated) handlers,
  // so a control that handles Escape can preventDefault to keep the dialog open.
  const offKeydown = on(node, "keydown", (event: KeyboardEvent) => {
    event.stopPropagation();
    if (!isTop(trap)) return;
    trap.options.onKeydown?.(event);
    if (event.defaultPrevented || event.isComposing) return;
    if (event.key === "Escape" && trap.options.onEscape) {
      event.preventDefault();
      trap.options.onEscape();
    } else if (event.key === "Tab" && !event.altKey && !event.ctrlKey && !event.metaKey) {
      handleTab(trap, event);
    }
  });

  // Focus can still land outside, e.g. on <body> when the focused control is removed.
  // Keys pressed then must not reach listeners behind the dialog either.
  const onWindowKeydown = (event: KeyboardEvent) => {
    if (!isTop(trap) || isInside(trap, event.target) || inForeignModal(event.target)) return;
    event.stopImmediatePropagation();
    if (event.key === "Escape") {
      event.preventDefault();
      trap.options.onEscape?.();
    } else if (event.key === "Tab") {
      event.preventDefault();
      focusEdge(trap, event.shiftKey);
    }
  };
  window.addEventListener("keydown", onWindowKeydown, true);

  const onFocusIn = (event: FocusEvent) => {
    if (!isTop(trap) || isInside(trap, event.target) || inForeignModal(event.target)) return;
    focusInitial(trap);
  };
  document.addEventListener("focusin", onFocusIn, true);

  // Let the dialog finish rendering (and any onMount state settle) before choosing a control.
  queueMicrotask(() => {
    if (isTop(trap) && node.isConnected && !isInside(trap, document.activeElement)) {
      focusInitial(trap);
    }
  });

  return {
    update(next) {
      trap.options = next ?? {};
    },
    destroy() {
      offKeydown();
      window.removeEventListener("keydown", onWindowKeydown, true);
      document.removeEventListener("focusin", onFocusIn, true);
      const index = stack.indexOf(trap);
      const wasTop = index === stack.length - 1;
      stack.splice(index, 1);
      if (wasTop) {
        restoreFocus(trap);
        return;
      }
      // A dialog opened over this one would return focus into this one; hand it our target.
      const above = stack[index];
      if (above.previous instanceof Node && node.contains(above.previous)) {
        above.previous = trap.previous;
      }
    },
  };
}
