# 21 Resume writing position

Manual/native extension; not yet an executable socket suite. Use owned fixtures
with enough prose to scroll. Capture the before/after state and record project,
scene, expanded beat, cursor offset and scroll position.

1. Open a later scene, expand its second beat, place the cursor mid-paragraph and
   scroll. Switch to a second fixture and back. Verify scene/beat/cursor/scroll.
2. Repeat in Page view and with a collapsed outline. Verify scene expansion does
   not toggle closed while restoring the selected scene.
3. Save/flush, quit and restart the native app with the same QA data directory.
   Reopen the exact fixture ID and verify restoration; a webview reload alone is
   not a native restart test.
4. Delete/archive the stored scene/beat in an owned fixture, then reopen: fallback
   must be usable with no crash, phantom selection or repeated restoration loop.
5. Verify autosave does not steal focus and pending prose/synopsis edits flush
   before closing. Exercise failed-write retention/retry/discard in fault-injected
   tests and document the boundary if no native failure was established.

Finish ownership cleanup before closing the QA session, or retain the exact IDs
in the run record for manual native-restart cleanup. Never identify fixtures by
name alone after sessionStorage has been lost.
