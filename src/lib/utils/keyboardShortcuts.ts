import definitions from "../shortcutDefinitions.json";

export type Bindings = Record<string, string>;
export const defaultBindings: Bindings = Object.fromEntries(
  definitions.map((def) => [def.id, def.binding])
);
export const isMac = () => /Mac|iPhone|iPad/.test(navigator.platform);
const namedKeys: Record<string, string> = {
  ",": "Comma",
  ".": "Period",
  "/": "Slash",
  "\\": "Backslash",
  ";": "Semicolon",
  "'": "Quote",
  "[": "BracketLeft",
  "]": "BracketRight",
  "-": "Minus",
  "=": "Equal",
  "`": "Backquote",
};
// Native editing and window management stay with the OS. Never steal text input.
export const reservedBindings = [
  "Mod+X",
  "Mod+C",
  "Mod+V",
  "Mod+Shift+V",
  "Mod+Alt+Shift+V",
  "Mod+A",
  "Mod+Z",
  "Mod+Shift+Z",
  "Mod+Y",
  "Mod+M",
  "Mod+H",
  "Mod+Alt+H",
];

export function shortcutFromEvent(event: KeyboardEvent, mac = isMac()): string | null {
  if (event.isComposing || event.getModifierState?.("AltGraph")) return null;
  if (!(mac ? event.metaKey : event.ctrlKey) || (mac ? event.ctrlKey : event.metaKey)) return null;
  // Prefer the logical key, regardless of its physical location (e.g. AZERTY M).
  let key = namedKeys[event.key] ?? event.key.toUpperCase();
  const recognized =
    /^[A-Z0-9]$/.test(key) ||
    Object.values(namedKeys).includes(key) ||
    /^F([1-9]|1[0-9]|2[0-4])$/.test(key);
  // Option/Shift can produce a symbol instead of a base key. Fall back only
  // for those symbols, never overwrite an already recognizable logical key.
  if (!recognized && (event.altKey || event.shiftKey)) {
    if (/^Key[A-Z]$/.test(event.code)) key = event.code.slice(3);
    else if (/^Digit[0-9]$/.test(event.code)) key = event.code.slice(5);
    else if (Object.values(namedKeys).includes(event.code)) key = event.code;
  }
  if (
    !/^[A-Z0-9]$/.test(key) &&
    !Object.values(namedKeys).includes(key) &&
    !/^F([1-9]|1[0-9]|2[0-4])$/.test(key)
  )
    return null;
  return `Mod+${event.altKey ? "Alt+" : ""}${event.shiftKey ? "Shift+" : ""}${key}`;
}

export function formatShortcut(binding: string, mac = isMac()): string {
  if (!binding) return "";
  const names: Record<string, string> = Object.fromEntries(
    Object.entries(namedKeys).map(([key, name]) => [name, key])
  );
  const parts = binding.split("+");
  if (mac)
    parts.sort((a, b) =>
      ["Alt", "Shift", "Mod"].indexOf(a) < 0
        ? 1
        : ["Alt", "Shift", "Mod"].indexOf(b) < 0
          ? -1
          : ["Alt", "Shift", "Mod"].indexOf(a) - ["Alt", "Shift", "Mod"].indexOf(b)
    );
  return parts
    .map((part) =>
      part === "Mod"
        ? mac
          ? "⌘"
          : "Ctrl"
        : part === "Alt"
          ? mac
            ? "⌥"
            : "Alt"
          : part === "Shift"
            ? mac
              ? "⇧"
              : "Shift"
            : (names[part] ?? part)
    )
    .join(mac ? "" : "+");
}

export function bindingProblem(id: string, binding: string, bindings: Bindings): string | null {
  if (!binding) return null;
  if (reservedBindings.includes(binding))
    return "This shortcut is reserved for standard text editing or window controls.";
  const conflict = definitions.find((def) => def.id !== id && bindings[def.id] === binding);
  return conflict
    ? `Already used by ${conflict.label}. Clear or change that shortcut first.`
    : null;
}
