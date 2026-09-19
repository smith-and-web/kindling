# 20 Keyboard shortcut customization

Socket: `npm run qa:visual -- --only 20`. Captures filtered bindings, recording,
and rejected reserved-key feedback. It cancels recording without saving a new
binding. These states run light/dark/narrow.

Expect aligned command names, readable current bindings, visible recording focus,
wrapping rows and an explanation on invalid/reserved keys. Filtered empty results
and long command labels should not hide Reset all to defaults.

Additional acceptance on the isolated QA database:

1. Snapshot backend shortcut preferences (these live outside localStorage). Set a
   valid unused binding, close Settings and invoke it with a **real OS key chord**.
2. Verify native menus, command palette and editor hints update immediately. Clear
   the binding and verify the old key no longer invokes the command.
3. Attempt a conflict with another command: identify the conflicting command in
   the error and preserve both existing bindings. Test standard edit/system keys.
4. Escape cancels; Tab leaves recording and advances focus; modifier-only/repeated
   events do not save. A failed persistence write leaves a retryable UI.
5. Restart and verify persistence; Reset all to defaults and verify both UI and
   native menus. Restore the captured backend settings in a finally block.

`q.restoreLocal()` cannot restore backend shortcut or author files. Do not run
these mutation checks against the user's regular/demo database. Synthetic
KeyboardEvents and menu-event dispatch do not test OS accelerator registration.
