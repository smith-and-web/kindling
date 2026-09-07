import { afterEach, expect, it, vi } from "vitest";
import { captureWritingFocus } from "./writingFocus";

afterEach(() => {
  document.body.innerHTML = "";
});

function setup() {
  document.body.innerHTML =
    '<main><div data-testid="scene-panel"><textarea>abcdef</textarea><div contenteditable="true">writing</div></div></main><button>Other</button>';
  const editor = document.querySelector("textarea")!;
  editor.focus();
  editor.setSelectionRange(2, 4, "backward");
  return editor;
}

it("restores textarea focus and a backward selection after inert is removed", () => {
  const editor = setup();
  const finish = captureWritingFocus(() => true);
  editor.blur();
  editor.setSelectionRange(0, 0);
  finish(true);
  expect(document.activeElement).toBe(editor);
  expect([editor.selectionStart, editor.selectionEnd, editor.selectionDirection]).toEqual([
    2,
    4,
    "backward",
  ]);
});

it.each(["success", "navigation", "unmount", "inert", "new focus", "new focus then blur"])(
  "does not restore after %s",
  (reason) => {
    const editor = setup();
    const finish = captureWritingFocus(() => reason !== "navigation");
    editor.blur();
    if (reason === "unmount") editor.remove();
    if (reason === "inert") editor.setAttribute("inert", "");
    if (reason.startsWith("new focus")) document.querySelector("button")!.focus();
    if (reason === "new focus then blur") document.querySelector("button")!.blur();
    finish(reason !== "success");
    expect(document.activeElement).not.toBe(editor);
  }
);

it("ignores non-writing focus", () => {
  setup();
  const button = document.querySelector("button")!;
  button.focus();
  const current = vi.fn();
  captureWritingFocus(current)(true);
  expect(current).not.toHaveBeenCalled();
  expect(document.activeElement).toBe(button);
});

it("restores a contenteditable range", () => {
  setup();
  const editor = document.querySelector("[contenteditable]") as HTMLElement;
  Object.defineProperty(editor, "isContentEditable", { value: true });
  editor.focus();
  const range = document.createRange();
  range.setStart(editor.firstChild!, 1);
  range.setEnd(editor.firstChild!, 4);
  window.getSelection()!.removeAllRanges();
  window.getSelection()!.addRange(range);
  const finish = captureWritingFocus(() => true);
  editor.blur();
  window.getSelection()!.removeAllRanges();
  finish(true);
  expect(document.activeElement).toBe(editor);
  expect(window.getSelection()!.toString()).toBe("rit");
});
