/**
 * Mock @tauri-apps/api/event for browser-only dev.
 * Native menu events don't exist in the browser, so listen() is a no-op.
 */

export type UnlistenFn = () => void;

export async function listen<T>(
  _event: string,
  _handler: (event: { payload: T }) => void
): Promise<UnlistenFn> {
  return () => {};
}

export async function emit(_event: string, _payload?: unknown): Promise<void> {
  // no-op
}
