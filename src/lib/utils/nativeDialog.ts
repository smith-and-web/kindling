/**
 * File pickers from @tauri-apps/plugin-dialog, counted as open modals while they are
 * showing so menu accelerators cannot act behind them (see isModalOpen).
 * Import `open` / `save` from here rather than from the plugin.
 */
import {
  open as pluginOpen,
  save as pluginSave,
  type OpenDialogOptions,
  type SaveDialogOptions,
} from "@tauri-apps/plugin-dialog";
import { trackNativeDialog } from "./modalFocus";

export function open<T extends OpenDialogOptions>(options?: T) {
  return trackNativeDialog(() => pluginOpen(options));
}

export function save(options?: SaveDialogOptions) {
  return trackNativeDialog(() => pluginSave(options));
}
