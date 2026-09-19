import { invoke } from "@tauri-apps/api/core";
import {
  defaultBindings,
  formatShortcut,
  shortcutFromEvent,
  type Bindings,
} from "../utils/keyboardShortcuts";

export class Shortcuts {
  bindings = $state<Bindings>({ ...defaultBindings });
  ready = $state(false);
  error = $state<string | null>(null);
  suspended = $state(false);
  recording = $state(false);
  private suspension = Promise.resolve();
  private loading: Promise<void> | null = null;

  load(): Promise<void> {
    if (this.ready) return Promise.resolve();
    if (this.loading) return this.loading;
    this.error = null;
    this.loading = invoke<Bindings>("get_keyboard_shortcuts")
      .then((bindings) => {
        this.bindings = { ...defaultBindings, ...bindings };
        this.ready = true;
      })
      .catch((error) => {
        this.error = `Could not load keyboard shortcuts: ${String(error)}`;
      })
      .finally(() => {
        this.loading = null;
      });
    return this.loading;
  }

  suspend(suspended: boolean): Promise<void> {
    this.suspension = this.suspension.then(async () => {
      try {
        await invoke("suspend_keyboard_shortcuts", { suspended });
        this.suspended = suspended;
        if (this.ready) this.error = null;
      } catch (error) {
        this.error = `Could not update native keyboard shortcuts: ${String(error)}`;
      }
    });
    return this.suspension;
  }

  async save(bindings: Bindings) {
    const saved = await invoke<Bindings>("set_keyboard_shortcuts", { bindings });
    this.bindings = { ...defaultBindings, ...saved };
    this.ready = true;
    this.error = null;
  }

  label(id: string) {
    return formatShortcut(this.bindings[id] ?? "");
  }
  match(event: KeyboardEvent) {
    const binding = shortcutFromEvent(event);
    return binding
      ? Object.keys(this.bindings).find((id) => this.bindings[id] === binding)
      : undefined;
  }
}
export const shortcuts = new Shortcuts();
